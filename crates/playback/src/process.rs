//! Bounded subprocess supervision; application policy is supplied by an observer.
use oxevu_filesystem::disk_usage;
use serde_json::Value;
use std::{future::Future, path::Path, pin::Pin, process::Stdio};
use tokio::{io::AsyncReadExt, process::Command, sync::watch};

pub type ObserverFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
/// Session lifecycle callbacks. Authentication and persistence stay in the application.
pub trait Observer: Sync {
    fn started(&self, pid: Option<u32>) -> ObserverFuture<'_, ()>;
    fn check(&self, ready: bool) -> ObserverFuture<'_, Result<(), String>>;
}
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Playback stopped")]
    Stopped,
    #[error("FFmpeg could not start")]
    Start,
    #[error("FFmpeg encoding failed")]
    Encoding,
    #[error("FFmpeg startup timed out")]
    Timeout,
    #[error("Scratch storage limit reached")]
    DiskLimit,
    #[error("{0}")]
    Policy(String),
}
pub async fn run(
    mut command: Command,
    mut cancel: watch::Receiver<bool>,
    root: &Path,
    limit: u64,
    ready: &Path,
    observer: &dyn Observer,
) -> Result<(), ProcessError> {
    if *cancel.borrow() {
        return Err(ProcessError::Stopped);
    }
    parent_death(&mut command);
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| ProcessError::Start)?;
    observer.started(child.id()).await;
    let stderr = child.stderr.take();
    let drain = tokio::spawn(async move {
        let mut kept = Vec::new();
        if let Some(mut pipe) = stderr {
            let mut buffer = [0; 4096];
            while let Ok(count) = pipe.read(&mut buffer).await {
                if count == 0 {
                    break;
                }
                let retain = count.min(16384usize.saturating_sub(kept.len()));
                kept.extend_from_slice(&buffer[..retain]);
            }
        }
        kept
    });
    let start = std::time::Instant::now();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(100));
    let result = loop {
        tokio::select! {
         status=child.wait()=>{break match status {Ok(status) if status.success()=>Ok(()),_=>Err(ProcessError::Encoding)}},
         _=cancel.changed()=>{break Err(ProcessError::Stopped)},
         _=tick.tick()=>{
          if *cancel.borrow() {break Err(ProcessError::Stopped);}
          let ready=tokio::fs::try_exists(ready).await.unwrap_or(false);
          if let Err(error)=observer.check(ready).await {break Err(ProcessError::Policy(error));}
          if !ready && start.elapsed().as_secs()>30 {break Err(ProcessError::Timeout);}
          if disk_usage(root).await.unwrap_or(u64::MAX)>=limit {break Err(ProcessError::DiskLimit);}
         }
        }
    };
    if result.is_err() {
        let _ = child.kill().await;
    }
    let _ = child.wait().await;
    let _ = drain.await;
    observer.started(None).await;
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

pub async fn probe(
    mut command: Command,
    mut cancel: watch::Receiver<bool>,
    seconds: u64,
) -> std::result::Result<Value, String> {
    if *cancel.borrow() {
        return Err("Playback stopped".into());
    }
    parent_death(&mut command);
    let mut child = command.spawn().map_err(|_| "ffprobe unavailable")?;
    let stdout = child.stdout.take().ok_or("ffprobe stdout unavailable")?;
    let mut bytes = Vec::new();
    let result = tokio::select! {
        result = tokio::time::timeout(std::time::Duration::from_secs(seconds), async {
            stdout.take(256 * 1024 + 1).read_to_end(&mut bytes).await?;
            if bytes.len() > 256 * 1024 { return Err(std::io::Error::other("probe output exceeded limit")); }
            child.wait().await
        }) => result,
        _ = cancel.changed() => { let _ = child.kill().await; let _ = child.wait().await; return Err("Playback stopped".into()); }
    };
    if !matches!(result, Ok(Ok(status)) if status.success()) {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return Err("Source probe failed".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "Invalid source metadata".into())
}
