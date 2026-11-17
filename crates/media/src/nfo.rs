//! Bounded Jellyfin/Kodi NFO metadata. No DTDs, external entities or filesystem writes.
use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};

pub fn parse(bytes: &[u8]) -> Result<Value, &'static str> {
    if bytes.len() > 1024 * 1024 {
        return Err("NFO exceeds 1 MiB");
    }
    let mut reader = Reader::from_reader(bytes);
    let mut stack = Vec::<String>::new();
    let mut text = String::new();
    let mut out = json!({});
    let mut unique = String::new();
    for _ in 0..30000 {
        match reader.read_event().map_err(|_| "invalid NFO XML")? {
            Event::Start(element) => {
                if stack.len() >= 32 {
                    return Err("NFO nesting too deep");
                }
                let name = String::from_utf8_lossy(element.name().as_ref()).to_string();
                if stack.is_empty()
                    && !matches!(
                        name.as_str(),
                        "movie" | "tvshow" | "episodedetails" | "season"
                    )
                {
                    return Err("unsupported NFO root");
                }
                if name == "uniqueid" {
                    unique = element
                        .attributes()
                        .flatten()
                        .find(|a| a.key.as_ref() == b"type")
                        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
                        .unwrap_or_default();
                }
                stack.push(name);
                text.clear();
            }
            Event::Text(value) => text.push_str(&value.decode().map_err(|_| "invalid NFO text")?),
            Event::CData(value) => text.push_str(&value.decode().map_err(|_| "invalid NFO text")?),
            Event::GeneralRef(value) => {
                let name = value.decode().map_err(|_| "invalid NFO entity")?;
                let encoded = format!("&{name};");
                text.push_str(
                    &quick_xml::escape::unescape(&encoded).map_err(|_| "unsupported NFO entity")?,
                );
            }
            Event::End(_) => {
                if stack.len() == 2 {
                    let key = stack.last().map(String::as_str).unwrap_or("");
                    let value = text.trim();
                    if !value.is_empty() {
                        match key {
                            "title" => out["title"] = json!(value),
                            "plot" => out["overview"] = json!(value),
                            "mpaa" => out["content_rating"] = json!(value),
                            "year" => {
                                if let Ok(year) = value.parse::<u32>() {
                                    out["year"] = json!(year);
                                }
                            }
                            "rating" => {
                                if let Ok(rating) = value.parse::<f64>()
                                    && rating.is_finite()
                                    && (0.0..=10.0).contains(&rating)
                                {
                                    out["rating"] = json!(rating);
                                    out["rating_source"] = json!("NFO");
                                }
                            }
                            "genre" | "studio" | "director" => {
                                let field = match key {
                                    "genre" => "genres",
                                    "studio" => "studios",
                                    _ => "creators",
                                };
                                if out.get(field).is_none() {
                                    out[field] = json!([]);
                                }
                                if let Some(values) = out[field].as_array_mut()
                                    && values.len() < 100
                                    && !values.contains(&json!(value))
                                {
                                    values.push(json!(value));
                                }
                            }
                            "imdbid" | "tmdbid" | "uniqueid" => {
                                let provider = match key {
                                    "imdbid" => "imdb",
                                    "tmdbid" => "tmdb",
                                    _ => &unique,
                                };
                                if provider == "tmdb"
                                    && let Ok(id) = value.parse::<u64>()
                                {
                                    out["provider_ids"]["tmdb"] = json!(id);
                                }
                                if provider == "imdb"
                                    && value.starts_with("tt")
                                    && value.len() <= 20
                                    && value[2..].bytes().all(|b| b.is_ascii_digit())
                                {
                                    out["provider_ids"]["imdb"] = json!(value);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                stack.pop();
                text.clear();
            }
            Event::DocType(_) => return Err("NFO document types are unsupported"),
            Event::Eof => {
                return if stack.is_empty() {
                    Ok(out)
                } else {
                    Err("incomplete NFO")
                };
            }
            _ => {}
        }
    }
    Err("NFO event limit exceeded")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_movie_fields_entities_and_ids_without_actor_names() {
        let value = parse(br#"<movie><title>A Knight&apos;s Tale</title><plot>One &amp; two</plot><year>2001</year><genre>Drama</genre><director>Director</director><actor><name>Actor</name></actor><uniqueid type="tmdb">9476</uniqueid></movie>"#).unwrap();
        assert_eq!(value["title"], "A Knight's Tale");
        assert_eq!(value["overview"], "One & two");
        assert_eq!(value["provider_ids"]["tmdb"], 9476);
        assert_eq!(value["creators"], json!(["Director"]));
    }
    #[test]
    fn rejects_dtds_and_excessive_input() {
        assert!(parse(b"<!DOCTYPE movie SYSTEM 'file:///etc/passwd'><movie/>").is_err());
        assert!(parse(&vec![b'x'; 1024 * 1024 + 1]).is_err());
    }
}
