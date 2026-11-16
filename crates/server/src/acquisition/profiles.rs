use super::*;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Profile {
    pub name: String,
    pub min_height: u32,
    pub max_height: u32,
    pub min_size_mb: u32,
    pub max_size_mb: u32,
    pub min_seeders: u32,
    pub codecs: Vec<String>,
    pub sources: Vec<String>,
    pub languages: Vec<String>,
    pub allow_hdr: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "1080p".into(),
            min_height: 720,
            max_height: 1080,
            min_size_mb: 0,
            max_size_mb: 20480,
            min_seeders: 1,
            codecs: vec!["h264".into(), "hevc".into(), "av1".into()],
            sources: Vec::new(),
            languages: Vec::new(),
            allow_hdr: false,
        }
    }
}
pub(super) async fn load(a: &App, id: &str) -> Result<Profile> {
    let raw: Option<String> =
        sqlx::query_scalar("SELECT data FROM acquisition_profiles WHERE id=?")
            .bind(id)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?;
    match raw {
        Some(raw) => serde_json::from_str(&raw).map_err(|_| bad("invalid acquisition profile")),
        None if id == "default" => Ok(Profile::default()),
        _ => Err(bad("unknown acquisition profile")),
    }
}
pub(super) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let rows = sqlx::query("SELECT id,data FROM acquisition_profiles ORDER BY name")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    let mut profiles = vec![json!({"id":"default","profile":load(&a,"default").await?})];
    for row in rows {
        let id: String = row.get("id");
        if id != "default" {
            profiles.push(json!({"id":id,"profile":decode(row.get("data"))?}));
        }
    }
    Ok(Json(json!({"profiles":profiles})))
}
pub(super) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(p): Json<Profile>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    if id.is_empty()
        || id.len() > 100
        || p.name.is_empty()
        || p.name.len() > 100
        || p.min_height > p.max_height
        || p.max_height > 4320
        || p.min_size_mb > p.max_size_mb
        || p.max_size_mb > 1_000_000
        || p.codecs.len() > 10
        || p.sources.len() > 20
        || p.languages.len() > 20
        || p.min_seeders > 100000
        || p.codecs
            .iter()
            .chain(&p.sources)
            .chain(&p.languages)
            .any(|s| s.len() > 50)
    {
        return Err(bad("invalid acquisition profile"));
    }
    sqlx::query("INSERT INTO acquisition_profiles VALUES(?,?,?) ON CONFLICT(id) DO UPDATE SET name=excluded.name,data=excluded.data")
        .bind(&id).bind(&p.name).bind(serde_json::to_string(&p).map_err(bad)?).execute(&a.pool).await.map_err(db)?;
    Ok(Json(json!({"id":id,"profile":p})))
}

pub(super) fn evaluate(p: &Profile, r: &Value) -> Vec<String> {
    let mut reasons = Vec::new();
    match r["height"].as_u64() {
        Some(n) if n >= p.min_height as u64 && n <= p.max_height as u64 => {}
        Some(_) => reasons.push("Resolution outside profile".into()),
        None => reasons.push("Resolution unknown".into()),
    }
    match r["size"].as_u64() {
        Some(n)
            if n >= p.min_size_mb as u64 * 1024 * 1024
                && n <= p.max_size_mb as u64 * 1024 * 1024 => {}
        Some(_) => reasons.push("Size outside profile".into()),
        None => reasons.push("Size unknown".into()),
    }
    if !r["seeders"]
        .as_u64()
        .is_some_and(|n| n >= p.min_seeders as u64)
    {
        reasons.push("Insufficient or unknown seeders".into());
    }
    for (field, allowed) in [
        ("codec", &p.codecs),
        ("source", &p.sources),
        ("language", &p.languages),
    ] {
        if !allowed.is_empty()
            && !r[field]
                .as_str()
                .is_some_and(|v| allowed.iter().any(|a| a.eq_ignore_ascii_case(v)))
        {
            reasons.push(format!("{field} not allowed or unknown"));
        }
    }
    if r["hdr"] == true && !p.allow_hdr {
        reasons.push("HDR disabled by profile".into());
    }
    reasons
}
