use super::*;
use crate::contract::acquisition_catalog_model::Trailer;

static JOBS: Semaphore = Semaphore::const_new(3);

pub(super) async fn lookup(a: &App, headers: &HeaderMap, kind: &str, id: u64) -> Result<Trailer> {
    user(a, headers).await?;
    title_key(kind, id)?;
    let _permit = JOBS.try_acquire().map_err(|_| bad("trailer lookup busy"))?;
    let config = integrations::load(a).await?;
    let videos = integrations::tmdb(&config, &format!("{kind}/{id}/videos"), &[]).await?;
    Ok(Trailer {
        youtube_id: youtube_trailer(&videos).map(str::to_owned),
    })
}

fn youtube_trailer(videos: &Value) -> Option<&str> {
    videos["results"]
        .as_array()?
        .iter()
        .filter_map(|video| {
            if video["site"] != "YouTube" || video["type"] != "Trailer" {
                return None;
            }
            let key = video["key"].as_str().filter(|key| {
                key.len() == 11
                    && key
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            })?;
            Some((video["official"] == true, key))
        })
        .max_by_key(|(official, _)| *official)
        .map(|(_, key)| key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_official_trailers_and_rejects_arbitrary_video_urls() {
        let result = json!({"results": [
            {"site":"YouTube","type":"Trailer","official":true,"key":"https://example.org/"},
            {"site":"YouTube","type":"Teaser","official":true,"key":"abcdefghijk"},
            {"site":"YouTube","type":"Trailer","official":true,"key":"AbCd_-12345"},
            {"site":"YouTube","type":"Trailer","official":false,"key":"01234567890"}
        ]});
        assert_eq!(youtube_trailer(&result), Some("AbCd_-12345"));
        assert_eq!(youtube_trailer(&json!({"results":[]})), None);
    }
}
