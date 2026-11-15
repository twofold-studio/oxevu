use super::cache_index::{self, Record};
use super::*;

pub(super) struct Hit {
    pub start: f64,
    pub end: f64,
    pub method: String,
    pub decoder: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
}
pub(super) async fn ranges(
    a: &App,
    path: &std::path::Path,
    method: &str,
    s: &settings::Settings,
) -> Vec<Value> {
    if !s.cache_enabled {
        return Vec::new();
    }
    let Some(expected) = key(a, path, method, s).await else {
        return Vec::new();
    };
    let Some(root) = root(a, false).await else {
        return Vec::new();
    };
    let _guard = a.playback.cache_jobs.lock().await;
    cache_index::records(&root, (s.cache_limit_gb * 1024. * 1024. * 1024.) as u64)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, record, _)| record.key == expected)
        .map(|(_, record, _)| json!({"start":record.start,"end":record.end}))
        .collect()
}
async fn root(a: &App, create: bool) -> Option<PathBuf> {
    let path = a.playback.scratch.parent()?.join("oxevu-cache");
    if create {
        match tokio::fs::create_dir(&path).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return None,
        }
    }
    tokio::fs::symlink_metadata(&path)
        .await
        .ok()?
        .is_dir()
        .then_some(path)
}
pub(super) async fn key(
    a: &App,
    path: &std::path::Path,
    method: &str,
    s: &settings::Settings,
) -> Option<String> {
    let source = source_probe::Identity::read(path).await.ok()?;
    let binary = if a.playback.ffmpeg.contains('/') {
        PathBuf::from(&a.playback.ffmpeg)
    } else {
        let mut executable = None;
        for directory in std::env::split_paths(&std::env::var_os("PATH")?) {
            let candidate = directory.join(&a.playback.ffmpeg);
            if tokio::fs::metadata(&candidate)
                .await
                .is_ok_and(|m| m.is_file())
            {
                executable = Some(candidate);
                break;
            }
        }
        executable?
    };
    let encoder = source_probe::Identity::read(&binary).await.ok()?;
    Some(json!({"pipeline":2,"source":source,"ffmpeg":encoder,"method":method,"gpu":s.gpu_index,"encoding":s.hardware_encoding,"decoding":s.hardware_decoding,"fallback":s.allow_cpu_fallback,"height":s.max_height,"bitrate":s.max_video_bitrate_mbps,"crf":s.cpu_crf}).to_string())
}
pub(super) async fn restore(
    a: &App,
    id: &str,
    path: &std::path::Path,
    start: f64,
    method: &str,
    s: &settings::Settings,
    duration: Option<f64>,
) -> Option<Hit> {
    if !s.cache_enabled {
        return None;
    }
    let expected = key(a, path, method, s).await?;
    let root = root(a, false).await?;
    let _guard = a.playback.cache_jobs.lock().await;
    let entries = cache_index::records(&root, (s.cache_limit_gb * 1024. * 1024. * 1024.) as u64)
        .await
        .ok()?;
    let (directory, mut record, _) = entries
        .into_iter()
        .filter(|(_, r, _)| {
            r.key == expected
                && r.start <= start
                && r.end > start
                && (r.end - start >= 30. || duration.is_some_and(|d| r.end >= d - 0.2))
        })
        .max_by(|a, b| a.1.end.total_cmp(&b.1.end))?;
    let mut anchor = record.start;
    let mut first = 0;
    while first + 1 < record.segments.len() && anchor + record.segments[first].seconds <= start {
        anchor += record.segments[first].seconds;
        first += 1;
    }
    let selected = &record.segments[first..];
    let target = a.playback.scratch.join(id);
    let result = async {
        let needed = runtime::disk_usage(&directory).await.ok()?;
        let used = runtime::disk_usage(&a.playback.scratch).await.ok()?;
        if used.saturating_add(needed) >= (s.scratch_limit_gb * 1024. * 1024. * 1024.) as u64 {
            return None;
        }
        tokio::fs::create_dir(&target).await.ok()?;
        for segment in selected {
            tokio::fs::hard_link(directory.join(&segment.file), target.join(&segment.file))
                .await
                .ok()?;
        }
        tokio::fs::write(target.join("index.m3u8"), cache_index::playlist(selected))
            .await
            .ok()?;
        if key(a, path, method, s).await.as_deref() != Some(expected.as_str()) {
            return None;
        }
        Some(())
    }
    .await;
    if result.is_none() {
        let _ = tokio::fs::remove_dir_all(&target).await;
        return None;
    }
    record.accessed = cache_index::now();
    if let Ok(bytes) = serde_json::to_vec(&record) {
        let _ = tokio::fs::write(directory.join("index.json"), bytes).await;
    }
    Some(Hit {
        start: anchor,
        end: record.end,
        method: record.method,
        decoder: record.decoder,
        width: record.width,
        height: record.height,
    })
}
#[expect(
    clippy::too_many_arguments,
    reason = "cache snapshots an immutable session plan and observed output"
)]
pub(super) async fn store(
    a: &App,
    id: &str,
    path: &std::path::Path,
    start: f64,
    method: &str,
    s: &settings::Settings,
    output: &Value,
    expected_key: Option<&str>,
) {
    if !s.cache_enabled {
        return;
    }
    let Some(key) = key(a, path, method, s).await else {
        return;
    };
    if Some(key.as_str()) != expected_key {
        return;
    }
    let source = a.playback.scratch.join(id);
    let Ok(playlist) = tokio::fs::read_to_string(source.join("index.m3u8")).await else {
        return;
    };
    let Some(segments) = cache_index::segments(&playlist) else {
        return;
    };
    let seconds = segments.iter().map(|s| s.seconds).sum::<f64>();
    if seconds < 4. {
        return;
    }
    let Some(root) = root(a, true).await else {
        return;
    };
    let _guard = a.playback.cache_jobs.lock().await;
    if tokio::fs::create_dir_all(&root).await.is_err() {
        return;
    }
    let Ok(mut entries) =
        cache_index::records(&root, (s.cache_limit_gb * 1024. * 1024. * 1024.) as u64).await
    else {
        return;
    };
    let Ok(size) = runtime::disk_usage(&source).await else {
        return;
    };
    let limit = (s.cache_limit_gb * 1024. * 1024. * 1024.) as u64;
    if size + 512 * 1024 >= limit {
        return;
    }
    entries.sort_by_key(|(_, r, _)| r.accessed);
    let mut used = runtime::disk_usage(&root).await.unwrap_or(u64::MAX);
    let mut files = entries
        .iter()
        .map(|(_, r, _)| r.segments.len())
        .sum::<usize>();
    let mut count = entries.len();
    for (directory, record, bytes) in entries {
        if used.saturating_add(size + 512 * 1024) < limit
            && count < 64
            && files + segments.len() < 8000
        {
            break;
        }
        if tokio::fs::remove_dir_all(directory).await.is_err() {
            return;
        }
        used = used.saturating_sub(bytes);
        files = files.saturating_sub(record.segments.len());
        count = count.saturating_sub(1);
    }
    if used.saturating_add(size + 512 * 1024) >= limit
        || count >= 64
        || files + segments.len() >= 8000
    {
        return;
    }
    let record = Record {
        key,
        start,
        end: start + seconds,
        method: output["method"].as_str().unwrap_or(method).into(),
        decoder: output["decoder"].as_str().unwrap_or("cpu").into(),
        width: output["width"].as_u64(),
        height: output["height"].as_u64(),
        accessed: cache_index::now(),
        segments,
    };
    let destination = root.join(uuid::Uuid::new_v4().to_string());
    let result = async {
        tokio::fs::create_dir(&destination).await.ok()?;
        for segment in &record.segments {
            tokio::fs::hard_link(source.join(&segment.file), destination.join(&segment.file))
                .await
                .ok()?;
        }
        let bytes = serde_json::to_vec(&record).ok()?;
        if bytes.len() > 512 * 1024 {
            return None;
        }
        tokio::fs::write(destination.join("index.json"), bytes)
            .await
            .ok()?;
        Some(())
    }
    .await;
    if result.is_none() {
        let _ = tokio::fs::remove_dir_all(destination).await;
    }
}
