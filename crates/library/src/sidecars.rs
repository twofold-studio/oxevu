//! Read local metadata without following sidecars outside the configured library.
use serde_json::{Value, json};
use std::path::Path;
use tokio::io::AsyncReadExt;

async fn read(path: &Path, root: &Path) -> Option<Value> {
    let path = tokio::fs::canonicalize(path).await.ok()?;
    if !path.starts_with(root) {
        return None;
    }
    let file = tokio::fs::File::open(path).await.ok()?;
    if !file.metadata().await.ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .await
        .ok()?;
    oxevu_media::nfo::parse(&bytes).ok()
}

pub async fn metadata(path: &Path, root: &Path, tv: bool) -> Option<Value> {
    let root = tokio::fs::canonicalize(root).await.ok()?;
    let path = tokio::fs::canonicalize(path).await.ok()?;
    if !path.starts_with(&root) {
        return None;
    }
    let parent = path.parent()?;
    let mut result = json!({});
    if tv {
        for ancestor in parent.ancestors().take_while(|p| p.starts_with(&root)) {
            if let Some(show) = read(&ancestor.join("tvshow.nfo"), &root).await {
                for field in [
                    "overview",
                    "year",
                    "rating",
                    "rating_source",
                    "genres",
                    "studios",
                    "creators",
                    "content_rating",
                ] {
                    if !show[field].is_null() {
                        result[format!("show_{field}")] = show[field].clone();
                    }
                }
                result["provider_ids"] = show["provider_ids"].clone();
                break;
            }
        }
        if let Some(season) = read(&parent.join("season.nfo"), &root).await {
            result["season_overview"] = season["overview"].clone();
        }
    }
    let local = match read(&path.with_extension("nfo"), &root).await {
        Some(local) => Some(local),
        None if !tv && parent != root => read(&parent.join("movie.nfo"), &root).await,
        None => None,
    };
    if let Some(mut local) = local {
        // Episode IDs identify the episode, not its parent series.
        if tv {
            local.as_object_mut()?.remove("provider_ids");
        }
        result.as_object_mut()?.extend(local.as_object()?.clone());
    }
    if !result[if tv { "show_overview" } else { "overview" }]
        .as_str()
        .is_some_and(|s| !s.trim().is_empty())
    {
        return None;
    }
    result["metadata_source"] = json!("nfo");
    let mut links = json!({});
    if let Some(id) = result["provider_ids"]["tmdb"].as_u64() {
        links["tmdb"] = json!(format!(
            "https://www.themoviedb.org/{}/{id}",
            if tv { "tv" } else { "movie" }
        ));
    }
    if let Some(id) = result["provider_ids"]["imdb"].as_str() {
        links["imdb"] = json!(format!("https://www.imdb.com/title/{id}/"));
    }
    result["external_urls"] = links;
    Some(result)
}
