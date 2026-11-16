use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Serialize)]
pub(super) struct Entry {
    pub id: String,
    pub title: String,
    pub root: PathBuf,
    pub path: PathBuf,
    pub data: Value,
    pub stamp: Option<unlink::Stamp>,
}
pub(super) async fn build(a: &App, ids: &[String]) -> Result<Vec<Entry>> {
    if ids.is_empty() || ids.len() > 1000 {
        return Err(bad("choose 1 to 1000 library items"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut entries = Vec::new();
    for id in ids {
        if !seen.insert(id) || id.len() > 128 {
            return Err(bad("invalid or duplicate library item"));
        }
        let row = sqlx::query("SELECT library_id,path,data FROM items WHERE id=?")
            .bind(id)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?
            .ok_or_else(|| bad("library item no longer exists"))?;
        let path = PathBuf::from(row.get::<String, _>("path"));
        let root = libraries::roots(a, &row.get::<String, _>("library_id"))
            .await?
            .into_iter()
            .map(PathBuf::from)
            .filter(|root| path.starts_with(root))
            .max_by_key(|root| root.components().count())
            .ok_or_else(|| bad("media is outside registered library folders"))?;
        let data: Value =
            serde_json::from_str(row.get("data")).map_err(|_| bad("invalid library item"))?;
        let inspected_root = root.clone();
        let inspected_path = path.clone();
        let inspected =
            tokio::task::spawn_blocking(move || unlink::inspect(&inspected_root, &inspected_path))
                .await
                .map_err(|_| bad("media inspection interrupted"))?;
        let stamp = match inspected {
            Ok(stamp) => stamp,
            Err(error)
                if data["file_available"] == false
                    && error.kind() == std::io::ErrorKind::NotFound =>
            {
                None
            }
            Err(_) => {
                return Err(bad(
                    "media folder is unavailable or contains a symlink; nothing was removed",
                ));
            }
        };
        if stamp.is_none() && data["file_available"] != false {
            return Err(bad(
                "media is missing or offline; restore its folder before removing it",
            ));
        }
        entries.push(Entry {
            id: id.clone(),
            title: data["title"].as_str().unwrap_or("Title").to_owned(),
            root,
            path,
            data,
            stamp,
        });
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(entries)
}
pub(super) fn token(entries: &[Entry], keep: bool) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&(entries, keep)).map_err(bad)?)
    ))
}
pub(super) fn summary(entries: &[Entry], keep: bool) -> Result<Value> {
    let size: u64 = entries
        .iter()
        .filter_map(|e| e.stamp.as_ref())
        .map(unlink::Stamp::size)
        .sum();
    let shared = entries
        .iter()
        .filter(|e| e.stamp.as_ref().is_some_and(|s| s.links > 1))
        .count();
    Ok(
        json!({"token":token(entries,keep)?,"keep_metadata":keep,"count":entries.len(),"bytes_selected":size,"shared_files_count":shared,
        "items":entries.iter().map(|e|json!({"id":e.id,"title":e.title,"size_bytes":e.stamp.as_ref().map(unlink::Stamp::size),"local_file":e.stamp.is_some(),"shared_links":e.stamp.as_ref().map_or(0,|s|s.links)})).collect::<Vec<_>>(),
        "notice":"Only selected library media files are deleted. Artwork, subtitles and torrent download copies remain. Hard links, open players or filesystem snapshots can delay or prevent disk-space reclamation."}),
    )
}
