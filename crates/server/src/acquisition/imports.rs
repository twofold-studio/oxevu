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
        let current: Option<String> = sqlx::query_scalar::<_, Option<String>>(
            "SELECT title_id FROM acquisition_downloads WHERE id=? AND state!='removed'",
        )
        .bind(download)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?
        .flatten();
        if current.as_deref() != Some(file.title_id.as_str()) {
            return Err(bad(
                "The download title changed. Preview the import again before publishing files.",
            ));
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
        "UPDATE acquisition_downloads SET data=json_remove(data,'$.import_warning','$.reimport_requested') WHERE id=?",
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
    let item = crate::media_files::restore_identity(a, &item, &title).await?;
    let mut metadata = imported_metadata(&title);
    metadata["episode_coverage"] = json!(file.coverage);
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
    if file.coverage.is_empty() {
        sqlx::query(
            "DELETE FROM acquisition_removed_units WHERE title_id=? AND season=-1 AND episode=-1",
        )
        .bind(&file.title_id)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    } else {
        for unit in &file.coverage {
            sqlx::query(
                "DELETE FROM acquisition_removed_units WHERE title_id=? AND season=? AND episode=?",
            )
            .bind(&file.title_id)
            .bind(unit.season)
            .bind(unit.episode)
            .execute(&a.pool)
            .await
            .map_err(db)?;
        }
    }
    Ok(())
}

fn imported_metadata(title: &Value) -> Value {
    let genres: Vec<&str> = title["genres"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|genre| genre["name"].as_str())
        .collect();
    let mut data = json!({
        "provider_ids": {"tmdb": title["provider_id"]},
        "year": title["year"].as_str().and_then(|s| s.parse::<u32>().ok()),
        "overview": title["overview"],
        "rating": title["rating"],
        "rating_source": "TMDB",
        "genres": genres,
        "metadata_attribution": "This product uses the TMDB API but is not endorsed or certified by TMDB."
    });
    for field in ["poster_url", "backdrop_url"] {
        if let Some(url) = title[field].as_str().filter(|url| {
            url.strip_prefix("/api/v1/acquisition/artwork/")
                .is_some_and(catalog::valid_image)
        }) {
            data[field] = json!(url);
        }
    }
    if let (Some(id), Some(kind @ ("movie" | "tv"))) =
        (title["provider_id"].as_u64(), title["kind"].as_str())
    {
        data["external_urls"] = json!({"tmdb": format!("https://www.themoviedb.org/{kind}/{id}")});
    }
    data
}
pub(super) async fn list_for(a: &App, id: &str) -> Result<Value> {
    let rows=sqlx::query("SELECT id,file_index,destination,state,error,item_id,coverage FROM acquisition_imports WHERE download_id=? ORDER BY file_index").bind(id).fetch_all(&a.pool).await.map_err(db)?;
    let mut items = Vec::new();
    for r in rows {
        items.push(json!({"id":r.get::<String,_>("id"),"file_index":r.get::<i64,_>("file_index"),"destination":r.get::<String,_>("destination"),"state":r.get::<String,_>("state"),"error":r.get::<Option<String>,_>("error"),"item_id":r.get::<Option<String>,_>("item_id"),"episodes":decode(r.get("coverage"))?}));
    }
    Ok(json!(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_details_import_keeps_artwork_genres_and_rating() {
        let title = json!({"provider_id":315162,"kind":"movie","year":"2022","rating":8.2,
            "poster_url":"/api/v1/acquisition/artwork/Poster123.jpg",
            "backdrop_url":"https://untrusted.example/image.jpg","genres":[{"name":"Animation"}]});
        let data = imported_metadata(&title);
        assert_eq!(data["poster_url"], title["poster_url"]);
        assert_eq!(data["genres"], json!(["Animation"]));
        assert_eq!(data["rating"], 8.2);
        assert_eq!(data["rating_source"], "TMDB");
        assert_eq!(data["year"], 2022);
        assert!(data.get("backdrop_url").is_none());
        assert_eq!(
            data["external_urls"]["tmdb"],
            "https://www.themoviedb.org/movie/315162"
        );
    }

    #[tokio::test]
    async fn media_details_migration_fills_missing_art_without_replacing_owner_metadata() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::raw_sql(
            "CREATE TABLE acquisition_titles(id TEXT, data TEXT, provider_id INTEGER, kind TEXT);
            CREATE TABLE acquisition_imports(item_id TEXT, title_id TEXT, state TEXT);
            CREATE TABLE item_metadata(item_id TEXT, data TEXT);",
        )
        .execute(&pool)
        .await
        .unwrap();
        let title = json!({"poster_url":"/api/v1/acquisition/artwork/Poster123.jpg","genres":[{"name":"Animation"}]}).to_string();
        sqlx::query("INSERT INTO acquisition_titles VALUES('title',?,315162,'movie')")
            .bind(title)
            .execute(&pool)
            .await
            .unwrap();
        for (id, poster) in [
            ("missing", None),
            (
                "matched",
                Some("/api/v1/items/matched/metadata/artwork/poster"),
            ),
        ] {
            let mut data = json!({"provider_ids":{"tmdb":315162},"rating":8.2});
            if let Some(poster) = poster {
                data["poster_url"] = json!(poster);
            }
            sqlx::query("INSERT INTO item_metadata VALUES(?,?)")
                .bind(id)
                .bind(data.to_string())
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO acquisition_imports VALUES(?,'title','scanned')")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();
        }
        sqlx::raw_sql(include_str!("../../migrations/0007_import_artwork.sql"))
            .execute(&pool)
            .await
            .unwrap();
        for (id, expected) in [
            ("missing", "/api/v1/acquisition/artwork/Poster123.jpg"),
            ("matched", "/api/v1/items/matched/metadata/artwork/poster"),
        ] {
            let raw: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id=?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
            let data: Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(data["poster_url"], expected);
            assert_eq!(data["rating"], 8.2);
            assert_eq!(data["genres"], json!(["Animation"]));
        }
    }
}
