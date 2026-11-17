pub(crate) async fn read(path: &std::path::Path) -> crate::Result<serde_json::Value> {
    oxevu_media::read(path).await.map_err(crate::bad)
}
