use super::Error::Storage as db;
use super::{Result, invalid as bad};
use oxevu_db::catalog::{self as repository, Snapshot};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};

/// Merge authorized records with metadata and per-user state, preserving local artwork.
pub fn project(snapshot: Snapshot, visible: &HashSet<String>) -> Result<Vec<Value>> {
    let metadata: HashMap<String, Value> = snapshot
        .metadata
        .into_iter()
        .map(|(id, data)| {
            Ok((
                id,
                serde_json::from_str(&data).map_err(|_| bad("invalid matched metadata"))?,
            ))
        })
        .collect::<Result<_>>()?;
    snapshot
        .items
        .into_iter()
        .filter(|item| visible.contains(&item.id))
        .map(|item| {
            let mut value: Value = serde_json::from_str(&item.data).map_err(bad)?;
            if let Some(enriched) = metadata.get(&item.id).and_then(Value::as_object)
                && let Some(object) = value.as_object_mut()
            {
                let poster = object
                    .get("poster_url")
                    .filter(|v| {
                        v.as_str()
                            .is_some_and(|url| url.contains("/artwork/poster"))
                    })
                    .cloned();
                object.extend(enriched.clone());
                if let Some(poster) = poster {
                    object.insert("poster_url".into(), poster);
                }
            }
            for kind in ["poster", "thumbnail", "backdrop"] {
                value
                    .as_object_mut()
                    .ok_or_else(|| bad("invalid catalog item"))?
                    .entry(format!("{kind}_url"))
                    .or_insert(Value::Null);
            }
            if value["poster_url"].as_str().is_none_or(str::is_empty)
                && value["rating_source"] == "IMDb"
                && value["provider_ids"]["imdb"].as_str().is_some()
            {
                value["poster_url"] =
                    json!(format!("/api/v1/items/{}/metadata/artwork/poster", item.id));
            }
            value["position_seconds"] = json!(item.position);
            value["completed"] = json!(item.completed);
            value["favorite"] = json!(snapshot.favorites.contains(&item.id));
            value["reaction"] = json!(snapshot.reactions.get(&item.id));
            Ok(value)
        })
        .collect()
}

pub async fn favorite(pool: &SqlitePool, user: &str, item: &str, body: &Value) -> Result<()> {
    let enabled = body["favorite"]
        .as_bool()
        .ok_or_else(|| bad("favorite must be boolean"))?;
    repository::favorite(pool, user, item, enabled)
        .await
        .map_err(db)
}

pub async fn progress(
    pool: &SqlitePool,
    user: &str,
    item: &str,
    body: &Value,
    settings: &oxevu_config::Settings,
) -> Result<()> {
    let position = body["position_seconds"]
        .as_f64()
        .filter(|p| p.is_finite() && *p >= 0.)
        .ok_or_else(|| bad("invalid position"))?;
    let completed = body["completed"]
        .as_bool()
        .ok_or_else(|| bad("completed required"))?;
    let raw = repository::item_data(pool, item)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("unknown item"))?;
    let item_data: Value = serde_json::from_str(&raw).map_err(bad)?;
    let (position, completed) = resume(
        item_data["duration_seconds"].as_f64(),
        position,
        completed,
        settings,
    );
    repository::progress(pool, user, item, position, completed)
        .await
        .map_err(db)
}

fn resume(
    duration: Option<f64>,
    mut position: f64,
    mut completed: bool,
    settings: &oxevu_config::Settings,
) -> (f64, bool) {
    if let Some(duration) = duration.filter(|d| *d > 0.) {
        let percent = position / duration * 100.;
        completed = completed || percent >= settings.resume_max_percent;
        if duration < f64::from(settings.resume_min_duration_seconds)
            || percent < settings.resume_min_percent
            || completed
        {
            position = 0.;
        }
    }
    (position, completed)
}
