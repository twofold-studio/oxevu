use super::*;
pub(crate) fn forbidden() -> Error {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error":"access denied"})),
    )
}
pub(crate) async fn profile(a: &App, id: &str) -> Result<Value> {
    let r=sqlx::query("SELECT users.id,username,COALESCE(is_admin,1) is_admin,COALESCE(disabled,0) disabled,COALESCE(can_play,1) can_play,COALESCE(can_transcode,1) can_transcode,COALESCE(syncplay_access,'create_join') syncplay_access,COALESCE(all_libraries,1) all_libraries FROM users LEFT JOIN user_access ON user_id=users.id WHERE users.id=?").bind(id).fetch_one(&a.pool).await.map_err(db)?;
    let grants: Vec<String> =
        sqlx::query_scalar("SELECT library_id FROM library_grants WHERE user_id=?")
            .bind(id)
            .fetch_all(&a.pool)
            .await
            .map_err(db)?;
    let acquisition = crate::acquisition::permissions(a, id).await?;
    Ok(
        json!({"id":id,"username":r.get::<String,_>("username"),"is_admin":r.get::<bool,_>("is_admin"),"disabled":r.get::<bool,_>("disabled"),"can_play":r.get::<bool,_>("can_play"),"can_transcode":r.get::<bool,_>("can_transcode"),"syncplay_access":r.get::<String,_>("syncplay_access"),"all_libraries":r.get::<bool,_>("all_libraries"),"library_ids":grants,"acquisition":acquisition}),
    )
}
pub(crate) async fn admin(a: &App, h: &HeaderMap) -> Result<Value> {
    let u = user(a, h).await?;
    if u["is_admin"] != true {
        return Err(forbidden());
    }
    Ok(u)
}
pub(crate) async fn authorize_item(a: &App, h: &HeaderMap, id: &str, play: bool) -> Result<Value> {
    let u = user(a, h).await?;
    authorize_profile(a, u, id, play).await
}
pub(crate) async fn authorize_profile(a: &App, u: Value, id: &str, play: bool) -> Result<Value> {
    if u["disabled"] == true {
        return Err(forbidden());
    }
    let library: Option<String> = sqlx::query_scalar("SELECT library_id FROM items WHERE id=?")
        .bind(id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    let library = library.ok_or_else(forbidden)?;
    let enabled: bool = sqlx::query_scalar(
        "SELECT COALESCE((SELECT enabled FROM library_options WHERE library_id=?),1)",
    )
    .bind(&library)
    .fetch_one(&a.pool)
    .await
    .map_err(db)?;
    if !enabled
        || (play && u["can_play"] != true)
        || (u["is_admin"] != true
            && u["all_libraries"] != true
            && !u["library_ids"]
                .as_array()
                .is_some_and(|v| v.contains(&json!(library))))
    {
        return Err(forbidden());
    }
    Ok(u)
}
pub(crate) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    admin(&a, &h).await?;
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM users ORDER BY username")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    let mut users = Vec::new();
    for id in ids {
        users.push(profile(&a, &id).await?)
    }
    Ok(Json(json!({"users":users})))
}
pub(crate) async fn create(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    admin(&a, &h).await?;
    save(&a, None, v).await
}
pub(crate) async fn update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    admin(&a, &h).await?;
    save(&a, Some(id), v).await
}
async fn save(a: &App, id: Option<String>, v: Value) -> Result<Json<Value>> {
    let name = v["username"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 100)
        .ok_or_else(|| bad("invalid username"))?;
    let mut flags = Vec::new();
    for key in [
        "is_admin",
        "disabled",
        "can_play",
        "can_transcode",
        "all_libraries",
    ] {
        flags.push(match v.get(key) {
            Some(value) => value
                .as_bool()
                .ok_or_else(|| bad(format!("invalid {key}")))?,
            None => key != "is_admin" && key != "disabled",
        })
    }
    let sync = v["syncplay_access"]
        .as_str()
        .filter(|s| matches!(*s, "create_join" | "join" | "disabled"))
        .ok_or_else(|| bad("invalid syncplay_access"))?;
    let grants: Vec<String> = serde_json::from_value(v["library_ids"].clone()).map_err(bad)?;
    let password = match v.get("password") {
        Some(p) => {
            let p = p
                .as_str()
                .filter(|p| (12..=1024).contains(&p.len()))
                .ok_or_else(|| bad("invalid password"))?;
            Some(hash(p.into()).await?)
        }
        None if id.is_some() => None,
        None => return Err(bad("password required")),
    };
    let existing = id.is_some();
    let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
    for grant in &grants {
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM libraries WHERE id=?")
            .bind(grant)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
        if n != 1 {
            return Err(bad("unknown library grant"));
        }
    }
    if existing {
        let old = profile(a, &id).await?;
        if old["is_admin"] == true && old["disabled"] == false && (!flags[0] || flags[1]) {
            let n:i64=sqlx::query_scalar("SELECT count(*) FROM users LEFT JOIN user_access ON user_id=users.id WHERE COALESCE(is_admin,1)=1 AND COALESCE(disabled,0)=0").fetch_one(&mut *tx).await.map_err(db)?;
            if n <= 1 {
                return Err(bad("cannot disable or demote last admin"));
            }
        }
        sqlx::query("UPDATE users SET username=?,password=COALESCE(?,password) WHERE id=?")
            .bind(name)
            .bind(&password)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    } else {
        sqlx::query("INSERT INTO users VALUES(?,?,?)")
            .bind(&id)
            .bind(name)
            .bind(&password)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    sqlx::query("INSERT INTO user_access VALUES(?,?,?,?,?,?,?) ON CONFLICT(user_id) DO UPDATE SET is_admin=excluded.is_admin,disabled=excluded.disabled,can_play=excluded.can_play,can_transcode=excluded.can_transcode,syncplay_access=excluded.syncplay_access,all_libraries=excluded.all_libraries").bind(&id).bind(flags[0]).bind(flags[1]).bind(flags[2]).bind(flags[3]).bind(sync).bind(flags[4]).execute(&mut *tx).await.map_err(db)?;
    sqlx::query("DELETE FROM library_grants WHERE user_id=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    for grant in grants {
        sqlx::query("INSERT OR IGNORE INTO library_grants VALUES(?,?)")
            .bind(&id)
            .bind(grant)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    sqlx::query("DELETE FROM sessions WHERE user_id=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(Json(json!({"user":profile(a,&id).await?})))
}
