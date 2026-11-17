use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    fs::File,
    io,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Component, Path},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Stamp {
    device: u64,
    inode: u64,
    size: u64,
    modified: i64,
    nanos: i64,
    pub links: u64,
    pub allocated: u64,
}
fn stamp(meta: &std::fs::Metadata) -> Stamp {
    Stamp {
        device: meta.dev(),
        inode: meta.ino(),
        size: meta.len(),
        modified: meta.mtime(),
        nanos: meta.mtime_nsec(),
        links: meta.nlink(),
        allocated: meta.blocks().saturating_mul(512),
    }
}
impl Stamp {
    pub(super) fn size(&self) -> u64 {
        self.size
    }
}

fn open_child(parent: &File, part: &std::ffi::OsStr) -> io::Result<File> {
    let part = CString::new(part.as_bytes())?;
    // SAFETY: parent stays open and part is a live NUL-terminated component.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            part.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: openat returned a newly owned descriptor.
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn parent(root: &Path, path: &Path) -> io::Result<(File, CString)> {
    if !root.is_absolute() {
        return Err(io::Error::other("library root must be absolute"));
    }
    let relative = path.strip_prefix(root).map_err(io::Error::other)?;
    let leaf = relative
        .file_name()
        .ok_or_else(|| io::Error::other("missing media filename"))?;
    let mut directory = File::open("/")?;
    for part in root
        .components()
        .chain(relative.parent().into_iter().flat_map(Path::components))
    {
        match part {
            Component::RootDir => {}
            Component::Normal(name) => directory = open_child(&directory, name)?,
            _ => return Err(io::Error::other("invalid media path")),
        }
    }
    Ok((directory, CString::new(leaf.as_bytes())?))
}
fn inspect_at(parent: &File, name: &CString) -> io::Result<Option<Stamp>> {
    // Open the file without following links, then inspect that descriptor.
    // SAFETY: the directory descriptor and NUL-terminated filename are live.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        let error = io::Error::last_os_error();
        return if error.kind() == io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(error)
        };
    }
    // SAFETY: openat returned a newly owned descriptor.
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::other("media must be a regular file"));
    }
    Ok(Some(stamp(&metadata)))
}
pub(super) fn inspect(root: &Path, path: &Path) -> io::Result<Option<Stamp>> {
    let (directory, name) = parent(root, path)?;
    inspect_at(&directory, &name)
}
pub(super) fn remove(root: &Path, path: &Path, expected: &Stamp) -> io::Result<()> {
    let (directory, name) = parent(root, path)?;
    if inspect_at(&directory, &name)?.as_ref() != Some(expected) {
        return Err(io::Error::other("media changed; preview removal again"));
    }
    // SAFETY: unlinkat removes only this entry in the already-open directory;
    // intermediate symlinks cannot redirect it outside the registered root.
    if unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), 0) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // The unlink has happened. A durability error must not make the caller
    // restore a "present" catalog state for a file that is already gone.
    let _ = directory.sync_all();
    Ok(())
}
