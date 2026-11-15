use anyhow::{Context, Result, ensure};
use librqbit::{AddTorrent, AddTorrentOptions, Session};
use std::{sync::Arc, time::Duration};

#[path = "torrent_spike/download.rs"]
mod download;
#[path = "torrent_spike/fixture.rs"]
mod fixture;
#[path = "torrent_spike/network.rs"]
mod network;
#[path = "torrent_spike/tracker.rs"]
mod tracker;

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<()> {
    ensure!(
        cfg!(target_os = "linux"),
        "this acceptance harness requires Linux loopback binding"
    );
    let args: Vec<String> = std::env::args().collect();
    if let [_, flag, stage, root] = args.as_slice() {
        ensure!(flag == "--download-stage", "unexpected arguments");
        return download::run_child(stage, std::path::Path::new(root)).await;
    }
    ensure!(args.len() == 1, "run torrent-spike without arguments");
    let mut tracker = tracker::Tracker::new().await?;
    let fixture = fixture::Fixture::new(tracker.url.clone()).await?;
    let mut sessions = Vec::new();
    let mut child = None;
    let result = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(90), run(&fixture, &mut sessions, &tracker, &mut child)) =>
            result.context("torrent acceptance timed out").and_then(|result| result),
        _ = tokio::signal::ctrl_c() => Err(anyhow::anyhow!("acceptance interrupted")),
    };
    // Stop all engines before TempDir removes their files, also on failure.
    let child_stopped = download::stop_child(&mut child).await;
    let mut stopped = Ok(());
    for session in &sessions {
        if let Err(error) = network::stop(session).await {
            stopped = Err(error);
        }
    }
    drop(sessions);
    let tracker_stopped = tracker.close().await;
    let fixture_removed = fixture.close();
    result?;
    child_stopped?;
    stopped?;
    tracker_stopped?;
    fixture_removed?;
    println!("PASS shutdown/listener cleanup; fixture files removed");
    Ok(())
}

async fn run(
    f: &fixture::Fixture,
    sessions: &mut Vec<Arc<Session>>,
    tracker: &tracker::Tracker,
    child: &mut Option<tokio::process::Child>,
) -> Result<()> {
    network::reject_missing_interface(f.root()).await?;
    let seed = network::open(f.root(), "seed", None, false).await?;
    sessions.push(seed.clone());
    let address = seed.listen_addr().context("seed listener missing")?;
    tracker
        .peers
        .port
        .store(address.port(), std::sync::atomic::Ordering::SeqCst);
    let seed_torrent = seed
        .add_torrent(
            AddTorrent::from_bytes(f.torrent.clone()),
            Some(AddTorrentOptions {
                output_folder: Some(f.source().to_string_lossy().into_owned()),
                overwrite: true,
                ..Default::default()
            }),
        )
        .await?
        .into_handle()
        .context("seed handle missing")?;
    seed_torrent.wait_until_completed().await?;
    ensure!(
        seed_torrent.with_metadata(|m| m.info.info().private)?,
        "fixture must be private"
    );

    download::exercise(f.root(), child).await?;
    ensure!(
        tracker
            .peers
            .announces
            .load(std::sync::atomic::Ordering::SeqCst)
            >= 2,
        "tracker was not exercised"
    );
    f.verify(&f.destination()).await?;
    ensure!(
        seed_torrent.stats().uploaded_bytes > 0,
        "seed did not account uploaded bytes"
    );
    println!("PASS persistent restart, resume, exact bytes and seeding accounting");

    verify_magnet(f, sessions, &seed).await?;
    Ok(())
}

async fn verify_magnet(
    f: &fixture::Fixture,
    sessions: &mut Vec<Arc<Session>>,
    seed: &Arc<Session>,
) -> Result<()> {
    // Private torrents intentionally refuse metadata exchange; magnets use a
    // separate public fixture, still restricted to the explicit loopback peer.
    let public_seed = seed
        .add_torrent(
            AddTorrent::from_bytes(f.public_torrent.clone()),
            Some(AddTorrentOptions {
                output_folder: Some(f.source().to_string_lossy().into_owned()),
                overwrite: true,
                ..Default::default()
            }),
        )
        .await?
        .into_handle()
        .context("public seed handle missing")?;
    public_seed.wait_until_completed().await?;
    let hash = public_seed.info_hash();
    let magnet_session = network::open(f.root(), "magnet", None, false).await?;
    sessions.push(magnet_session.clone());
    let magnet = format!("magnet:?xt=urn:btih:{}", hash.as_string());
    let handle = magnet_session
        .add_torrent(
            AddTorrent::from_url(magnet),
            Some(AddTorrentOptions {
                initial_peers: Some(vec![seed.listen_addr().context("seed listener missing")?]),
                only_files_regex: Some("main\\.bin$".into()),
                ..Default::default()
            }),
        )
        .await?
        .into_handle()
        .context("magnet handle missing")?;
    handle.wait_until_completed().await?;
    ensure!(handle.info_hash() == hash, "magnet metadata differs");
    f.verify(handle.output_folder()).await?;
    println!("PASS magnet metadata exchange over explicit loopback peer");
    Ok(())
}
