use super::*;

pub fn infohash(raw: &Value) -> Option<String> {
    if let Some(hash) = raw["infohash"]
        .as_str()
        .filter(|hash| matches!(hash.len(), 40 | 64) && hash.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return Some(hash.to_ascii_lowercase());
    }
    ["magneturl", "guid", "download_url", "link"]
        .into_iter()
        .find_map(|field| {
            librqbit::Magnet::parse(raw[field].as_str()?)
                .ok()?
                .as_id20()
                .map(|id| id.as_string())
        })
}

pub fn key(indexer: &str, raw: &Value) -> String {
    use sha2::{Digest, Sha256};
    let identity = infohash(raw)
        .or_else(|| {
            ["guid", "title", "download_url"]
                .into_iter()
                .find_map(|field| {
                    raw[field]
                        .as_str()
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned)
                })
        })
        .unwrap_or_default();
    format!(
        "{:x}",
        Sha256::digest(format!("{indexer}\0{identity}").as_bytes())
    )
}

pub fn group(mut releases: Vec<Value>) -> Vec<Value> {
    releases.sort_by(|a, b| {
        b["accepted"]
            .as_bool()
            .cmp(&a["accepted"].as_bool())
            .then_with(|| b["seeders"].as_u64().cmp(&a["seeders"].as_u64()))
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    let mut ids = std::collections::HashSet::new();
    let mut hashes = std::collections::HashMap::new();
    let mut grouped: Vec<Value> = Vec::new();
    for mut release in releases {
        if !ids.insert(release["id"].as_str().unwrap_or_default().to_owned()) {
            continue;
        }
        if let Some(hash) = infohash(&release) {
            if let Some(&index) = hashes.get(&hash) {
                let row: &mut Value = &mut grouped[index];
                if row["indexer_id"] != release["indexer_id"] {
                    let source = json!({"indexer_id":release["indexer_id"],"indexer_name":release["indexer_name"]});
                    if let Some(sources) = row["also_on"].as_array_mut()
                        && !sources.contains(&source)
                    {
                        sources.push(source);
                    }
                }
                continue;
            }
            hashes.insert(hash, grouped.len());
        }
        release["also_on"] = json!([]);
        grouped.push(release);
    }
    grouped
}
