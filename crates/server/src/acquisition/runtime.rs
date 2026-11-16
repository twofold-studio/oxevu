use super::*;
use librqbit::{ListenerMode, ListenerOptions, Session, SessionOptions, limits::LimitsConfig};
use std::num::NonZeroU32;
use tokio::sync::Mutex;

pub(crate) struct Runtime {
    pub directory: PathBuf,
    pub gate: Mutex<()>,
    pub import_jobs: Arc<Semaphore>,
    engine: Mutex<Option<Arc<Session>>>,
    stopping: std::sync::atomic::AtomicBool,
}
impl Runtime {
    pub fn new(data_dir: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            directory: data_dir.join("acquisition"),
            gate: Mutex::new(()),
            engine: Mutex::new(None),
            stopping: std::sync::atomic::AtomicBool::new(false),
            import_jobs: Arc::new(Semaphore::new(1)),
        })
    }
    pub(super) async fn engine(&self, c: &config::Config) -> Result<Arc<Session>> {
        if !c.enabled || self.is_stopping() {
            return Err(bad("downloads are disabled or shutting down"));
        }
        let mut slot = self.engine.lock().await;
        if let Some(session) = slot.as_ref() {
            return Ok(session.clone());
        }
        tokio::fs::create_dir_all(&c.download_dir)
            .await
            .map_err(|_| bad("download folder unavailable"))?;
        let limits = LimitsConfig {
            download_bps: NonZeroU32::new(c.download_limit_kib.saturating_mul(1024)),
            upload_bps: NonZeroU32::new(c.upload_limit_kib.saturating_mul(1024)),
        };
        let session = Session::new_with_opts(
            PathBuf::from(&c.download_dir),
            SessionOptions {
                // SQLite owns job intent. Re-add paused and recheck files after a
                // restart instead of letting engine persistence start unbounded jobs.
                persistence: None,
                dht: if c.dht {
                    SessionOptions::default().dht.map(|mut d| {
                        d.persistence = None;
                        d
                    })
                } else {
                    None
                },
                bind_device_name: c.bind_interface.clone(),
                disable_local_service_discovery: true,
                listen: Some(ListenerOptions {
                    listen_addr: ([0, 0, 0, 0], c.listen_port).into(),
                    mode: ListenerMode::TcpAndUtp,
                    enable_upnp_port_forwarding: false,
                    ..Default::default()
                }),
                ratelimits: limits,
                peer_limit: Some(128),
                concurrent_init_limit: Some(2),
                runtime_worker_threads: Some(2),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| bad("torrent engine could not start; check storage, port and interface"))?;
        *slot = Some(session.clone());
        Ok(session)
    }
    pub async fn stop_engine(&self) {
        if let Some(session) = self.engine.lock().await.take() {
            session.stop().await;
        }
    }
    pub(super) async fn current(&self) -> Option<Arc<Session>> {
        self.engine.lock().await.clone()
    }
    pub(super) fn is_stopping(&self) -> bool {
        self.stopping.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub(crate) async fn shutdown(&self) {
        self.stopping
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.stop_engine().await;
    }
}
