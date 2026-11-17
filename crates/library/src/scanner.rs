use super::Error::Storage as db;
use super::{Error, Result, invalid as bad};
use oxevu_db::scanning as repository;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

pub trait Artwork: Sync {
    fn attach<'a>(
        &'a self,
        data: &'a mut Value,
        path: &'a Path,
        root: &'a Path,
        tv: bool,
        errors: &'a mut Vec<String>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}
pub async fn run(pool: &SqlitePool, id: &str, artwork: &dyn Artwork) -> Result<Value> {
    let library = match oxevu_db::libraries::get(pool, id).await {
        Ok(library) => library,
        Err(sqlx::Error::RowNotFound) => return Err(bad("unknown library")),
        Err(error) => return Err(Error::Storage(error)),
    };
    if !library.enabled {
        return Err(bad("library disabled"));
    }
    let roots = super::roots::roots(pool, id).await?;
    let tv = library.kind == "tv";
    let mut dirs = Vec::new();
    let mut errors = Vec::new();
    for root in roots {
        let root = PathBuf::from(root);
        if tokio::fs::metadata(&root).await.is_ok() {
            dirs.push((root.clone(), root));
        } else {
            errors.push("Library root offline; existing catalog retained".into());
        }
    }
    let online_roots: Vec<PathBuf> = dirs.iter().map(|(_, root)| root.clone()).collect();
    let mut scanned = 0;

    let mut visited = 0;
    let re = regex::Regex::new(r"(?i)^(.*?)s(\d{1,3})e(\d{1,3})").map_err(bad)?;
    while let Some((dir, root)) = dirs.pop() {
        let mut entries = tokio::fs::read_dir(dir).await.map_err(bad)?;
        while let Some(entry) = entries.next_entry().await.map_err(bad)? {
            visited += 1;
            if visited > 100000 {
                return Err(bad("scan entry limit exceeded"));
            }
            let ty = entry.file_type().await.map_err(bad)?;
            if ty.is_symlink() {
                continue;
            }
            let path = entry.path();
            if ty.is_dir() {
                dirs.push((path, root.clone()));
                continue;
            }
            if !path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                ["mp4", "mkv", "webm", "mov", "avi", "m4v", "ts"]
                    .contains(&s.to_ascii_lowercase().as_str())
            }) {
                continue;
            }
            let metadata = oxevu_media::read(&path).await.ok();
            let Some(meta) = metadata else {
                errors.push(format!(
                    "Could not probe {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
                continue;
            };
            let tracks = meta["streams"].as_array();
            let codec = |kind: &str| {
                tracks
                    .and_then(|t| t.iter().find(|s| s["codec_type"] == kind))
                    .and_then(|s| s["codec_name"].as_str())
                    .map(str::to_owned)
            };
            if codec("video").is_none() {
                continue;
            }
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .replace('.', " ");
            let episode = re.captures(&stem);
            if tv && episode.is_none() {
                errors.push(format!("Episode name requires SxxExx: {stem}"));
                continue;
            }
            let old = repository::existing_id(pool, path.to_string_lossy().as_ref())
                .await
                .map_err(db)?;
            let item_id = old.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let title = if tv {
                episode
                    .as_ref()
                    .and_then(|captures| captures.get(0))
                    .map(|matched| {
                        oxevu_media::episode_title::clean(
                            &stem,
                            matched.end(),
                            episode
                                .as_ref()
                                .and_then(|c| c.get(3))
                                .and_then(|n| n.as_str().parse().ok())
                                .unwrap_or(0),
                        )
                    })
                    .unwrap_or_else(|| stem.trim().to_owned())
            } else {
                stem.trim().to_owned()
            };
            let mut data = json!({"id":item_id,"library_id":id,"title":title,"kind":if tv{"episode"}else{"movie"},"show_title":show_title(&path, &root, episode.as_ref().and_then(|c|c.get(1)).map(|s|s.as_str())),"season_number":episode.as_ref().and_then(|c|c.get(2)).and_then(|s|s.as_str().parse::<u32>().ok()),"episode_number":episode.as_ref().and_then(|c|c.get(3)).and_then(|s|s.as_str().parse::<u32>().ok()),"duration_seconds":meta["format"]["duration"].as_str().and_then(|s|s.parse::<f64>().ok()),"container":meta["format"]["format_name"],"video_codec":codec("video"),"audio_codec":codec("audio"),"position_seconds":0,"completed":false});
            let transfer = tracks
                .and_then(|t| t.iter().find(|v| v["codec_type"] == "video"))
                .and_then(|v| v["color_transfer"].as_str());
            data["color_transfer"] = json!(transfer);
            data["file_available"] = json!(true);
            data["file_removed_at"] = Value::Null;
            oxevu_media::attach_facts(&mut data, &meta);
            data["hdr"] = json!(tracks.is_some_and(|t| t.iter().any(|v| {
                matches!(
                    v["color_transfer"].as_str(),
                    Some("smpte2084" | "arib-std-b67")
                ) || v["side_data_list"].as_array().is_some_and(|list| {
                    list.iter().any(|d| {
                        d["side_data_type"]
                            .as_str()
                            .is_some_and(|s| s.contains("DOVI"))
                    })
                })
            })));
            artwork
                .attach(&mut data, &path, &root, tv, &mut errors)
                .await;
            repository::upsert(
                pool,
                &item_id,
                id,
                path.to_string_lossy().as_ref(),
                &data.to_string(),
            )
            .await
            .map_err(db)?;
            scanned += 1;
        }
    }
    // Only remove genuinely missing files beneath roots that were online for this scan.
    let rows = repository::present(pool, id).await.map_err(db)?;
    for row in rows {
        let path = PathBuf::from(&row.path);
        if online_roots.iter().any(|root| path.starts_with(root))
            && matches!(tokio::fs::metadata(&path).await,Err(e) if e.kind()==std::io::ErrorKind::NotFound)
        {
            repository::remove_missing(pool, &row.id)
                .await
                .map_err(db)?;
        }
    }
    Ok(json!({"scanned":scanned,"errors":errors}))
}

// A configured TV root uses show folders; release filenames can differ by season.
fn show_title(
    path: &std::path::Path,
    root: &std::path::Path,
    fallback: Option<&str>,
) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    if relative.components().count() > 1 {
        return relative
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().into_owned());
    }
    fallback.map(|title| title.trim().trim_end_matches('-').trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::show_title;
    use std::path::Path;

    #[test]
    fn seasons_with_different_release_names_share_the_show_folder() {
        let root = Path::new("/tv");
        for (path, release_title) in [
            ("/tv/Arcane/Season 01/episode.mkv", "Arcane (2021) - "),
            ("/tv/Arcane/Season 02/episode.mkv", "Arcane "),
        ] {
            assert_eq!(
                show_title(Path::new(path), root, Some(release_title)).as_deref(),
                Some("Arcane")
            );
        }
        assert_eq!(
            show_title(Path::new("/tv/Arcane.S01E01.mkv"), root, Some("Arcane ")).as_deref(),
            Some("Arcane")
        );
    }
}
