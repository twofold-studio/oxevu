use super::*;
use std::ops::Deref;

pub(crate) struct Runtime {
    inner: Arc<oxevu_transfers::engine::Runtime>,
}
impl Deref for Runtime {
    type Target = oxevu_transfers::engine::Runtime;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl Runtime {
    pub fn new(data: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            inner: oxevu_transfers::engine::Runtime::new(data),
        })
    }
    pub(super) async fn engine(&self, config: &config::Config) -> Result<Arc<librqbit::Session>> {
        let options = oxevu_transfers::engine::EngineOptions {
            enabled: config.enabled,
            download_dir: config.download_dir.clone(),
            download_limit_kib: config.download_limit_kib,
            upload_limit_kib: config.upload_limit_kib,
            dht: config.dht,
            bind_interface: config.bind_interface.clone(),
            listen_port: config.listen_port,
        };
        self.inner.engine(&options).await.map_err(bad)
    }
}
