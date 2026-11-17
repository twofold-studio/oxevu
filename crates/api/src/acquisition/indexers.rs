use super::*;
use oxevu_indexers::native::Provider;

static HTTP_JOBS: Semaphore = Semaphore::const_new(3);

#[derive(Clone, Deserialize, Serialize, Default, utoipa::ToSchema)]
#[serde(default, deny_unknown_fields)]
#[schema(as = IndexerInput)]
pub(super) struct Indexer {
    pub name: String,
    pub endpoint: String,
    pub api_key: String,
    pub enabled: bool,
    pub download_hosts: Vec<String>,
    #[schema(value_type = Option<crate::contract::acquisition_transfer_model::IndexerCapabilities>)]
    pub capabilities: Value,
}
impl Indexer {
    pub(super) fn provider(&self) -> Option<Provider> {
        Provider::from_endpoint(&self.endpoint)
    }

    fn validate(&self) -> Result<()> {
        let url = endpoint(&self.endpoint)?;
        if self.name.trim().is_empty()
            || self.name.len() > 100
            || self.api_key.len() > 4096
            || self.download_hosts.len() > 10
            || self
                .download_hosts
                .iter()
                .any(|h| h.len() > 253 || h.contains('/') || h.contains('@') || h.contains(':'))
            || url.query().is_some()
        {
            return Err(bad("invalid indexer configuration"));
        }
        Ok(())
    }
    fn response(&self, id: &str) -> Value {
        json!({"id":id,"name":self.name,"endpoint":self.endpoint,"enabled":self.enabled,
            "api_key_set":!self.api_key.is_empty(),"download_hosts":self.download_hosts,"capabilities":self.capabilities})
    }
}
pub(super) fn endpoint(raw: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(raw).map_err(|_| bad("invalid indexer URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || raw.len() > 4096
    {
        return Err(bad(
            "indexer URL must be HTTP(S) without embedded credentials or fragment",
        ));
    }
    Ok(url)
}
pub(super) async fn load(a: &App, id: &str) -> Result<Indexer> {
    let raw: Option<String> =
        sqlx::query_scalar("SELECT data FROM acquisition_indexers WHERE id=?")
            .bind(id)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?;
    serde_json::from_str(&raw.ok_or_else(|| bad("unknown indexer"))?)
        .map_err(|_| bad("invalid indexer configuration"))
}
pub(super) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "indexers").await?;
    let rows = sqlx::query("SELECT id,data FROM acquisition_indexers ORDER BY name")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    let mut indexers = Vec::new();
    for row in rows {
        let i: Indexer =
            serde_json::from_str(row.get("data")).map_err(|_| bad("invalid indexer"))?;
        indexers.push(i.response(row.get("id")));
    }
    Ok(Json(json!({"indexers":indexers})))
}
// Download managers need search names/counts, not endpoint configuration or keys.
pub(super) async fn sources(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let rows = sqlx::query("SELECT id,name FROM acquisition_indexers WHERE json_extract(data,'$.enabled')=1 ORDER BY name LIMIT 20").fetch_all(&a.pool).await.map_err(db)?;
    Ok(Json(
        json!({"sources":rows.into_iter().map(|row| json!({"indexer_id":row.get::<String,_>("id"),"indexer_name":row.get::<String,_>("name")})).collect::<Vec<_>>()}),
    ))
}
pub(super) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(mut i): Json<Indexer>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "indexers").await?;
    if id.is_empty() || id.len() > 100 {
        return Err(bad("invalid indexer id"));
    }
    i.validate()?;
    if let Ok(old) = load(&a, &id).await {
        if i.api_key.is_empty() {
            i.api_key = old.api_key;
        }
        i.capabilities = old.capabilities;
    }
    sqlx::query("INSERT INTO acquisition_indexers VALUES(?,?,?) ON CONFLICT(id) DO UPDATE SET name=excluded.name,data=excluded.data")
        .bind(&id).bind(&i.name).bind(serde_json::to_string(&i).map_err(bad)?).execute(&a.pool).await.map_err(db)?;
    Ok(Json(json!({"indexer":i.response(&id)})))
}
pub(super) async fn test(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "indexers").await?;
    let mut i = load(&a, &id).await?;
    i.capabilities = capabilities(&i).await?;
    sqlx::query("UPDATE acquisition_indexers SET data=? WHERE id=?")
        .bind(serde_json::to_string(&i).map_err(bad)?)
        .bind(&id)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    Ok(Json(json!({"indexer":i.response(&id)})))
}
pub(super) async fn capabilities(i: &Indexer) -> Result<Value> {
    if let Some(provider) = i.provider() {
        results(i, &[("limit", "1".into())]).await?;
        return Ok(json!({
            "search": true,
            "movie-search": true,
            "tv-search": provider != Provider::Yts,
            "max_limit": 100,
            "categories": if provider == Provider::Yts { vec!["2000"] } else { vec!["2000", "5000"] },
        }));
    }
    torznab::parse(&request(i, &[("t", "caps".into())]).await?, true)
}

pub(super) async fn results(i: &Indexer, params: &[(&str, String)]) -> Result<Value> {
    let bytes = request(i, params).await?;
    match i.provider() {
        Some(provider) => provider.parse(&bytes).map_err(bad),
        None => torznab::parse(&bytes, false),
    }
}

async fn request(i: &Indexer, params: &[(&str, String)]) -> Result<Vec<u8>> {
    let _permit = HTTP_JOBS
        .acquire()
        .await
        .map_err(|_| failure("indexer requests unavailable", true))?;
    let mut url = endpoint(&i.endpoint)?;
    let client = integrations::client()?;
    let request = if let Some(provider) = i.provider() {
        native_request(&client, provider, url, params)
    } else {
        {
            let mut query = url.query_pairs_mut();
            query.extend_pairs(params.iter().map(|(k, v)| (*k, v.as_str())));
            if !i.api_key.is_empty() {
                query.append_pair("apikey", &i.api_key);
            }
        }
        client.get(url)
    };
    let response = request
        .send()
        .await
        .map_err(|_| failure("indexer unavailable or request timed out", true))?;
    response_bytes(response).await
}

fn native_request(
    client: &reqwest::Client,
    provider: Provider,
    mut url: reqwest::Url,
    params: &[(&str, String)],
) -> reqwest::RequestBuilder {
    let get = |key| {
        params
            .iter()
            .find(|(name, _)| *name == key)
            .map_or("", |(_, value)| value.as_str())
    };
    let query = get("q");
    let offset = get("offset").parse::<u32>().unwrap_or(0);
    let limit = get("limit").parse::<u32>().unwrap_or(100).clamp(1, 100);
    match provider {
        Provider::Knaben => {
            let mut body = json!({
                "from": offset,
                "size": limit,
                "hide_unsafe": true,
                "hide_xxx": true,
            });
            // An empty query returns no hits. Omit it to request recent releases.
            if !query.is_empty() {
                body["query"] = json!(query);
                body["search_field"] = json!("title");
            }
            client.post(url).json(&body)
        }
        Provider::Nyaa => client
            .get(url)
            .query(&[("page", "rss"), ("q", query), ("c", "1_0")]),
        Provider::PirateBay if query.is_empty() => {
            url.set_path("/precompiled/data_top100_recent.json");
            client.get(url)
        }
        Provider::PirateBay => client.get(url).query(&[("q", query), ("cat", "200")]),
        Provider::Yts => {
            let page = (offset / 50 + 1).to_string();
            let count = limit.min(50).to_string();
            client.get(url).query(&[
                ("query_term", query),
                ("limit", count.as_str()),
                ("page", page.as_str()),
                ("sort_by", "date_added"),
            ])
        }
    }
}

pub(super) fn failure(message: &str, retryable: bool) -> Error {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error":message,"retryable":retryable})),
    )
}
pub(super) async fn response_bytes(response: reqwest::Response) -> Result<Vec<u8>> {
    let status = response.status();
    if status.is_redirection() {
        return Err(failure(
            "indexer returned a redirect to an unsupported location",
            false,
        ));
    }
    if status.as_u16() == 429 {
        return Err(failure("indexer is rate limiting (HTTP 429)", true));
    }
    if !status.is_success() {
        return Err(failure(
            &format!("indexer returned HTTP {}", status.as_u16()),
            status.is_server_error(),
        ));
    }
    integrations::bytes(response, 2 * 1024 * 1024).await
}
pub(super) enum TorrentSource {
    Bytes(Vec<u8>),
    Magnet(String),
}
pub(super) async fn fetch_torrent(i: &Indexer, raw: &str) -> Result<TorrentSource> {
    tokio::time::timeout(std::time::Duration::from_secs(30), fetch_source(i, raw))
        .await
        .map_err(|_| failure("torrent retrieval timed out", true))?
}
async fn fetch_source(i: &Indexer, raw: &str) -> Result<TorrentSource> {
    let _permit = HTTP_JOBS
        .acquire()
        .await
        .map_err(|_| failure("indexer requests unavailable", true))?;
    let base = endpoint(&i.endpoint)?;
    let mut url = base
        .join(raw)
        .map_err(|_| bad("invalid torrent download URL"))?;
    endpoint(url.as_str())?;
    if base.origin() != url.origin()
        && (url.scheme() != "https"
            || !i
                .download_hosts
                .iter()
                .any(|host| Some(host.as_str()) == url.host_str()))
    {
        return Err(bad(
            "torrent host is not allowed by this indexer configuration",
        ));
    }
    let origin = url.origin();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| bad("HTTP unavailable"))?;
    for hop in 0..=1 {
        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|_| failure("torrent retrieval failed or timed out", true))?;
        if matches!(response.status().as_u16(), 301 | 302 | 307 | 308) {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    failure(
                        "indexer returned a redirect to an unsupported location",
                        false,
                    )
                })?;
            if location.starts_with("magnet:?") {
                if location.len() > 16384
                    || librqbit::Magnet::parse(location)
                        .ok()
                        .and_then(|magnet| magnet.as_id20())
                        .is_none()
                {
                    return Err(bad("invalid or unsupported magnet redirect"));
                }
                return Ok(TorrentSource::Magnet(location.to_owned()));
            }
            let target = url
                .join(location)
                .map_err(|_| bad("invalid torrent redirect"))?;
            endpoint(target.as_str())?;
            if hop != 0 || target.origin() != origin {
                return Err(failure(
                    "indexer returned a redirect to an unsupported location",
                    false,
                ));
            }
            url = target;
            continue;
        }
        return Ok(TorrentSource::Bytes(response_bytes(response).await?));
    }
    Err(failure("indexer redirect limit reached", false))
}
