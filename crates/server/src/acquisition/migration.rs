use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Input {
    pub source: String,
    pub records: Value,
    #[serde(default)]
    pub users: BTreeMap<String, String>,
    #[serde(default)]
    pub tmdb_ids: BTreeMap<String, u64>,
    #[serde(default)]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}
#[derive(Serialize)]
struct Record {
    key: String,
    kind: String,
    provider_id: u64,
    title: String,
    owner: Option<String>,
    status: String,
    scope: scope::Scope,
    enabled: bool,
}
fn normalize(input: &Input) -> Result<Vec<Record>> {
    if !matches!(input.source.as_str(), "radarr" | "sonarr" | "seerr") {
        return Err(bad("supported exports: radarr, sonarr, seerr"));
    }
    let rows = input
        .records
        .as_array()
        .or_else(|| input.records["results"].as_array())
        .ok_or_else(|| bad("export must be an array or a Seerr results object"))?;
    if rows.is_empty()
        || rows.len() > 1000
        || input.users.len() > 1000
        || input.tmdb_ids.len() > 1000
    {
        return Err(bad("migration requires 1 to 1000 records"));
    }
    let mut records = Vec::new();
    for row in rows {
        let external = row["id"]
            .as_u64()
            .ok_or_else(|| bad("export record needs a numeric id"))?
            .to_string();
        let request = input.source == "seerr";
        if request && !matches!(row["type"].as_str(), Some("movie" | "tv")) {
            return Err(bad("request export has an unknown media type"));
        }
        let kind = if input.source == "sonarr" || (request && row["type"] == "tv") {
            "tv"
        } else {
            "movie"
        };
        let id = input
            .tmdb_ids
            .get(&external)
            .copied()
            .or_else(|| {
                if request {
                    row["media"]["tmdbId"].as_u64()
                } else {
                    row["tmdbId"].as_u64()
                }
            })
            .filter(|id| *id > 0)
            .ok_or_else(|| bad(format!("record {external} needs an explicit TMDB mapping")))?;
        title_key(kind, id)?;
        let mut scope = scope::Scope::default();
        if kind == "tv" {
            let seasons: Vec<u32> = row["seasons"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|s| request || s["monitored"] == true)
                .filter_map(|s| {
                    s["seasonNumber"]
                        .as_u64()
                        .filter(|n| *n <= 999)
                        .map(|n| n as u32)
                })
                .collect();
            if !seasons.is_empty() {
                scope.mode = "seasons".into();
                scope.seasons = seasons;
            }
        }
        let owner = if request {
            let old = row["requestedBy"]["id"]
                .as_u64()
                .ok_or_else(|| bad("request has no owner id"))?
                .to_string();
            Some(
                input
                    .users
                    .get(&old)
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .ok_or_else(|| {
                        bad(format!("request owner {old} needs an Oxevu user mapping"))
                    })?,
            )
        } else {
            None
        };
        let status = if request {
            match row["status"].as_u64() {
                Some(1) => "pending",
                Some(2) => "approved",
                Some(3) => "declined",
                _ => return Err(bad("unknown request approval status")),
            }
        } else {
            "approved"
        };
        records.push(Record {
            key: format!("{}:{external}", input.source),
            kind: kind.into(),
            provider_id: id,
            title: row["title"]
                .as_str()
                .unwrap_or("Provider title resolved on import")
                .chars()
                .take(500)
                .collect(),
            owner,
            status: status.into(),
            scope,
            enabled: row["monitored"] == true,
        });
    }
    Ok(records)
}
fn token(input: &Input) -> Result<String> {
    // The confirmation binds the exact export and every mapping, never a live URL.
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(&json!({"source":input.source,"records":input.records,"users":input.users,"tmdb_ids":input.tmdb_ids,"profile_id":input.profile_id})).map_err(bad)?)))
}
pub(super) async fn preview(
    State(a): State<App>,
    h: HeaderMap,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let records = normalize(&input)?;
    validate_users(&a, &records).await?;
    profiles::load(&a, input.profile_id.as_deref().unwrap_or("default")).await?;
    Ok(Json(
        json!({"records":records,"token":token(&input)?,"count":records.len(),"automatic":false,
        "notice":"Existing monitors are preserved. New monitors have no destination and automatic acquisition stays off. Requests keep their mapped owner; availability is recalculated from Oxevu."}),
    ))
}
async fn validate_users(a: &App, records: &[Record]) -> Result<()> {
    for record in records {
        if a.acquisition.is_stopping() {
            break;
        }
        if let Some(owner) = &record.owner {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=?)")
                .bind(owner)
                .fetch_one(&a.pool)
                .await
                .map_err(db)?;
            if !exists {
                return Err(bad("mapped request owner does not exist"));
            }
        }
    }
    Ok(())
}
pub(super) async fn apply(
    State(a): State<App>,
    h: HeaderMap,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    if input.token.as_deref() != Some(token(&input)?.as_str()) {
        return Err(bad(
            "preview the exact export and mappings before importing",
        ));
    }
    let records = normalize(&input)?;
    validate_users(&a, &records).await?;
    let profile = input.profile_id.as_deref().unwrap_or("default");
    profiles::load(&a, profile).await?;
    let mut results = Vec::new();
    // Catalog I/O happens outside SQL transactions. Completed rows are idempotent;
    // interrupted batches can be submitted again without repeating previous rows.
    for record in records {
        match import_record(&a, &record, profile).await {
            Ok(id) => results.push(json!({"key":record.key,"title_id":id,"imported":true})),
            Err(e) => {
                results.push(json!({"key":record.key,"imported":false,"error":e.1.0["error"]}))
            }
        }
    }
    event(
        &a,
        "migration",
        "Migration batch reviewed; automatic acquisition remains off for imported titles",
    )
    .await?;
    Ok(Json(json!({"results":results})))
}
async fn import_record(a: &App, r: &Record, profile: &str) -> Result<String> {
    let title = catalog::ensure_title(a, &r.kind, r.provider_id).await?;
    let id = title["id"]
        .as_str()
        .ok_or_else(|| bad("provider title missing"))?;
    let mut scope = r.scope.clone();
    scope.normalize(&r.kind, "1970-01-01")?;
    let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
    let done: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM acquisition_migrations WHERE source_key=?)",
    )
    .bind(&r.key)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    if done {
        return Ok(id.into());
    }
    if let Some(owner) = &r.owner {
        sqlx::query("INSERT OR IGNORE INTO acquisition_requests(id,title_id,user_id,scope,status,note) VALUES(?,?,?,?,?,?)")
            .bind(uuid::Uuid::new_v4().to_string()).bind(id).bind(owner).bind(serde_json::to_string(&scope).map_err(bad)?).bind(&r.status).bind(format!("Imported from {}",r.key)).execute(&mut *tx).await.map_err(db)?;
    } else {
        sqlx::query("INSERT OR IGNORE INTO acquisition_monitors(title_id,enabled,scope,profile_id,automatic) VALUES(?,?,?,?,0)")
            .bind(id).bind(r.enabled).bind(serde_json::to_string(&scope).map_err(bad)?).bind(profile).execute(&mut *tx).await.map_err(db)?;
    }
    sqlx::query("INSERT INTO acquisition_migrations VALUES(?,?,unixepoch())")
        .bind(&r.key)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(id.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn export_preview_binds_mappings_preserves_owners_and_replays_once() {
        let a = crate::tests::app().await;
        let admin = crate::tests::account(&a, "admin", true).await;
        let viewer = crate::tests::account(&a, "viewer", false).await;
        super::super::tests::title(&a, "movie", 123).await;
        let mut input = Input {
            source: "seerr".into(),
            records: json!({"results":[{"id":9,"type":"movie","media":{"tmdbId":123},"requestedBy":{"id":4},"status":2}]}),
            users: BTreeMap::from([("4".into(), "viewer".into())]),
            tmdb_ids: BTreeMap::new(),
            profile_id: None,
            token: None,
        };
        assert_eq!(
            preview(
                State(a.clone()),
                viewer,
                Json(serde_json::from_value(serde_json::to_value(&input).unwrap()).unwrap())
            )
            .await
            .unwrap_err()
            .0,
            StatusCode::FORBIDDEN
        );
        input.token = Some(token(&input).unwrap());
        for _ in 0..2 {
            let _ = apply(
                State(a.clone()),
                admin.clone(),
                Json(serde_json::from_value(serde_json::to_value(&input).unwrap()).unwrap()),
            )
            .await
            .unwrap();
        }
        let rows = sqlx::query("SELECT user_id,status FROM acquisition_requests")
            .fetch_all(&a.pool)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<String, _>("user_id"), "viewer");
        assert_eq!(rows[0].get::<String, _>("status"), "approved");
        input.users.insert("4".into(), "admin".into());
        assert!(apply(State(a.clone()), admin, Json(input)).await.is_err());
    }
    #[test]
    fn sonarr_requires_a_mapping_when_tmdb_is_absent_and_preserves_seasons() {
        let mut input = Input {
            source: "sonarr".into(),
            records: json!([{"id":1,"title":"Fixture","tvdbId":222,"monitored":true,"seasons":[{"seasonNumber":1,"monitored":true},{"seasonNumber":2,"monitored":false}]}]),
            users: BTreeMap::new(),
            tmdb_ids: BTreeMap::new(),
            profile_id: None,
            token: None,
        };
        assert!(normalize(&input).is_err());
        input.tmdb_ids.insert("1".into(), 123);
        let rows = normalize(&input).unwrap();
        assert_eq!(rows[0].scope.seasons, vec![1]);
        assert!(rows[0].enabled);
    }
}
