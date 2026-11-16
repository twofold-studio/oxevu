use super::*;

pub(super) fn facts(name: &str) -> Result<Value> {
    let lower = name.to_ascii_lowercase();
    let resolution = regex::Regex::new(r"(?i)\b(4320|2160|1080|720|576|480)p\b").map_err(bad)?;
    let height = resolution
        .captures(name)
        .and_then(|c| c[1].parse::<u32>().ok());
    let codec = if ["x265", "h265", "h.265", "hevc"]
        .iter()
        .any(|s| lower.contains(s))
    {
        Some("hevc")
    } else if ["x264", "h264", "h.264", "avc"]
        .iter()
        .any(|s| lower.contains(s))
    {
        Some("h264")
    } else if lower.contains("av1") {
        Some("av1")
    } else {
        None
    };
    let source = if lower.contains("web-dl") || lower.contains("webdl") {
        Some("webdl")
    } else if lower.contains("webrip") {
        Some("webrip")
    } else if lower.contains("bluray") || lower.contains("blu-ray") {
        Some("bluray")
    } else if lower.contains("hdtv") {
        Some("hdtv")
    } else {
        None
    };
    let episode = regex::Regex::new(r"(?i)\bS(\d{1,3})(?:E(\d{1,4}))?(?:[ -]?E(\d{1,4}))?")
        .map_err(bad)?
        .captures(name);
    let season = episode.as_ref().and_then(|c| c[1].parse::<u32>().ok());
    let ep = episode
        .as_ref()
        .and_then(|c| c.get(2))
        .and_then(|m| m.as_str().parse::<u32>().ok());
    let end = episode
        .as_ref()
        .and_then(|c| c.get(3))
        .and_then(|m| m.as_str().parse::<u32>().ok());
    let year = regex::Regex::new(r"\b((?:19|20)\d{2})\b")
        .map_err(bad)?
        .captures(name)
        .and_then(|c| c[1].parse::<u32>().ok());
    let language = if lower.contains("bulgarian") || lower.contains(".bg.") {
        Some("bg")
    } else if lower.contains("english") || lower.contains(".eng.") {
        Some("en")
    } else {
        None
    };
    Ok(
        json!({"height":height,"codec":codec,"source":source,"season":season,"episode":ep,"episode_end":end,"year":year,"language":language,
        "hdr":lower.contains("hdr")||lower.contains("dolby.vision")||lower.contains("dovi")}),
    )
}

pub(super) fn identity_matches(title: &Value, release: &Value) -> Result<bool> {
    if release["tmdbid"].as_u64() == title["provider_id"].as_u64() && release["tmdbid"].is_number()
    {
        return Ok(true);
    }
    let name = release["title"].as_str().unwrap_or("");
    let boundary = regex::Regex::new(
        r"(?i)[ ._\-(](?:S\d{1,3}(?:E\d{1,4})?|(?:19|20)\d{2}|(?:2160|1080|720|480)p)\b",
    )
    .map_err(bad)?;
    let name = &name[..boundary.find(name).map(|m| m.start()).unwrap_or(name.len())];
    let normalize = |s: &str| {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let matches = normalize(name) == normalize(title["title"].as_str().unwrap_or(""));
    let year_ok = title["kind"] == "tv"
        || release["year"]
            .as_u64()
            .zip(title["year"].as_str().and_then(|s| s.parse::<u64>().ok()))
            .is_some_and(|(a, b)| a == b);
    Ok(matches && year_ok)
}
