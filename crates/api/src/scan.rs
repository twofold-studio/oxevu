use super::*;
pub(super) async fn scan(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    scan_library(a, id).await
}
pub(crate) async fn scan_library(a: App, id: String) -> Result<Json<Value>> {
    let _permit = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("scan already running"))?;
    let artwork = ArtworkAdapter(&a);
    crate::library::scanner::run(&a.pool, &id, &artwork)
        .await
        .map(Json)
        .map_err(crate::library::response)
}
struct ArtworkAdapter<'a>(&'a App);
impl crate::library::scanner::Artwork for ArtworkAdapter<'_> {
    fn attach<'a>(
        &'a self,
        data: &'a mut Value,
        path: &'a std::path::Path,
        root: &'a std::path::Path,
        tv: bool,
        errors: &'a mut Vec<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(artwork::attach(self.0, data, path, root, tv, errors))
    }
}
