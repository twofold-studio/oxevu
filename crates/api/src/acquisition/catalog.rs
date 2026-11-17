use super::*;
static PROVIDER_JOBS: Semaphore = Semaphore::const_new(3);

#[derive(Deserialize)]
pub(super) struct Search {
    #[serde(default)]
    pub query: String,
    #[serde(default = "movie")]
    pub kind: String,
    #[serde(default = "first_page")]
    pub page: u32,
}
fn movie() -> String {
    "movie".into()
}
fn first_page() -> u32 {
    1
}

pub(super) async fn discover(
    State(a): State<App>,
    h: HeaderMap,
    Query(q): Query<Search>,
) -> Result<Json<Value>> {
    user(&a, &h).await?;
    title_key(&q.kind, 1)?;
    if q.query.len() > 200 || !(1..=20).contains(&q.page) {
        return Err(bad("invalid catalog search"));
    }
    let _permit = PROVIDER_JOBS
        .try_acquire()
        .map_err(|_| bad("catalog busy; try again"))?;
    let config = integrations::load(&a).await?;
    let route = if q.query.trim().is_empty() {
        format!("{}/popular", q.kind)
    } else {
        format!("search/{}", q.kind)
    };
    let data = integrations::tmdb(
        &config,
        &route,
        &[
            ("query", q.query),
            ("page", q.page.to_string()),
            ("include_adult", "false".into()),
        ],
    )
    .await?;
    let local = crate::catalog::catalog(&a, &h).await?;
    let mut results = Vec::new();
    for value in data["results"].as_array().into_iter().flatten().take(20) {
        let Some(id) = value["id"].as_u64() else {
            continue;
        };
        let mut title = summary(&q.kind, id, value)?;
        title["availability"] = availability(&a, &title, &local).await?;
        results.push(title);
    }
    Ok(Json(
        json!({"results":results,"page":q.page,"total_pages":data["total_pages"],"attribution":"This product uses the TMDB API but is not endorsed or certified by TMDB."}),
    ))
}

pub(super) fn summary(kind: &str, id: u64, data: &Value) -> Result<Value> {
    let key = title_key(kind, id)?;
    let name = if kind == "tv" {
        &data["name"]
    } else {
        &data["title"]
    };
    if !name
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 500)
    {
        return Err(bad("provider returned invalid title"));
    }
    let date = if kind == "tv" {
        &data["first_air_date"]
    } else {
        &data["release_date"]
    };
    let poster = data["poster_path"]
        .as_str()
        .and_then(|path| path.strip_prefix('/'))
        .filter(|path| valid_image(path))
        .map(|path| format!("/api/v1/acquisition/artwork/{path}"));
    let backdrop = data["backdrop_path"]
        .as_str()
        .and_then(|path| path.strip_prefix('/'))
        .filter(|path| valid_image(path))
        .map(|path| format!("/api/v1/acquisition/artwork/{path}"));
    Ok(
        json!({"id":key,"provider":"tmdb","provider_id":id,"kind":kind,"title":name,
        "year":date.as_str().and_then(|s|s.get(..4)),"overview":data["overview"],
        "poster_url":poster,"backdrop_url":backdrop,"rating":data["vote_average"],"genres":data["genres"],
        "release_date":date,"seasons":data["seasons"],"status":data["status"]}),
    )
}

pub(super) async fn ensure_title(a: &App, kind: &str, id: u64) -> Result<Value> {
    let key = title_key(kind, id)?;
    if let Some(raw) =
        sqlx::query_scalar::<_, String>("SELECT data FROM acquisition_titles WHERE id=?")
            .bind(key)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?
    {
        return decode(&raw);
    }
    refresh(a, kind, id).await
}

pub(super) async fn refresh(a: &App, kind: &str, id: u64) -> Result<Value> {
    let _permit = PROVIDER_JOBS
        .try_acquire()
        .map_err(|_| bad("catalog busy; try again"))?;
    let config = integrations::load(a).await?;
    let data = integrations::tmdb(&config, &format!("{kind}/{id}"), &[]).await?;
    let title = summary(kind, id, &data)?;
    let mut episodes = Vec::new();
    if kind == "tv" {
        let seasons = data["seasons"]
            .as_array()
            .ok_or_else(|| bad("provider returned no seasons"))?;
        if seasons.len() > 100 {
            return Err(bad("series exceeds 100-season catalog limit"));
        }
        for season in seasons {
            let Some(number) = season["season_number"].as_u64().filter(|n| *n <= 999) else {
                continue;
            };
            let details =
                integrations::tmdb(&config, &format!("tv/{id}/season/{number}"), &[]).await?;
            for episode in details["episodes"].as_array().into_iter().flatten() {
                if episodes.len() >= 10000 {
                    return Err(bad("series exceeds episode limit"));
                }
                if let Some(n) = episode["episode_number"].as_u64().filter(|n| *n <= 9999) {
                    episodes.push((number, n, episode.clone()));
                }
            }
        }
    }
    let mut tx = a.pool.begin().await.map_err(db)?;
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES(?,?,?,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data,refreshed_at=unixepoch()")
        .bind(title["id"].as_str()).bind(kind).bind(id as i64).bind(title.to_string()).execute(&mut *tx).await.map_err(db)?;
    for (season, episode, value) in episodes {
        sqlx::query("INSERT INTO acquisition_episodes VALUES(?,?,?,?,?) ON CONFLICT(title_id,season,episode) DO UPDATE SET data=excluded.data,air_date=excluded.air_date")
            .bind(title["id"].as_str()).bind(season as i64).bind(episode as i64).bind(value.to_string()).bind(value["air_date"].as_str()).execute(&mut *tx).await.map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    Ok(title)
}

pub(super) async fn details(
    State(a): State<App>,
    h: HeaderMap,
    Path((kind, id)): Path<(String, u64)>,
) -> Result<Json<Value>> {
    user(&a, &h).await?;
    let mut title = ensure_title(&a, &kind, id).await?;
    let local = crate::catalog::catalog(&a, &h).await?;
    title["availability"] = availability(&a, &title, &local).await?;
    let rows = sqlx::query("SELECT season,episode,data,air_date FROM acquisition_episodes WHERE title_id=? ORDER BY season,episode")
        .bind(title["id"].as_str()).fetch_all(&a.pool).await.map_err(db)?;
    let mut episodes = Vec::new();
    for r in rows {
        let mut v = decode(r.get("data"))?;
        let season: i64 = r.get("season");
        let episode: i64 = r.get("episode");
        v["available"] = json!(local.iter().any(|item| matches_item(&title, item)
            && wanted::covers(item, season as u32, episode as u32)));
        episodes.push(v);
    }
    Ok(Json(json!({"title":title,"episodes":episodes})))
}

pub(super) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "requests").await?;
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT data FROM acquisition_titles ORDER BY refreshed_at DESC LIMIT 1000",
    )
    .fetch_all(&a.pool)
    .await
    .map_err(db)?;
    let local = crate::catalog::catalog(&a, &h).await?;
    let mut titles = Vec::new();
    for raw in rows {
        let mut t = decode(&raw)?;
        t["availability"] = availability(&a, &t, &local).await?;
        titles.push(t);
    }
    Ok(Json(json!({"titles":titles})))
}

pub(super) fn matches_item(title: &Value, item: &Value) -> bool {
    let kind = if item["kind"] == "episode" {
        "tv"
    } else {
        "movie"
    };
    let provider = &item["provider_ids"]["tmdb"];
    let id = provider
        .as_u64()
        .or_else(|| provider.as_str()?.parse().ok());
    item["file_available"] != false
        && kind == title["kind"]
        && id.is_some()
        && id == title["provider_id"].as_u64()
}

pub(super) async fn availability(a: &App, title: &Value, local: &[Value]) -> Result<Value> {
    let mut available = Vec::new();
    let mut offline = false;
    for item in local.iter().filter(|item| matches_item(title, item)) {
        let path: Option<String> = sqlx::query_scalar("SELECT path FROM items WHERE id=?")
            .bind(item["id"].as_str())
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?;
        if let Some(path) = path {
            if tokio::fs::metadata(path).await.is_ok() {
                available.push(item.clone());
            } else {
                offline = true;
            }
        }
    }
    let requested: i64=sqlx::query_scalar("SELECT count(*) FROM acquisition_requests WHERE title_id=? AND status IN ('pending','approved')")
        .bind(title["id"].as_str()).fetch_one(&a.pool).await.map_err(db)?;
    Ok(
        json!({"items":available.iter().map(|i|json!({"id":i["id"],"season":i["season_number"],"episode":i["episode_number"]})).collect::<Vec<_>>(),"offline":offline,"requested":requested>0}),
    )
}

pub(super) fn valid_image(file: &str) -> bool {
    file.len() <= 150
        && file.ends_with(".jpg")
        && file
            .trim_end_matches(".jpg")
            .bytes()
            .all(|b| b.is_ascii_alphanumeric())
}
pub(super) async fn artwork(
    State(a): State<App>,
    h: HeaderMap,
    Path(file): Path<String>,
) -> Result<Response> {
    user(&a, &h).await?;
    if !valid_image(&file) {
        return Err(bad("invalid artwork"));
    }
    let response = integrations::client()?
        .get(format!("https://image.tmdb.org/t/p/w342/{file}"))
        .send()
        .await
        .map_err(|_| bad("artwork unavailable"))?;
    let bytes = integrations::bytes(response, 8 * 1024 * 1024).await?;
    if !bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Err(bad("invalid artwork response"));
    }
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "image/jpeg"),
            (axum::http::header::CACHE_CONTROL, "private, max-age=86400"),
        ],
        bytes,
    )
        .into_response())
}
