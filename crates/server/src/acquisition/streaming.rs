//! Verified torrent reads. Sparse on-disk files are never served directly.
use super::*;
use axum::{
    body::{Body, Bytes},
    http::{Method, header},
};
use librqbit::ManagedTorrent;
use std::{
    io::{Error, ErrorKind, SeekFrom},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    sync::OwnedSemaphorePermit,
};

pub(super) const MAX_READERS: usize = 8;

#[derive(Clone)]
pub(crate) struct Source {
    pub download: String,
    pub index: usize,
    pub size: u64,
    pub mime: &'static str,
    title: String,
    hash: String,
}

pub(crate) async fn for_item(a: &App, item: &str) -> Result<Option<Source>> {
    let row = sqlx::query("SELECT s.download_id,s.file_index,s.title_id,d.infohash,d.data FROM acquisition_stream_items s JOIN acquisition_downloads d ON d.id=s.download_id JOIN items i ON i.id=s.item_id WHERE s.item_id=? AND COALESCE(json_extract(i.data,'$.file_available'),1)=0")
        .bind(item).fetch_optional(&a.pool).await.map_err(db)?;
    let Some(row) = row else { return Ok(None) };
    let data = decode(row.get("data"))?;
    let index = row.get::<i64, _>("file_index") as usize;
    let file = data["files"]
        .get(index)
        .ok_or_else(|| bad("Download metadata is not ready."))?;
    let path = file["path"]
        .as_str()
        .filter(|p| auto_import::is_video(p))
        .ok_or_else(|| bad("Not a supported video file."))?;
    let source = Source {
        download: row.get("download_id"),
        index,
        size: file["size"]
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or_else(|| bad("Download size is unknown."))?,
        title: row.get("title_id"),
        hash: row
            .get::<Option<String>, _>("infohash")
            .ok_or_else(|| bad("Torrent metadata is not ready."))?,
        mime: match std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("mp4" | "m4v" | "mov") => "video/mp4",
            Some("webm") => "video/webm",
            Some("mkv") => "video/x-matroska",
            Some("ts") => "video/mp2t",
            _ => "video/x-msvideo",
        },
    };
    source.handle(a).await?;
    Ok(Some(source))
}

impl Source {
    async fn handle(&self, a: &App) -> Result<Arc<ManagedTorrent>> {
        if a.acquisition.is_stopping() || !config::load(a).await?.enabled {
            return Err(bad("Downloads are disabled."));
        }
        let row = sqlx::query(
            "SELECT state,title_id,infohash,data FROM acquisition_downloads WHERE id=?",
        )
        .bind(&self.download)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("Download no longer exists."))?;
        let data = decode(row.get("data"))?;
        if row.get::<Option<String>, _>("title_id").as_deref() != Some(&self.title)
            || row.get::<Option<String>, _>("infohash").as_deref() != Some(&self.hash)
        {
            return Err(bad("Download identity changed. Refresh this title."));
        }
        if !matches!(
            row.get::<String, _>("state").as_str(),
            "downloading" | "seeding" | "completed"
        ) {
            return Err(bad(
                "This download is paused, queued or unavailable. Resume it in Downloads first.",
            ));
        }
        if data["input"]["selected_files"]
            .as_array()
            .is_some_and(|v| !v.iter().any(|i| i.as_u64() == Some(self.index as u64)))
        {
            return Err(bad("This video is no longer selected for download."));
        }
        let session = a
            .acquisition
            .current()
            .await
            .ok_or_else(|| bad("Torrent engine is not ready."))?;
        let handle = downloads::find(&session, Some(&self.hash))
            .ok_or_else(|| bad("Torrent is not ready. Wait for it to resume."))?;
        let stats = handle.stats();
        if !stats.finished && stats.state.to_string() != "live" {
            return Err(bad("This download is waiting for queue capacity."));
        }
        if handle
            .only_files()
            .is_some_and(|v| !v.contains(&self.index))
        {
            return Err(bad("Video is not selected."));
        }
        Ok(handle)
    }

    pub(crate) async fn response(&self, a: App, request: Request) -> Result<Response> {
        self.response_with_cancel(a, request, None).await
    }

    pub(crate) async fn response_with_cancel(
        &self,
        a: App,
        request: Request,
        cancel: Option<tokio::sync::watch::Receiver<bool>>,
    ) -> Result<Response> {
        if cancel.as_ref().is_some_and(|rx| *rx.borrow()) {
            return Err(bad("Playback stopped."));
        }
        let raw = request
            .headers()
            .get(header::RANGE)
            .map(|h| h.to_str().map_err(|_| ()));
        let range = raw.transpose().and_then(|raw| byte_range(raw, self.size));
        let Ok(range) = range else {
            return Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{}", self.size))
                .header(header::CACHE_CONTROL, "no-store")
                .body(Body::empty())
                .map_err(bad);
        };
        let (start, end, partial) = range;
        let length = end - start + 1;
        let handle = self.handle(&a).await?;
        let mut response = Response::builder()
            .status(if partial {
                StatusCode::PARTIAL_CONTENT
            } else {
                StatusCode::OK
            })
            .header(header::ACCEPT_RANGES, "bytes")
            .header(header::CONTENT_TYPE, self.mime)
            .header(header::CONTENT_LENGTH, length)
            .header(header::CACHE_CONTROL, "no-store");
        if partial {
            response = response.header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{}", self.size),
            );
        }
        if request.method() == Method::HEAD {
            return response.body(Body::empty()).map_err(bad);
        }
        let permit = a
            .acquisition
            .streams
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error":"Too many active torrent readers."})),
                )
            })?;
        let mut reader = tokio::time::timeout(Duration::from_secs(10), handle.stream(self.index))
            .await
            .map_err(|_| bad("Torrent reader initialization timed out."))?
            .map_err(|_| bad("Torrent reader unavailable."))?;
        reader
            .seek(SeekFrom::Start(start))
            .await
            .map_err(|_| bad("Cannot seek this torrent."))?;
        let state = ReadState {
            reader: Box::new(reader),
            remaining: length,
            a,
            source: self.clone(),
            _permit: permit,
            cancel,
            checked: std::time::Instant::now(),
        };
        let body = Body::from_stream(futures_util::stream::try_unfold(
            state,
            |mut state| async move {
                if state.remaining == 0 {
                    return Ok::<_, Error>(None);
                }
                let mut buf = vec![0u8; state.remaining.min(64 * 1024) as usize];
                let n = state.read(&mut buf).await?;
                if n == 0 {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Torrent ended before its advertised size",
                    ));
                }
                state.remaining -= n as u64;
                buf.truncate(n);
                Ok(Some((Bytes::from(buf), state)))
            },
        ));
        response.body(body).map_err(bad)
    }
}

struct ReadState {
    reader: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
    remaining: u64,
    a: App,
    source: Source,
    _permit: OwnedSemaphorePermit,
    cancel: Option<tokio::sync::watch::Receiver<bool>>,
    checked: std::time::Instant,
}
impl ReadState {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.as_ref().is_some_and(|rx| *rx.borrow()) {
            return Err(Error::new(ErrorKind::Interrupted, "Playback stopped"));
        }
        if self.checked.elapsed() >= Duration::from_secs(1) {
            self.source
                .handle(&self.a)
                .await
                .map_err(|_| Error::new(ErrorKind::Interrupted, "Download unavailable"))?;
            self.checked = std::time::Instant::now();
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        loop {
            tokio::select! {
                _=async {match &mut self.cancel {Some(rx)=>{let _=rx.changed().await;},None=>std::future::pending::<()>().await}}=>return Err(Error::new(ErrorKind::Interrupted,"Playback stopped")),
                value=self.reader.read(buf)=>return value,
                _=tokio::time::sleep_until(deadline)=>return Err(Error::new(ErrorKind::TimedOut,"Waiting for verified torrent pieces timed out")),
                _=tokio::time::sleep(Duration::from_secs(1))=>{
                    self.source.handle(&self.a).await.map_err(|_|Error::new(ErrorKind::Interrupted,"Download paused, removed or unavailable"))?;
                }
            }
        }
    }
}

/// Single RFC-style byte ranges, including suffixes. Multi-ranges are rejected.
fn byte_range(raw: Option<&str>, size: u64) -> std::result::Result<(u64, u64, bool), ()> {
    if size == 0 {
        return Err(());
    }
    let Some(raw) = raw else {
        return Ok((0, size - 1, false));
    };
    let (start, end) = raw
        .strip_prefix("bytes=")
        .and_then(|r| r.split_once('-'))
        .ok_or(())?;
    if !start.bytes().all(|b| b.is_ascii_digit()) || !end.bytes().all(|b| b.is_ascii_digit()) {
        return Err(());
    }
    if end.contains(',') {
        return Err(());
    }
    if start.is_empty() {
        let suffix = end.parse::<u64>().map_err(|_| ())?;
        if suffix == 0 {
            return Err(());
        }
        return Ok((size.saturating_sub(suffix), size - 1, true));
    }
    let start = start.parse::<u64>().map_err(|_| ())?;
    let end = if end.is_empty() {
        size - 1
    } else {
        end.parse::<u64>().map_err(|_| ())?.min(size - 1)
    };
    if start >= size || end < start {
        return Err(());
    }
    Ok((start, end, true))
}

#[cfg(test)]
mod conversion_tests;
#[cfg(test)]
mod tests;
