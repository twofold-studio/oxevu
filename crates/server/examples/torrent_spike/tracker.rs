use anyhow::{Context, Result};
use axum::{Router, extract::State, routing::get};
use std::sync::{
    Arc,
    atomic::{AtomicU16, AtomicUsize, Ordering},
};

#[derive(Default)]
pub struct Peers {
    pub port: AtomicU16,
    pub announces: AtomicUsize,
}

pub struct Tracker {
    pub peers: Arc<Peers>,
    pub url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Tracker {
    pub async fn new() -> Result<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/announce", listener.local_addr()?);
        let peers = Arc::new(Peers::default());
        let app = Router::new()
            .route("/announce", get(announce))
            .with_state(peers.clone());
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Ok(Self { peers, url, task })
    }

    pub async fn close(&mut self) -> Result<()> {
        self.task.abort();
        match (&mut self.task).await {
            Err(error) if error.is_cancelled() => Ok(()),
            result => result.context("tracker task failed"),
        }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn announce(State(peers): State<Arc<Peers>>) -> Vec<u8> {
    peers.announces.fetch_add(1, Ordering::SeqCst);
    let port = peers.port.load(Ordering::SeqCst);
    // Minimal compact IPv4 response, pointing only to our generated-data seed.
    let mut response = b"d8:intervali1e5:peers6:".to_vec();
    response.extend_from_slice(&[127, 0, 0, 1]);
    response.extend_from_slice(&port.to_be_bytes());
    response.push(b'e');
    response
}
