use super::*;
use librqbit::AddTorrent;

pub(super) async fn source(
    a: &App,
    job: &str,
    data: &Value,
    input: &downloads::Input,
) -> Result<AddTorrent<'static>> {
    if let Some(raw) = data["cached_torrent"]
        .as_str()
        .or(input.torrent_base64.as_deref())
    {
        return Ok(AddTorrent::from_bytes(downloads::decode_torrent(raw)?));
    }
    if let Some(magnet) = data["cached_magnet"].as_str().or(input.magnet.as_deref()) {
        return Ok(AddTorrent::from_url(magnet.to_owned()));
    }
    let id = input
        .release_id
        .as_deref()
        .ok_or_else(|| bad("download source missing"))?;
    let row = sqlx::query("SELECT indexer_id,data FROM acquisition_releases WHERE id=?")
        .bind(id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let release = decode(row.get("data"))?;
    if let Some(uri) = release["magneturl"].as_str().or_else(|| {
        release["download_url"]
            .as_str()
            .filter(|s| s.starts_with("magnet:?"))
    }) {
        return Ok(AddTorrent::from_url(uri.to_owned()));
    }
    let indexer = indexers::load(a, row.get("indexer_id")).await?;
    let uri = release["download_url"]
        .as_str()
        .ok_or_else(|| bad("release has no torrent URL"))?;
    match indexers::fetch_torrent(&indexer, uri).await? {
        indexers::TorrentSource::Bytes(bytes) => Ok(AddTorrent::from_bytes(bytes)),
        indexers::TorrentSource::Magnet(magnet) => {
            sqlx::query("UPDATE acquisition_downloads SET data=json_set(data,'$.cached_magnet',?) WHERE id=? AND state!='removed'")
                .bind(&magnet).bind(job).execute(&a.pool).await.map_err(db)?;
            Ok(AddTorrent::from_url(magnet))
        }
    }
}
