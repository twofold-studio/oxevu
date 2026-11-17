use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Knaben,
    Nyaa,
    PirateBay,
    Yts,
}

impl Provider {
    /// Match canonical endpoints so existing Torznab configurations retain their meaning.
    pub fn from_endpoint(endpoint: &str) -> Option<Self> {
        match endpoint {
            "https://api.knaben.org/v1" => Some(Self::Knaben),
            "https://nyaa.si/" => Some(Self::Nyaa),
            "https://apibay.org/q.php" => Some(Self::PirateBay),
            "https://movies-api.accel.li/api/v2/list_movies.json" => Some(Self::Yts),
            _ => None,
        }
    }

    pub fn parse(self, bytes: &[u8]) -> Result<Value> {
        match self {
            Self::Knaben => parse_rows(&decode(bytes)?["hits"], |row| {
                let mut release = magnet_release(row, "title", "hash", "bytes", "peers")?;
                if let Some(uri) = row["magnetUrl"]
                    .as_str()
                    .filter(|uri| uri.starts_with("magnet:?") && uri.len() <= 16384)
                {
                    release["magneturl"] = json!(uri);
                }
                Some(release)
            }),
            Self::PirateBay => parse_rows(&decode(bytes)?, |row| {
                magnet_release(row, "name", "info_hash", "size", "leechers")
            }),
            Self::Yts => parse_yts(&decode(bytes)?),
            Self::Nyaa => torznab::parse(bytes, false),
        }
    }
}

fn decode(bytes: &[u8]) -> Result<Value> {
    serde_json::from_slice(bytes).map_err(|_| bad("invalid indexer JSON"))
}

fn parse_rows(rows: &Value, normalize: impl Fn(&Value) -> Option<Value>) -> Result<Value> {
    let rows = rows
        .as_array()
        .ok_or_else(|| bad("invalid indexer result list"))?;
    Ok(json!(
        rows.iter()
            .filter_map(normalize)
            .take(100)
            .collect::<Vec<_>>()
    ))
}

fn number(value: &Value) -> Option<u64> {
    value.as_u64().or_else(|| value.as_str()?.parse().ok())
}

fn magnet(hash: &Value) -> Option<String> {
    let hash = hash.as_str()?;
    (hash.len() == 40
        && hash.bytes().all(|b| b.is_ascii_hexdigit())
        && hash.bytes().any(|b| b != b'0'))
    .then(|| format!("magnet:?xt=urn:btih:{hash}"))
}

fn magnet_release(row: &Value, title: &str, hash: &str, size: &str, peers: &str) -> Option<Value> {
    Some(json!({
        "title": row[title].as_str()?,
        "magneturl": magnet(&row[hash])?,
        "size": number(&row[size]),
        "seeders": number(&row["seeders"]),
        "peers": number(&row[peers]),
    }))
}

fn parse_yts(body: &Value) -> Result<Value> {
    if body["status"] != "ok" {
        return Err(bad("YTS rejected the search"));
    }
    let data = &body["data"];
    if data["movie_count"] == 0 {
        return Ok(json!([]));
    }
    let movies = data["movies"]
        .as_array()
        .ok_or_else(|| bad("invalid YTS result list"))?;
    let releases = movies
        .iter()
        .flat_map(|movie| {
            movie["torrents"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(move |torrent| {
                    let title = movie["title_english"]
                        .as_str()
                        .or_else(|| movie["title"].as_str())?;
                    let year = number(&movie["year"])?;
                    let quality = torrent["quality"].as_str()?;
                    let source = torrent["type"].as_str().unwrap_or("");
                    let codec = torrent["video_codec"].as_str().unwrap_or("");
                    Some(json!({
                        "title": format!("{title} {year} {quality} {source} {codec} YTS"),
                        "magneturl": magnet(&torrent["hash"])?,
                        "size": number(&torrent["size_bytes"]),
                        "seeders": number(&torrent["seeds"]),
                        "peers": number(&torrent["peers"]),
                        "imdb": movie["imdb_code"],
                    }))
                })
        })
        .take(100)
        .collect::<Vec<_>>();
    Ok(json!(releases))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nyaa_normalizes_rss_peer_counts_and_size() {
        let rows = Provider::Nyaa.parse(br#"<rss xmlns:nyaa="https://nyaa.si/xmlns/nyaa"><channel><item>
            <title>Example &amp; friends</title><link>https://nyaa.si/download/42.torrent</link>
            <nyaa:size>1.5 GiB</nyaa:size><nyaa:seeders>12</nyaa:seeders><nyaa:leechers>3</nyaa:leechers>
            </item></channel></rss>"#).unwrap();
        assert_eq!(
            rows[0],
            json!({"title":"Example & friends","link":"https://nyaa.si/download/42.torrent",
            "size":1610612736_u64,"seeders":12,"peers":3})
        );
    }

    #[test]
    fn json_sources_normalize_magnets_and_skip_placeholder_hashes() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let knaben =
            json!({"hits":[{"title":"Example","hash":hash,"bytes":42,"seeders":2,"peers":1}]});
        let pirate_bay = json!([
            {"name":"No results","info_hash":"0000000000000000000000000000000000000000"},
            {"name":"Example","info_hash":hash,"size":"42","seeders":"2","leechers":"1"}
        ]);
        let expected = json!([{"title":"Example","magneturl":format!("magnet:?xt=urn:btih:{hash}"),
            "size":42,"seeders":2,"peers":1}]);
        assert_eq!(
            Provider::Knaben
                .parse(knaben.to_string().as_bytes())
                .unwrap(),
            expected
        );
        assert_eq!(
            Provider::PirateBay
                .parse(pirate_bay.to_string().as_bytes())
                .unwrap(),
            expected
        );
    }

    #[test]
    fn yts_keeps_release_quality_and_handles_empty_search() {
        let bytes = br#"{"status":"ok","data":{"movies":[{"title":"Example","year":2024,
            "torrents":[{"quality":"1080p","type":"bluray","hash":"0123456789abcdef0123456789abcdef01234567"}]}]}}"#;
        assert_eq!(
            Provider::Yts.parse(bytes).unwrap()[0]["title"],
            "Example 2024 1080p bluray  YTS"
        );
        assert_eq!(
            Provider::Yts
                .parse(br#"{"status":"ok","data":{"movie_count":0}}"#)
                .unwrap(),
            json!([])
        );
    }
}
