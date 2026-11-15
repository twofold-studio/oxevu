use super::*;
use std::process::Stdio;
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{Mutex, watch},
};

pub(crate) struct Runtime {
    lock: tokio::fs::File,
    pub shutting_down: std::sync::atomic::AtomicBool,
    pub probe_cache: Mutex<Option<(u32, std::time::Instant, Value)>>,
    pub ffmpeg: String,
    pub scratch: PathBuf,
    pub sessions: Mutex<std::collections::HashMap<String, Entry>>,
}
pub(crate) struct Entry {
    pub pid: Option<u32>,
    pub owner: String,
    pub item: String,
    pub value: Value,
    pub touched: std::time::Instant,
    pub idle: u32,
    pub transcode: bool,
    pub cancel: watch::Sender<bool>,
    pub done: watch::Receiver<bool>,
}
impl Runtime {
    pub async fn new(data: &std::path::Path, pool: &SqlitePool) -> Result<Arc<Self>> {
        let base = std::env::var_os("OXEVU_TRANSCODE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| data.join("transcodes"));
        let base = resolve_candidate(&base).await?;
        let roots: Vec<String> =
            sqlx::query_scalar("SELECT path FROM library_roots UNION SELECT path FROM libraries")
                .fetch_all(pool)
                .await
                .map_err(db)?;
        for root in roots {
            let root = tokio::fs::canonicalize(&root)
                .await
                .unwrap_or_else(|_| PathBuf::from(root));
            if base.starts_with(&root) || root.starts_with(&base) {
                return Err(bad("scratch overlaps media root"));
            }
        }
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| bad("source directory unavailable"))?;
        if base.starts_with(source) || source.starts_with(&base) {
            return Err(bad("scratch overlaps source root"));
        }
        tokio::fs::create_dir_all(&base).await.map_err(bad)?;
        let lock = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(base.join(".oxevu-playback.lock"))
            .await
            .map_err(bad)?;
        use std::os::fd::AsRawFd;
        // SAFETY: flock receives a live file descriptor; LOCK_NB never blocks the executor.
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(bad("scratch is owned by another server"));
        }
        let scratch = base.join("oxevu-sessions");
        if tokio::fs::try_exists(&scratch).await.map_err(bad)? {
            tokio::fs::remove_dir_all(&scratch).await.map_err(bad)?;
        }
        tokio::fs::create_dir(&scratch).await.map_err(bad)?;
        Ok(Arc::new(Self {
            lock,
            probe_cache: Mutex::new(None),
            shutting_down: std::sync::atomic::AtomicBool::new(false),
            ffmpeg: std::env::var("OXEVU_FFMPEG").unwrap_or_else(|_| "ffmpeg".into()),
            scratch,
            sessions: Mutex::new(Default::default()),
        }))
    }
    pub async fn shutdown(&self) {
        self.shutting_down
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let entries: Vec<Entry> = self.sessions.lock().await.drain().map(|(_, e)| e).collect();
        for mut entry in entries {
            let _ = entry.cancel.send(true);
            while !*entry.done.borrow() {
                if entry.done.changed().await.is_err() {
                    break;
                }
            }
        }
        let _ = tokio::fs::remove_dir_all(&self.scratch).await;
        use std::os::fd::AsRawFd;
        // SAFETY: the lock file remains open throughout shutdown.
        let _ = unsafe { libc::flock(self.lock.as_raw_fd(), libc::LOCK_UN) };
    }
}
pub async fn capabilities(a: &App, admin: bool) -> Value {
    let settings = crate::settings::load(a).await.unwrap_or_default();
    let r = &a.playback;
    let mut cache = r.probe_cache.lock().await;
    if let Some((gpu, time, value)) = &*cache
        && *gpu == settings.gpu_index
        && time.elapsed().as_secs() < 60
    {
        return redact(value.clone(), admin);
    }
    let available = probe(&r.ffmpeg, &["-version".into()]).await;
    let nvenc = available
        && probe(
            &r.ffmpeg,
            &[
                "-nostdin",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=320x180:rate=24",
                "-frames:v",
                "1",
                "-c:v",
                "h264_nvenc",
                "-gpu",
            ]
            .map(String::from)
            .into_iter()
            .chain([
                settings.gpu_index.to_string(),
                "-f".into(),
                "null".into(),
                "-".into(),
            ])
            .collect::<Vec<_>>(),
        )
        .await;
    let fixture = r
        .scratch
        .join(format!("probe-{}.h264", uuid::Uuid::new_v4()));
    let mut encode = [
        "-nostdin",
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=size=320x180:rate=24",
        "-frames:v",
        "2",
        "-c:v",
        "libx264",
        "-f",
        "h264",
    ]
    .map(String::from)
    .to_vec();
    encode.push(fixture.to_string_lossy().into_owned());
    let decoded = available
        && probe(&r.ffmpeg, &encode).await
        && probe(
            &r.ffmpeg,
            &[
                "-nostdin".into(),
                "-v".into(),
                "error".into(),
                "-hwaccel".into(),
                "cuda".into(),
                "-hwaccel_device".into(),
                settings.gpu_index.to_string(),
                "-hwaccel_output_format".into(),
                "cuda".into(),
                "-i".into(),
                fixture.to_string_lossy().into_owned(),
                "-vf".into(),
                "scale_cuda=w=160:h=90:format=nv12,hwdownload,format=nv12".into(),
                "-f".into(),
                "null".into(),
                "-".into(),
            ],
        )
        .await;
    let _ = tokio::fs::remove_file(fixture).await;
    let value = json!({"ffmpeg_available":available,"nvenc_available":nvenc,"hardware_decode_available":decoded,"ffmpeg_path":Some(&r.ffmpeg),"scratch_path":Some(r.scratch.to_string_lossy()),"notes": [if !available {"FFmpeg execution failed"}else if !nvenc{"Selected GPU failed an actual H264 NVENC encode probe"}else{"Selected GPU passed an actual H264 NVENC encode probe"},if decoded {"CUDA H264 decode and scale probe passed; codec support depends on GPU; HDR tone mapping unsupported"} else {"CUDA decode/scale probe failed; HDR tone mapping unsupported"}]});
    *cache = Some((settings.gpu_index, std::time::Instant::now(), value.clone()));
    redact(value, admin)
}
fn redact(mut value: Value, admin: bool) -> Value {
    if !admin {
        value["ffmpeg_path"] = Value::Null;
        value["scratch_path"] = Value::Null;
    }
    value
}
async fn probe(binary: &str, args: &[String]) -> bool {
    let mut command = Command::new(binary);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    parent_death(&mut command);
    matches!(tokio::time::timeout(std::time::Duration::from_secs(10),command.status()).await,Ok(Ok(s)) if s.success())
}
pub async fn disk_usage(root: &std::path::Path) -> std::io::Result<u64> {
    let mut dirs = vec![root.to_path_buf()];
    let mut bytes = 0u64;
    let mut count = 0;
    while let Some(dir) = dirs.pop() {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            // Another session can finish while the shared scratch tree is sampled.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && dir != root => continue,
            Err(error) => return Err(error),
        };
        while let Some(e) = entries.next_entry().await? {
            count += 1;
            if count > 8192 {
                return Err(std::io::Error::other("scratch file limit exceeded"));
            }
            let Some(meta) = sampled_metadata(&e.path()).await? else {
                continue;
            };
            if meta.is_dir() {
                dirs.push(e.path())
            } else {
                bytes = bytes.saturating_add(meta.len())
            }
        }
    }
    Ok(bytes)
}
// FFmpeg atomically renames temporary HLS files while this sampler walks them.
// A missing file contributes no bytes; permission and I/O failures still fail closed.
async fn sampled_metadata(path: &std::path::Path) -> std::io::Result<Option<std::fs::Metadata>> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub async fn run_process(
    mut command: Command,
    mut cancel: watch::Receiver<bool>,
    root: &std::path::Path,
    limit: u64,
    ready: &std::path::Path,
    a: &App,
    id: &str,
) -> std::result::Result<(), String> {
    if *cancel.borrow() {
        return Err("Playback stopped".into());
    }
    parent_death(&mut command);
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "FFmpeg could not start".to_string())?;
    if let Some(e) = a.playback.sessions.lock().await.get_mut(id) {
        e.pid = child.id();
    }
    let stderr = child.stderr.take();
    // Drain continuously, retaining no more than 16 KiB. Diagnostics never expose source paths.
    let drain = tokio::spawn(async move {
        let mut kept = Vec::new();
        if let Some(mut pipe) = stderr {
            let mut buf = [0; 4096];
            while let Ok(n) = pipe.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                let retain = n.min(16384usize.saturating_sub(kept.len()));
                kept.extend_from_slice(&buf[..retain]);
            }
        }
        kept
    });
    let start = std::time::Instant::now();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(100));
    let result = loop {
        tokio::select! {
            status=child.wait()=>{break match status{Ok(s) if s.success()=>Ok(()),_=>Err("FFmpeg encoding failed".into())}},
            _=cancel.changed()=>{break Err("Playback stopped".into())},
            _=tick.tick()=>{
                if *cancel.borrow(){break Err("Playback stopped".into())}
                let mut sessions=a.playback.sessions.lock().await;
                let Some(entry)=sessions.get_mut(id) else{break Err("Playback stopped".into())};
                if entry.touched.elapsed().as_secs()>u64::from(entry.idle){break Err("Playback idle lease expired".into())}
                if tokio::fs::try_exists(ready).await.unwrap_or(false){entry.value["status"]=json!("ready");}
                else if start.elapsed().as_secs()>30{break Err("FFmpeg startup timed out".into())}
                let owner=entry.owner.clone();let item=entry.item.clone();
                drop(sessions);
                let authorized=match users::profile(a,&owner).await {Ok(u) if u["can_transcode"]==true=>users::authorize_profile(a,u,&item,true).await.is_ok(),_=>false};
                if !authorized{break Err("Playback permission revoked".into())}
                if disk_usage(root).await.unwrap_or(u64::MAX)>=limit{break Err("Scratch storage limit reached".into())}
            }
        }
    };
    if result.is_err() {
        let _ = child.kill().await;
    }
    let _ = child.wait().await;
    let _ = drain.await;
    if let Some(e) = a.playback.sessions.lock().await.get_mut(id) {
        e.pid = None;
    }
    result
}

// Linux kills children if the server crashes, before stale scratch is removed on restart.
pub fn parent_death(command: &mut Command) {
    #[cfg(target_os = "linux")]
    {
        let parent = std::process::id();
        // SAFETY: only async-signal-safe libc calls execute between fork and exec.
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::getppid() as u32 != parent {
                    return Err(std::io::Error::other("parent exited"));
                }
                Ok(())
            });
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = command;
}

async fn resolve_candidate(path: &std::path::Path) -> Result<PathBuf> {
    let mut parent = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(bad)?.join(path)
    };
    let mut missing = Vec::new();
    loop {
        match tokio::fs::canonicalize(&parent).await {
            Ok(mut existing) => {
                for component in missing.iter().rev() {
                    existing.push(component)
                }
                return Ok(existing);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    parent
                        .file_name()
                        .ok_or_else(|| bad("invalid scratch path"))?
                        .to_owned(),
                );
                if !parent.pop() {
                    return Err(bad("invalid scratch path"));
                }
            }
            Err(_) => return Err(bad("scratch path is inaccessible")),
        }
    }
}

#[cfg(test)]
mod disk_tests {
    use super::*;
    #[tokio::test]
    async fn disappearing_atomic_output_is_not_a_disk_limit_failure() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir(&root).await.unwrap();
        let file = root.join("index.m3u8.tmp");
        tokio::fs::write(&file, b"playlist").await.unwrap();
        let mut entries = tokio::fs::read_dir(&root).await.unwrap();
        let entry = entries.next_entry().await.unwrap().unwrap();
        tokio::fs::rename(&file, root.join("index.m3u8"))
            .await
            .unwrap();
        assert!(sampled_metadata(&entry.path()).await.unwrap().is_none());
        assert_eq!(disk_usage(&root).await.unwrap(), 8);
        tokio::fs::remove_dir_all(&root).await.unwrap();
    }
}
