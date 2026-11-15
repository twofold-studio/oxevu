use anyhow::{Context, Result, ensure};
use librqbit::{AddTorrent, AddTorrentOptions, ManagedTorrent, Session};
use std::{path::Path, sync::Arc, time::Duration};

use crate::network;

pub async fn exercise(root: &Path, child: &mut Option<tokio::process::Child>) -> Result<()> {
    let started = std::time::Instant::now();
    for stage in ["initial", "resume"] {
        *child = Some(
            tokio::process::Command::new(std::env::current_exe()?)
                .arg("--download-stage")
                .arg(stage)
                .arg(root)
                .kill_on_drop(true)
                .spawn()?,
        );
        let status = child
            .as_mut()
            .context("downloader process missing")?
            .wait()
            .await?;
        child.take();
        ensure!(
            status.success(),
            "downloader stage {stage} failed: {status}"
        );
    }
    // Token-bucket bursts and restart overhead make this a coarse limit check,
    // not a throughput benchmark or a promise about production resource use.
    ensure!(
        started.elapsed() >= Duration::from_secs(4),
        "rate limit did not constrain the transfer"
    );
    Ok(())
}

pub async fn stop_child(child: &mut Option<tokio::process::Child>) -> Result<()> {
    if let Some(mut child) = child.take() {
        if child.try_wait()?.is_none() {
            child.start_kill()?;
        }
        child.wait().await.context("downloader reaping failed")?;
    }
    Ok(())
}

pub async fn run_child(stage: &str, root: &Path) -> Result<()> {
    let session = network::open(root, "download", Some(256 * 1024), true).await?;
    let result = tokio::time::timeout(Duration::from_secs(40), async {
        match stage {
            "initial" => initial(root, &session).await,
            "resume" => resume(root, &session).await,
            _ => anyhow::bail!("unknown downloader stage"),
        }
    })
    .await
    .context("downloader stage timed out")
    .and_then(|result| result);
    let stopped = network::stop(&session).await;
    result?;
    stopped
}

fn options(root: &Path) -> AddTorrentOptions {
    AddTorrentOptions {
        output_folder: Some(root.join("download/files").to_string_lossy().into_owned()),
        only_files_regex: Some("main\\.bin$".into()),
        overwrite: true,
        ..Default::default()
    }
}

async fn initial(root: &Path, session: &Arc<Session>) -> Result<()> {
    let bytes = tokio::fs::read(root.join("fixture.torrent")).await?;
    let handle = session
        .add_torrent(AddTorrent::from_bytes(bytes), Some(options(root)))
        .await?
        .into_handle()
        .context("download handle missing")?;
    loop {
        let stats = handle.stats();
        ensure!(stats.error.is_none(), "download failed: {:?}", stats.error);
        if stats.progress_bytes > 0 {
            ensure!(
                !stats.finished,
                "fixture finished before pause; limit test inconclusive"
            );
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    session.pause(&handle).await?;
    let paused = handle.stats().progress_bytes;
    tokio::time::sleep(Duration::from_millis(350)).await;
    ensure!(
        handle.stats().progress_bytes == paused,
        "paused transfer advanced"
    );
    ensure!(
        handle.only_files().is_some_and(|files| files.len() == 1),
        "file selection lost"
    );
    tokio::fs::write(root.join("paused-bytes"), paused.to_string()).await?;
    println!("PASS selective download and pause at {paused} bytes; downloader process exiting");
    Ok(())
}

async fn resume(root: &Path, session: &Arc<Session>) -> Result<()> {
    let paused: u64 = tokio::fs::read_to_string(root.join("paused-bytes"))
        .await?
        .parse()?;
    let handle = session
        .with_torrents(|torrents| torrents.next().map(|(_, handle)| handle.clone()))
        .context("persistent session did not restore torrent")?;
    ensure!(
        handle.only_files().is_some_and(|files| files.len() == 1),
        "restored selection lost"
    );
    handle.wait_until_initialized().await?;
    ensure!(
        handle.stats().progress_bytes >= paused,
        "restart lost verified partial progress"
    );
    session.unpause(&handle).await?;
    handle.wait_until_completed().await?;
    ensure!(
        handle
            .stats()
            .file_progress
            .iter()
            .filter(|bytes| **bytes > 0)
            .count()
            == 1,
        "unselected file received data"
    );
    println!("PASS new process restored {paused} verified bytes and completed the selected file");
    verify_remove(root, session, &handle).await
}

async fn verify_remove(
    root: &Path,
    session: &Arc<Session>,
    handle: &Arc<ManagedTorrent>,
) -> Result<()> {
    let bytes = tokio::fs::read(root.join("fixture.torrent")).await?;
    let duplicate = session
        .add_torrent(AddTorrent::from_bytes(bytes), Some(options(root)))
        .await?;
    ensure!(
        matches!(
            duplicate,
            librqbit::AddTorrentResponse::AlreadyManaged(_, _)
        ),
        "infohash was not deduplicated"
    );
    session.delete(handle.id().into(), false).await?;
    ensure!(
        session.with_torrents(|torrents| torrents.next().is_none()),
        "removed job remains managed"
    );
    println!("PASS deduplication and remove-job-without-deleting-files");
    Ok(())
}
