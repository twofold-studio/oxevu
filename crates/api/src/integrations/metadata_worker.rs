//! One bounded metadata job at a time; durable failures survive scans/restarts.
use super::*;

pub(crate) async fn run(a: App) {
    loop {
        let result = tick(&a).await;
        if let Err(error) = &result {
            eprintln!("metadata queue unavailable: {}", error.0);
        }
        tokio::time::sleep(std::time::Duration::from_secs(
            if matches!(result, Ok(true)) { 1 } else { 15 },
        ))
        .await;
    }
}

async fn tick(a: &App) -> Result<bool> {
    let Ok(_permit) = a.scan.clone().try_acquire_owned() else {
        return Ok(false);
    };
    let Some(job) = oxevu_db::metadata_jobs::next(&a.pool).await.map_err(db)? else {
        return Ok(false);
    };
    let config = load(a).await?;
    let tmdb = config.tmdb.enabled && !config.tmdb.api_key.is_empty();
    let omdb = config.omdb.enabled && !config.omdb.api_key.is_empty();
    let provider = super::metadata::ProviderSource(config);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(90),
        oxevu_library::automatic_metadata::enrich(&a.pool, &job, &provider, tmdb, omdb),
    )
    .await;
    let error = match result {
        Ok(Ok(())) => return Ok(true),
        Ok(Err(error)) => error.to_string(),
        Err(_) => "Metadata lookup timed out; use Identify or wait for retry".into(),
    };
    oxevu_db::metadata_jobs::failed(&a.pool, &job.id, &error)
        .await
        .map_err(db)?;
    Ok(true)
}
