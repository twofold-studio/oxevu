use std::{path::Path, process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("seek preview timed out")]
    Timeout,
    #[error("could not start seek preview")]
    Process(#[from] std::io::Error),
    #[error("no preview frame at this position")]
    Unavailable,
}

pub async fn jpeg(ffmpeg: &str, source: &Path, seconds: u32) -> Result<Vec<u8>, Error> {
    let output = tokio::time::timeout(
        Duration::from_secs(5),
        Command::new(ffmpeg)
            .args([
                "-nostdin",
                "-v",
                "error",
                "-threads",
                "1",
                "-protocol_whitelist",
                "file",
                "-format_whitelist",
                "matroska,webm,mov,mp4,m4a,3gp,3g2,mj2,avi,mpegts",
                "-ss",
            ])
            .arg(seconds.to_string())
            .arg("-i")
            .arg(source)
            .args([
                "-map",
                "0:V:0",
                "-frames:v",
                "1",
                "-filter_threads",
                "1",
                "-vf",
                "scale=256:144:force_original_aspect_ratio=decrease:force_divisible_by=2",
                "-threads",
                "1",
                "-c:v",
                "mjpeg",
                "-f",
                "image2pipe",
                "pipe:1",
            ])
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| Error::Timeout)??;
    if !output.status.success() || !output.stdout.starts_with(b"\xff\xd8\xff") {
        return Err(Error::Unavailable);
    }
    Ok(output.stdout)
}
