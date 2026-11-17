use super::*;
use std::{collections::VecDeque, os::unix::fs::MetadataExt, process::Stdio};
use tokio::{process::Command, sync::watch};

#[derive(Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Identity {
    path: PathBuf,
    device: u64,
    inode: u64,
    size: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
impl Identity {
    pub(super) async fn read(path: &std::path::Path) -> std::io::Result<Self> {
        let metadata = tokio::fs::metadata(path).await?;
        Ok(Self {
            path: path.to_owned(),
            device: metadata.dev(),
            inode: metadata.ino(),
            size: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        })
    }
}
#[derive(Default)]
pub(crate) struct Cache(VecDeque<(Identity, Value)>);
impl Cache {
    fn get(&mut self, identity: &Identity) -> Option<Value> {
        let index = self.0.iter().position(|(key, _)| key == identity)?;
        let entry = self.0.remove(index)?;
        let value = entry.1.clone();
        self.0.push_back(entry);
        Some(value)
    }
    fn insert(&mut self, identity: Identity, value: Value) {
        self.0.retain(|(key, _)| key.path != identity.path);
        while self.0.len() >= 64 {
            self.0.pop_front();
        }
        self.0.push_back((identity, value));
    }
}
pub(super) async fn metadata(
    a: &App,
    path: &std::path::Path,
    cancel: watch::Receiver<bool>,
) -> std::result::Result<Value, String> {
    if *cancel.borrow() {
        return Err("Playback stopped".into());
    }
    let identity = Identity::read(path)
        .await
        .map_err(|_| "Source unavailable")?;
    if let Some(value) = a.playback.source_metadata.lock().await.get(&identity) {
        return Ok(value);
    }
    let mut command = Command::new("ffprobe");
    command.args(["-v", "error", "-protocol_whitelist", "file,pipe", "-show_entries",
        "stream=codec_type,codec_name,width,height,color_transfer,pix_fmt:stream_side_data=side_data_type:format=bit_rate", "-of", "json"])
        .arg(path).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true);
    let value = probe(command, cancel, 15).await?;
    if Identity::read(path).await.ok().as_ref() == Some(&identity) {
        a.playback
            .source_metadata
            .lock()
            .await
            .insert(identity, value.clone());
    }
    Ok(value)
}

pub(super) async fn progressive(
    input: &source::Input,
    cancel: watch::Receiver<bool>,
) -> std::result::Result<Value, String> {
    let mut command = Command::new("ffprobe");
    command.args(["-v", "error"]);
    input.arguments(&mut command);
    command.args(["-show_entries","stream=codec_type,codec_name,width,height,color_transfer,pix_fmt:stream_side_data=side_data_type:format=bit_rate,duration","-of","json"])
        .arg(input.path()).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true);
    probe(command, cancel, 90).await
}

use oxevu_playback::process::probe;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cached_probe_is_invalidated_when_source_changes() {
        let a = crate::tests::app().await;
        let path = a.playback.scratch.join("probe-fixture");
        tokio::fs::write(&path, b"not a video").await.unwrap();
        let identity = Identity::read(&path).await.unwrap();
        a.playback
            .source_metadata
            .lock()
            .await
            .insert(identity, json!({"cached":true}));
        let (_cancel, receiver) = watch::channel(false);
        assert_eq!(
            metadata(&a, &path, receiver.clone()).await.unwrap()["cached"],
            true
        );
        tokio::fs::write(&path, b"changed invalid source")
            .await
            .unwrap();
        assert!(metadata(&a, &path, receiver).await.is_err());
        a.playback.shutdown().await;
    }
}
