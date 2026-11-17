use super::*;
pub(crate) async fn roots(a: &App, id: &str) -> Result<Vec<String>> {
    let roots: Vec<String> =
        sqlx::query_scalar("SELECT path FROM library_roots WHERE library_id=? ORDER BY rowid")
            .bind(id)
            .fetch_all(&a.pool)
            .await
            .map_err(db)?;
    if !roots.is_empty() {
        return Ok(roots);
    }
    sqlx::query_scalar("SELECT path FROM libraries WHERE id=?")
        .bind(id)
        .fetch_all(&a.pool)
        .await
        .map_err(db)
}
pub(crate) async fn item_root(a: &App, id: &str, path: &std::path::Path) -> Result<PathBuf> {
    let lib: String = sqlx::query_scalar("SELECT library_id FROM items WHERE id=?")
        .bind(id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let path = tokio::fs::canonicalize(path).await.map_err(bad)?;
    for root in roots(a, &lib).await? {
        if let Ok(root) = tokio::fs::canonicalize(root).await
            && path.starts_with(&root)
        {
            return Ok(root);
        }
    }
    Err(bad("file outside library"))
}
pub(crate) async fn contains(a: &App, id: &str, path: &std::path::Path) -> Result<bool> {
    Ok(item_root(a, id, path).await.is_ok())
}
async fn representation(a: &App, id: &str) -> Result<Value> {
    let r=sqlx::query("SELECT libraries.*,COALESCE(enabled,1) enabled FROM libraries LEFT JOIN library_options ON library_id=libraries.id WHERE libraries.id=?").bind(id).fetch_one(&a.pool).await.map_err(db)?;
    Ok(
        json!({"id":id,"name":r.get::<String,_>("name"),"kind":r.get::<String,_>("kind"),"path":r.get::<String,_>("path"),"paths":roots(a,id).await?,"enabled":r.get::<bool,_>("enabled")}),
    )
}
pub(crate) async fn libraries(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM libraries")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    let mut out = Vec::new();
    for id in ids {
        out.push(representation(&a, &id).await?)
    }
    Ok(Json(json!({"libraries":out})))
}
pub(crate) async fn add_library(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    save(&a, None, v).await
}
pub(crate) async fn update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    save(&a, Some(id), v).await
}
async fn save(a: &App, id: Option<String>, v: Value) -> Result<Json<Value>> {
    let _permit = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("scan already running"))?;
    let name = v["name"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 200)
        .ok_or_else(|| bad("invalid name"))?;
    let kind = v["kind"]
        .as_str()
        .filter(|s| matches!(*s, "movies" | "tv"))
        .ok_or_else(|| bad("invalid kind"))?;
    let enabled = v["enabled"].as_bool().unwrap_or(true);
    let input: Vec<String> = if v.get("paths").is_some() {
        serde_json::from_value(v["paths"].clone()).map_err(bad)?
    } else {
        vec![
            v["path"]
                .as_str()
                .ok_or_else(|| bad("paths required"))?
                .into(),
        ]
    };
    if input.is_empty() || input.len() > 32 {
        return Err(bad("provide 1–32 roots"));
    }
    let existing = id.is_some();
    let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let other:Vec<String>=sqlx::query_scalar("SELECT path FROM library_roots WHERE library_id!=? UNION SELECT path FROM libraries WHERE id!=?").bind(&id).bind(&id).fetch_all(&a.pool).await.map_err(db)?;
    let mut protected = vec![
        tokio::fs::canonicalize(std::env::current_dir().map_err(bad)?)
            .await
            .map_err(bad)?,
    ];
    if let Some(source) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
    {
        protected.push(source.to_path_buf());
    }
    if let Some(data) = a.artwork_cache.parent() {
        protected.push(data.to_path_buf());
    }
    if let Some(scratch) = std::env::var_os("OXEVU_TRANSCODE_DIR") {
        protected.push(PathBuf::from(scratch));
    }
    protected.push(a.playback.scratch.clone());
    let mut paths: Vec<PathBuf> = Vec::new();
    for input in input {
        let path = tokio::fs::canonicalize(input)
            .await
            .map_err(|_| bad("root must be an accessible directory"))?;
        if !tokio::fs::metadata(&path).await.map_err(bad)?.is_dir() {
            return Err(bad("root must be directory"));
        }
        for other in paths
            .iter()
            .cloned()
            .chain(other.iter().map(PathBuf::from))
            .chain(protected.iter().cloned())
        {
            let other = tokio::fs::canonicalize(&other).await.unwrap_or(other);
            if path.starts_with(&other) || other.starts_with(&path) {
                return Err(bad("overlapping or protected root"));
            }
        }
        paths.push(path);
    }
    let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
    if existing {
        let n = sqlx::query("UPDATE libraries SET name=?,path=?,kind=? WHERE id=?")
            .bind(name)
            .bind(paths[0].to_string_lossy().as_ref())
            .bind(kind)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        if n.rows_affected() != 1 {
            return Err(bad("unknown library"));
        }
    } else {
        sqlx::query("INSERT INTO libraries VALUES(?,?,?,?)")
            .bind(&id)
            .bind(name)
            .bind(paths[0].to_string_lossy().as_ref())
            .bind(kind)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    sqlx::query("DELETE FROM library_roots WHERE library_id=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    for path in &paths {
        sqlx::query("INSERT INTO library_roots VALUES(?,?)")
            .bind(&id)
            .bind(path.to_string_lossy().as_ref())
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    sqlx::query("INSERT INTO library_options VALUES(?,?) ON CONFLICT(library_id) DO UPDATE SET enabled=excluded.enabled").bind(&id).bind(enabled).execute(&mut *tx).await.map_err(db)?;
    // Removed roots immediately cease to expose catalog entries; media is never changed.
    let items = sqlx::query("SELECT id,path FROM items WHERE library_id=?")
        .bind(&id)
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?;
    for item in items {
        if !paths
            .iter()
            .any(|p| PathBuf::from(item.get::<String, _>("path")).starts_with(p))
        {
            let item: String = item.get("id");
            sqlx::query("DELETE FROM progress WHERE item_id=?")
                .bind(&item)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("DELETE FROM items WHERE id=?")
                .bind(item)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
    }
    tx.commit().await.map_err(db)?;
    Ok(Json(json!({"library":representation(a,&id).await?})))
}
pub(crate) async fn remove(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    users::admin(&a, &h).await?;
    let _permit = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("scan already running"))?;
    let mut tx = a.pool.begin().await.map_err(db)?;
    for query in [
        "DELETE FROM progress WHERE item_id IN(SELECT id FROM items WHERE library_id=?)",
        "DELETE FROM items WHERE library_id=?",
        "DELETE FROM library_grants WHERE library_id=?",
        "DELETE FROM library_roots WHERE library_id=?",
        "DELETE FROM library_options WHERE library_id=?",
        "DELETE FROM libraries WHERE id=?",
    ] {
        sqlx::query(query)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    Ok(StatusCode::NO_CONTENT)
}
