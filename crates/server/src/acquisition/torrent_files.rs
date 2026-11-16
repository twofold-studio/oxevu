use super::*;
use std::path::Component;

pub(super) fn validate(bytes: &[u8], limit: u64) -> Result<(String, Value)> {
    let meta = librqbit::torrent_from_bytes(bytes)
        .map_err(|_| bad("invalid or unsupported torrent metadata"))?;
    let hash = meta.info_hash.as_string();
    if meta.info.data.private && meta.iter_announce().count() > 1 {
        return Err(bad(
            "private multi-tracker torrents require an engine compatibility update",
        ));
    }
    let info = meta
        .info
        .data
        .validate()
        .map_err(|_| bad("invalid torrent paths or piece layout"))?;
    let mut files = Vec::new();
    let mut size = 0_u64;
    for (index, f) in info.iter_file_details().enumerate() {
        if index >= 10000 {
            return Err(bad("torrent exceeds file-count limit"));
        }
        let path = f.filename.to_pathbuf();
        if path.as_os_str().is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || path.to_string_lossy().contains('\\')
        {
            return Err(bad("torrent contains an unsafe file path"));
        }
        size = size
            .checked_add(f.len)
            .ok_or_else(|| bad("torrent size overflow"))?;
        if size > limit {
            return Err(bad("torrent exceeds acquisition storage limit"));
        }
        files.push(json!({"index":index,"path":path.to_string_lossy(),"size":f.len}));
    }
    Ok((hash, json!(files)))
}
