use super::*;
use tokio::sync::Mutex;

pub(crate) struct Runtime {
    inner: oxevu_transfers::engine::Runtime,
    pub directory: PathBuf,
    pub gate: Mutex<()>,
    pub import_jobs: Arc<Semaphore>,
    pub(super) streams: Arc<Semaphore>,
}
impl Runtime {
    pub async fn shutdown(&self) {
        self.inner.shutdown().await;
    }
    pub async fn stop_engine(&self) {
        self.inner.stop_engine().await;
    }
    pub(super) async fn current(&self) -> Option<Arc<librqbit::Session>> {
        self.inner.current().await
    }
    pub(super) fn is_stopping(&self) -> bool {
        self.inner.is_stopping()
    }
    #[cfg(test)]
    pub(super) async fn use_test_engine(&self, session: Arc<librqbit::Session>) {
        self.inner.use_test_engine(session).await;
    }

    pub fn new(data: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            inner: oxevu_transfers::engine::Runtime::new(),
            directory: data.join("acquisition"),
            gate: Mutex::new(()),
            import_jobs: Arc::new(Semaphore::new(1)),
            streams: Arc::new(Semaphore::new(oxevu_transfers::engine::MAX_READERS)),
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
