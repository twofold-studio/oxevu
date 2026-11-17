use super::*;

#[derive(Debug, PartialEq)]
struct Guess {
    kind: &'static str,
    title: String,
    year: Option<u64>,
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn guess(path: &str) -> Result<Option<Guess>> {
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|p| p.to_str())
        .unwrap_or(path)
        .replace(['.', '_'], " ");
    let prefix = regex::Regex::new(r"^\[[^\]]+\]\s*").map_err(bad)?;
    let name = prefix.replace(&name, "");
    let facts = release_facts::facts(&name)?;
    let boundary = regex::Regex::new(r"(?i)(?:^|[\s(\[])(?:S\d{1,3}(?:E\d{1,4})?|(?:19|20)\d{2}|(?:4320|2160|1080|720|576|480)p|web[- ]?(?:dl|rip)|blu[- ]?ray|x26[45]|h26[45]|hevc)\b").map_err(bad)?;
    let title = name[..boundary.find(&name).map_or(name.len(), |m| m.start())]
        .trim()
        .trim_matches(['-', '(', ')', '[', ']'])
        .trim();
    if title.len() < 2 || title.len() > 200 {
        return Ok(None);
    }
    Ok(Some(Guess {
        kind: if facts["season"].is_number() {
            "tv"
        } else {
            "movie"
        },
        title: title.to_owned(),
        year: facts["year"].as_u64(),
    }))
}

fn from_files(data: &Value) -> Result<Option<Guess>> {
    let mut found = None;
    for file in data["files"].as_array().into_iter().flatten() {
        let path = file["path"].as_str().unwrap_or("");
        if !auto_import::is_video(path) {
            continue;
        }
        if data["input"]["selected_files"]
            .as_array()
            .is_some_and(|selection| !selection.contains(&file["index"]))
        {
            continue;
        }
        let Some(next) = guess(path)? else {
            return Ok(None);
        };
        if found.as_ref().is_some_and(|prior: &Guess| {
            prior.kind != next.kind
                || normalized(&prior.title) != normalized(&next.title)
                || prior.year != next.year
        }) {
            return Ok(None);
        }
        found = Some(next);
    }
    Ok(found)
}

fn matches(guess: &Guess, value: &Value) -> bool {
    let name = if guess.kind == "tv" { "name" } else { "title" };
    let original = if guess.kind == "tv" {
        "original_name"
    } else {
        "original_title"
    };
    let same_name = [name, original].iter().any(|key| {
        value[*key]
            .as_str()
            .is_some_and(|n| normalized(n) == normalized(&guess.title))
    });
    let date = if guess.kind == "tv" {
        "first_air_date"
    } else {
        "release_date"
    };
    same_name
        && guess.year.is_none_or(|year| {
            value[date]
                .as_str()
                .and_then(|s| s.get(..4))
                .and_then(|s| s.parse::<u64>().ok())
                == Some(year)
        })
}

async fn find(a: &App, guess: &Guess) -> Result<Option<String>> {
    let c = integrations::load(a).await?;
    let result = integrations::tmdb(
        &c,
        &format!("search/{}", guess.kind),
        &[
            ("query", guess.title.clone()),
            ("include_adult", "false".into()),
            ("page", "1".into()),
        ],
    )
    .await?;
    let candidates: Vec<_> = result["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| matches(guess, v))
        .collect();
    if candidates.len() != 1 {
        return Ok(None);
    }
    let Some(id) = candidates[0]["id"].as_u64() else {
        return Ok(None);
    };
    let title = catalog::ensure_title(a, guess.kind, id).await?;
    Ok(title["id"].as_str().map(str::to_owned))
}

pub(super) async fn identify(a: &App, id: &str) -> Result<()> {
    let raw: Option<String> = sqlx::query_scalar("SELECT data FROM acquisition_downloads WHERE id=? AND title_id IS NULL AND state!='removed'")
        .bind(id).fetch_optional(&a.pool).await.map_err(db)?;
    let Some(raw) = raw else {
        return Ok(());
    };
    let data = decode(&raw)?;
    if data["files"].as_array().is_none_or(Vec::is_empty) {
        return Ok(());
    }
    let guessed = from_files(&data)?;
    let result = if let Some(guess) = guessed {
        tokio::time::timeout(std::time::Duration::from_secs(60), find(a, &guess))
            .await
            .unwrap_or_else(|_| Err(bad("recognition timed out")))
    } else {
        Ok(None)
    };
    let (title, state, message) = match result {
        Ok(Some(title)) => (
            Some(title),
            "matched",
            "Catalog title recognized automatically.",
        ),
        Ok(None) => (
            None,
            "needs_review",
            "No unique title match. Choose the correct catalog title before importing.",
        ),
        Err(_) => (
            None,
            "unavailable",
            "Metadata lookup unavailable. Enable TMDB in Integrations; recognition will retry.",
        ),
    };
    let recognition = json!({"state":state,"message":message});
    sqlx::query("UPDATE acquisition_downloads SET title_id=COALESCE(title_id,?),data=json_set(data,'$.recognition',json(?),'$.recognition_retry_at',unixepoch()+300) WHERE id=? AND title_id IS NULL AND state!='removed'")
        .bind(&title).bind(recognition.to_string()).bind(id).execute(&a.pool).await.map_err(db)?;
    event(a, id, message).await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    kind: String,
    provider_id: u64,
}

pub(super) async fn associate(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Identity>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    title_key(&input.kind, input.provider_id)?;
    let _permit = a
        .acquisition
        .import_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("An import is active; wait before changing its title."))?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM acquisition_downloads WHERE id=? AND state!='removed')",
    )
    .bind(&id)
    .fetch_one(&a.pool)
    .await
    .map_err(db)?;
    if !exists {
        return Err(bad("unknown download"));
    }
    let imported: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acquisition_imports WHERE download_id=? AND state IN ('copying','published','scanned'))").bind(&id).fetch_one(&a.pool).await.map_err(db)?;
    if imported {
        return Err(bad(
            "This transfer already has an import. Edit its metadata in the library instead.",
        ));
    }
    let title = catalog::ensure_title(&a, &input.kind, input.provider_id).await?;
    sqlx::query("UPDATE acquisition_downloads SET title_id=?,data=json_set(json_remove(data,'$.import_warning'),'$.input.title_id',?,'$.recognition',json('{\"state\":\"matched\",\"message\":\"Title chosen manually.\"}')) WHERE id=? AND state!='removed'")
        .bind(title["id"].as_str()).bind(title["id"].as_str()).bind(&id).execute(&a.pool).await.map_err(db)?;
    Ok(Json(json!({"download":downloads::view(&a,&id).await?})))
}

pub(super) async fn tick(a: &App) -> Result<()> {
    let c = config::load(a).await?;
    if !c.enabled || !c.auto_identify {
        return Ok(());
    }
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM acquisition_downloads WHERE title_id IS NULL AND state!='removed' AND json_array_length(json_extract(data,'$.files'))>0 AND COALESCE(json_extract(data,'$.recognition_retry_at'),0)<=unixepoch() ORDER BY created_at LIMIT 3")
        .fetch_all(&a.pool).await.map_err(db)?;
    for id in ids {
        if a.acquisition.is_stopping() {
            break;
        }
        identify(a, &id).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn automatic_import_recognizes_release_names_and_rejects_mixed_packs() {
        assert_eq!(
            guess("Godzilla.Minus.One.2023.1080p.BluRay.x265.DDP5.1.mkv").unwrap(),
            Some(Guess {
                kind: "movie",
                title: "Godzilla Minus One".into(),
                year: Some(2023)
            })
        );
        assert_eq!(
            guess("[Group] Arcane.S02E03.1080p.WEB-DL.mkv").unwrap(),
            Some(Guess {
                kind: "tv",
                title: "Arcane".into(),
                year: None
            })
        );
        let files = json!({"files":[{"path":"Arcane.S01E01.mkv","index":0},{"path":"Arcane.S01E02.mkv","index":1},{"path":"sample.mkv","index":2}]});
        assert_eq!(from_files(&files).unwrap().unwrap().title, "Arcane");
        assert!(from_files(&json!({"files":[{"path":"Movie.One.2023.mkv"},{"path":"Different.Movie.2023.mkv"}]})).unwrap().is_none());
    }
    #[test]
    fn automatic_import_requires_exact_title_and_matching_release_year() {
        let guess = Guess {
            kind: "movie",
            title: "Godzilla Minus One".into(),
            year: Some(2023),
        };
        assert!(matches(
            &guess,
            &json!({"title":"Godzilla Minus One","release_date":"2023-11-03"})
        ));
        assert!(!matches(
            &guess,
            &json!({"title":"Godzilla","release_date":"2023-11-03"})
        ));
        assert!(!matches(
            &guess,
            &json!({"title":"Godzilla Minus One","release_date":"2022-11-03"})
        ));
    }
}
