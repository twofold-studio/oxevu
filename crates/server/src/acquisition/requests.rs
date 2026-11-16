use super::scope::Scope;
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Input {
    pub kind: String,
    pub provider_id: u64,
    #[serde(default)]
    pub scope: Scope,
}

pub(super) async fn create(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Input>,
) -> Result<Json<Value>> {
    let u = access::require(&a, &h, "request").await?;
    let title = catalog::ensure_title(&a, &v.kind, v.provider_id).await?;
    let id = create_for(&a, &u, &title, v.scope).await?;
    Ok(Json(json!({"request":view(&a,&id,&h).await?})))
}

pub(super) async fn create_for(
    a: &App,
    u: &Value,
    title: &Value,
    mut scope: Scope,
) -> Result<String> {
    let uid = u["id"].as_str().ok_or_else(users::forbidden)?;
    let title_id = title["id"].as_str().ok_or_else(|| bad("unknown title"))?;
    let today: String = sqlx::query_scalar("SELECT date('now')")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    scope.normalize(
        title["kind"]
            .as_str()
            .ok_or_else(|| bad("invalid title kind"))?,
        &today,
    )?;
    validate_selection(a, title_id, &scope).await?;
    let p = permissions(a, uid).await?;
    if u["is_admin"] != true && !p.can_request {
        return Err(users::forbidden());
    }
    let raw = serde_json::to_string(&scope).map_err(bad)?;
    let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
    let existing: Option<String>=sqlx::query_scalar("SELECT id FROM acquisition_requests WHERE title_id=? AND user_id=? AND scope=? AND status IN ('pending','approved','fulfilled')")
        .bind(title_id).bind(uid).bind(&raw).fetch_optional(&mut *tx).await.map_err(db)?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let count: i64=sqlx::query_scalar("SELECT count(*) FROM acquisition_requests WHERE user_id=? AND status IN ('pending','approved')")
        .bind(uid).fetch_one(&mut *tx).await.map_err(db)?;
    if u["is_admin"] != true && count >= p.request_limit as i64 {
        return Err(bad("active request limit reached"));
    }
    let status = if u["is_admin"] == true || p.auto_approve {
        "approved"
    } else {
        "pending"
    };
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO acquisition_requests(id,title_id,user_id,scope,status) VALUES(?,?,?,?,?)",
    )
    .bind(&id)
    .bind(title_id)
    .bind(uid)
    .bind(raw)
    .bind(status)
    .execute(&mut *tx)
    .await
    .map_err(db)?;
    tx.commit().await.map_err(db)?;
    event(
        a,
        &id,
        if status == "approved" {
            "Request approved"
        } else {
            "Request submitted"
        },
    )
    .await?;
    Ok(id)
}

async fn validate_selection(a: &App, title: &str, scope: &Scope) -> Result<()> {
    for season in &scope.seasons {
        let n: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM acquisition_episodes WHERE title_id=? AND season=?",
        )
        .bind(title)
        .bind(season)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
        if n == 0 {
            return Err(bad("selected season is not in the provider catalog"));
        }
    }
    for e in &scope.episodes {
        let n: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM acquisition_episodes WHERE title_id=? AND season=? AND episode=?",
        )
        .bind(title)
        .bind(e.season)
        .bind(e.episode)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
        if n != 1 {
            return Err(bad("selected episode is not in the provider catalog"));
        }
    }
    Ok(())
}

pub(super) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    let p = permissions(&a, u["id"].as_str().ok_or_else(users::forbidden)?).await?;
    let manage = u["is_admin"] == true || p.manage_requests;
    let ids:Vec<String>=sqlx::query_scalar("SELECT id FROM acquisition_requests WHERE (? OR user_id=?) ORDER BY created_at DESC,id LIMIT 1000")
        .bind(manage).bind(u["id"].as_str()).fetch_all(&a.pool).await.map_err(db)?;
    let mut requests = Vec::new();
    for id in ids {
        requests.push(view(&a, &id, &h).await?);
    }
    Ok(Json(json!({"requests":requests})))
}

pub(super) async fn view(a: &App, id: &str, h: &HeaderMap) -> Result<Value> {
    let r=sqlx::query("SELECT r.*,t.data title,u.username FROM acquisition_requests r JOIN acquisition_titles t ON t.id=r.title_id JOIN users u ON u.id=r.user_id WHERE r.id=?")
        .bind(id).fetch_optional(&a.pool).await.map_err(db)?.ok_or_else(||bad("unknown request"))?;
    let title = decode(r.get("title"))?;
    let available = catalog::availability(a, &title, &crate::catalog::catalog(a, h).await?).await?;
    Ok(
        json!({"id":id,"title":title,"user_id":r.get::<String,_>("user_id"),"username":r.get::<String,_>("username"),
        "scope":decode(r.get("scope"))?,"status":r.get::<String,_>("status"),"note":r.get::<String,_>("note"),
        "created_at":r.get::<i64,_>("created_at"),"availability":available,"stage":request_status::stage(a,title["id"].as_str().ok_or_else(||bad("invalid title"))?,r.get("status")).await?}),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Decision {
    pub status: String,
    #[serde(default)]
    pub note: String,
}

pub(super) async fn update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Decision>,
) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    if v.note.len() > 2000 {
        return Err(bad("note exceeds 2000 characters"));
    }
    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM acquisition_requests WHERE id=?")
            .bind(&id)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?;
    let owner = owner.ok_or_else(|| bad("unknown request"))?;
    if v.status == "cancelled" {
        if u["id"] != owner {
            access::require(&a, &h, "requests").await?;
        }
    } else {
        access::require(&a, &h, "requests").await?;
        if !matches!(v.status.as_str(), "approved" | "declined") {
            return Err(bad("invalid request decision"));
        }
    }
    let result=sqlx::query("UPDATE acquisition_requests SET status=?,note=?,updated_at=unixepoch() WHERE id=? AND status!='cancelled'")
        .bind(&v.status).bind(v.note).bind(&id).execute(&a.pool).await.map_err(db)?;
    if result.rows_affected() != 1 {
        return Err(bad("request has been cancelled"));
    }
    event(&a, &id, &format!("Request {}", v.status)).await?;
    Ok(Json(json!({"request":view(&a,&id,&h).await?})))
}
