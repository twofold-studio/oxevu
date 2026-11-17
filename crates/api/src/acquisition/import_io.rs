use super::*;
pub(super) use oxevu_filesystem::stamp;
use std::path::Path;
pub(super) async fn probe(path: &str) -> Result<()> {
    let data = crate::media_probe::read(Path::new(path)).await?;
    if !data["streams"]
        .as_array()
        .is_some_and(|streams| streams.iter().any(|stream| stream["codec_type"] == "video"))
    {
        return Err(bad("download has no video stream"));
    }
    Ok(())
}

pub(super) fn publish(
    f: &import_plan::File,
    id: &str,
    copy: bool,
    runtime: &Runtime,
) -> std::io::Result<String> {
    publish_mode(f, id, copy, false, runtime)
}
pub(super) fn publish_mode(
    f: &import_plan::File,
    id: &str,
    copy: bool,
    force_copy: bool,
    runtime: &Runtime,
) -> std::io::Result<String> {
    let plan = oxevu_filesystem::Publication {
        root: &f.root,
        source: &f.source,
        destination: &f.destination,
        root_device: f.root_device,
        source_stamp: &f.source_stamp,
    };
    oxevu_filesystem::publish_mode(&plan, id, copy, force_copy, &|| runtime.is_stopping())
}
