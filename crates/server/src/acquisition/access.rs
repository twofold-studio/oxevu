use super::*;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Permissions {
    pub can_request: bool,
    pub auto_approve: bool,
    pub manage_requests: bool,
    pub manage_downloads: bool,
    pub manage_indexers: bool,
    pub request_limit: u32,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            can_request: true,
            auto_approve: false,
            manage_requests: false,
            manage_downloads: false,
            manage_indexers: false,
            request_limit: 10,
        }
    }
}

pub(crate) async fn permissions(a: &App, id: &str) -> Result<Permissions> {
    let row = sqlx::query("SELECT * FROM acquisition_permissions WHERE user_id=?")
        .bind(id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    Ok(match row {
        Some(r) => Permissions {
            can_request: r.get("can_request"),
            auto_approve: r.get("auto_approve"),
            manage_requests: r.get("manage_requests"),
            manage_downloads: r.get("manage_downloads"),
            manage_indexers: r.get("manage_indexers"),
            request_limit: r.get("request_limit"),
        },
        None => Permissions::default(),
    })
}

pub(super) async fn require(a: &App, h: &HeaderMap, permission: &str) -> Result<Value> {
    let u = user(a, h).await?;
    if u["is_admin"] == true {
        return Ok(u);
    }
    let p = permissions(a, u["id"].as_str().ok_or_else(users::forbidden)?).await?;
    let allowed = match permission {
        "request" => p.can_request,
        "requests" => p.manage_requests,
        "downloads" => p.manage_downloads,
        "indexers" => p.manage_indexers,
        "catalog" => p.manage_requests || p.manage_downloads,
        _ => false,
    };
    if !allowed {
        return Err(users::forbidden());
    }
    Ok(u)
}

pub(super) async fn get(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    Ok(Json(
        json!({"permissions":permissions(&a,u["id"].as_str().ok_or_else(users::forbidden)?).await?}),
    ))
}

pub(super) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(p): Json<Permissions>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    if p.request_limit > 1000 {
        return Err(bad("request limit must be 0 to 1000"));
    }
    let exists: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    if exists != 1 {
        return Err(bad("unknown user"));
    }
    sqlx::query("INSERT INTO acquisition_permissions VALUES(?,?,?,?,?,?,?) ON CONFLICT(user_id) DO UPDATE SET can_request=excluded.can_request,auto_approve=excluded.auto_approve,manage_requests=excluded.manage_requests,manage_downloads=excluded.manage_downloads,manage_indexers=excluded.manage_indexers,request_limit=excluded.request_limit")
        .bind(&id).bind(p.can_request).bind(p.auto_approve).bind(p.manage_requests).bind(p.manage_downloads).bind(p.manage_indexers).bind(p.request_limit).execute(&a.pool).await.map_err(db)?;
    Ok(Json(json!({"permissions":p})))
}
