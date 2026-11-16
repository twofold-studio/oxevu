use super::*;

// Claims reserve catalog units, not entire shows. A long-running seed must not
// prevent acquiring the next episode, and competing packs must not grab twice.
pub(super) async fn candidates(a: &App, input: &downloads::Input) -> Result<Vec<(i64, i64)>> {
    if !input.only_wanted {
        return Ok(Vec::new());
    }
    let title_id = input
        .title_id
        .as_deref()
        .ok_or_else(|| bad("automatic transfer requires a title"))?;
    let release = if let Some(release_id) = &input.release_id {
        let raw: String = sqlx::query_scalar("SELECT data FROM acquisition_releases WHERE id=?")
            .bind(release_id)
            .fetch_one(&a.pool)
            .await
            .map_err(db)?;
        decode(&raw)?
    } else {
        Value::Null
    };
    let files = if let Some(raw) = &input.torrent_base64 {
        Some(torrent_files::validate(&downloads::decode_torrent(raw)?, u64::MAX)?.1)
    } else {
        None
    };
    if release.is_null() && files.is_none() {
        return Err(bad(
            "wanted-only magnet selection requires a release identity",
        ));
    }
    let raw: String = sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id=?")
        .bind(title_id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let title = decode(&raw)?;
    let items = wanted::inventory(a, &title).await?;
    if items.iter().any(|i| i["online"] != true) {
        return Err(bad(
            "existing library media is offline; automatic acquisition suspended",
        ));
    }
    let units = wanted::units(a, &title, &items).await?;
    let season = release["season"].as_i64();
    let first = release["episode"].as_i64();
    let last = release["episode_end"].as_i64().or(first);
    let mut candidates: Vec<(i64, i64)> = units
        .iter()
        .filter(|u| u["wanted"] == true && u["available"] != true)
        .filter(|u| {
            release.is_null()
                || title["kind"] == "movie"
                || (u["season"].as_i64() == season
                    && first.is_none_or(|first| {
                        u["episode"]
                            .as_i64()
                            .is_some_and(|n| n >= first && n <= last.unwrap_or(first))
                    }))
        })
        .map(|u| {
            (
                u["season"].as_i64().unwrap_or(-1),
                u["episode"].as_i64().unwrap_or(-1),
            )
        })
        .collect();
    if let Some(files) = files {
        candidates.retain(|(season, episode)| file_covers(&files, None, *season, *episode));
    }
    Ok(candidates)
}
fn file_covers(files: &Value, selected: Option<&[usize]>, season: i64, episode: i64) -> bool {
    files.as_array().into_iter().flatten().any(|file| {
        let path = file["path"].as_str().unwrap_or("");
        if !auto_import::is_video(path)
            || selected.is_some_and(|v| {
                !v.contains(&(file["index"].as_u64().unwrap_or(u64::MAX) as usize))
            })
        {
            return false;
        }
        season < 0
            || auto_import::file_coverage(path).is_ok_and(|coverage| {
                coverage
                    .iter()
                    .any(|e| e.season as i64 == season && e.episode as i64 == episode)
            })
    })
}
pub(super) async fn retain_files(
    a: &App,
    id: &str,
    files: &Value,
    selected: Option<&[usize]>,
) -> Result<()> {
    let rows =
        sqlx::query("SELECT title_id,season,episode FROM acquisition_claims WHERE download_id=?")
            .bind(id)
            .fetch_all(&a.pool)
            .await
            .map_err(db)?;
    for row in rows {
        let season: i64 = row.get("season");
        let episode: i64 = row.get("episode");
        if !file_covers(files, selected, season, episode) {
            sqlx::query(
                "DELETE FROM acquisition_claims WHERE download_id=? AND season=? AND episode=?",
            )
            .bind(id)
            .bind(season)
            .bind(episode)
            .execute(&a.pool)
            .await
            .map_err(db)?;
        }
    }
    Ok(())
}
pub(super) async fn reserve(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    title: &str,
    units: &[(i64, i64)],
) -> Result<(Vec<(i64, i64)>, Option<String>)> {
    sqlx::query("DELETE FROM acquisition_claims WHERE title_id=? AND download_id IN (SELECT id FROM acquisition_downloads WHERE state IN ('removed','error','duplicate'))")
        .bind(title).execute(&mut **tx).await.map_err(db)?;
    let mut free = Vec::new();
    let mut existing = None;
    for (season, episode) in units {
        let owner:Option<String>=sqlx::query_scalar("SELECT download_id FROM acquisition_claims WHERE title_id=? AND season=? AND episode=?")
            .bind(title).bind(season).bind(episode).fetch_optional(&mut **tx).await.map_err(db)?;
        if owner.is_some() {
            existing = owner;
        } else {
            free.push((*season, *episode));
        }
    }
    Ok((free, existing))
}
pub(super) async fn unclaimed(a: &App, title: &str, units: &[Value]) -> Result<Vec<Value>> {
    let rows=sqlx::query("SELECT c.season,c.episode FROM acquisition_claims c JOIN acquisition_downloads d ON d.id=c.download_id WHERE c.title_id=? AND d.state NOT IN ('removed','error','duplicate')")
        .bind(title).fetch_all(&a.pool).await.map_err(db)?;
    Ok(units
        .iter()
        .filter(|u| {
            !rows.iter().any(|r| {
                r.get::<i64, _>("season") == u["season"].as_i64().unwrap_or(-1)
                    && r.get::<i64, _>("episode") == u["episode"].as_i64().unwrap_or(-1)
            })
        })
        .cloned()
        .collect())
}
