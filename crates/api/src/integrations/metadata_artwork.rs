//! Authenticated provider images; never proxy an arbitrary URL from metadata.
use super::*;

fn omdb_poster(value: &Value) -> Option<&str> {
    value.as_str().filter(|s| {
        s.starts_with("https://m.media-amazon.com/images/")
            && s.len() <= 2048
            && !s.chars().any(char::is_whitespace)
    })
}

pub(super) async fn get(a: App, h: HeaderMap, id: String, kind: String) -> Result<Response> {
    item_data(&a, &h, &id).await?;
    let field = match kind.as_str() {
        "poster" => "metadata_poster",
        "backdrop" => "metadata_backdrop",
        "thumbnail" => "metadata_thumbnail",
        "season_poster" => "metadata_season_poster",
        _ => return Err(bad("unknown artwork kind")),
    };
    let raw: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let data: Value = serde_json::from_str(&raw).map_err(|_| bad("invalid metadata"))?;
    let url = if let Some(path) = oxevu_library::identification::image(&data[field]) {
        format!("https://image.tmdb.org/t/p/w780{path}")
    } else if kind == "poster" {
        if let Some(url) = omdb_poster(&data["omdb_poster"]) {
            url.to_owned()
        } else {
            let imdb = data["provider_ids"]["imdb"]
                .as_str()
                .filter(|s| {
                    s.starts_with("tt")
                        && (3..=20).contains(&s.len())
                        && s[2..].bytes().all(|b| b.is_ascii_digit())
                })
                .ok_or_else(|| bad("artwork unavailable"))?;
            let details =
                super::metadata::omdb(&load(&a).await?, &[("i", imdb.to_owned())]).await?;
            let url = omdb_poster(&details["Poster"])
                .ok_or_else(|| bad("provider has no poster"))?
                .to_owned();
            sqlx::query("UPDATE item_metadata SET data=json_set(data,'$.omdb_poster',?) WHERE item_id=? AND json_extract(data,'$.provider_ids.imdb')=? AND json_extract(data,'$.metadata_poster') IS NULL")
                .bind(&url).bind(&id).bind(imdb).execute(&a.pool).await.map_err(db)?;
            url
        }
    } else {
        return Err(bad("artwork unavailable"));
    };
    let response = client()?
        .get(url)
        .send()
        .await
        .map_err(|_| bad("artwork request failed"))?;
    let bytes = bytes(response, 8 * 1024 * 1024).await?;
    let mime = if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "image/jpeg"
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else {
        return Err(bad("invalid provider artwork"));
    };
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, mime),
            (axum::http::header::CACHE_CONTROL, "private, max-age=86400"),
        ],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_allows_omdb_image_origin() {
        assert!(omdb_poster(&json!("https://m.media-amazon.com/images/M/poster.jpg")).is_some());
        for value in [
            "http://m.media-amazon.com/images/M/a.jpg",
            "https://m.media-amazon.com.evil.test/images/a.jpg",
            "https://127.0.0.1/images/a.jpg",
            "https://m.media-amazon.com@evil.test/images/a.jpg",
        ] {
            assert!(omdb_poster(&json!(value)).is_none());
        }
    }
}
