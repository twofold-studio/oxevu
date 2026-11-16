use super::*;
use scope::Scope;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Input {
    pub enabled: bool,
    pub scope: Scope,
    pub library_id: Option<String>,
    pub root: Option<String>,
    pub profile_id: String,
    pub automatic: bool,
}
pub(super) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "catalog").await?;
    let rows=sqlx::query("SELECT t.id,t.data,m.enabled,m.scope,m.library_id,m.root,m.profile_id,m.automatic,m.next_search,m.failures FROM acquisition_titles t LEFT JOIN acquisition_monitors m ON m.title_id=t.id WHERE m.title_id IS NOT NULL OR EXISTS(SELECT 1 FROM acquisition_requests r WHERE r.title_id=t.id AND r.status IN ('approved','fulfilled')) ORDER BY t.refreshed_at DESC LIMIT 1000").fetch_all(&a.pool).await.map_err(db)?;
    let mut titles = Vec::new();
    for r in rows {
        let title = decode(r.get("data"))?;
        let items = wanted::inventory(&a, &title).await?;
        let units = wanted::units(&a, &title, &items).await?;
        titles.push(json!({"title":title,"units":units,"monitor":{"enabled":r.get::<Option<bool>,_>("enabled").unwrap_or(true),"scope":r.get::<Option<String>,_>("scope").map(|s|decode(&s)).transpose()?.unwrap_or(json!({"mode":"requests","seasons":[],"episodes":[],"cutoff":null})),"library_id":r.get::<Option<String>,_>("library_id"),"root":r.get::<Option<String>,_>("root"),"profile_id":r.get::<Option<String>,_>("profile_id").unwrap_or("default".into()),"automatic":r.get::<Option<bool>,_>("automatic").unwrap_or(false),"next_search":r.get::<Option<i64>,_>("next_search"),"failures":r.get::<Option<i64>,_>("failures")}}));
    }
    Ok(Json(json!({"titles":titles})))
}

pub(super) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(mut v): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let kind: Option<String> = sqlx::query_scalar("SELECT kind FROM acquisition_titles WHERE id=?")
        .bind(&id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    let kind = kind.ok_or_else(|| bad("unknown title"))?;
    if v.scope.mode == "requests" {
        v.scope.seasons.clear();
        v.scope.episodes.clear();
        v.scope.cutoff = None;
    } else {
        let previous: Option<String> =
            sqlx::query_scalar("SELECT scope FROM acquisition_monitors WHERE title_id=?")
                .bind(&id)
                .fetch_optional(&a.pool)
                .await
                .map_err(db)?;
        let previous = previous
            .map(|raw| serde_json::from_str::<Scope>(&raw))
            .transpose()
            .map_err(bad)?;
        let mut today: String = sqlx::query_scalar("SELECT date('now')")
            .fetch_one(&a.pool)
            .await
            .map_err(db)?;
        if let Some(old) = previous.filter(|old| old.mode == v.scope.mode)
            && let Some(cutoff) = old.cutoff
        {
            today = cutoff;
        }
        v.scope.normalize(&kind, &today)?;
    }
    profiles::load(&a, &v.profile_id).await?;
    let mut device = None;
    if let (Some(library), Some(root)) = (&v.library_id, &v.root) {
        let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM libraries l JOIN library_roots r ON r.library_id=l.id WHERE l.id=? AND r.path=? AND l.kind=?)")
            .bind(library).bind(root).bind(if kind=="tv"{"tv"}else{"movies"}).fetch_one(&a.pool).await.map_err(db)?;
        if !valid {
            return Err(bad("select a library root matching the title type"));
        }
        use std::os::unix::fs::MetadataExt;
        let meta = tokio::fs::metadata(root)
            .await
            .map_err(|_| bad("library root is offline"))?;
        if !meta.is_dir() {
            return Err(bad("library root is not a directory"));
        }
        device = Some(meta.dev() as i64);
    } else if v.library_id.is_some() || v.root.is_some() || v.automatic {
        return Err(bad(
            "choose a library and root before enabling automatic acquisition",
        ));
    }
    sqlx::query("INSERT INTO acquisition_monitors(title_id,enabled,scope,library_id,root,profile_id,automatic,root_device) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(title_id) DO UPDATE SET enabled=excluded.enabled,scope=excluded.scope,library_id=excluded.library_id,root=excluded.root,profile_id=excluded.profile_id,automatic=excluded.automatic,root_device=excluded.root_device,next_search=0,failures=0")
        .bind(&id).bind(v.enabled).bind(serde_json::to_string(&v.scope).map_err(bad)?).bind(&v.library_id).bind(&v.root).bind(&v.profile_id).bind(v.automatic).bind(device).execute(&a.pool).await.map_err(db)?;
    Ok(Json(json!({"monitor":v})))
}

pub(crate) async fn run(a: App) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        timer.tick().await;
        if a.acquisition.is_stopping() {
            break;
        }
        if let Err(e) = tick(&a).await {
            let _ = event(
                &a,
                "monitoring",
                e.1.0["error"].as_str().unwrap_or("monitoring failed"),
            )
            .await;
        }
    }
}
pub(super) async fn tick(a: &App) -> Result<()> {
    wanted::fulfill(a).await?;
    let c = config::load(a).await?;
    if !c.enabled {
        return Ok(());
    }
    feeds::tick(a).await?;
    let rows=sqlx::query("SELECT m.*,t.kind,t.provider_id,t.refreshed_at,t.data title FROM acquisition_monitors m JOIN acquisition_titles t ON t.id=m.title_id WHERE m.enabled=1 AND m.automatic=1 AND m.next_search<=unixepoch() ORDER BY m.next_search LIMIT 5").fetch_all(&a.pool).await.map_err(db)?;
    for r in rows {
        if a.acquisition.is_stopping() {
            break;
        }
        let id: String = r.get("title_id");
        // Reserve the search before external I/O; crashes retry after the interval.
        sqlx::query("UPDATE acquisition_monitors SET next_search=unixepoch()+? WHERE title_id=?")
            .bind(c.search_interval_minutes * 60)
            .bind(&id)
            .execute(&a.pool)
            .await
            .map_err(db)?;
        let result = search_missing(a, &r).await;
        if let Err(e) = result {
            sqlx::query("UPDATE acquisition_monitors SET failures=failures+1,next_search=unixepoch()+MIN(86400,300*(failures+1)*(failures+1)) WHERE title_id=?").bind(&id).execute(&a.pool).await.map_err(db)?;
            event(
                a,
                &id,
                e.1.0["error"].as_str().unwrap_or("scheduled search failed"),
            )
            .await?;
        } else {
            sqlx::query("UPDATE acquisition_monitors SET failures=0 WHERE title_id=?")
                .bind(&id)
                .execute(&a.pool)
                .await
                .map_err(db)?;
        }
    }
    Ok(())
}

async fn search_missing(a: &App, r: &sqlx::sqlite::SqliteRow) -> Result<()> {
    let id: String = r.get("title_id");
    let root = r
        .get::<Option<String>, _>("root")
        .ok_or_else(|| bad("automatic title has no destination"))?;
    use std::os::unix::fs::MetadataExt;
    let meta = tokio::fs::metadata(root)
        .await
        .map_err(|_| bad("library root offline; acquisition suspended"))?;
    if Some(meta.dev() as i64) != r.get::<Option<i64>, _>("root_device") {
        return Err(bad("library mount changed; acquisition suspended"));
    }
    let enabled: bool = sqlx::query_scalar(
        "SELECT COALESCE((SELECT enabled FROM library_options WHERE library_id=?),1)",
    )
    .bind(r.get::<Option<String>, _>("library_id"))
    .fetch_one(&a.pool)
    .await
    .map_err(db)?;
    if !enabled {
        return Err(bad("destination library disabled"));
    }
    let now: i64 = sqlx::query_scalar("SELECT unixepoch()")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let title =
        if r.get::<String, _>("kind") == "tv" && now - r.get::<i64, _>("refreshed_at") > 86400 {
            catalog::refresh(a, "tv", r.get::<i64, _>("provider_id") as u64).await?
        } else {
            decode(r.get("title"))?
        };
    let items = wanted::inventory(a, &title).await?;
    if items.iter().any(|i| i["online"] != true) {
        return Err(bad(
            "existing library files offline or unverified; acquisition suspended",
        ));
    }
    let today: String = sqlx::query_scalar("SELECT date('now')")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let units = claims::unclaimed(a, &id, &wanted::units(a, &title, &items).await?).await?;
    let Some(unit) = units.iter().find(|u| {
        u["wanted"] == true
            && u["available"] != true
            && u["air_date"]
                .as_str()
                .is_some_and(|date| !date.is_empty() && date <= today.as_str())
    }) else {
        return Ok(());
    };
    let q = releases::Search {
        indexer_ids: None,
        season: unit["season"].as_u64().map(|n| n as u32),
        episode: unit["episode"].as_u64().map(|n| n as u32),
        profile_id: r.get("profile_id"),
        offset: 0,
    };
    let result = releases::search_for(a, &id, &q).await?;
    if let Some(candidate) = result["releases"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|v| {
            v["accepted"] == true
                && v["identity_match"] == true
                && v["blocked"] != true
                && (q.season.is_none()
                    || (v["season"] == json!(q.season)
                        && (v["episode"] == json!(q.episode) || v["episode"].is_null())))
        })
    {
        let input = downloads::Input {
            release_id: candidate["id"].as_str().map(str::to_owned),
            profile_id: q.profile_id,
            only_wanted: true,
            ..Default::default()
        };
        downloads::enqueue(a, input).await?;
        event(a, &id, "Scheduled search queued a missing title release").await?;
    }
    Ok(())
}
