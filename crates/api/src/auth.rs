use super::*;
static PASSWORD_JOBS: Semaphore = Semaphore::const_new(4);
fn password_permit() -> Result<tokio::sync::SemaphorePermit<'static>> {
    PASSWORD_JOBS.try_acquire().map_err(|_| {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":"authentication busy; try again later"})),
        )
    })
}
pub(crate) async fn user(a: &App, h: &HeaderMap) -> Result<Value> {
    let token = h
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';')
                .find_map(|s| s.trim().strip_prefix("oxevu_session="))
        })
        .unwrap_or("");
    let row=sqlx::query("SELECT users.id,username FROM sessions JOIN users ON users.id=sessions.user_id WHERE sessions.id=? AND expires>unixepoch()").bind(token).fetch_optional(&a.pool).await.map_err(db)?;
    let row = row.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({"error":"sign in required"})),
    ))?;
    let u = users::profile(a, &row.get::<String, _>(0)).await?;
    if u["disabled"] == true {
        return Err(users::forbidden());
    }
    Ok(u)
}
pub(crate) async fn setup(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Response> {
    if h.get("x-oxevu-setup-token").and_then(|v| v.to_str().ok()) != Some(&a.token) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"local setup token required"})),
        ));
    }
    let (name, password) = credentials(&v)?;
    let hash = hash(password.to_owned()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let result =
        sqlx::query("INSERT INTO users SELECT ?,?,? WHERE NOT EXISTS(SELECT 1 FROM users)")
            .bind(&id)
            .bind(name)
            .bind(hash)
            .execute(&a.pool)
            .await
            .map_err(db)?;
    if result.rows_affected() != 1 {
        return Err(bad("setup already completed"));
    }
    sqlx::query("INSERT INTO user_access(user_id,is_admin) VALUES(?,1)")
        .bind(&id)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    session(&a, id, name).await
}
pub(crate) fn credentials(v: &Value) -> Result<(&str, &str)> {
    let n = v["username"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 100)
        .ok_or_else(|| bad("invalid username"))?;
    let p = v["password"]
        .as_str()
        .filter(|s| s.len() >= 12 && s.len() <= 1024)
        .ok_or_else(|| bad("password must be 12–1024 bytes"))?;
    Ok((n, p))
}
pub(crate) async fn hash(p: String) -> Result<String> {
    let permit = password_permit()?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
        argon2::Argon2::default()
            .hash_password(p.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|h| h.to_string())
            .map_err(|_| bad("password hashing failed"))
    })
    .await
    .map_err(bad)?
}
pub(crate) async fn login(State(a): State<App>, Json(v): Json<Value>) -> Result<Response> {
    let (n, p) = credentials(&v)?;
    sqlx::query("DELETE FROM login_attempts WHERE expires<=unixepoch()")
        .execute(&a.pool)
        .await
        .map_err(db)?;
    let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM login_attempts")
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
    let attempts: Option<i64> =
        sqlx::query_scalar("SELECT attempts FROM login_attempts WHERE username=?")
            .bind(n)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db)?;
    if attempts.is_some_and(|n| n >= 5) || (attempts.is_none() && count >= 1024) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":"try again later"})),
        ));
    }
    sqlx::query("INSERT INTO login_attempts VALUES(?,1,unixepoch()+300) ON CONFLICT(username) DO UPDATE SET attempts=attempts+1").bind(n).execute(&mut *tx).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    let r = sqlx::query("SELECT id,password FROM users WHERE username=?")
        .bind(n)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("invalid credentials"))?;
    let hashed: String = r.get(1);
    let p = p.to_owned();
    let permit = password_permit()?;
    let valid = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        use argon2::{PasswordHash, PasswordVerifier};
        PasswordHash::new(&hashed).is_ok_and(|h| {
            argon2::Argon2::default()
                .verify_password(p.as_bytes(), &h)
                .is_ok()
        })
    })
    .await
    .map_err(bad)?;
    if !valid {
        return Err(bad("invalid credentials"));
    }
    let id: String = r.get(0);
    if users::profile(&a, &id).await?["disabled"] == true {
        return Err(bad("invalid credentials"));
    }
    sqlx::query("DELETE FROM login_attempts WHERE username=?")
        .bind(n)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    session(&a, id, n).await
}
pub(crate) async fn session(a: &App, id: String, _name: &str) -> Result<Response> {
    sqlx::query("DELETE FROM sessions WHERE expires<=unixepoch()")
        .execute(&a.pool)
        .await
        .map_err(db)?;
    sqlx::query("DELETE FROM sessions WHERE user_id=? AND id NOT IN (SELECT id FROM sessions WHERE user_id=? ORDER BY expires DESC LIMIT 31)").bind(&id).bind(&id).execute(&a.pool).await.map_err(db)?;
    let token = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO sessions VALUES(?,?,unixepoch()+604800)")
        .bind(&token)
        .bind(&id)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    let secure = if a.origin.starts_with("https:") {
        "; Secure"
    } else {
        ""
    };
    Ok((
        [(
            axum::http::header::SET_COOKIE,
            format!(
                "oxevu_session={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800{secure}"
            ),
        )],
        Json(crate::contract::UserEnvelope {
            user: crate::contract::decode(users::profile(a, &id).await?)?,
        }),
    )
        .into_response())
}
pub(crate) async fn logout(State(a): State<App>, h: HeaderMap) -> Result<Response> {
    user(&a, &h).await?;
    let cookie = h
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .find_map(|v| v.trim().strip_prefix("oxevu_session="))
        })
        .unwrap_or("");
    sqlx::query("DELETE FROM sessions WHERE id=?")
        .bind(cookie)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    Ok((
        StatusCode::NO_CONTENT,
        [(
            axum::http::header::SET_COOKIE,
            "oxevu_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        )],
    )
        .into_response())
}

pub(crate) async fn password(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<StatusCode> {
    let u = user(&a, &h).await?;
    let current = v["current_password"]
        .as_str()
        .ok_or_else(|| bad("current password required"))?
        .to_owned();
    let new = v["new_password"]
        .as_str()
        .filter(|s| (12..=1024).contains(&s.len()))
        .ok_or_else(|| bad("invalid new password"))?;
    let old: String = sqlx::query_scalar("SELECT password FROM users WHERE id=?")
        .bind(u["id"].as_str())
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let permit = password_permit()?;
    let valid = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        use argon2::{PasswordHash, PasswordVerifier};
        PasswordHash::new(&old).is_ok_and(|h| {
            argon2::Argon2::default()
                .verify_password(current.as_bytes(), &h)
                .is_ok()
        })
    })
    .await
    .map_err(bad)?;
    if !valid {
        return Err(bad("invalid current password"));
    }
    let hashed = hash(new.into()).await?;
    let mut tx = a.pool.begin().await.map_err(db)?;
    sqlx::query("UPDATE users SET password=? WHERE id=?")
        .bind(hashed)
        .bind(u["id"].as_str())
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    sqlx::query("DELETE FROM sessions WHERE user_id=?")
        .bind(u["id"].as_str())
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(StatusCode::NO_CONTENT)
}
