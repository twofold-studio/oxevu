use super::*;

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Search {
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub profile_id: Option<String>,
    pub offset: u32,
}
pub(super) async fn search(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(q): Json<Search>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    Ok(Json(search_for(&a, &id, &q).await?))
}
pub(super) async fn search_for(a: &App, id: &str, q: &Search) -> Result<Value> {
    if q.offset > 1000 || q.season.is_some_and(|n| n > 999) || q.episode.is_some_and(|n| n > 9999) {
        return Err(bad("invalid release search"));
    }
    let raw: Option<String> = sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id=?")
        .bind(id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    let title = decode(&raw.ok_or_else(|| bad("unknown title"))?)?;
    let profile = profiles::load(a, q.profile_id.as_deref().unwrap_or("default")).await?;
    let ids:Vec<String>=sqlx::query_scalar("SELECT id FROM acquisition_indexers WHERE json_extract(data,'$.enabled')=1 ORDER BY name LIMIT 20").fetch_all(&a.pool).await.map_err(db)?;
    let mut releases = Vec::new();
    let mut errors = Vec::new();
    for indexer_id in ids {
        let indexer = indexers::load(a, &indexer_id).await?;
        match search_indexer(a, &indexer_id, &indexer, &title, q, &profile).await {
            Ok(mut found) => releases.append(&mut found),
            Err(e) => errors.push(json!({"indexer_id":indexer_id,"error":e.1.0["error"]})),
        }
    }
    releases.sort_by(|a, b| {
        b["accepted"]
            .as_bool()
            .cmp(&a["accepted"].as_bool())
            .then_with(|| b["seeders"].as_u64().cmp(&a["seeders"].as_u64()))
    });
    // Releases not associated with jobs expire. Credentials stay server-side.
    sqlx::query("DELETE FROM acquisition_releases WHERE created_at<unixepoch()-604800 AND id NOT IN (SELECT release_id FROM acquisition_downloads WHERE release_id IS NOT NULL)").execute(&a.pool).await.map_err(db)?;
    Ok(json!({"releases":releases,"errors":errors,"offset":q.offset}))
}

async fn search_indexer(
    a: &App,
    indexer_id: &str,
    i: &indexers::Indexer,
    title: &Value,
    q: &Search,
    p: &profiles::Profile,
) -> Result<Vec<Value>> {
    let caps = if i.capabilities.is_object() {
        i.capabilities.clone()
    } else {
        indexers::capabilities(i).await?
    };
    let tv = title["kind"] == "tv";
    let mode = if tv && caps["tv-search"] == true {
        "tvsearch"
    } else if !tv && caps["movie-search"] == true {
        "movie"
    } else if caps["search"] == true {
        "search"
    } else {
        return Err(bad("indexer cannot search this content type"));
    };
    let mut query = title["title"]
        .as_str()
        .ok_or_else(|| bad("title missing"))?
        .to_owned();
    if !tv && let Some(year) = title["year"].as_str() {
        query.push_str(&format!(" {year}"));
    }
    if mode == "search"
        && let Some(season) = q.season
    {
        query.push_str(&format!(" S{season:02}"));
        if let Some(episode) = q.episode {
            query.push_str(&format!("E{episode:02}"));
        }
    }
    let mut params = vec![
        ("t", mode.into()),
        ("q", query),
        ("cat", if tv { "5000" } else { "2000" }.into()),
        ("extended", "1".into()),
        (
            "limit",
            caps["max_limit"]
                .as_u64()
                .unwrap_or(100)
                .min(100)
                .to_string(),
        ),
        ("offset", q.offset.to_string()),
    ];
    let supported = caps["tv-search_params"].as_str().unwrap_or("");
    if mode == "tvsearch" {
        for (name, value) in [("season", q.season), ("ep", q.episode)] {
            if supported.split(',').any(|p| p == name)
                && let Some(v) = value
            {
                params.push((name, v.to_string()));
            }
        }
    }
    let response = indexers::request(i, &params).await?;
    let parsed = torznab::parse(&response, false)?;
    store_results(a, indexer_id, i, title, &parsed, p).await
}

pub(super) async fn store_results(
    a: &App,
    indexer_id: &str,
    i: &indexers::Indexer,
    title: &Value,
    parsed: &Value,
    p: &profiles::Profile,
) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for raw in parsed.as_array().into_iter().flatten() {
        let Some(name) = raw["title"]
            .as_str()
            .filter(|s| s.len() <= 1000 && !s.is_empty())
        else {
            continue;
        };
        let mut r = raw.clone();
        if r["download_url"].is_null() {
            r["download_url"] = r["link"].clone();
        }
        if !r["download_url"].is_string() && !r["magneturl"].is_string() {
            continue;
        }
        let facts = release_facts::facts(name)?;
        for (key, value) in facts
            .as_object()
            .ok_or_else(|| bad("invalid release facts"))?
        {
            r[key] = value.clone();
        }
        let reasons = profiles::evaluate(p, &r);
        r["accepted"] = json!(reasons.is_empty());
        r["reasons"] = json!(reasons);
        r["identity_match"] = json!(release_facts::identity_matches(title, &r)?);
        r["indexer_id"] = json!(indexer_id);
        r["indexer_name"] = json!(i.name);
        let fingerprint = format!("{indexer_id}\0{}", r["guid"].as_str().unwrap_or(name));
        use sha2::{Digest, Sha256};
        let key = format!("{:x}", Sha256::digest(fingerprint.as_bytes()));
        r["id"] = json!(key);
        sqlx::query("INSERT INTO acquisition_releases(id,title_id,indexer_id,data) VALUES(?,?,?,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data,created_at=unixepoch()")
            .bind(&key).bind(title["id"].as_str()).bind(indexer_id).bind(r.to_string()).execute(&a.pool).await.map_err(db)?;
        let blocked: bool = sqlx::query_scalar(
            "SELECT blocked_until>unixepoch() FROM acquisition_releases WHERE id=?",
        )
        .bind(&key)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
        r["blocked"] = json!(blocked);
        if blocked {
            r["accepted"] = json!(false);
            r["reasons"]
                .as_array_mut()
                .ok_or_else(|| bad("invalid release evaluation"))?
                .push(json!("Previously failed release is temporarily blocked"));
        }
        result.push(public(&r));
    }
    Ok(result)
}

pub(super) fn public(raw: &Value) -> Value {
    let mut v = raw.clone();
    if let Some(map) = v.as_object_mut() {
        for field in ["download_url", "link", "guid", "magneturl"] {
            map.remove(field);
        }
    }
    v
}
