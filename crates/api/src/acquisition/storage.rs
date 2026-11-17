use super::*;
use std::path::{Component, Path};

pub(super) async fn canonical_candidate(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(bad("absolute path without parent traversal required"));
    }
    let mut ancestor = path.to_path_buf();
    let mut suffix = Vec::new();
    loop {
        match tokio::fs::canonicalize(&ancestor).await {
            Ok(mut resolved) => {
                for part in suffix.iter().rev() {
                    resolved.push(part);
                }
                return Ok(resolved);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let name = ancestor
                    .file_name()
                    .ok_or_else(|| bad("path has no existing ancestor"))?
                    .to_owned();
                suffix.push(name);
                ancestor.pop();
            }
            Err(_) => return Err(bad("cannot resolve storage path")),
        }
    }
}
pub(super) fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}
pub(super) async fn validate_download_root(a: &App, path: &Path) -> Result<()> {
    let path = canonical_candidate(path).await?;
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = tokio::fs::canonicalize(source)
        .await
        .map_err(|_| bad("source path unavailable"))?;
    if overlaps(&path, &source) || path == Path::new("/") {
        return Err(bad(
            "download folder must be outside source and filesystem root",
        ));
    }
    let roots: Vec<String> =
        sqlx::query_scalar("SELECT path FROM library_roots UNION SELECT path FROM libraries")
            .fetch_all(&a.pool)
            .await
            .map_err(db)?;
    for root in roots {
        if overlaps(&path, &canonical_candidate(Path::new(&root)).await?) {
            return Err(bad("downloads and library roots must not overlap"));
        }
    }
    let data = a
        .acquisition
        .directory
        .parent()
        .ok_or_else(|| bad("data directory unavailable"))?;
    let data = canonical_candidate(data).await?;
    if data.starts_with(&path) {
        return Err(bad("download folder cannot contain application state"));
    }
    for protected in [
        a.artwork_cache.clone(),
        a.playback
            .scratch
            .parent()
            .unwrap_or(&a.playback.scratch)
            .to_path_buf(),
        a.acquisition.directory.join("engine"),
    ] {
        if overlaps(&path, &canonical_candidate(&protected).await?) {
            return Err(bad("download folder overlaps application cache"));
        }
    }
    Ok(())
}
pub(super) async fn used_bytes(path: PathBuf) -> Result<u64> {
    tokio::task::spawn_blocking(move || {
        let mut dirs = vec![path];
        let mut size = 0_u64;
        let mut visited = 0;
        while let Some(dir) = dirs.pop() {
            for entry in
                std::fs::read_dir(dir).map_err(|_| bad("cannot inspect download storage"))?
            {
                visited += 1;
                if visited > 100000 {
                    return Err(bad("download storage exceeds entry limit"));
                }
                let entry = entry.map_err(|_| bad("cannot inspect download storage"))?;
                let meta = entry
                    .metadata()
                    .map_err(|_| bad("cannot inspect download storage"))?;
                if entry
                    .file_type()
                    .map_err(|_| bad("cannot inspect download storage"))?
                    .is_symlink()
                {
                    continue;
                }
                if meta.is_dir() {
                    dirs.push(entry.path());
                } else if meta.is_file() {
                    size = size.saturating_add(meta.len());
                }
            }
        }
        Ok(size)
    })
    .await
    .map_err(|_| bad("storage accounting interrupted"))?
}
pub(super) async fn free_bytes(path: PathBuf) -> Result<u64> {
    tokio::task::spawn_blocking(move || {
        use std::os::unix::ffi::OsStrExt;
        let name = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| bad("invalid storage path"))?;
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: name is NUL terminated and statvfs writes the supplied struct on success.
        if unsafe { libc::statvfs(name.as_ptr(), stats.as_mut_ptr()) } != 0 {
            return Err(bad("cannot measure free storage"));
        }
        // SAFETY: the successful call initialized every field.
        let stats = unsafe { stats.assume_init() };
        Ok(stats.f_bavail.saturating_mul(stats.f_frsize))
    })
    .await
    .map_err(|_| bad("storage check interrupted"))?
}
