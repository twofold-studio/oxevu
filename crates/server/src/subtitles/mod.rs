use crate::*;
use axum::http::header;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

const LIMIT: u64 = 8 * 1024 * 1024;
struct Track {
    value: Value,
    source: PathBuf,
    stream: Option<u64>,
}

async fn output(mut command: Command, limit: u64) -> Result<Vec<u8>> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|_| bad("subtitle tool unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| bad("subtitle output unavailable"))?;
    let mut bytes = Vec::new();
    let result = tokio::time::timeout(std::time::Duration::from_secs(60), async {
        stdout.take(limit + 1).read_to_end(&mut bytes).await?;
        if bytes.len() as u64 > limit {
            return Err(std::io::Error::other("output too large"));
        }
        child.wait().await
    })
    .await;
    if !matches!(result, Ok(Ok(status)) if status.success()) {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return Err(bad("subtitle extraction failed or exceeded limits"));
    }
    Ok(bytes)
}

async fn tracks(a: &App, h: &HeaderMap, id: &str) -> Result<Vec<Track>> {
    users::authorize_item(a, h, id, true).await?;
    let path: String = sqlx::query_scalar("SELECT path FROM items WHERE id=?")
        .bind(id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let path = tokio::fs::canonicalize(path)
        .await
        .map_err(|_| bad("source unavailable"))?;
    let root = libraries::item_root(a, id, &path).await?;
    let mut probe = Command::new("ffprobe");
    probe
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-select_streams",
            "s",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(&path);
    let meta: Value = serde_json::from_slice(&output(probe, 256 * 1024).await?)
        .map_err(|_| bad("invalid subtitle metadata"))?;
    let mut tracks = Vec::new();
    for stream in meta["streams"].as_array().into_iter().flatten().take(128) {
        let Some(index) = stream["index"].as_u64() else {
            continue;
        };
        let codec = stream["codec_name"].as_str().unwrap_or("unknown");
        let supported = matches!(
            codec,
            "subrip" | "ass" | "ssa" | "webvtt" | "mov_text" | "text"
        );
        let language = stream["tags"]["language"].as_str().unwrap_or("und");
        let label = stream["tags"]["title"].as_str().unwrap_or(language);
        let track_id = format!("stream-{index}");
        tracks.push(Track { value: json!({"id":track_id,"label":label,"language":language,"codec":codec,"default":stream["disposition"]["default"]==1,"forced":stream["disposition"]["forced"]==1,"supported":supported,"url":if supported {Some(format!("/api/v1/items/{id}/subtitles/{track_id}"))}else{None}}), source:path.clone(),stream:Some(index) });
    }
    let parent = path.parent().ok_or_else(|| bad("invalid media path"))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| bad("invalid media name"))?;
    let mut entries = tokio::fs::read_dir(parent).await.map_err(bad)?;
    let mut sidecars = Vec::new();
    let mut count = 0;
    while let Some(entry) = entries.next_entry().await.map_err(bad)? {
        count += 1;
        if count > 10000 {
            return Err(bad("subtitle directory entry limit exceeded"));
        }
        let p = entry.path();
        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(ext.as_str(), "srt" | "vtt" | "ass" | "ssa") {
            continue;
        }
        let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if name != stem && !name.strip_prefix(stem).is_some_and(|s| s.starts_with('.')) {
            continue;
        }
        let p = tokio::fs::canonicalize(p).await.map_err(bad)?;
        if !p.starts_with(&root) {
            continue;
        }
        if tokio::fs::metadata(&p).await.map_err(bad)?.len() > LIMIT {
            continue;
        }
        sidecars.push((p, ext));
    }
    sidecars.sort_by(|a, b| a.0.cmp(&b.0));
    for (index, (source, codec)) in sidecars.into_iter().take(128).enumerate() {
        let label = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Subtitles");
        let language = label
            .strip_prefix(stem)
            .unwrap_or("")
            .trim_start_matches('.');
        let track_id = format!("sidecar-{index}");
        tracks.push(Track {value:json!({"id":track_id,"label":if language.is_empty(){"External subtitles"}else{language},"language":language,"codec":codec,"default":false,"forced":language.contains("forced"),"supported":true,"url":format!("/api/v1/items/{id}/subtitles/{track_id}")}),source,stream:None});
    }
    let downloads = sqlx::query(
        "SELECT id,language,file FROM item_subtitles WHERE item_id=? ORDER BY id LIMIT 128",
    )
    .bind(id)
    .fetch_all(&a.pool)
    .await
    .map_err(db)?;
    let cache = a
        .artwork_cache
        .parent()
        .ok_or_else(|| bad("data directory unavailable"))?
        .join("subtitles");
    for row in downloads {
        let source = PathBuf::from(row.get::<String, _>("file"));
        let Ok(source) = tokio::fs::canonicalize(source).await else {
            continue;
        };
        if !source.starts_with(&cache) {
            continue;
        }
        let track_id = format!("download-{}", row.get::<String, _>("id"));
        let language: String = row.get("language");
        tracks.push(Track {value:json!({"id":track_id,"label":format!("{language} (OpenSubtitles)"),"language":language,"codec":"vtt","default":false,"forced":false,"supported":true,"url":format!("/api/v1/items/{id}/subtitles/{track_id}")}),source,stream:None});
    }
    Ok(tracks)
}

pub(crate) async fn list(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let _permit = a
        .artwork_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("subtitle worker busy"))?;
    Ok(Json(
        json!({"tracks":tracks(&a,&h,&id).await?.into_iter().map(|t|t.value).collect::<Vec<_>>()}),
    ))
}
pub(crate) async fn serve(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, track)): Path<(String, String)>,
) -> Result<Response> {
    let _permit = a
        .artwork_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("subtitle worker busy"))?;
    let selected = tracks(&a, &h, &id)
        .await?
        .into_iter()
        .find(|t| t.value["id"] == track)
        .ok_or_else(|| bad("unknown subtitle track"))?;
    if selected.value["supported"] != true {
        return Err(bad("image subtitles require burn-in, which is unavailable"));
    }
    Ok((
        [
            (header::CONTENT_TYPE, "text/vtt; charset=utf-8"),
            (header::CACHE_CONTROL, "private, no-store"),
        ],
        convert(&a, &selected.source, selected.stream).await?,
    )
        .into_response())
}
pub(crate) async fn convert(
    a: &App,
    source: &std::path::Path,
    stream: Option<u64>,
) -> Result<Vec<u8>> {
    let mut command = Command::new(&a.playback.ffmpeg);
    command
        .args([
            "-nostdin",
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-i",
        ])
        .arg(source);
    if let Some(index) = stream {
        command.args(["-map", &format!("0:{index}")]);
    }
    command.args(["-f", "webvtt", "pipe:1"]);
    output(command, LIMIT).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn external_subtitles_require_item_access_and_stay_inside_library() {
        let a = crate::tests::app().await;
        let h = crate::tests::account(&a, "owner", true).await;
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir_all(&root).await.unwrap();
        let file = root.join("film.mp4");
        let status = Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=64x64:rate=1",
                "-t",
                "1",
                "-c:v",
                "libx264",
            ])
            .arg(&file)
            .status()
            .await
            .unwrap();
        assert!(status.success());
        tokio::fs::write(
            root.join("film.en.srt"),
            b"1\n00:00:00,000 --> 00:00:00,800\nHello\n",
        )
        .await
        .unwrap();
        let outside = root.with_extension("srt");
        tokio::fs::write(&outside, b"secret").await.unwrap();
        std::os::unix::fs::symlink(&outside, root.join("film.fr.srt")).unwrap();
        sqlx::query("INSERT INTO libraries VALUES('l','Test',?,'movies')")
            .bind(root.to_string_lossy().as_ref())
            .execute(&a.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO items VALUES('i','l',?,'{}')")
            .bind(file.to_string_lossy().as_ref())
            .execute(&a.pool)
            .await
            .unwrap();
        assert!(
            list(State(a.clone()), HeaderMap::new(), Path("i".into()))
                .await
                .is_err()
        );
        let tracks = tracks(&a, &h, "i").await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].value["language"], "en");
        let content = convert(&a, &tracks[0].source, None).await.unwrap();
        assert!(String::from_utf8(content).unwrap().contains("WEBVTT"));
        let _ = tokio::fs::remove_dir_all(root).await;
        let _ = tokio::fs::remove_file(outside).await;
    }
}
