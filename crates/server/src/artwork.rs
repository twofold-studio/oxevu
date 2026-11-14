use super::*;
use std::path::Path as FilePath;

fn missing() -> Error {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error":"artwork not found"})),
    )
}

async fn contained(path: &FilePath, root: &FilePath) -> Option<PathBuf> {
    let path = tokio::fs::canonicalize(path).await.ok()?;
    (path.starts_with(root) && tokio::fs::metadata(&path).await.ok()?.is_file()).then_some(path)
}

pub(super) async fn discover(
    path: &FilePath,
    root: &FilePath,
    tv: bool,
    kind: &str,
) -> Option<PathBuf> {
    let root = tokio::fs::canonicalize(root).await.ok()?;
    let path = contained(path, &root).await?;
    let parent = path.parent()?;
    let stem = path.file_stem()?.to_str()?;
    let suffix = if kind == "thumbnail" { "thumb" } else { kind };
    let mut names = vec![parent.join(format!("{stem}-{suffix}"))];
    if kind != "thumbnail" {
        let mut directory = Some(parent);
        while let Some(dir) = directory {
            // Generic artwork at a library root is ambiguous between playable items.
            if dir == root {
                break;
            }
            names.push(dir.join(kind));
            if kind == "poster" {
                names.push(dir.join("folder"));
            }
            if !tv {
                break;
            }
            directory = dir.parent();
        }
    }
    for name in names {
        for extension in ["jpg", "jpeg", "png", "webp"] {
            let candidate =
                name.with_file_name(format!("{}.{}", name.file_name()?.to_str()?, extension));
            if let Some(image) = contained(&candidate, &root).await {
                return Some(image);
            }
        }
    }
    None
}

pub(super) async fn attach(
    a: &App,
    data: &mut Value,
    path: &FilePath,
    root: &FilePath,
    tv: bool,
    errors: &mut Vec<String>,
) {
    for kind in ["poster", "thumbnail", "backdrop"] {
        let available = discover(path, root, tv, kind).await.is_some();
        let available = if !available && kind == "thumbnail" {
            match generate(a, path, root, data["id"].as_str().unwrap_or_default()).await {
                Ok(_) => true,
                Err(_) => {
                    errors.push(format!(
                        "Could not generate thumbnail for {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    false
                }
            }
        } else {
            available
        };
        data[format!("{kind}_url")] = if available {
            json!(format!(
                "/api/v1/items/{}/artwork/{kind}",
                data["id"].as_str().unwrap_or_default()
            ))
        } else {
            Value::Null
        };
    }
}

async fn generate(a: &App, path: &FilePath, root: &FilePath, id: &str) -> Result<PathBuf> {
    let _permit = a.artwork_jobs.acquire().await.map_err(bad)?;
    let root = tokio::fs::canonicalize(root).await.map_err(bad)?;
    let source = contained(path, &root).await.ok_or_else(missing)?;
    tokio::fs::create_dir_all(&a.artwork_cache)
        .await
        .map_err(bad)?;
    let destination = a.artwork_cache.join(format!("{id}.jpg"));
    if tokio::fs::try_exists(&destination).await.map_err(bad)? {
        return Ok(destination);
    }
    let temporary = a
        .artwork_cache
        .join(format!("{}.jpg", uuid::Uuid::new_v4()));
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        tokio::process::Command::new("ffmpeg")
            .args(["-nostdin", "-v", "error", "-threads", "1", "-i"])
            .arg(source)
            .args([
                "-frames:v",
                "1",
                "-vf",
                "scale=640:360:force_original_aspect_ratio=decrease",
                "-threads",
                "1",
                "-fs",
                "2097152",
                "-y",
            ])
            .arg(&temporary)
            .kill_on_drop(true)
            .status(),
    )
    .await;
    if !matches!(result, Ok(Ok(status)) if status.success()) {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(missing());
    }
    tokio::fs::rename(&temporary, &destination)
        .await
        .map_err(bad)?;
    Ok(destination)
}

pub(super) async fn serve(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, kind)): Path<(String, String)>,
) -> Result<Response> {
    user(&a, &h).await?;
    if !matches!(kind.as_str(), "poster" | "thumbnail" | "backdrop") {
        return Err(missing());
    }
    let row = sqlx::query("SELECT items.path,libraries.path root,libraries.kind FROM items JOIN libraries ON library_id=libraries.id WHERE items.id=?")
        .bind(&id).fetch_optional(&a.pool).await.map_err(db)?.ok_or_else(missing)?;
    let path = PathBuf::from(row.get::<String, _>("path"));
    let root = PathBuf::from(row.get::<String, _>("root"));
    let image = match discover(&path, &root, row.get::<String, _>("kind") == "tv", &kind).await {
        Some(image) => image,
        None if kind == "thumbnail" => generate(&a, &path, &root, &id).await?,
        None => return Err(missing()),
    };
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(image).await.map_err(|_| missing())?;
    if file.metadata().await.map_err(bad)?.len() > 20 * 1024 * 1024 {
        return Err(missing());
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(20 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(bad)?;
    let mime = if bytes.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        return Err(missing());
    };
    Ok(([(axum::http::header::CONTENT_TYPE, mime)], bytes).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sidecars_inherit_without_crossing_library_or_items() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let season = root.join("Show/Season 02");
        tokio::fs::create_dir_all(&season).await.unwrap();
        for name in [
            "Anna.mkv",
            "Screen.mkv",
            "Anna-poster.jpg",
            "poster.jpg",
            "Show/folder.jpg",
            "Show/Season 02/Show S02E01.mkv",
        ] {
            tokio::fs::write(root.join(name), b"fixture").await.unwrap();
        }
        assert!(
            discover(&root.join("Anna.mkv"), &root, false, "poster")
                .await
                .is_some()
        );
        assert!(
            discover(&root.join("Screen.mkv"), &root, false, "poster")
                .await
                .is_none()
        );
        assert_eq!(
            discover(&season.join("Show S02E01.mkv"), &root, true, "poster").await,
            Some(root.join("Show/folder.jpg"))
        );
        assert!(
            discover(&root.join("Anna.mkv"), &root, false, "backdrop")
                .await
                .is_none()
        );
        let outside = root.with_extension("jpg");
        tokio::fs::write(&outside, b"outside").await.unwrap();
        std::os::unix::fs::symlink(&outside, root.join("Screen-backdrop.jpg")).unwrap();
        assert!(
            discover(&root.join("Screen.mkv"), &root, false, "backdrop")
                .await
                .is_none()
        );
        tokio::fs::remove_dir_all(root).await.unwrap();
        tokio::fs::remove_file(outside).await.unwrap();
    }
}
