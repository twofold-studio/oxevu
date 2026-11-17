use oxevu_filesystem as runtime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Deserialize, Serialize)]
pub struct Segment {
    pub file: String,
    pub seconds: f64,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Record {
    pub key: String,
    pub start: f64,
    pub end: f64,
    pub method: String,
    pub decoder: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub accessed: u64,
    pub segments: Vec<Segment>,
}
impl Record {
    fn valid(&self) -> bool {
        if !self.start.is_finite()
            || !self.end.is_finite()
            || self.start < 0.
            || self.end <= self.start
            || self.segments.is_empty()
            || self.segments.len() > 8000
        {
            return false;
        }
        let mut names = std::collections::HashSet::new();
        for segment in &self.segments {
            if !valid_file(&segment.file)
                || !segment.seconds.is_finite()
                || segment.seconds <= 0.
                || segment.seconds > 60.
                || !names.insert(&segment.file)
            {
                return false;
            }
        }
        let seconds = self.segments.iter().map(|s| s.seconds).sum::<f64>();
        (self.end - self.start - seconds).abs() < 0.01
    }
}
fn valid_file(name: &str) -> bool {
    name.strip_prefix("segment")
        .and_then(|s| s.strip_suffix(".ts"))
        .is_some_and(|s| !s.is_empty() && s.len() <= 20 && s.bytes().all(|b| b.is_ascii_digit()))
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn segments(playlist: &str) -> Option<Vec<Segment>> {
    let mut duration = None;
    let mut segments = Vec::new();
    for line in playlist.lines() {
        if let Some(value) = line.strip_prefix("#EXTINF:") {
            let seconds = value.split(',').next()?.parse::<f64>().ok()?;
            if !seconds.is_finite() || !(0. ..=60.).contains(&seconds) || seconds == 0. {
                return None;
            }
            duration = Some(seconds);
        } else if !line.starts_with('#') && !line.is_empty() {
            if !valid_file(line) {
                return None;
            }
            segments.push(Segment {
                file: line.to_owned(),
                seconds: duration.take()?,
            });
            if segments.len() > 8000 {
                return None;
            }
        }
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}
pub fn playlist(segments: &[Segment]) -> String {
    let target = segments
        .iter()
        .map(|s| s.seconds.ceil() as u64)
        .max()
        .unwrap_or(2);
    let mut value = format!(
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:{target}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n"
    );
    for segment in segments {
        value.push_str(&format!(
            "#EXTINF:{:.6},\n{}\n",
            segment.seconds, segment.file
        ));
    }
    value.push_str("#EXT-X-ENDLIST\n");
    value
}
pub async fn records(
    root: &std::path::Path,
    limit: u64,
) -> std::io::Result<Vec<(PathBuf, Record, u64)>> {
    let mut entries = tokio::fs::read_dir(root).await?;
    let mut records = Vec::new();
    let mut count = 0;
    while let Some(entry) = entries.next_entry().await? {
        count += 1;
        if count > 128 {
            return Err(std::io::Error::other("cache entry limit exceeded"));
        }
        if uuid::Uuid::parse_str(&entry.file_name().to_string_lossy()).is_err() {
            continue;
        }
        let directory = entry.path();
        if entry.file_type().await?.is_symlink() {
            tokio::fs::remove_file(directory).await?;
            continue;
        }
        let index = directory.join("index.json");
        let record = async {
            let meta = tokio::fs::symlink_metadata(&index).await.ok()?;
            if !meta.is_file() || meta.len() > 512 * 1024 {
                return None;
            }
            let data = tokio::fs::read(&index).await.ok()?;
            let record = serde_json::from_slice::<Record>(&data).ok()?;
            if !record.valid() {
                return None;
            }
            for segment in &record.segments {
                let meta = tokio::fs::symlink_metadata(directory.join(&segment.file))
                    .await
                    .ok()?;
                if !meta.is_file() {
                    return None;
                }
            }
            Some(record)
        }
        .await;
        let Some(record) = record else {
            tokio::fs::remove_dir_all(directory).await?;
            continue;
        };
        let size = runtime::disk_usage(&directory).await?;
        records.push((directory, record, size));
    }
    records.sort_by_key(|(_, record, _)| record.accessed);
    let mut used = runtime::disk_usage(root).await?;
    while used > limit && !records.is_empty() {
        let (path, _, bytes) = records.remove(0);
        tokio::fs::remove_dir_all(path).await?;
        used = used.saturating_sub(bytes);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn reduced_cache_limit_evicts_entries() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let entry = root.join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir_all(&entry).await.unwrap();
        tokio::fs::write(entry.join("segment000000.ts"), b"fixture")
            .await
            .unwrap();
        let record = Record {
            key: "fixture".into(),
            start: 0.,
            end: 2.,
            method: "cpu".into(),
            decoder: "cpu".into(),
            width: None,
            height: None,
            accessed: now(),
            segments: vec![Segment {
                file: "segment000000.ts".into(),
                seconds: 2.,
            }],
        };
        tokio::fs::write(
            entry.join("index.json"),
            serde_json::to_vec(&record).unwrap(),
        )
        .await
        .unwrap();
        assert!(records(&root, 0).await.unwrap().is_empty());
        assert!(!entry.exists());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
    #[test]
    fn frozen_playlist_keeps_segment_durations_and_rejects_paths() {
        let parsed =
            segments("#EXTM3U\n#EXTINF:2.002,\nsegment000000.ts\n#EXTINF:1.5,\nsegment000001.ts\n")
                .unwrap();
        assert_eq!(parsed.iter().map(|s| s.seconds).sum::<f64>(), 3.502);
        assert!(playlist(&parsed).ends_with("#EXT-X-ENDLIST\n"));
        assert!(segments("#EXTINF:2,\n../outside.ts\n").is_none());
        assert!(segments("#EXTINF:NaN,\nsegment0.ts\n").is_none());
    }
}
