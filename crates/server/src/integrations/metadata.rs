use super::*;
use axum::extract::Query;
#[derive(Deserialize)]
pub(crate) struct Search {
    provider: String,
    query: String,
}
async fn tmdb(c: &Config, route: &str, query: &[(&str, String)]) -> Result<Value> {
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
            data["results"].as_array().into_iter().flatten().take(20).map(|v|json!({"provider":"tmdb","id":v["id"].as_u64().map(|n|n.to_string()),"title":if tv{&v["name"]}else{&v["title"]},"year":if tv{v["first_air_date"].as_str()}else{v["release_date"].as_str()}.map(|s|s.chars().take(4).collect::<String>()),"kind":if tv{"tv"}else{"movie"},"overview":v["overview"],"poster_url":null})).collect::<Vec<_>>()
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
            data["Search"].as_array().into_iter().flatten().take(20).map(|v|json!({"provider":"omdb","id":v["imdbID"],"title":v["Title"],"year":v["Year"],"kind":if tv{"tv"}else{"movie"},"overview":null,"poster_url":null})).collect()
        }
        _ => return Err(bad("unsupported metadata provider")),
    };
    Ok(Json(json!({"results":result})))
}
fn strings(v: &Value, key: &str) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v[key].as_str().map(str::to_owned))
        .collect()
}
fn csv(v: &Value) -> Vec<String> {
    v.as_str()
        .filter(|s| *s != "N/A")
        .map(|s| s.split(", ").map(str::to_owned).collect())
        .unwrap_or_default()
}
fn date_year(v: &Value) -> Option<u32> {
    v.as_str()?.get(..4)?.parse().ok()
}
fn rating(v: &Value) -> Option<f64> {
    v.as_str()?.parse().ok()
}
fn image(v: &Value) -> Option<String> {
    v.as_str()
        .filter(|p| p.starts_with('/') && !p.contains("..") && p.len() < 200)
        .map(str::to_owned)
}
#[derive(Deserialize)]
pub(crate) struct Match {
    provider: String,
    id: String,
}
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
    let tv = item["kind"] == "episode";
    let c = load(&a).await?;
    let mut enrichment = match m.provider.as_str() {
        "tmdb" => {
            let number = m.id.parse::<u64>().map_err(|_| bad("invalid TMDB id"))?;
            let data = tmdb(
                &c,
                &format!("{}/{number}", if tv { "tv" } else { "movie" }),
                &[(
                    "append_to_response",
                    "credits,external_ids,release_dates,content_ratings".into(),
                )],
            )
            .await?;
            let imdb = data["imdb_id"]
                .as_str()
                .or_else(|| data["external_ids"]["imdb_id"].as_str());
            let creators = if tv {
                strings(&data["created_by"], "name")
            } else {
                data["credits"]["crew"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|v| v["job"] == "Director")
                    .filter_map(|v| v["name"].as_str().map(str::to_owned))
                    .collect()
            };
            json!({"title":if tv{&data["name"]}else{&data["title"]},"overview":data["overview"],"year":date_year(if tv{&data["first_air_date"]}else{&data["release_date"]}),"end_year":date_year(&data["last_air_date"]),"rating":data["vote_average"],"genres":strings(&data["genres"],"name"),"studios":strings(&data["production_companies"],"name"),"creators":creators,"provider_ids":{"tmdb":number,"imdb":imdb},"external_urls":{"tmdb":format!("https://www.themoviedb.org/{}/{number}",if tv{"tv"}else{"movie"}),"imdb":imdb.map(|s|format!("https://www.imdb.com/title/{s}/"))},"metadata_poster":image(&data["poster_path"]),"metadata_backdrop":image(&data["backdrop_path"]),"metadata_attribution":"This product uses the TMDB API but is not endorsed or certified by TMDB."})
        }
        "omdb" => {
            if !m.id.starts_with("tt")
                || !m.id[2..].bytes().all(|b| b.is_ascii_digit())
                || m.id.len() > 20
            {
                return Err(bad("invalid IMDb id"));
            }
            let data = omdb(&c, &[("i", m.id.clone()), ("plot", "full".into())]).await?;
            json!({"title":data["Title"],"overview":data["Plot"],"year":date_year(&data["Year"]),"rating":rating(&data["imdbRating"]),"content_rating":data["Rated"],"genres":csv(&data["Genre"]),"studios":csv(&data["Production"]),"creators":csv(&data["Director"]),"provider_ids":{"imdb":m.id},"external_urls":{"imdb":format!("https://www.imdb.com/title/{}/",m.id)}})
        }
        _ => return Err(bad("unsupported metadata provider")),
    };
    if tv {
        for field in [
            "overview",
            "year",
            "end_year",
            "rating",
            "genres",
            "studios",
            "creators",
            "content_rating",
        ] {
            enrichment[format!("show_{field}")] = enrichment[field].clone();
        }
    }
    let rows = if tv {
        sqlx::query("SELECT id,data FROM items WHERE library_id=? AND json_extract(data,'$.show_title')=? ORDER BY id LIMIT 101").bind(item["library_id"].as_str()).bind(item["show_title"].as_str()).fetch_all(&a.pool).await.map_err(db)?
    } else {
        sqlx::query("SELECT id,data FROM items WHERE id=?")
            .bind(&id)
            .fetch_all(&a.pool)
            .await
            .map_err(db)?
    };
    if rows.len() > 100 {
        return Err(bad("series metadata limit is 100 episodes per match"));
    }
    let mut updates = Vec::new();
    for row in rows {
        let source: Value =
            serde_json::from_str(row.get("data")).map_err(|_| bad("invalid item metadata"))?;
        if tv && source["show_title"] != item["show_title"] {
            continue;
        }
        if updates.len() >= 100 {
            return Err(bad("series metadata limit is 100 episodes per match"));
        }
        let mut data = enrichment.clone();
        if tv {
            data["show_title"] = source["show_title"].clone();
            data["title"] = source["title"].clone();
            if m.provider == "tmdb"
                && let (Some(season), Some(episode)) = (
                    source["season_number"].as_u64(),
                    source["episode_number"].as_u64(),
                )
            {
                let episode = tmdb(
                    &c,
                    &format!("tv/{}/season/{season}/episode/{episode}", m.id),
                    &[],
                )
                .await?;
                data["title"] = episode["name"].clone();
                data["overview"] = episode["overview"].clone();
                data["metadata_thumbnail"] = json!(image(&episode["still_path"]));
            }
        }
        let item_id: String = row.get("id");
        for (field, kind) in [
            ("metadata_poster", "poster"),
            ("metadata_backdrop", "backdrop"),
            ("metadata_thumbnail", "thumbnail"),
        ] {
            if data[field].is_string() {
                data[format!("{kind}_url")] =
                    json!(format!("/api/v1/items/{item_id}/metadata/artwork/{kind}"));
            }
        }
        data["studio_images"]=json!(data["studios"].as_array().into_iter().flatten().filter_map(|s|s.as_str()).map(|name|json!({"name":name,"url":format!("/api/v1/items/{item_id}/metadata/studios/{}",studios::slug(name))})).collect::<Vec<_>>());
        updates.push((item_id, data));
    }
    let mut tx = a.pool.begin().await.map_err(db)?;
    for (item_id, data) in updates {
        sqlx::query("INSERT INTO item_metadata VALUES(?,?) ON CONFLICT(item_id) DO UPDATE SET data=excluded.data").bind(item_id).bind(data.to_string()).execute(&mut *tx).await.map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    catalog::item(State(a), h, Path(id)).await
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
