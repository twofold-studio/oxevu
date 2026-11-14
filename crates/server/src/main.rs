use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
mod artwork;
mod assets;
mod episode_title;
mod scan;
type Error = (StatusCode, Json<Value>);
type Result<T> = std::result::Result<T, Error>;
fn bad(s: impl ToString) -> Error {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error":s.to_string()})),
    )
}
fn db(e: sqlx::Error) -> Error {
    eprintln!("database operation failed: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error":"database operation failed"})),
    )
}
#[derive(Clone)]
struct App {
    pool: SqlitePool,
    token: String,
    origin: String,
    scan: Arc<Semaphore>,
    artwork_jobs: Arc<Semaphore>,
    artwork_cache: PathBuf,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dir = std::env::var_os("OXEVU_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/oxevu")
        });
    tokio::fs::create_dir_all(&dir).await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(dir.join("oxevu.sqlite"))
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await?;
    sqlx::migrate!().run(&pool).await?;
    let token = uuid::Uuid::new_v4().to_string();
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(dir.join("setup-token"))?;
    use std::io::Write;
    f.write_all(token.as_bytes())?;
    let bind = std::env::var("OXEVU_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let address: std::net::SocketAddr = bind.parse()?;
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await?;
    anyhow::ensure!(
        users > 0 || address.ip().is_loopback(),
        "initial setup requires a loopback OXEVU_BIND"
    );
    let origin = std::env::var("OXEVU_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let app = App {
        pool,
        token,
        origin,
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: dir.join("artwork"),
    };
    let api = Router::new()
        .route("/auth/status", get(status))
        .route("/auth/setup", post(setup))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/libraries", get(libraries).post(add_library))
        .route("/libraries/{id}/scan", post(scan::scan))
        .route("/items", get(items))
        .route("/items/{id}", get(item))
        .route("/items/{id}/stream", get(stream))
        .route("/items/{id}/artwork/{kind}", get(artwork::serve))
        .route("/items/{id}/progress", put(progress));
    let router = Router::new()
        .nest("/api/v1", api)
        .route("/healthz", get(|| async { "ok" }))
        .fallback(assets::serve)
        .layer(middleware::from_fn_with_state(app.clone(), origin_check))
        .with_state(app);
    println!("Oxevu listening on {bind}");
    axum::serve(tokio::net::TcpListener::bind(bind).await?, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
async fn origin_check(State(a): State<App>, r: Request, n: Next) -> Response {
    if !matches!(
        *r.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    ) && r.headers().get("origin").and_then(|v| v.to_str().ok()) != Some(&a.origin)
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"same-origin Origin header required"})),
        )
            .into_response();
    }
    n.run(r).await
}
async fn user(a: &App, h: &HeaderMap) -> Result<Value> {
    let token = h
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';')
                .find_map(|s| s.trim().strip_prefix("oxevu_session="))
        })
        .unwrap_or("");
    let row=sqlx::query("SELECT users.id,username FROM sessions JOIN users ON users.id=sessions.user_id WHERE sessions.id=? AND expires>unixepoch()").bind(token).fetch_optional(&a.pool).await.map_err(db)?;
    row.map(|r| json!({"id":r.get::<String,_>(0),"username":r.get::<String,_>(1)}))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"sign in required"})),
        ))
}
async fn status(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    Ok(Json(
        json!({"setup_required":count==0,"user":user(&a,&h).await.ok()}),
    ))
}
async fn setup(State(a): State<App>, h: HeaderMap, Json(v): Json<Value>) -> Result<Response> {
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
    session(&a, id, name).await
}
fn credentials(v: &Value) -> Result<(&str, &str)> {
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
async fn hash(p: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
        argon2::Argon2::default()
            .hash_password(p.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|h| h.to_string())
            .map_err(|_| bad("password hashing failed"))
    })
    .await
    .map_err(bad)?
}
async fn login(State(a): State<App>, Json(v): Json<Value>) -> Result<Response> {
    let (n, p) = credentials(&v)?;
    let r = sqlx::query("SELECT id,password FROM users WHERE username=?")
        .bind(n)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("invalid credentials"))?;
    let hashed: String = r.get(1);
    let p = p.to_owned();
    let valid = tokio::task::spawn_blocking(move || {
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
    session(&a, r.get(0), n).await
}
async fn session(a: &App, id: String, name: &str) -> Result<Response> {
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
        Json(json!({"user":{"id":id,"username":name}})),
    )
        .into_response())
}
async fn logout(State(a): State<App>, h: HeaderMap) -> Result<Response> {
    let u = user(&a, &h).await?;
    sqlx::query("DELETE FROM sessions WHERE user_id=?")
        .bind(u["id"].as_str())
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
async fn libraries(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    user(&a, &h).await?;
    let rows = sqlx::query("SELECT * FROM libraries")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    Ok(Json(
        json!({"libraries":rows.iter().map(|r|json!({"id":r.get::<String,_>("id"),"name":r.get::<String,_>("name"),"path":r.get::<String,_>("path"),"kind":r.get::<String,_>("kind")})).collect::<Vec<_>>()}),
    ))
}
async fn add_library(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    user(&a, &h).await?;
    let name = v["name"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| bad("name required"))?;
    let kind = v["kind"]
        .as_str()
        .filter(|s| matches!(*s, "movies" | "tv"))
        .ok_or_else(|| bad("invalid kind"))?;
    let path = tokio::fs::canonicalize(v["path"].as_str().ok_or_else(|| bad("path required"))?)
        .await
        .map_err(bad)?;
    if !path.is_dir() {
        return Err(bad("root must be directory"));
    }
    let path = path.to_string_lossy();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO libraries VALUES(?,?,?,?)")
        .bind(&id)
        .bind(name)
        .bind(path.as_ref())
        .bind(kind)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    Ok(Json(
        json!({"library":{"id":id,"name":name,"path":path,"kind":kind}}),
    ))
}
async fn catalog(a: &App, h: &HeaderMap) -> Result<Vec<Value>> {
    let u = user(a, h).await?;
    let rows=sqlx::query("SELECT items.id,data,CAST(COALESCE(position,0.0) AS REAL) position,COALESCE(completed,0) completed FROM items LEFT JOIN progress ON item_id=items.id AND user_id=?").bind(u["id"].as_str()).fetch_all(&a.pool).await.map_err(db)?;
    rows.iter()
        .map(|r| {
            let mut v: Value = serde_json::from_str(r.get("data")).map_err(bad)?;
            for kind in ["poster", "thumbnail", "backdrop"] {
                v.as_object_mut()
                    .ok_or_else(|| bad("invalid catalog item"))?
                    .entry(format!("{kind}_url"))
                    .or_insert(Value::Null);
            }
            v["position_seconds"] = json!(r.try_get::<f64, _>("position").map_err(db)?);
            v["completed"] = json!(r.try_get::<bool, _>("completed").map_err(db)?);
            Ok(v)
        })
        .collect()
}
async fn items(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    Ok(Json(json!({"items":catalog(&a,&h).await?})))
}
async fn item(State(a): State<App>, h: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>> {
    Ok(Json(
        json!({"item":catalog(&a,&h).await?.into_iter().find(|v|v["id"]==id).ok_or_else(||(StatusCode::NOT_FOUND,Json(json!({"error":"item not found"}))))?}),
    ))
}
async fn progress(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<StatusCode> {
    let u = user(&a, &h).await?;
    let p = v["position_seconds"]
        .as_f64()
        .filter(|p| p.is_finite() && *p >= 0.)
        .ok_or_else(|| bad("invalid position"))?;
    let c = v["completed"]
        .as_bool()
        .ok_or_else(|| bad("completed required"))?;
    if sqlx::query_scalar::<_, i64>("SELECT count(*) FROM items WHERE id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?
        == 0
    {
        return Err(bad("unknown item"));
    }
    sqlx::query("INSERT INTO progress VALUES(?,?,?,?) ON CONFLICT(user_id,item_id) DO UPDATE SET position=excluded.position,completed=excluded.completed").bind(u["id"].as_str()).bind(id).bind(p).bind(c).execute(&a.pool).await.map_err(db)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn stream(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    r: Request,
) -> Result<Response> {
    user(&a, &h).await?;
    let row=sqlx::query("SELECT items.path,items.data,libraries.path root FROM items JOIN libraries ON library_id=libraries.id WHERE items.id=?").bind(id).fetch_optional(&a.pool).await.map_err(db)?.ok_or_else(||bad("unknown item"))?;
    let path = tokio::fs::canonicalize(row.get::<String, _>("path"))
        .await
        .map_err(bad)?;
    if !path.starts_with(PathBuf::from(row.get::<String, _>("root"))) {
        return Err(bad("file outside library"));
    }
    use tower_http::services::ServeFile;
    let data: Value = serde_json::from_str(row.get("data")).map_err(bad)?;
    let container = data["container"].as_str().unwrap_or("");
    let mime = if path.extension().is_some_and(|ext| ext == "webm") {
        "video/webm"
    } else if container.contains("matroska") {
        "video/x-matroska"
    } else if container.contains("mp4") || container.contains("mov") {
        "video/mp4"
    } else if container.contains("mpegts") {
        "video/mp2t"
    } else {
        "application/octet-stream"
    };
    Ok(ServeFile::new_with_mime(path, &mime.parse().map_err(bad)?)
        .try_call(r)
        .await
        .map_err(bad)?
        .into_response())
}

#[cfg(test)]
mod tests;
