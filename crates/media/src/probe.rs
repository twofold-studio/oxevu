use serde_json::{Value, json};

/// A bounded probe failure safe to report without exposing the media path.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ProbeError(&'static str);
type Result<T> = std::result::Result<T, ProbeError>;
fn bad(message: &'static str) -> ProbeError {
    ProbeError(message)
}
use std::{path::Path, process::Stdio};
use tokio::io::AsyncReadExt;

/// Keep bounded technical facts from the probe, never its filesystem paths or tags.
pub fn attach_facts(item: &mut Value, probe: &Value) {
    let tracks = probe["streams"].as_array();
    let video = tracks.and_then(|v| v.iter().find(|v| v["codec_type"] == "video"));
    let audio = tracks.and_then(|v| v.iter().find(|v| v["codec_type"] == "audio"));
    let number = |value: &Value| {
        value
            .as_u64()
            .or_else(|| value.as_str()?.parse::<u64>().ok())
    };
    item["size_bytes"] = json!(number(&probe["format"]["size"]));
    item["bitrate"] = json!(number(&probe["format"]["bit_rate"]));
    item["video_width"] = json!(video.and_then(|v| number(&v["width"])));
    item["video_height"] = json!(video.and_then(|v| number(&v["height"])));
    item["video_bitrate"] = json!(video.and_then(|v| number(&v["bit_rate"])));
    item["audio_bitrate"] = json!(audio.and_then(|v| number(&v["bit_rate"])));
    item["audio_channels"] = json!(audio.and_then(|v| number(&v["channels"])));
    item["audio_channel_layout"] = json!(audio.and_then(|v| v["channel_layout"].as_str()));
}

pub async fn read(path: &Path) -> Result<Value> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_details_probe_reads_numeric_and_string_facts_without_leaking_paths() {
        let mut item = json!({});
        attach_facts(
            &mut item,
            &json!({"format":{"size":"12345","bit_rate":"9000000","filename":"/private/movie.mkv"},
            "streams":[{"codec_type":"video","width":1920,"height":1080,"bit_rate":"8500000"},
                       {"codec_type":"audio","channels":6,"channel_layout":"5.1","bit_rate":"500000"}]}),
        );
        assert_eq!(item["size_bytes"], 12345);
        assert_eq!(item["bitrate"], 9000000);
        assert_eq!(item["video_width"], 1920);
        assert_eq!(item["audio_channels"], 6);
        assert_eq!(item["audio_channel_layout"], "5.1");
        assert!(item.get("filename").is_none());
        attach_facts(&mut item, &json!({}));
        assert!(item["bitrate"].is_null());
        assert!(item["audio_channels"].is_null());
    }
}
