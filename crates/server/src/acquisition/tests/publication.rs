use super::super::*;
use std::os::unix::fs::{MetadataExt, symlink};

#[test]
fn publication_refuses_overwrite_symlinks_and_changed_sources() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("library");
    let source = directory.path().join("source.mp4");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(&source, b"original fixture").unwrap();
    let meta = std::fs::metadata(&source).unwrap();
    let runtime = Runtime::new(directory.path().join("app"));
    let mut plan = import_plan::File {
        file_index: 0,
        source: source.to_string_lossy().into_owned(),
        destination: root.join("film.mp4").to_string_lossy().into_owned(),
        size: meta.len(),
        coverage: Vec::new(),
        title_id: String::new(),
        root_device: meta.dev(),
        root: root.to_string_lossy().into_owned(),
        source_stamp: import_io::stamp(&meta),
    };
    std::fs::write(&plan.destination, b"existing different media").unwrap();
    assert!(import_io::publish(&plan, "fixture", false, &runtime).is_err());
    assert_eq!(
        std::fs::read(&plan.destination).unwrap(),
        b"existing different media"
    );
    std::fs::remove_file(&plan.destination).unwrap();
    let elsewhere = directory.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();
    symlink(&elsewhere, root.join("escape")).unwrap();
    plan.destination = root.join("escape/film.mp4").to_string_lossy().into_owned();
    assert!(import_io::publish(&plan, "fixture", false, &runtime).is_err());
    assert!(!elsewhere.join("film.mp4").exists());
    plan.destination = root.join("film.mp4").to_string_lossy().into_owned();
    std::fs::write(&source, b"changed fixture after preview").unwrap();
    assert!(import_io::publish(&plan, "fixture", false, &runtime).is_err());
    assert!(!root.join("film.mp4").exists());
}
