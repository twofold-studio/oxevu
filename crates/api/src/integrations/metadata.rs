use super::*;
use axum::extract::Query;
#[derive(Deserialize)]
pub(crate) struct Search {
    provider: String,
    query: String,
}
pub(crate) async fn tmdb(c: &Config, route: &str, query: &[(&str, String)]) -> Result<Value> {
    if !c.tmdb.enabled || c.tmdb.api_key.is_empty() {
        return Err(bad("TMDB requires an enabled application API key"));
    }
    let mut query = query.to_vec();
    let mut request = client()?.get(format!("https://api.themoviedb.org/3/{route}"));
    if c.tmdb.api_key.starts_with("eyJ") {
        request = request.bearer_auth(&c.tmdb.api_key);
    } else {
        query.push(("api_key", c.tmdb.api_key.clone()));
    }
    json_response(
        request
            .query(&query)
            .send()
            .await
            .map_err(|_| bad("TMDB request failed"))?,
    )
    .await
}
async fn omdb(c: &Config, query: &[(&str, String)]) -> Result<Value> {
    if !c.omdb.enabled || c.omdb.api_key.is_empty() {
        return Err(bad("OMDB requires an enabled API key"));
    }
    let mut query = query.to_vec();
    query.push(("apikey", c.omdb.api_key.clone()));
    let data = json_response(
        client()?
            .get("https://www.omdbapi.com/")
            .query(&query)
            .send()
            .await
            .map_err(|_| bad("OMDB request failed"))?,
    )
    .await?;
    if data["Response"] == "False" {
        return Err(bad("OMDB found no matching title"));
    }
    Ok(data)
}
pub(crate) async fn search(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<Search>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    if q.query.trim().is_empty() || q.query.len() > 200 {
        return Err(bad("query must contain 1 to 200 characters"));
    }
    let item = item_data(&a, &h, &id).await?;
    let c = load(&a).await?;
    let tv = item["kind"] == "episode";
    let result = match q.provider.as_str() {
        "tmdb" => {
            let data = tmdb(
                &c,
                if tv { "search/tv" } else { "search/movie" },
                &[("query", q.query), ("include_adult", "false".into())],
            )
            .await?;
            data["results"].as_array().into_iter().flatten().take(20).map(|v|json!({"provider":"tmdb","id":v["id"].as_u64().map(|n|n.to_string()),"title":if tv{&v["name"]}else{&v["title"]},"year":if tv{v["first_air_date"].as_str()}else{v["release_date"].as_str()}.map(|s|s.chars().take(4).collect::<String>()),"kind":if tv{"tv"}else{"movie"},"overview":v["overview"],"poster_url":image(&v["poster_path"]).map(|path|format!("https://image.tmdb.org/t/p/w185{path}"))})).collect::<Vec<_>>()
        }
        "omdb" => {
            let data = omdb(
                &c,
                &[
                    ("s", q.query),
                    ("type", if tv { "series" } else { "movie" }.into()),
                ],
            )
            .await?;
            data["Search"].as_array().into_iter().flatten().take(20).map(|v|json!({"provider":"omdb","id":v["imdbID"],"title":v["Title"],"year":v["Year"],"kind":if tv{"tv"}else{"movie"},"overview":null,"poster_url":v["Poster"].as_str().filter(|url|url.starts_with("https://") && url.len() <= 2048)})).collect()
        }
        _ => return Err(bad("unsupported metadata provider")),
    };
    Ok(Json(json!({"results":result})))
}
pub(crate) use crate::library::identification::Selection as Match;
use crate::library::identification::image;
pub(crate) async fn apply(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(m): Json<Match>,
) -> Result<Json<Value>> {
    tokio::time::timeout(std::time::Duration::from_secs(90), apply_inner(a, h, id, m))
        .await
        .map_err(|_| bad("metadata matching timed out"))?
}
async fn apply_inner(a: App, h: HeaderMap, id: String, m: Match) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let _permit = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("metadata or scan job already running"))?;
    let item = item_data(&a, &h, &id).await?;
    let provider = ProviderSource(load(&a).await?);
    crate::library::identification::apply(&a.pool, &id, item, m, &provider)
        .await
        .map_err(crate::library::response)?;
    catalog::item(State(a), h, Path(id)).await
}
struct ProviderSource(Config);
fn provider_error(error: crate::Error) -> crate::library::Error {
    crate::library::Error::Invalid(
        error.1.0["error"]
            .as_str()
            .unwrap_or("metadata provider failed")
            .to_owned(),
    )
}
impl crate::library::identification::MetadataSource for ProviderSource {
    fn tmdb<'a>(
        &'a self,
        route: &'a str,
        query: &'a [(&'a str, String)],
    ) -> crate::library::identification::MetadataFuture<'a> {
        Box::pin(async move { tmdb(&self.0, route, query).await.map_err(provider_error) })
    }
    fn omdb<'a>(
        &'a self,
        query: &'a [(&'a str, String)],
    ) -> crate::library::identification::MetadataFuture<'a> {
        Box::pin(async move { omdb(&self.0, query).await.map_err(provider_error) })
    }
}
pub(crate) async fn artwork(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, kind)): Path<(String, String)>,
) -> Result<Response> {
    item_data(&a, &h, &id).await?;
    let field = match kind.as_str() {
        "poster" => "metadata_poster",
        "backdrop" => "metadata_backdrop",
        "thumbnail" => "metadata_thumbnail",
        _ => return Err(bad("unknown artwork kind")),
    };
    let raw: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let data: Value = serde_json::from_str(&raw).map_err(|_| bad("invalid metadata"))?;
    let image = image(&data[field]).ok_or_else(|| bad("artwork unavailable"))?;
    let response = client()?
        .get(format!("https://image.tmdb.org/t/p/w780{image}"))
        .send()
        .await
        .map_err(|_| bad("artwork request failed"))?;
    let bytes = bytes(response, 8 * 1024 * 1024).await?;
    if !bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Err(bad("invalid provider artwork"));
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
