use super::*;

static HTTP_JOBS: Semaphore = Semaphore::const_new(3);

#[derive(Clone, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Indexer {
    pub name: String,
    pub endpoint: String,
    pub api_key: String,
    pub enabled: bool,
    pub download_hosts: Vec<String>,
    pub capabilities: Value,
}
impl Indexer {
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
    torznab::parse(&request(i, &[("t", "caps".into())]).await?, true)
}
pub(super) async fn request(i: &Indexer, params: &[(&str, String)]) -> Result<Vec<u8>> {
    let _permit = HTTP_JOBS
        .try_acquire()
        .map_err(|_| bad("indexer request limit reached"))?;
    let mut url = endpoint(&i.endpoint)?;
    {
        let mut query = url.query_pairs_mut();
        query.extend_pairs(params.iter().map(|(k, v)| (*k, v.as_str())));
        if !i.api_key.is_empty() {
            query.append_pair("apikey", &i.api_key);
        }
    }
    let response = integrations::client()?
        .get(url)
        .send()
        .await
        .map_err(|_| bad("indexer unavailable"))?;
    integrations::bytes(response, 2 * 1024 * 1024).await
}
pub(super) async fn fetch_torrent(i: &Indexer, raw: &str) -> Result<Vec<u8>> {
    let _permit = HTTP_JOBS
        .try_acquire()
        .map_err(|_| bad("indexer request limit reached"))?;
    let base = endpoint(&i.endpoint)?;
    let url = base
        .join(raw)
        .map_err(|_| bad("invalid torrent download URL"))?;
    endpoint(url.as_str())?;
    let same_origin = base.origin() == url.origin();
    if !same_origin
        && (!i
            .download_hosts
            .iter()
            .any(|h| Some(h.as_str()) == url.host_str())
            || url.scheme() != "https")
    {
        return Err(bad(
            "torrent host is not allowed by this indexer configuration",
        ));
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| bad("HTTP unavailable"))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| bad("torrent retrieval failed"))?;
    integrations::bytes(response, 2 * 1024 * 1024).await
}
