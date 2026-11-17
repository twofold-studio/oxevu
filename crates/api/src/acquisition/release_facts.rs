use super::*;
pub(super) fn facts(name: &str) -> Result<Value> {
    oxevu_acquisition::release_facts::facts(name).map_err(bad)
}
pub(super) fn identity_matches(title: &Value, release: &Value) -> Result<bool> {
    oxevu_acquisition::release_facts::identity_matches(title, release).map_err(bad)
}
