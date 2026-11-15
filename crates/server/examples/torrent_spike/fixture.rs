use anyhow::{Context, Result, ensure};
use librqbit::{CreateTorrentOptions, create_torrent, spawn_utils::BlockingSpawner};
use std::path::{Path, PathBuf};

pub struct Fixture {
    directory: tempfile::TempDir,
    pub torrent: Vec<u8>,
    pub public_torrent: Vec<u8>,
}

impl Fixture {
    pub async fn new(tracker: String) -> Result<Self> {
        let directory = tempfile::Builder::new()
            .prefix("oxevu-torrent-spike-")
            .tempdir()?;
        let source = directory.path().join("source");
        tokio::fs::create_dir(&source).await?;
        // Deterministic, generated data: no existing media or public swarm needed.
        let mut state = 0x1234_5678_u32;
        let payload: Vec<u8> = (0..2 * 1024 * 1024)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state as u8
            })
            .collect();
        tokio::fs::write(source.join("main.bin"), payload).await?;
        tokio::fs::write(source.join("extra.bin"), vec![0x5a; 64 * 1024]).await?;
        let mut torrent = create_torrent(
            &source,
            CreateTorrentOptions {
                name: Some("oxevu-fixture"),
                piece_length: Some(16 * 1024),
                trackers: vec![tracker],
            },
            &BlockingSpawner::new(2),
        )
        .await?;
        let public_torrent = torrent.as_bytes()?.to_vec();
        // The creator has no private option; serialize the updated dictionary.
        // Each peer reparses these bytes and computes the resulting infohash.
        torrent.meta.info.data.private = true;
        let torrent = torrent.as_bytes()?.to_vec();
        tokio::fs::write(directory.path().join("fixture.torrent"), &torrent).await?;
        Ok(Self {
            torrent,
            public_torrent,
            directory,
        })
    }

    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn source(&self) -> PathBuf {
        self.root().join("source")
    }
    pub fn destination(&self) -> PathBuf {
        self.root().join("download/files")
    }

    pub async fn verify(&self, folder: &Path) -> Result<()> {
        let expected = tokio::fs::read(self.source().join("main.bin")).await?;
        let actual = tokio::fs::read(folder.join("main.bin"))
            .await
            .context("downloaded file missing")?;
        ensure!(actual == expected, "downloaded bytes differ");
        Ok(())
    }

    pub fn close(self) -> Result<()> {
        self.directory.close().context("fixture cleanup failed")
    }
}
