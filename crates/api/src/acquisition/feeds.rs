use super::*;

pub(super) async fn tick(a: &App) -> Result<()> {
    let titles=sqlx::query("SELECT m.*,t.data title FROM acquisition_monitors m JOIN acquisition_titles t ON t.id=m.title_id WHERE m.enabled=1 AND m.automatic=1 LIMIT 1000").fetch_all(&a.pool).await.map_err(db)?;
    if titles.is_empty() {
        return Ok(());
    }
    let indexers:Vec<String>=sqlx::query_scalar("SELECT i.id FROM acquisition_indexers i LEFT JOIN acquisition_feeds f ON f.indexer_id=i.id WHERE json_extract(i.data,'$.enabled')=1 AND COALESCE(f.next_poll,0)<=unixepoch() LIMIT 3").fetch_all(&a.pool).await.map_err(db)?;
    for id in indexers {
        sqlx::query("INSERT INTO acquisition_feeds VALUES(?,unixepoch()+900) ON CONFLICT(indexer_id) DO UPDATE SET next_poll=excluded.next_poll").bind(&id).execute(&a.pool).await.map_err(db)?;
        let i = indexers::load(a, &id).await?;
        let result = indexers::results(
            &i,
            &[
                ("t", "search".into()),
                ("cat", "2000,5000".into()),
                ("limit", "100".into()),
                ("extended", "1".into()),
            ],
        )
        .await;
        let parsed = match result {
            Ok(v) => v,
            Err(e) => {
                event(
                    a,
                    &id,
                    e.1.0["error"].as_str().unwrap_or("feed unavailable"),
                )
                .await?;
                continue;
            }
        };
        for row in &titles {
            let title = decode(row.get("title"))?;
            let items = wanted::inventory(a, &title).await?;
            if items.iter().any(|item| item["online"] != true) {
                continue;
            }
            use std::os::unix::fs::MetadataExt;
            let Some(root) = row.get::<Option<String>, _>("root") else {
                continue;
            };
            if !tokio::fs::metadata(root)
                .await
                .is_ok_and(|m| Some(m.dev() as i64) == row.get::<Option<i64>, _>("root_device"))
            {
                continue;
            }
            let library_enabled: bool = sqlx::query_scalar(
                "SELECT COALESCE((SELECT enabled FROM library_options WHERE library_id=?),1)",
            )
            .bind(row.get::<Option<String>, _>("library_id"))
            .fetch_one(&a.pool)
            .await
            .map_err(db)?;
            if !library_enabled {
                continue;
            }
            let units = claims::unclaimed(
                a,
                title["id"].as_str().ok_or_else(|| bad("invalid title"))?,
                &wanted::units(a, &title, &items).await?,
            )
            .await?;
            let today: String = sqlx::query_scalar("SELECT date('now')")
                .fetch_one(&a.pool)
                .await
                .map_err(db)?;
            let profile_id = row
                .get::<Option<String>, _>("profile_id")
                .unwrap_or("default".into());
            let profile = profiles::load(a, &profile_id).await?;
            for raw in parsed.as_array().into_iter().flatten() {
                let mut facts = raw.clone();
                if let Some(name) = raw["title"].as_str() {
                    for (k, v) in release_facts::facts(name)?
                        .as_object()
                        .ok_or_else(|| bad("invalid feed facts"))?
                    {
                        facts[k] = v.clone();
                    }
                }
                if !release_facts::identity_matches(&title, &facts)? {
                    continue;
                }
                let coverage_matches = units.iter().any(|u| {
                    u["wanted"] == true
                        && u["available"] != true
                        && u["air_date"]
                            .as_str()
                            .is_some_and(|d| !d.is_empty() && d <= today.as_str())
                        && (title["kind"] == "movie"
                            || (u["season"] == facts["season"]
                                && (facts["episode"].is_null()
                                    || u["episode"] == facts["episode"])))
                });
                if !coverage_matches {
                    continue;
                }
                let records =
                    releases::store_results(a, &id, &i, &title, &json!([raw]), &profile).await?;
                if let Some(r) = records.first().filter(|v| v["accepted"] == true) {
                    let _ = downloads::enqueue(
                        a,
                        downloads::Input {
                            release_id: r["id"].as_str().map(str::to_owned),
                            profile_id: Some(profile_id.clone()),
                            only_wanted: true,
                            ..Default::default()
                        },
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}
