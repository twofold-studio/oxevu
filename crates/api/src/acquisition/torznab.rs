use super::*;
pub(super) fn parse(xml: &[u8], caps: bool) -> Result<Value> {
    oxevu_indexers::torznab::parse(xml, caps).map_err(bad)
}
