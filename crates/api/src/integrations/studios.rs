use super::*;
pub(super) fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
pub(crate) async fn studio(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, name)): Path<(String, String)>,
) -> Result<Response> {
    item_data(&a, &h, &id).await?;
    if load(&a).await?.studio_images["enabled"] != true {
        return Err(bad("studio images disabled"));
    }
    if name.is_empty()
        || name.len() > 100
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(bad("invalid studio name"));
    }
    let allowed = [
        "arte",
        "at-x",
        "comedy-central",
        "dr1",
        "e",
        "fox",
        "france-2",
        "france-3",
        "france-4",
        "france-5",
        "france-tv-slash",
        "france-tv",
        "game-show-network",
        "hbo",
        "m6",
        "netflix",
        "showtime",
        "starz",
        "syfy",
        "tbs",
        "the-cw",
        "tnt",
        "usa-network",
    ];
    if !allowed.contains(&name.as_str()) {
        return Err(bad("studio artwork unavailable"));
    }
    let _permit = a
        .artwork_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("artwork worker busy"))?;
    let raw: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id=?")
        .bind(id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let data: Value = serde_json::from_str(&raw).map_err(|_| bad("invalid metadata"))?;
    if !data["studios"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| s.as_str())
        .any(|s| slug(s) == name)
    {
        return Err(bad("studio not associated with item"));
    }
    let cache = a.artwork_cache.join("studios");
    tokio::fs::create_dir_all(&cache)
        .await
        .map_err(|_| bad("cannot create artwork cache"))?;
    let file = cache.join(format!("{name}.svg"));
    let content = match tokio::fs::read(&file).await {
        Ok(bytes) => bytes,
        Err(_) => {
            // Fixed public source; no user-supplied host or path. Original artwork
            // is CC-BY-SA 4.0, credited in integration settings and documentation.
            let url = format!(
                "https://raw.githubusercontent.com/jellyfin/jellyfin-artwork/master/studios/{name}/thumb.svg"
            );
            let data = bytes(
                client()?
                    .get(url)
                    .send()
                    .await
                    .map_err(|_| bad("studio artwork request failed"))?,
                512 * 1024,
            )
            .await?;
            let text = std::str::from_utf8(&data).map_err(|_| bad("invalid studio artwork"))?;
            if !text.contains("<svg") {
                return Err(bad("invalid studio artwork"));
            }
            tokio::fs::write(file, &data)
                .await
                .map_err(|_| bad("cannot cache artwork"))?;
            data
        }
    };
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "image/svg+xml"),
            (
                axum::http::header::CONTENT_SECURITY_POLICY,
                "sandbox; default-src 'none'",
            ),
            (axum::http::header::CACHE_CONTROL, "private, max-age=86400"),
        ],
        content,
    )
        .into_response())
}
