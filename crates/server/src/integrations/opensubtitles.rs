use super::*;
use axum::extract::Query;
#[derive(Deserialize)]
pub(crate) struct Search {
    language: String,
}
fn configured(c: &Config) -> Result<&Provider> {
    let p = &c.opensubtitles;
    if !p.enabled || p.api_key.is_empty() || p.username.is_empty() || p.password.is_empty() {
        return Err(bad(
            "OpenSubtitles requires enabled API key, username and password",
        ));
    }
    Ok(p)
}
fn language(language: &str) -> Result<()> {
    if !(2..=8).contains(&language.len())
        || !language
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'-')
    {
        return Err(bad("invalid subtitle language"));
    }
    Ok(())
}
pub(crate) async fn subtitle_search(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<Search>,
) -> Result<Json<Value>> {
    users::authorize_item(&a, &h, &id, true).await?;
    language(&q.language)?;
    let item = item_data(&a, &h, &id).await?;
    let c = load(&a).await?;
    let p = configured(&c)?;
    let query = if item["kind"] == "episode" {
        item["show_title"].as_str()
    } else {
        item["title"].as_str()
    }
    .unwrap_or("");
    let mut parameters = vec![
        ("query", query.to_owned()),
        ("languages", q.language.clone()),
    ];
    if let Some(season) = item["season_number"].as_u64() {
        parameters.push(("season_number", season.to_string()));
    }
    if let Some(episode) = item["episode_number"].as_u64() {
        parameters.push(("episode_number", episode.to_string()));
    }
    let data = json_response(
        client()?
            .get("https://api.opensubtitles.com/api/v1/subtitles")
            .header("Api-Key", &p.api_key)
            .query(&parameters)
            .send()
            .await
            .map_err(|_| bad("OpenSubtitles search failed"))?,
    )
    .await?;
    let results=data["data"].as_array().into_iter().flatten().take(30).flat_map(|row|{
        let attributes=&row["attributes"];
        attributes["files"].as_array().into_iter().flatten().take(5).map(move |file|json!({"id":file["file_id"].as_u64().map(|v|v.to_string()),"label":attributes["release"],"language":attributes["language"],"hearing_impaired":attributes["hearing_impaired"],"download_count":attributes["download_count"]}))
    }).collect::<Vec<_>>();
    Ok(Json(json!({"results":results})))
}
#[derive(Deserialize)]
pub(crate) struct Download {
    id: String,
    language: String,
}
pub(crate) async fn subtitle_download(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(d): Json<Download>,
) -> Result<Json<Value>> {
    let u = users::authorize_item(&a, &h, &id, true).await?;
    if u["can_transcode"] != true {
        return Err(users::forbidden());
    }
    language(&d.language)?;
    let file_id =
        d.id.parse::<u64>()
            .map_err(|_| bad("invalid subtitle file id"))?;
    let _permit = a
        .artwork_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("subtitle worker busy"))?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM item_subtitles")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    if count >= 64 {
        return Err(bad("downloaded subtitle cache limit reached"));
    }
    let item_count: i64 = sqlx::query_scalar("SELECT count(*) FROM item_subtitles WHERE item_id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    if item_count >= 16 {
        return Err(bad("item subtitle limit reached"));
    }
    let c = load(&a).await?;
    let p = configured(&c)?;
    let http = client()?;
    let login = json_response(
        http.post("https://api.opensubtitles.com/api/v1/login")
            .header("Api-Key", &p.api_key)
            .json(&json!({"username":p.username,"password":p.password}))
            .send()
            .await
            .map_err(|_| bad("OpenSubtitles login failed"))?,
    )
    .await?;
    let token = login["token"]
        .as_str()
        .ok_or_else(|| bad("OpenSubtitles login did not return a token"))?;
    let response = json_response(
        http.post("https://api.opensubtitles.com/api/v1/download")
            .header("Api-Key", &p.api_key)
            .bearer_auth(token)
            .json(&json!({"file_id":file_id,"sub_format":"srt"}))
            .send()
            .await
            .map_err(|_| bad("OpenSubtitles download request failed"))?,
    )
    .await?;
    let url = reqwest::Url::parse(
        response["link"]
            .as_str()
            .ok_or_else(|| bad("OpenSubtitles returned no download link"))?,
    )
    .map_err(|_| bad("invalid subtitle download link"))?;
    if url.scheme() != "https"
        || !url
            .host_str()
            .is_some_and(|s| s == "opensubtitles.com" || s.ends_with(".opensubtitles.com"))
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(bad("untrusted subtitle download host"));
    }
    let data = bytes(
        http.get(url)
            .send()
            .await
            .map_err(|_| bad("subtitle download failed"))?,
        8 * 1024 * 1024,
    )
    .await?;
    let directory = a
        .artwork_cache
        .parent()
        .ok_or_else(|| bad("data directory unavailable"))?
        .join("subtitles");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|_| bad("cannot create subtitle cache"))?;
    if crate::playback::runtime::disk_usage(&directory)
        .await
        .unwrap_or(u64::MAX)
        > 504 * 1024 * 1024
    {
        return Err(bad("downloaded subtitle disk limit reached"));
    }
    let track = uuid::Uuid::new_v4().to_string();
    let source = directory.join(format!("{track}.srt"));
    if tokio::fs::write(&source, data).await.is_err() {
        let _ = tokio::fs::remove_file(&source).await;
        return Err(bad("cannot cache subtitle"));
    }
    let converted = crate::subtitles::convert(&a, &source, None).await;
    let _ = tokio::fs::remove_file(&source).await;
    let converted = converted?;
    let target = directory.join(format!("{track}.vtt"));
    if tokio::fs::write(&target, converted).await.is_err() {
        let _ = tokio::fs::remove_file(&target).await;
        return Err(bad("cannot save subtitle"));
    }
    let result = sqlx::query("INSERT INTO item_subtitles VALUES(?,?,?,?)")
        .bind(&track)
        .bind(&id)
        .bind(&d.language)
        .bind(target.to_string_lossy().as_ref())
        .execute(&a.pool)
        .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(target).await;
    }
    result.map_err(db)?;
    Ok(Json(
        json!({"track":{"id":format!("download-{track}"),"label":format!("{} (OpenSubtitles)",d.language),"language":d.language,"codec":"vtt","default":false,"forced":false,"supported":true,"url":format!("/api/v1/items/{id}/subtitles/download-{track}")}}),
    ))
}
