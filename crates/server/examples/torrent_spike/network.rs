use anyhow::{Context, Result, ensure};
use librqbit::{
    ListenerMode, ListenerOptions, Session, SessionOptions, SessionPersistenceConfig,
    limits::LimitsConfig,
};
use std::{num::NonZeroU32, path::Path, sync::Arc, time::Duration};

fn options() -> SessionOptions {
    SessionOptions {
        dht: None,
        bind_device_name: Some("lo".into()),
        disable_trackers: false,
        disable_local_service_discovery: true,
        ipv4_only: true,
        listen: Some(ListenerOptions {
            listen_addr: ([127, 0, 0, 1], 0).into(),
            mode: ListenerMode::TcpOnly,
            enable_upnp_port_forwarding: false,
            ..Default::default()
        }),
        concurrent_init_limit: Some(2),
        runtime_worker_threads: Some(2),
        peer_limit: Some(4),
        ..Default::default()
    }
}

pub async fn open(
    root: &Path,
    name: &str,
    limit: Option<u32>,
    persist: bool,
) -> Result<Arc<Session>> {
    let folder = root.join(name);
    tokio::fs::create_dir_all(folder.join("files")).await?;
    let mut config = options();
    config.ratelimits = LimitsConfig {
        download_bps: limit.and_then(NonZeroU32::new),
        upload_bps: None,
    };
    config.persistence = persist.then(|| SessionPersistenceConfig::Json {
        folder: Some(folder.join("state")),
    });
    let session = Session::new_with_opts(folder.join("files"), config).await?;
    ensure!(
        session.get_dht().is_none(),
        "DHT must stay off in this harness"
    );
    Ok(session)
}

pub async fn reject_missing_interface(root: &Path) -> Result<()> {
    let mut config = options();
    config.bind_device_name = Some("oxevu-no-iface".into());
    match Session::new_with_opts(root.join("invalid-interface"), config).await {
        Err(_) => {
            println!("PASS missing network interface fails closed at startup");
            Ok(())
        }
        Ok(session) => {
            session.stop().await;
            anyhow::bail!("missing interface was silently accepted")
        }
    }
}

pub async fn stop(session: &Session) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(5), session.stop())
        .await
        .context("engine shutdown timed out")?;
    if let Some(address) = session.listen_addr() {
        let connected = tokio::time::timeout(
            Duration::from_millis(300),
            tokio::net::TcpStream::connect(address),
        )
        .await;
        ensure!(
            !matches!(connected, Ok(Ok(_))),
            "listener remains open after shutdown"
        );
    }
    Ok(())
}
