use super::{Error, Result, invalid as bad};
use oxevu_db::identification as repository;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::{future::Future, pin::Pin};
#[cfg(test)]
#[path = "identification_tests.rs"]
mod tests;
pub type MetadataFuture<'a> = Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>>;
pub trait MetadataSource: Sync {
    fn tmdb<'a>(&'a self, route: &'a str, query: &'a [(&'a str, String)]) -> MetadataFuture<'a>;
    fn omdb<'a>(&'a self, query: &'a [(&'a str, String)]) -> MetadataFuture<'a>;
}
#[derive(Deserialize)]
pub struct Selection {
    provider: String,
    id: String,
}
fn strings(v: &Value, key: &str) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v[key].as_str().map(str::to_owned))
        .collect()
}
fn csv(v: &Value) -> Vec<String> {
    v.as_str()
        .filter(|s| *s != "N/A")
        .map(|s| s.split(", ").map(str::to_owned).collect())
        .unwrap_or_default()
}
fn date_year(v: &Value) -> Option<u32> {
    v.as_str()?.get(..4)?.parse().ok()
}
fn rating(v: &Value) -> Option<f64> {
    v.as_str()?.parse().ok()
}
pub fn image(v: &Value) -> Option<String> {
    v.as_str()
        .filter(|p| p.starts_with('/') && !p.contains("..") && p.len() < 200)
        .map(str::to_owned)
}
pub fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
pub async fn apply(
    pool: &SqlitePool,
    id: &str,
    item: Value,
    m: Selection,
    provider: &dyn MetadataSource,
) -> Result<()> {
    let tv = item["kind"] == "episode";
    let mut enrichment = match m.provider.as_str() {
        "tmdb" => {
            let number = m.id.parse::<u64>().map_err(|_| bad("invalid TMDB id"))?;
            let data = provider
                .tmdb(
                    &format!("{}/{number}", if tv { "tv" } else { "movie" }),
                    &[(
                        "append_to_response",
                        "credits,external_ids,release_dates,content_ratings".into(),
                    )],
                )
                .await?;
            let imdb = data["imdb_id"]
                .as_str()
                .or_else(|| data["external_ids"]["imdb_id"].as_str());
            let creators = if tv {
                strings(&data["created_by"], "name")
            } else {
                data["credits"]["crew"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|v| v["job"] == "Director")
                    .filter_map(|v| v["name"].as_str().map(str::to_owned))
                    .collect()
            };
            json!({"title":if tv{&data["name"]}else{&data["title"]},"overview":data["overview"],"year":date_year(if tv{&data["first_air_date"]}else{&data["release_date"]}),"end_year":date_year(&data["last_air_date"]),"rating":data["vote_average"],"genres":strings(&data["genres"],"name"),"studios":strings(&data["production_companies"],"name"),"creators":creators,"provider_ids":{"tmdb":number,"imdb":imdb},"external_urls":{"tmdb":format!("https://www.themoviedb.org/{}/{number}",if tv{"tv"}else{"movie"}),"imdb":imdb.map(|s|format!("https://www.imdb.com/title/{s}/"))},"metadata_poster":image(&data["poster_path"]),"metadata_backdrop":image(&data["backdrop_path"]),"metadata_attribution":"This product uses the TMDB API but is not endorsed or certified by TMDB."})
        }
        "omdb" => {
            if !m.id.starts_with("tt")
                || !m.id[2..].bytes().all(|b| b.is_ascii_digit())
                || m.id.len() > 20
            {
                return Err(bad("invalid IMDb id"));
            }
            let data = provider
                .omdb(&[("i", m.id.clone()), ("plot", "full".into())])
                .await?;
            json!({"title":data["Title"],"overview":data["Plot"],"year":date_year(&data["Year"]),"rating":rating(&data["imdbRating"]),"content_rating":data["Rated"],"genres":csv(&data["Genre"]),"studios":csv(&data["Production"]),"creators":csv(&data["Director"]),"provider_ids":{"imdb":m.id},"external_urls":{"imdb":format!("https://www.imdb.com/title/{}/",m.id)}})
        }
        _ => return Err(bad("unsupported metadata provider")),
    };
    enrichment["rating_source"] = json!(if m.provider == "tmdb" { "TMDB" } else { "IMDb" });
    if tv {
        for field in [
            "overview",
            "year",
            "end_year",
            "rating",
            "rating_source",
            "genres",
            "studios",
            "creators",
            "content_rating",
        ] {
            enrichment[format!("show_{field}")] = enrichment[field].clone();
        }
    }
    let rows = repository::candidates(
        pool,
        id,
        tv.then(|| (item["library_id"].as_str(), item["show_title"].as_str())),
    )
    .await
    .map_err(Error::Storage)?;
    if rows.len() > 100 {
        return Err(bad("series metadata limit is 100 episodes per match"));
    }
    let mut updates = Vec::new();
    let mut seasons = std::collections::HashMap::<u64, Value>::new();
    for row in rows {
        let source: Value =
            serde_json::from_str(&row.data).map_err(|_| bad("invalid item metadata"))?;
        if tv && source["show_title"] != item["show_title"] {
            continue;
        }
        if updates.len() >= 100 {
            return Err(bad("series metadata limit is 100 episodes per match"));
        }
        let mut data = enrichment.clone();
        if tv {
            data["rating"] = Value::Null;
            data["rating_source"] = Value::Null;
            data["show_title"] = source["show_title"].clone();
            data["title"] = source["title"].clone();
            if m.provider == "tmdb"
                && let (Some(season), Some(episode)) = (
                    source["season_number"].as_u64(),
                    source["episode_number"].as_u64(),
                )
            {
                if let std::collections::hash_map::Entry::Vacant(entry) = seasons.entry(season) {
                    entry.insert(
                        provider
                            .tmdb(&format!("tv/{}/season/{season}", m.id), &[])
                            .await?,
                    );
                }
                if let Some(details) = seasons.get(&season) {
                    data["season_overview"] = details["overview"].clone();
                    data["season_rating"] = details["vote_average"].clone();
                }
                let episode = provider
                    .tmdb(
                        &format!("tv/{}/season/{season}/episode/{episode}", m.id),
                        &[],
                    )
                    .await?;
                data["title"] = episode["name"].clone();
                data["overview"] = episode["overview"].clone();
                data["rating"] = episode["vote_average"].clone();
                data["rating_source"] = json!("TMDB");
                data["metadata_thumbnail"] = json!(image(&episode["still_path"]));
            }
        }
        let item_id = row.id;
        for (field, kind) in [
            ("metadata_poster", "poster"),
            ("metadata_backdrop", "backdrop"),
            ("metadata_thumbnail", "thumbnail"),
        ] {
            if data[field].is_string() {
                data[format!("{kind}_url")] =
                    json!(format!("/api/v1/items/{item_id}/metadata/artwork/{kind}"));
            }
        }
        data["studio_images"]=json!(data["studios"].as_array().into_iter().flatten().filter_map(|s|s.as_str()).map(|name|json!({"name":name,"url":format!("/api/v1/items/{item_id}/metadata/studios/{}",slug(name))})).collect::<Vec<_>>());
        updates.push((item_id, data.to_string()));
    }
    repository::publish(pool, &updates)
        .await
        .map_err(Error::Storage)?;
    Ok(())
}
