//! Safe, checksum-verified, no-overwrite media publication and disk accounting.
mod disk;
pub use disk::disk_usage;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

/// Immutable facts approved by the import preview.
pub struct Publication<'a> {
    pub root: &'a str,
    pub source: &'a str,
    pub destination: &'a str,
    pub source_stamp: &'a str,
    pub root_device: u64,
}
pub fn stamp(m: &std::fs::Metadata) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime(),
        m.mtime_nsec()
    )
}

pub fn publish_mode(
    f: &Publication<'_>,
    id: &str,
    copy: bool,
    force_copy: bool,
    cancelled: &dyn Fn() -> bool,
) -> std::io::Result<String> {
    let root = Path::new(f.root);
    let target = Path::new(f.destination);
    let source = Path::new(f.source);
    if std::fs::canonicalize(root)? != root || std::fs::metadata(root)?.dev() != f.root_device {
        return Err(std::io::Error::other("library mount changed"));
    }
    let parent = target
        .parent()
        .ok_or_else(|| std::io::Error::other("destination parent missing"))?;
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| std::io::Error::other("destination outside library"))?;
    let mut directory = root.to_path_buf();
    for part in relative.components() {
        if !matches!(part, std::path::Component::Normal(_)) {
            return Err(std::io::Error::other("invalid destination path"));
        }
        directory.push(part);
        match std::fs::symlink_metadata(&directory) {
            Ok(m) if !m.file_type().is_symlink() && m.is_dir() => {}
            Ok(_) => {
                return Err(std::io::Error::other(
                    "destination ancestor is not a real directory",
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => std::fs::create_dir(&directory)?,
            Err(e) => return Err(e),
        }
    }
    let checksum = hash(source, cancelled)?;
    if target.try_exists()? {
        if hash(target, cancelled)? == checksum {
            return Ok(checksum);
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "existing media differs; overwrite refused",
        ));
    }
    if stamp(&std::fs::metadata(source)?) != f.source_stamp {
        return Err(std::io::Error::other("source changed after preview"));
    }
    let staging = parent.join(format!(".oxevu-import-{id}.partial"));
    // This name belongs to the journal entry, so a retry can remove its leftover.
    if staging.try_exists()? {
        std::fs::remove_file(&staging)?;
    }
    let result = (|| {
        if force_copy {
            copy_file(source, &staging, cancelled)?;
        } else {
            match std::fs::hard_link(source, &staging) {
                Ok(()) => {}
                Err(e) if copy && e.raw_os_error() == Some(libc::EXDEV) => {
                    copy_file(source, &staging, cancelled)?
                }
                Err(e) => return Err(e),
            }
        }
        if hash(&staging, cancelled)? != checksum {
            return Err(std::io::Error::other("copy verification failed"));
        }
        std::fs::File::open(&staging)?.sync_all()?;
        // Atomic no-replace publication; an existing destination is never removed.
        std::fs::hard_link(&staging, target)?;
        std::fs::File::open(parent)?.sync_all()?;
        Ok(checksum)
    })();
    let _ = std::fs::remove_file(staging);
    result
}

fn copy_file(source: &Path, target: &Path, cancelled: &dyn Fn() -> bool) -> std::io::Result<()> {
    let mut input = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(source)?;
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o644)
        .open(target)?;
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        if cancelled() {
            return Err(std::io::Error::other("import interrupted"));
        }
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        output.write_all(&buffer[..n])?;
    }
    output.sync_all()
}
pub fn hash(path: &Path, cancelled: &dyn Fn() -> bool) -> std::io::Result<String> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::other("media is not a regular file"));
    }
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        if cancelled() {
            return Err(std::io::Error::other("import interrupted"));
        }
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
