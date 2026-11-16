use crate::*;
use std::{path::Path, process::Stdio};
use tokio::io::AsyncReadExt;

pub(crate) async fn read(path: &Path) -> Result<Value> {
    let mut child = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| bad("ffprobe unavailable"))?;
    let result = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        let mut bytes = Vec::new();
        child
            .stdout
            .take()
            .ok_or_else(|| bad("probe output unavailable"))?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| bad("probe failed"))?;
        if bytes.len() > 1024 * 1024 {
            return Err(bad("probe output exceeds limit"));
        }
        if !child
            .wait()
            .await
            .map_err(|_| bad("probe failed"))?
            .success()
        {
            return Err(bad("not a playable media file"));
        }
        serde_json::from_slice(&bytes).map_err(|_| bad("invalid probe output"))
    })
    .await
    .unwrap_or_else(|_| Err(bad("media validation timed out")));
    if result.is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    result
}
