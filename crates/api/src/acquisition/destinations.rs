use super::*;
use std::os::unix::fs::MetadataExt;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Destination {
    pub library_id: String,
    pub root: String,
    /// Captured by the server on save; never trusted from a client.
    #[serde(default)]
    pub root_device: Option<u64>,
}

async fn registered(a: &App, kind: &str, target: &Destination) -> Result<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM libraries l JOIN library_roots r ON r.library_id=l.id LEFT JOIN library_options o ON o.library_id=l.id WHERE l.id=? AND l.kind=? AND r.path=? AND COALESCE(o.enabled,1)=1)")
        .bind(&target.library_id).bind(kind).bind(&target.root).fetch_one(&a.pool).await.map_err(db)
}

pub(super) async fn validate(a: &App, kind: &str, target: &mut Destination) -> Result<()> {
    if !registered(a, kind, target).await? {
        return Err(bad(
            "choose an enabled library and folder matching the media type",
        ));
    }
    let meta = tokio::fs::metadata(&target.root)
        .await
        .map_err(|_| bad("default import folder is offline"))?;
    if !meta.is_dir() {
        return Err(bad("default import folder must be a directory"));
    }
    target.root_device = Some(meta.dev());
    Ok(())
}

pub(super) async fn resolve(a: &App, kind: &str) -> Result<Option<Destination>> {
    let c = config::load(a).await?;
    let selected = if kind == "tv" {
        c.series_destination
    } else {
        c.movie_destination
    };
    if let Some(target) = selected {
        if !registered(a, kind, &target).await? {
            return Err(bad(
                "default import folder is no longer enabled; choose a new default",
            ));
        }
        return Ok(Some(target));
    }
    let row = sqlx::query("SELECT l.id,r.path FROM libraries l JOIN library_roots r ON r.library_id=l.id LEFT JOIN library_options o ON o.library_id=l.id WHERE l.kind=? AND COALESCE(o.enabled,1)=1 ORDER BY l.rowid,r.rowid LIMIT 1")
        .bind(kind).fetch_optional(&a.pool).await.map_err(db)?;
    Ok(row.map(|r| Destination {
        library_id: r.get("id"),
        root: r.get("path"),
        root_device: None,
    }))
}

pub(super) async fn for_title(a: &App, title: &str, kind: &str) -> Result<Option<Destination>> {
    let row = sqlx::query("SELECT library_id,root,root_device FROM acquisition_monitors WHERE title_id=? AND library_id IS NOT NULL AND root IS NOT NULL")
        .bind(title).fetch_optional(&a.pool).await.map_err(db)?;
    if let Some(r) = row {
        return Ok(Some(Destination {
            library_id: r.get("library_id"),
            root: r.get("root"),
            root_device: r.get::<Option<i64>, _>("root_device").map(|d| d as u64),
        }));
    }
    resolve(a, if kind == "tv" { "tv" } else { "movies" }).await
}

pub(super) async fn get(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    Ok(Json(
        json!({"movies":resolve(&a,"movies").await?,"tv":resolve(&a,"tv").await?}),
    ))
}
