use super::*;
use import_plan::{File, Input};

pub(super) async fn preview(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let files = import_plan::build(&a, &id, &input).await?;
    Ok(Json(
        json!({"files":files,"token":import_plan::token(&files)?,"copy_fallback":config::load(&a).await?.copy_fallback}),
    ))
}
pub(super) async fn apply(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let files = import_plan::build(&a, &id, &input).await?;
    if input.token.as_deref() != Some(import_plan::token(&files)?.as_str()) {
        return Err(bad(
            "preview the current import destinations before confirming",
        ));
    }
    execute(&a, &id, &input.library_id, files).await?;
    Ok(Json(json!({"imports":list_for(&a,&id).await?})))
}

pub(super) async fn execute(
    a: &App,
    download: &str,
    library: &str,
    files: Vec<File>,
) -> Result<()> {
    let _permit = a
        .acquisition
        .import_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("another import is active"))?;
    for file in files {
        let config = config::load(a).await?;
        if !config.import_enabled {
            return Err(bad("enable imports explicitly in acquisition settings"));
        }
        let registered: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM library_roots r JOIN libraries l ON l.id=r.library_id JOIN acquisition_titles t ON t.id=? LEFT JOIN library_options o ON o.library_id=l.id WHERE r.library_id=? AND r.path=? AND COALESCE(o.enabled,1)=1 AND l.kind=CASE t.kind WHEN 'tv' THEN 'tv' ELSE 'movies' END)",
        )
        .bind(&file.title_id)
        .bind(library)
        .bind(&file.root)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
        if !registered {
            return Err(bad(
                "import destination is no longer a configured library root",
            ));
        }
        import_io::probe(&file.source).await?;
        if config.copy_fallback {
            let free = storage::free_bytes(PathBuf::from(&file.root)).await?;
            if free
                < file
                    .size
                    .saturating_add(config.min_free_gb as u64 * 1024 * 1024 * 1024)
            {
                return Err(bad("not enough free library storage for copy fallback"));
            }
        }
        let existing:Option<String>=sqlx::query_scalar("SELECT id FROM acquisition_imports WHERE download_id=? AND file_index=? AND season=? AND episode=?")
            .bind(download).bind(file.file_index as i64).bind(file.coverage.first().map_or(-1,|e|e.season as i64)).bind(file.coverage.first().map_or(-1,|e|e.episode as i64)).fetch_optional(&a.pool).await.map_err(db)?;
        let id = existing.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let previous =
            sqlx::query("SELECT checksum,destination FROM acquisition_imports WHERE id=?")
                .bind(&id)
                .fetch_optional(&a.pool)
                .await
                .map_err(db)?;
        if previous
            .as_ref()
            .is_some_and(|r| r.get::<String, _>("destination") != file.destination)
        {
            return Err(bad("existing import journal has a different destination"));
        }
        let expected_checksum = previous
            .as_ref()
            .and_then(|r| r.get::<Option<String>, _>("checksum"));
        let coverage = serde_json::to_string(&file.coverage).map_err(bad)?;
        sqlx::query("INSERT INTO acquisition_imports(id,download_id,file_index,title_id,season,episode,destination,source,state,coverage,library_id,plan) VALUES(?,?,?,?,?,?,?,?,'copying',?,?,?) ON CONFLICT(id) DO UPDATE SET state='copying',error=NULL,plan=excluded.plan")
            .bind(&id).bind(download).bind(file.file_index as i64).bind(&file.title_id).bind(file.coverage.first().map_or(-1,|e|e.season as i64)).bind(file.coverage.first().map_or(-1,|e|e.episode as i64))
            .bind(&file.destination).bind(&file.source).bind(coverage).bind(library).bind(serde_json::to_string(&file).map_err(bad)?).execute(&a.pool).await.map_err(db)?;
        let f = file.clone();
        let job_id = id.clone();
        let runtime = a.acquisition.clone();
        let result = tokio::task::spawn_blocking(move || {
            import_io::publish(&f, &job_id, config.copy_fallback, &runtime)
        })
        .await
        .map_err(|_| bad("import task interrupted"))?;
        match result {
            Ok(checksum) => {
                if expected_checksum
                    .as_ref()
                    .is_some_and(|expected| expected != &checksum)
                {
                    return Err(bad("published media changed since the verified import"));
                }
                sqlx::query(
                    "UPDATE acquisition_imports SET state='published',checksum=? WHERE id=?",
                )
                .bind(checksum)
                .bind(&id)
                .execute(&a.pool)
                .await
                .map_err(db)?;
            }
            Err(e) => {
                sqlx::query("UPDATE acquisition_imports SET state='failed',error=? WHERE id=?")
                    .bind(e.to_string())
                    .bind(&id)
                    .execute(&a.pool)
                    .await
                    .map_err(db)?;
                return Err(bad(format!("import failed: {e}")));
            }
        }
        let _ = scan::scan_library(a.clone(), library.to_owned()).await?;
        attach(a, &id, &file).await?;
        event(a, download, "Media imported; seeding source retained").await?;
    }
    sqlx::query(
        "UPDATE acquisition_downloads SET data=json_remove(data,'$.import_warning') WHERE id=?",
    )
    .bind(download)
    .execute(&a.pool)
    .await
    .map_err(db)?;
    Ok(())
}

async fn attach(a: &App, id: &str, file: &File) -> Result<()> {
    let item: Option<String> = sqlx::query_scalar("SELECT id FROM items WHERE path=?")
        .bind(&file.destination)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    let item =
        item.ok_or_else(|| bad("published media has not entered the library; retry scan"))?;
    let raw: String = sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id=?")
        .bind(&file.title_id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let title = decode(&raw)?;
    let mut metadata = json!({"provider_ids":{"tmdb":title["provider_id"]},"year":title["year"].as_str().and_then(|s|s.parse::<u32>().ok()),"overview":title["overview"],"rating":title["rating"],"rating_source":"TMDB","episode_coverage":file.coverage});
    if let Some(first) = file.coverage.first() {
        let raw: String = sqlx::query_scalar(
            "SELECT data FROM acquisition_episodes WHERE title_id=? AND season=? AND episode=?",
        )
        .bind(&file.title_id)
        .bind(first.season)
        .bind(first.episode)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
        let e = decode(&raw)?;
        metadata["title"] = e["name"].clone();
        metadata["overview"] = e["overview"].clone();
        metadata["rating"] = e["vote_average"].clone();
        metadata["show_title"] = title["title"].clone();
    } else {
        metadata["title"] = title["title"].clone();
    }
    sqlx::query("INSERT INTO item_metadata VALUES(?,?) ON CONFLICT(item_id) DO UPDATE SET data=json_patch(item_metadata.data,excluded.data)").bind(&item).bind(metadata.to_string()).execute(&a.pool).await.map_err(db)?;
    sqlx::query("UPDATE acquisition_imports SET state='scanned',item_id=?,error=NULL WHERE id=?")
        .bind(item)
        .bind(id)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    Ok(())
}
pub(super) async fn list_for(a: &App, id: &str) -> Result<Value> {
    let rows=sqlx::query("SELECT id,file_index,destination,state,error,item_id,coverage FROM acquisition_imports WHERE download_id=? ORDER BY file_index").bind(id).fetch_all(&a.pool).await.map_err(db)?;
    let mut items = Vec::new();
    for r in rows {
        items.push(json!({"id":r.get::<String,_>("id"),"file_index":r.get::<i64,_>("file_index"),"destination":r.get::<String,_>("destination"),"state":r.get::<String,_>("state"),"error":r.get::<Option<String>,_>("error"),"item_id":r.get::<Option<String>,_>("item_id"),"episodes":decode(r.get("coverage"))?}));
    }
    Ok(json!(items))
}
