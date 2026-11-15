use crate::*;
use serde::Deserialize;
use std::path::Path as FilePath;

static JOBS: Semaphore = Semaphore::const_new(4);

#[derive(Deserialize)]
pub(crate) struct Input {
    path: String,
}

pub(crate) async fn suggest(
    State(a): State<App>,
    headers: HeaderMap,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &headers).await?;
    let _permit = JOBS.try_acquire().map_err(|_| bad("folder lookup busy"))?;
    tokio::time::timeout(std::time::Duration::from_secs(2), directories(&input.path))
        .await
        .map_err(|_| bad("folder lookup timed out"))?
        .map(Json)
}

async fn directories(input: &str) -> Result<Value> {
    if input.len() > 4096 || input.contains('\0') || !FilePath::new(input).is_absolute() {
        return Err(bad("type an absolute folder path"));
    }
    let (parent, prefix) = if input.ends_with('/') {
        (FilePath::new(input), "")
    } else {
        let path = FilePath::new(input);
        (
            path.parent().ok_or_else(|| bad("invalid folder path"))?,
            path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        )
    };
    let parent = tokio::fs::canonicalize(parent)
        .await
        .map_err(|_| bad("folder unavailable"))?;
    // Virtual trees are neither useful library roots nor safe places to enumerate.
    let blocked = |path: &FilePath| {
        ["/proc", "/sys", "/dev", "/nix/store"]
            .iter()
            .any(|p| path.starts_with(p))
    };
    if blocked(&parent) {
        return Err(bad("folder unavailable"));
    }
    let mut entries = tokio::fs::read_dir(&parent)
        .await
        .map_err(|_| bad("folder unavailable"))?;
    let mut paths = Vec::new();
    let mut count = 0;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| bad("folder unavailable"))?
    {
        count += 1;
        if count > 2048 {
            break;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !name.starts_with(prefix) || (name.starts_with('.') && !prefix.starts_with('.')) {
            continue;
        }
        let path = entry.path();
        let Ok(resolved) = tokio::fs::canonicalize(&path).await else {
            continue;
        };
        if blocked(&resolved)
            || !tokio::fs::metadata(&resolved)
                .await
                .is_ok_and(|m| m.is_dir())
        {
            continue;
        }
        paths.push(json!({"name":name,"path":format!("{}/", path.to_string_lossy())}));
    }
    paths.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let truncated = count > 2048 || paths.len() > 20;
    paths.truncate(20);
    Ok(json!({"paths":paths,"truncated":truncated}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn suggestions_require_admin_and_include_only_matching_directories() {
        let a = crate::tests::app().await;
        let viewer = crate::tests::account(&a, "viewer", false).await;
        assert!(
            suggest(State(a), viewer, Json(Input { path: "/".into() }))
                .await
                .is_err()
        );
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir_all(root.join("Movies"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(root.join("TV")).await.unwrap();
        tokio::fs::write(root.join("Movie.mkv"), b"fixture")
            .await
            .unwrap();
        let result = directories(&format!("{}/Mo", root.display()))
            .await
            .unwrap();
        assert_eq!(result["paths"].as_array().unwrap().len(), 1);
        assert_eq!(result["paths"][0]["name"], "Movies");
        assert!(directories("relative").await.is_err());
        assert!(directories("/proc/").await.is_err());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
