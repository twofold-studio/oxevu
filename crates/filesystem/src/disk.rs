pub async fn disk_usage(root: &std::path::Path) -> std::io::Result<u64> {
    let mut dirs = vec![root.to_path_buf()];
    let mut bytes = 0u64;
    let mut count = 0;
    while let Some(dir) = dirs.pop() {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            // Another session can finish while the shared scratch tree is sampled.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && dir != root => continue,
            Err(error) => return Err(error),
        };
        while let Some(e) = entries.next_entry().await? {
            count += 1;
            if count > 8192 {
                return Err(std::io::Error::other("scratch file limit exceeded"));
            }
            let Some(meta) = sampled_metadata(&e.path()).await? else {
                continue;
            };
            if meta.is_dir() {
                dirs.push(e.path())
            } else {
                bytes = bytes.saturating_add(meta.len())
            }
        }
    }
    Ok(bytes)
}
// FFmpeg atomically renames temporary HLS files while this sampler walks them.
// A missing file contributes no bytes; permission and I/O failures still fail closed.
async fn sampled_metadata(path: &std::path::Path) -> std::io::Result<Option<std::fs::Metadata>> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod disk_tests {
    use super::*;
    #[tokio::test]
    async fn disappearing_atomic_output_is_not_a_disk_limit_failure() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir(&root).await.unwrap();
        let file = root.join("index.m3u8.tmp");
        tokio::fs::write(&file, b"playlist").await.unwrap();
        let mut entries = tokio::fs::read_dir(&root).await.unwrap();
        let entry = entries.next_entry().await.unwrap().unwrap();
        tokio::fs::rename(&file, root.join("index.m3u8"))
            .await
            .unwrap();
        assert!(sampled_metadata(&entry.path()).await.unwrap().is_none());
        assert_eq!(disk_usage(&root).await.unwrap(), 8);
        tokio::fs::remove_dir_all(&root).await.unwrap();
    }
}
