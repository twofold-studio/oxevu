use super::*;
use std::process::Stdio;
use tokio::{
    process::Command,
    sync::{Mutex, watch},
};

pub(crate) struct Runtime {
    lock: tokio::fs::File,
    pub shutting_down: std::sync::atomic::AtomicBool,
    pub probe_cache: Mutex<Option<(u32, std::time::Instant, Value)>>,
    pub source_metadata: Mutex<super::source_probe::Cache>,
    pub cache_jobs: Mutex<()>,
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
            source_metadata: Mutex::new(Default::default()),
            cache_jobs: Mutex::new(()),
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
pub(crate) use oxevu_filesystem::disk_usage;

struct SessionObserver<'a> {
    app: &'a App,
    id: &'a str,
}
impl oxevu_playback::process::Observer for SessionObserver<'_> {
    fn started(&self, pid: u32) -> oxevu_playback::process::ObserverFuture<'_, ()> {
        Box::pin(async move {
            if let Some(entry) = self.app.playback.sessions.lock().await.get_mut(self.id) {
                entry.pid = Some(pid);
            }
        })
    }
    fn finished(&self) -> oxevu_playback::process::ObserverFuture<'_, ()> {
        Box::pin(async move {
            if let Some(entry) = self.app.playback.sessions.lock().await.get_mut(self.id) {
                entry.pid = None;
            }
        })
    }
    fn tick(
        &self,
        ready: bool,
    ) -> oxevu_playback::process::ObserverFuture<
        '_,
        std::result::Result<(), oxevu_playback::process::PolicyError>,
    > {
        Box::pin(async move {
            let mut sessions = self.app.playback.sessions.lock().await;
            let entry = sessions
                .get_mut(self.id)
                .ok_or(oxevu_playback::process::PolicyError::Stopped)?;
            if entry.touched.elapsed().as_secs() > u64::from(entry.idle) {
                return Err(oxevu_playback::process::PolicyError::IdleExpired);
            }
            if ready {
                entry.value["status"] = json!("ready");
            }
            let owner = entry.owner.clone();
            let item = entry.item.clone();
            drop(sessions);
            let authorized = match users::profile(self.app, &owner).await {
                Ok(user) if user["can_transcode"] == true => {
                    users::authorize_profile(self.app, user, &item, true)
                        .await
                        .is_ok()
                }
                _ => false,
            };
            if !authorized {
                return Err(oxevu_playback::process::PolicyError::PermissionRevoked);
            }
            Ok(())
        })
    }
}
pub async fn run_process(
    command: Command,
    cancel: watch::Receiver<bool>,
    root: &std::path::Path,
    limit: u64,
    ready: &std::path::Path,
    app: &App,
    id: &str,
) -> std::result::Result<(), oxevu_playback::process::ProcessError> {
    oxevu_playback::process::run(
        command,
        cancel,
        root,
        limit,
        ready,
        &SessionObserver { app, id },
    )
    .await
}
pub use oxevu_playback::process::parent_death;

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
