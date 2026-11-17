//! Conservative automatic identification. Ambiguous matches require user input.
use super::{
    Result,
    identification::{self, MetadataSource, Selection},
    invalid as bad,
};
use oxevu_db::metadata_jobs::{self as repository, Pending};
use serde_json::Value;
use sqlx::SqlitePool;
use std::path::Path;
#[cfg(test)]
#[path = "automatic_metadata_tests.rs"]
mod tests;

fn normalized(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn title(item: &Value, _path: &Path) -> (String, Option<String>) {
    let raw = if item["kind"] == "episode" {
        item["show_title"].as_str()
    } else {
        item["title"].as_str()
    }
    .or_else(|| item["title"].as_str())
    .unwrap_or("");
    let raw = raw.replace(['.', '_'], " ");
    let words: Vec<_> = raw.split_whitespace().collect();
    let year_at = words.iter().rposition(|word| {
        let word = word.trim_matches(['(', ')', '[', ']']);
        word.len() == 4
            && word
                .parse::<u32>()
                .is_ok_and(|year| (1888..=2100).contains(&year))
    });
    let year = year_at.map(|i| words[i].trim_matches(['(', ')', '[', ']']).to_owned());
    let title = words[..year_at.unwrap_or(words.len())]
        .join(" ")
        .trim_end_matches([' ', '-', '('])
        .to_owned();
    (title, year)
}

pub async fn enrich(
    pool: &SqlitePool,
    job: &Pending,
    provider: &dyn MetadataSource,
    tmdb: bool,
    omdb: bool,
) -> Result<()> {
    let item: Value =
        serde_json::from_str(&job.data).map_err(|_| bad("invalid catalog metadata"))?;
    let path = Path::new(&job.path);
    let root = super::roots::item_root(pool, &job.id, path).await?;
    let tv = item["kind"] == "episode";
    if let Some(data) = super::sidecars::metadata(path, &root, tv).await {
        repository::publish_unmatched(pool, &job.id, &data.to_string()).await?;
        return Ok(());
    }
    let (title, year) = title(&item, path);
    if title.is_empty() {
        return Err(bad("Cannot infer a title; use Identify"));
    }
    let mut failure = "No unique matching title found; use Identify".to_owned();
    for name in ["tmdb", "omdb"] {
        if (name == "tmdb" && !tmdb) || (name == "omdb" && !omdb) {
            continue;
        }
        let selection = async {
            let data = if name == "tmdb" {
                let mut query = vec![("query", title.clone()), ("include_adult", "false".into())];
                if let Some(year) = &year {
                    query.push((
                        if tv { "first_air_date_year" } else { "year" },
                        year.clone(),
                    ));
                }
                provider
                    .tmdb(if tv { "search/tv" } else { "search/movie" }, &query)
                    .await?
            } else {
                let mut query = vec![
                    ("s", title.clone()),
                    ("type", if tv { "series" } else { "movie" }.into()),
                ];
                if let Some(year) = &year {
                    query.push(("y", year.clone()));
                }
                provider.omdb(&query).await?
            };
            let matches: Vec<_> = data[if name == "tmdb" { "results" } else { "Search" }]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|v| {
                    let candidate = v[if name == "tmdb" {
                        if tv { "name" } else { "title" }
                    } else {
                        "Title"
                    }]
                    .as_str()
                    .unwrap_or("");
                    let date = v[if name == "tmdb" {
                        if tv { "first_air_date" } else { "release_date" }
                    } else {
                        "Year"
                    }]
                    .as_str()
                    .unwrap_or("");
                    normalized(candidate) == normalized(&title)
                        && year.as_ref().is_none_or(|y| date.starts_with(y))
                })
                .collect();
            if matches.len() != 1 {
                return Err(bad("No unique matching title found; use Identify"));
            }
            let id = if name == "tmdb" {
                matches[0]["id"].as_u64().map(|n| n.to_string())
            } else {
                matches[0]["imdbID"].as_str().map(str::to_owned)
            }
            .ok_or_else(|| bad("Provider returned no title ID"))?;
            identification::apply_automatic(
                pool,
                &job.id,
                item.clone(),
                Selection {
                    provider: name.into(),
                    id,
                },
                provider,
            )
            .await
        }
        .await;
        match selection {
            Ok(()) => return Ok(()),
            Err(error) => failure = error.to_string(),
        }
    }
    if !tmdb && !omdb {
        failure =
            "No local metadata found; configure TMDB or OMDb, then identify this title".into();
    }
    Err(bad(failure))
}
