//! Ignore explicitly named release samples, never arbitrary short films.
use std::path::Path;

pub(crate) fn sample(path: &Path) -> bool {
    path.components().any(|part| {
        part.as_os_str()
            .to_str()
            .is_some_and(|s| s.eq_ignore_ascii_case("sample") || s.eq_ignore_ascii_case("samples"))
    }) || path.file_stem().and_then(|s| s.to_str()).is_some_and(|s| {
        let s = s.to_ascii_lowercase();
        s == "sample"
            || [".sample", "-sample", "_sample", " sample"]
                .iter()
                .any(|suffix| s.ends_with(suffix))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn skips_release_samples_without_a_duration_cutoff() {
        for file in [
            "/movies/Film/Sample/clip.avi",
            "/movies/Film/film_sample.mkv",
        ] {
            assert!(sample(Path::new(file)));
        }
        for file in [
            "/movies/Film/film.mkv",
            "/movies/Sample People (2025)/film.mkv",
            "/movies/Sample People (2025)/Sample.People.2025.mkv",
            "/movies/short-film.mkv",
        ] {
            assert!(!sample(Path::new(file)));
        }
    }
}
