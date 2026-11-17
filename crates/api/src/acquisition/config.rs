use super::*;

#[derive(Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(default, deny_unknown_fields)]
#[schema(as = AcquisitionConfig)]
pub(super) struct Config {
    #[schema(required)]
    pub enabled: bool,
    #[schema(required)]
    pub download_dir: String,
    #[schema(required)]
    pub listen_port: u16,
    #[schema(required)]
    pub bind_interface: Option<String>,
    #[schema(required)]
    pub dht: bool,
    #[schema(required)]
    pub max_active_downloads: u32,
    #[schema(required)]
    pub download_limit_kib: u32,
    #[schema(required)]
    pub upload_limit_kib: u32,
    #[schema(required)]
    pub min_free_gb: u32,
    #[schema(required)]
    pub storage_limit_gb: u32,
    #[schema(required)]
    pub seed_ratio: f64,
    #[schema(required)]
    pub seed_hours: u32,
    #[schema(required)]
    pub search_interval_minutes: u32,
    #[schema(required)]
    pub import_enabled: bool,
    #[schema(required)]
    pub copy_fallback: bool,
    #[schema(required)]
    pub auto_identify: bool,
    #[schema(required)]
    pub movie_destination: Option<destinations::Destination>,
    #[schema(required)]
    pub series_destination: Option<destinations::Destination>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            download_dir: String::new(),
            listen_port: 6882,
            bind_interface: None,
            dht: true,
            max_active_downloads: 2,
            download_limit_kib: 0,
            upload_limit_kib: 0,
            min_free_gb: 10,
            storage_limit_gb: 100,
            seed_ratio: 1.0,
            seed_hours: 72,
            search_interval_minutes: 60,
            import_enabled: false,
            copy_fallback: false,
            auto_identify: true,
            movie_destination: None,
            series_destination: None,
        }
    }
}
pub(super) async fn load(a: &App) -> Result<Config> {
    let raw: Option<String> = sqlx::query_scalar("SELECT data FROM acquisition_config WHERE id=1")
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    let mut c: Config = raw
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|_| bad("invalid acquisition settings"))?
        .unwrap_or_default();
    if c.download_dir.is_empty() {
        c.download_dir = a
            .acquisition
            .directory
            .join("downloads")
            .to_string_lossy()
            .into_owned();
    }
    Ok(c)
}
pub(super) async fn get(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    Ok(Json(
        json!({"settings":load(&a).await?,"engine":"librqbit 9.1.0","network_policy":"direct_allowed"}),
    ))
}
pub(super) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Json(mut c): Json<Config>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    validate(&c)?;
    let old = load(&a).await?;
    for (kind, target, previous) in [
        ("movies", &mut c.movie_destination, &old.movie_destination),
        ("tv", &mut c.series_destination, &old.series_destination),
    ] {
        if let Some(target) = target {
            if let Some(previous) = previous
                .as_ref()
                .filter(|old| old.library_id == target.library_id && old.root == target.root)
            {
                // Unrelated settings, including disabling imports, remain editable
                // while a saved destination is offline. Keep its original mount pin.
                target.root_device = previous.root_device;
            } else {
                destinations::validate(&a, kind, target).await?;
            }
        }
    }
    let root = super::storage::canonical_candidate(std::path::Path::new(&c.download_dir)).await?;
    super::storage::validate_download_root(&a, &root).await?;
    c.download_dir = root.to_string_lossy().into_owned();
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM acquisition_downloads WHERE state!='removed'")
            .fetch_one(&a.pool)
            .await
            .map_err(db)?;
    if n > 0 && old.download_dir != c.download_dir {
        return Err(bad("remove existing jobs before changing download storage"));
    }
    let _gate = a.acquisition.gate.lock().await;
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data")
        .bind(serde_json::to_string(&c).map_err(bad)?).execute(&a.pool).await.map_err(db)?;
    if old.enabled != c.enabled
        || old.download_dir != c.download_dir
        || old.listen_port != c.listen_port
        || old.bind_interface != c.bind_interface
        || old.dht != c.dht
    {
        a.acquisition.stop_engine().await;
    } else if let Some(session) = a.acquisition.current().await {
        session
            .ratelimits
            .set_download_bps(std::num::NonZeroU32::new(c.download_limit_kib * 1024));
        session
            .ratelimits
            .set_upload_bps(std::num::NonZeroU32::new(c.upload_limit_kib * 1024));
    }
    Ok(Json(
        json!({"settings":c,"engine":"librqbit 9.1.0","network_policy":"direct_allowed"}),
    ))
}
fn validate(c: &Config) -> Result<()> {
    if c.download_dir.len() > 4096
        || !PathBuf::from(&c.download_dir).is_absolute()
        || c.max_active_downloads == 0
        || c.max_active_downloads > 16
        || c.download_limit_kib > 1_000_000
        || c.upload_limit_kib > 1_000_000
        || (c.download_limit_kib > 0 && c.download_limit_kib < 16)
        || (c.upload_limit_kib > 0 && c.upload_limit_kib < 16)
        || c.min_free_gb > 100_000
        || c.storage_limit_gb == 0
        || c.storage_limit_gb > 100_000
        || !c.seed_ratio.is_finite()
        || !(0.0..=100.0).contains(&c.seed_ratio)
        || c.seed_hours > 87600
        || !(5..=1440).contains(&c.search_interval_minutes)
        || c.bind_interface.as_ref().is_some_and(|s| {
            s.is_empty()
                || s.len() > 15
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
    {
        return Err(bad("invalid acquisition settings"));
    }
    Ok(())
}
