use super::*;
use scope::{Episode, Scope};

pub(super) async fn inventory(a: &App, title: &Value) -> Result<Vec<Value>> {
    let rows=sqlx::query("SELECT i.id,i.library_id,i.path,json_patch(i.data,COALESCE(m.data,'{}')) data FROM items i LEFT JOIN item_metadata m ON m.item_id=i.id WHERE CAST(COALESCE(json_extract(m.data,'$.provider_ids.tmdb'),json_extract(i.data,'$.provider_ids.tmdb')) AS INTEGER)=?")
        .bind(title["provider_id"].as_i64()).fetch_all(&a.pool).await.map_err(db)?;
    let mut items = Vec::new();
    for row in rows {
        let mut item = decode(row.get("data"))?;
        if !catalog::matches_item(title, &item) {
            continue;
        }
        item["library_id"] = json!(row.get::<String, _>("library_id"));
        item["id"] = json!(row.get::<String, _>("id"));
        item["online"] = json!(
            tokio::fs::metadata(row.get::<String, _>("path"))
                .await
                .is_ok()
        );
        items.push(item);
    }
    Ok(items)
}
pub(super) fn covers(item: &Value, season: u32, episode: u32) -> bool {
    (item["season_number"] == season && item["episode_number"] == episode)
        || item["episode_coverage"].as_array().is_some_and(|v| {
            v.iter()
                .any(|e| e["season"] == season && e["episode"] == episode)
        })
}
pub(super) async fn units(a: &App, title: &Value, items: &[Value]) -> Result<Vec<Value>> {
    let id = title["id"].as_str().ok_or_else(|| bad("invalid title"))?;
    let raw:Vec<String>=sqlx::query_scalar("SELECT scope FROM acquisition_requests WHERE title_id=? AND status IN ('approved','fulfilled')")
        .bind(id).fetch_all(&a.pool).await.map_err(db)?;
    let mut scopes = Vec::new();
    for raw in raw {
        scopes.push(serde_json::from_str::<Scope>(&raw).map_err(|_| bad("invalid wanted scope"))?);
    }
    if let Some(raw) = sqlx::query_scalar::<_, String>(
        "SELECT scope FROM acquisition_monitors WHERE title_id=? AND enabled=1",
    )
    .bind(id)
    .fetch_optional(&a.pool)
    .await
    .map_err(db)?
    {
        let scope: Scope =
            serde_json::from_str(&raw).map_err(|_| bad("invalid monitoring scope"))?;
        if scope.mode != "requests" {
            scopes.push(scope);
        }
    }
    if title["kind"] == "movie" {
        return Ok(vec![
            json!({"season":null,"episode":null,"name":title["title"],"air_date":title["release_date"],
            "wanted":!scopes.is_empty(),"available":items.iter().any(|i|i["online"]==true),"offline":items.iter().any(|i|i["online"]!=true),
            "item_id":items.iter().find(|i|i["online"]==true).map(|i|&i["id"])}),
        ]);
    }
    let rows=sqlx::query("SELECT season,episode,air_date,data FROM acquisition_episodes WHERE title_id=? ORDER BY season,episode").bind(id).fetch_all(&a.pool).await.map_err(db)?;
    let mut units = Vec::new();
    for r in rows {
        let season = r.get::<i64, _>("season") as u32;
        let episode = r.get::<i64, _>("episode") as u32;
        let date = r.get::<Option<String>, _>("air_date");
        let info = decode(r.get("data"))?;
        let available = items
            .iter()
            .find(|i| i["online"] == true && covers(i, season, episode));
        units.push(json!({"season":season,"episode":episode,"name":info["name"],"air_date":date,
            "wanted":scopes.iter().any(|s|s.covers(season,episode,date.as_deref())),"available":available.is_some(),
            "offline":items.iter().any(|i|i["online"]!=true && covers(i,season,episode)),"item_id":available.map(|i|&i["id"])}));
    }
    Ok(units)
}

pub(super) async fn fulfill(a: &App) -> Result<()> {
    let rows=sqlx::query("SELECT r.id,r.user_id,r.status,r.scope,t.data title FROM acquisition_requests r JOIN acquisition_titles t ON t.id=r.title_id WHERE r.status IN ('approved','fulfilled') LIMIT 1000").fetch_all(&a.pool).await.map_err(db)?;
    for r in rows {
        let title = decode(r.get("title"))?;
        let scope: Scope =
            serde_json::from_str(r.get("scope")).map_err(|_| bad("invalid request scope"))?;
        let u = users::profile(a, r.get("user_id")).await?;
        let items: Vec<Value> = inventory(a, &title)
            .await?
            .into_iter()
            .filter(|i| {
                u["disabled"] != true
                    && (u["is_admin"] == true
                        || u["all_libraries"] == true
                        || u["library_ids"]
                            .as_array()
                            .is_some_and(|v| v.contains(&i["library_id"])))
            })
            .collect();
        let complete = if title["kind"] == "movie" {
            items.iter().any(|i| i["online"] == true)
        } else {
            let rows = sqlx::query(
                "SELECT season,episode,air_date FROM acquisition_episodes WHERE title_id=?",
            )
            .bind(title["id"].as_str())
            .fetch_all(&a.pool)
            .await
            .map_err(db)?;
            let selected: Vec<Episode> = rows
                .iter()
                .filter(|r| {
                    scope.covers(
                        r.get::<i64, _>("season") as u32,
                        r.get::<i64, _>("episode") as u32,
                        r.get::<Option<String>, _>("air_date").as_deref(),
                    )
                })
                .map(|r| Episode {
                    season: r.get::<i64, _>("season") as u32,
                    episode: r.get::<i64, _>("episode") as u32,
                })
                .collect();
            !selected.is_empty()
                && selected.iter().all(|e| {
                    items
                        .iter()
                        .any(|i| i["online"] == true && covers(i, e.season, e.episode))
                })
        };
        let status = if complete { "fulfilled" } else { "approved" };
        if r.get::<String, _>("status") != status {
            sqlx::query("UPDATE acquisition_requests SET status=?,updated_at=unixepoch() WHERE id=? AND status IN ('approved','fulfilled')")
                .bind(status).bind(r.get::<String,_>("id")).execute(&a.pool).await.map_err(db)?;
            event(
                a,
                r.get("id"),
                if complete {
                    "Request is available in an accessible library"
                } else {
                    "Requested media is missing or inaccessible"
                },
            )
            .await?;
        }
    }
    Ok(())
}
