use super::*;
use axum::http::header;
use std::path::Path;

pub(super) enum Source {
    Local(PathBuf),
    Download(acquisition::streaming::Source),
}
impl Source {
    pub fn local(&self) -> Option<&Path> {
        match self {
            Self::Local(path) => Some(path),
            Self::Download(_) => None,
        }
    }
    pub async fn resolve(
        a: &App,
        item: &str,
        row: &sqlx::sqlite::SqliteRow,
        data: &Value,
    ) -> Result<Self> {
        if let Some(source) = acquisition::streaming::for_item(a, item).await? {
            return Ok(Self::Download(source));
        }
        if data["file_available"] == false {
            return Err(bad(
                "This title has no local media. Find a release to download it again.",
            ));
        }
        let path = tokio::fs::canonicalize(row.get::<String, _>("path"))
            .await
            .map_err(|_| bad("source unavailable"))?;
        libraries::item_root(a, item, &path).await?;
        Ok(Self::Local(path))
    }
    pub async fn prepare(&self, a: &App) -> std::result::Result<Input, String> {
        match self {
            Self::Local(path) => Ok(Input {
                path: path.clone(),
                bridge: None,
            }),
            Self::Download(source) => {
                let bridge = Bridge::start(a.clone(), source.clone()).await?;
                Ok(Input {
                    path: PathBuf::from(format!("http://{}/media", bridge.address)),
                    bridge: Some(bridge),
                })
            }
        }
    }
}

/// A worker-owned loopback range bridge. Neither the credential nor its address is
/// returned to clients. Dropping it aborts all readers and releases piece priorities.
struct Bridge {
    address: std::net::SocketAddr,
    key: String,
    shutdown: tokio::sync::watch::Sender<bool>,
}
impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
    }
}
#[derive(Clone)]
struct BridgeState {
    app: App,
    source: acquisition::streaming::Source,
    key: String,
    cancel: tokio::sync::watch::Receiver<bool>,
}
impl Bridge {
    async fn start(
        app: App,
        source: acquisition::streaming::Source,
    ) -> std::result::Result<Self, String> {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|_| "Cannot open playback reader")?;
        let address = listener
            .local_addr()
            .map_err(|_| "Cannot inspect playback reader")?;
        let key = uuid::Uuid::new_v4().to_string();
        let (shutdown, mut cancel) = tokio::sync::watch::channel(false);
        let state = BridgeState {
            app,
            source,
            key: key.clone(),
            cancel: cancel.clone(),
        };
        let route = Router::new()
            .route("/media", axum::routing::get(bridge_read))
            .with_state(state);
        tokio::spawn(async move {
            let _ = axum::serve(listener, route)
                .with_graceful_shutdown(async move {
                    let _ = cancel.changed().await;
                })
                .await;
        });
        Ok(Self {
            address,
            key,
            shutdown,
        })
    }
}
async fn bridge_read(State(s): State<BridgeState>, request: Request) -> Result<Response> {
    if request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        != Some(format!("Bearer {}", s.key).as_str())
    {
        return Err(users::forbidden());
    }
    s.source
        .response_with_cancel(s.app, request, Some(s.cancel))
        .await
}

pub(super) struct Input {
    path: PathBuf,
    bridge: Option<Bridge>,
}
impl Input {
    #[cfg(test)]
    pub fn local(path: PathBuf) -> Self {
        Self { path, bridge: None }
    }
    pub fn arguments(&self, command: &mut tokio::process::Command) {
        if let Some(bridge) = &self.bridge {
            // Only media demuxers are allowed. HTTP playlists/manifests must not
            // turn the private reader into a way to fetch arbitrary URLs.
            command
                .args([
                    "-protocol_whitelist",
                    "http,tcp",
                    "-format_whitelist",
                    "mov,matroska,webm,avi,mpegts,mpeg",
                    "-http_proxy",
                    "",
                    "-headers",
                ])
                .arg(format!("Authorization: Bearer {}\r\n", bridge.key));
        } else {
            command.args(["-protocol_whitelist", "file,pipe"]);
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn progressive(&self) -> bool {
        self.bridge.is_some()
    }
}
