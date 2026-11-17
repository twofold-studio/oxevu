use crate::*;
use std::sync::Arc;

pub const MAX_READERS: usize = 8;
/// Torrent engine configuration; independent of request/import policy.
pub struct EngineOptions {
    pub enabled: bool,
    pub download_dir: String,
    pub download_limit_kib: u32,
    pub upload_limit_kib: u32,
    pub dht: bool,
    pub bind_interface: Option<String>,
    pub listen_port: u16,
}

use librqbit::{ListenerMode, ListenerOptions, Session, SessionOptions, limits::LimitsConfig};
use std::num::NonZeroU32;
use tokio::sync::Mutex;

pub struct Runtime {
    engine: Mutex<Option<Arc<Session>>>,
    stopping: std::sync::atomic::AtomicBool,
}
impl Runtime {
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(None),
            stopping: std::sync::atomic::AtomicBool::new(false),
        }
    }
    pub async fn engine(&self, c: &EngineOptions) -> Result<Arc<Session>> {
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
            std::path::PathBuf::from(&c.download_dir),
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
                // rqbit holds one blocking permit for each FileStream's entire
                // lifetime. Reserve two more for piece writes and initialization.
                runtime_worker_threads: Some(MAX_READERS + 2),
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
    pub async fn current(&self) -> Option<Arc<Session>> {
        self.engine.lock().await.clone()
    }
    #[cfg(feature = "test-support")]
    pub async fn use_test_engine(&self, session: Arc<Session>) {
        *self.engine.lock().await = Some(session);
    }
    pub fn is_stopping(&self) -> bool {
        self.stopping.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub async fn shutdown(&self) {
        self.stopping
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.stop_engine().await;
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
