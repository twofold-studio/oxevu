/// Extract the title after SxxExx, leaving filesystem names untouched.
/// Release markers are removed only as suffixes so title punctuation survives.
pub fn clean(stem: &str, episode_end: usize, fallback_number: u32) -> String {
    let tail = &stem[episode_end..];
    let title = tail.trim_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '_' | '.'));
    let words: Vec<&str> = title.split_whitespace().collect();
    let end = words
        .iter()
        .position(|word| {
            let token = word
                .trim_matches(|c| matches!(c, '(' | '[' | ')' | ']'))
                .to_ascii_lowercase();
            matches!(
                token.as_str(),
                "480p"
                    | "576p"
                    | "720p"
                    | "1080p"
                    | "1080i"
                    | "2160p"
                    | "4320p"
                    | "web-dl"
                    | "webrip"
                    | "bluray"
                    | "bdrip"
                    | "hdtv"
                    | "x264"
                    | "x265"
                    | "h264"
                    | "h265"
                    | "hevc"
                    | "repack"
                    | "proper"
            )
        })
        .unwrap_or(words.len());
    let cleaned = words[..end].join(" ");
    let cleaned =
        cleaned.trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '(' | '['));
    if cleaned.is_empty() {
        format!("Episode {fallback_number}")
    } else {
        cleaned.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::clean;
    #[test]
    fn strips_release_suffixes_and_preserves_episode_title() {
        for (stem, end, expected) in [
            (
                "Arcane (2021) - S01E01 - Welcome to the Playground (1080p NF WEB-DL x265 t3nzin)",
                22,
                "Welcome to the Playground",
            ),
            (
                "Arcane S02E01 Heavy Is the Crown 1080p NF WEB-DL DDP5 1 Atmos H 264-FLUX",
                13,
                "Heavy Is the Crown",
            ),
            ("Show S01E01 1:23:45", 11, "1:23:45"),
            ("Show S01E01", 11, "Episode 1"),
        ] {
            assert_eq!(clean(stem, end, 1), expected);
        }
    }
}
