use super::*;
use scope::Episode;
use std::path::Path;

#[derive(Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = ImportMapping)]
pub(super) struct Mapping {
    pub file_index: usize,
    #[serde(default)]
    #[schema(value_type = Vec<crate::contract::acquisition_catalog_model::EpisodeRef>)]
    pub episodes: Vec<Episode>,
}
#[derive(Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = ImportInput)]
pub(super) struct Input {
    pub library_id: String,
    pub root: String,
    pub mappings: Vec<Mapping>,
    #[serde(default)]
    pub token: Option<String>,
}
#[derive(Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[schema(as = ImportFile)]
pub(super) struct File {
    pub file_index: usize,
    pub source: String,
    pub destination: String,
    pub size: u64,
    #[schema(value_type = Vec<crate::contract::acquisition_catalog_model::EpisodeRef>)]
    pub coverage: Vec<Episode>,
    pub title_id: String,
    pub root_device: u64,
    pub root: String,
    pub source_stamp: String,
}

pub(super) async fn build(a: &App, id: &str, input: &Input) -> Result<Vec<File>> {
    if input.mappings.is_empty() || input.mappings.len() > 1000 {
        return Err(bad("select between 1 and 1000 files to import"));
    }
    let c = config::load(a).await?;
    let row = sqlx::query(
        "SELECT title_id,data FROM acquisition_downloads WHERE id=? AND state!='removed'",
    )
    .bind(id)
    .fetch_optional(&a.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| bad("unknown download"))?;
    let title_id: String = row
        .get::<Option<String>, _>("title_id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| bad("This torrent has not been recognized yet. Wait for recognition or associate it with a catalog title before importing."))?;
    let title_raw: String = sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id=?")
        .bind(&title_id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("The associated catalog title is missing. Identify this torrent again before importing."))?;
    let title = decode(&title_raw)?;
    let data = decode(row.get("data"))?;
    let kind = if title["kind"] == "tv" {
        "tv"
    } else {
        "movies"
    };
    if kind == "movies" && input.mappings.len() != 1 {
        return Err(bad("select one main video file for a movie"));
    }
    let registered:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM libraries l JOIN library_roots r ON r.library_id=l.id LEFT JOIN library_options o ON o.library_id=l.id WHERE l.id=? AND l.kind=? AND r.path=? AND COALESCE(o.enabled,1)=1)")
        .bind(&input.library_id).bind(kind).bind(&input.root).fetch_one(&a.pool).await.map_err(db)?;
    if !registered {
        return Err(bad(
            "choose an enabled library root matching the title type",
        ));
    }
    let root = tokio::fs::canonicalize(&input.root)
        .await
        .map_err(|_| bad("library root offline; import suspended"))?;
    let root_meta = tokio::fs::metadata(&root)
        .await
        .map_err(|_| bad("library root offline"))?;
    if !root_meta.is_dir() {
        return Err(bad("library root is not a directory"));
    }
    use std::os::unix::fs::MetadataExt;
    let device = root_meta.dev();
    let download_root = tokio::fs::canonicalize(Path::new(&c.download_dir).join(id))
        .await
        .map_err(|_| bad("download files unavailable"))?;
    if storage::overlaps(&root, &download_root) {
        return Err(bad("download and library roots overlap"));
    }
    let files = data["files"]
        .as_array()
        .ok_or_else(|| bad("torrent metadata is not ready"))?;
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for mapping in &input.mappings {
        if !seen.insert(mapping.file_index) {
            return Err(bad("each source file can be mapped only once"));
        }
        let file = files
            .get(mapping.file_index)
            .ok_or_else(|| bad("invalid torrent file index"))?;
        let source = download_root.join(
            file["path"]
                .as_str()
                .ok_or_else(|| bad("invalid source file"))?,
        );
        let source = tokio::fs::canonicalize(&source)
            .await
            .map_err(|_| bad("source file missing"))?;
        if !source.starts_with(&download_root) {
            return Err(bad("source file escapes download storage"));
        }
        let expected = file["size"]
            .as_u64()
            .ok_or_else(|| bad("unknown source size"))?;
        let metadata = tokio::fs::metadata(&source)
            .await
            .map_err(|_| bad("source unavailable"))?;
        if !metadata.is_file() || metadata.len() != expected {
            return Err(bad("source size changed; recheck download before import"));
        }
        if data["stats"]["file_progress"]
            .as_array()
            .and_then(|v| v.get(mapping.file_index))
            .and_then(Value::as_u64)
            != Some(expected)
        {
            return Err(bad("source file is not fully verified"));
        }
        let extension = source
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_ascii_lowercase)
            .filter(|s| ["mkv", "mp4", "webm", "mov", "avi", "m4v", "ts"].contains(&s.as_str()))
            .ok_or_else(|| bad("only supported video files can be imported"))?;
        let name = safe_name(
            title["title"]
                .as_str()
                .ok_or_else(|| bad("title missing"))?,
        );
        let year = title["year"]
            .as_str()
            .filter(|s| s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit()))
            .map(|s| format!(" ({s})"))
            .unwrap_or_default();
        let mut coverage = mapping.episodes.clone();
        coverage.sort();
        coverage.dedup();
        let relative = if kind == "movies" {
            if !coverage.is_empty() {
                return Err(bad("movie mappings cannot contain episode numbers"));
            }
            PathBuf::from(format!("{name}{year}")).join(format!("{name}{year}.{extension}"))
        } else {
            validate_coverage(a, &title_id, &coverage).await?;
            let first = coverage
                .first()
                .ok_or_else(|| bad("episode mapping required"))?;
            let last = coverage
                .last()
                .ok_or_else(|| bad("episode mapping required"))?;
            let range = if coverage.len() > 1 {
                format!("-E{:02}", last.episode)
            } else {
                String::new()
            };
            PathBuf::from(format!("{name}{year}"))
                .join(format!("Season {:02}", first.season))
                .join(format!(
                    "{name} - S{:02}E{:02}{range}.{extension}",
                    first.season, first.episode
                ))
        };
        let destination = root.join(relative);
        if tokio::fs::try_exists(&destination)
            .await
            .map_err(|_| bad("cannot inspect destination"))?
        {
            let ours:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acquisition_imports WHERE download_id=? AND file_index=? AND destination=?)")
                .bind(id).bind(mapping.file_index as i64).bind(destination.to_string_lossy().as_ref()).fetch_one(&a.pool).await.map_err(db)?;
            if !ours {
                return Err(bad(
                    "destination already exists; existing media will not be overwritten",
                ));
            }
        }
        result.push(File {
            file_index: mapping.file_index,
            source: source.to_string_lossy().into_owned(),
            destination: destination.to_string_lossy().into_owned(),
            size: expected,
            coverage,
            title_id: title_id.clone(),
            root_device: device,
            root: root.to_string_lossy().into_owned(),
            source_stamp: super::import_io::stamp(&metadata),
        });
    }
    Ok(result)
}

async fn validate_coverage(a: &App, title: &str, episodes: &[Episode]) -> Result<()> {
    if episodes.is_empty() || episodes.len() > 20 {
        return Err(bad("map 1 to 20 consecutive episodes per file"));
    }
    for pair in episodes.windows(2) {
        if pair[0].season != pair[1].season || pair[1].episode != pair[0].episode + 1 {
            return Err(bad(
                "multi-episode files require consecutive episodes in one season",
            ));
        }
    }
    for e in episodes {
        let known:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acquisition_episodes WHERE title_id=? AND season=? AND episode=?)").bind(title).bind(e.season).bind(e.episode).fetch_one(&a.pool).await.map_err(db)?;
        if !known {
            return Err(bad("episode not in provider catalog"));
        }
    }
    Ok(())
}
pub(super) fn safe_name(raw: &str) -> String {
    let name: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || "/\\:*?\"<>|".contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let trimmed = name.trim().trim_matches('.').trim();
    let result: String = trimmed.chars().take(120).collect();
    if result.is_empty() {
        "Untitled".into()
    } else {
        result
    }
}
pub(super) fn token(plan: &[File]) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(plan).map_err(bad)?)
    ))
}
