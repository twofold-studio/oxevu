use super::*;
pub(super) fn validate(bytes: &[u8], limit: u64) -> Result<(String, Value)> {
    oxevu_transfers::torrent_files::validate(bytes, limit).map_err(bad)
}
