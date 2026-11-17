//! Prefer unique episode names when release numbering differs from a provider.
use serde_json::Value;

fn title(value: &str) -> String {
    // Multi-episode releases such as S06E01E02 leave E02 before the title.
    let value = value
        .split_once(' ')
        .filter(|(prefix, _)| {
            prefix
                .strip_prefix(['E', 'e'])
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .map_or(value, |(_, title)| title);
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn resolve<'a>(season: &'a Value, source: &Value) -> Option<&'a Value> {
    let episodes = season["episodes"].as_array()?;
    let name = title(source["title"].as_str().unwrap_or(""));
    let mut matches = episodes
        .iter()
        .filter(|e| !name.is_empty() && title(e["name"].as_str().unwrap_or("")) == name);
    if let Some(episode) = matches.next()
        && matches.next().is_none()
    {
        return Some(episode);
    }
    let number = source["episode_number"].as_u64()?;
    episodes
        .iter()
        .find(|e| e["episode_number"].as_u64() == Some(number))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn unique_titles_resolve_combined_and_shifted_house_numbering() {
        let season = json!({"episodes":[{"episode_number":1,"name":"Broken"},{"episode_number":2,"name":"Epic Fail"},{"episode_number":3,"name":"The Tyrant"},{"episode_number":21,"name":"Help Me"}]});
        for (local, name, expected) in [
            (1, "E02 Broken", 1),
            (3, "Epic Fail", 2),
            (22, "Help Me", 21),
        ] {
            assert_eq!(
                resolve(&season, &json!({"episode_number":local,"title":name})).unwrap()["episode_number"],
                expected
            );
        }
        assert!(resolve(&season, &json!({"episode_number":99,"title":"Unknown"})).is_none());
    }
}
