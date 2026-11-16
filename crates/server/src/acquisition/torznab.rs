use super::*;
use quick_xml::{Reader, events::Event};

pub(super) fn parse(xml: &[u8], caps: bool) -> Result<Value> {
    if xml.len() > 2 * 1024 * 1024 {
        return Err(bad("indexer response too large"));
    }
    let xml = std::str::from_utf8(xml).map_err(|_| bad("indexer XML is not UTF-8"))?;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut result = Vec::new();
    let mut item: Option<Value> = None;
    let mut field = String::new();
    let mut text_buffer = String::new();
    let mut capabilities = json!({"search":false,"tv-search":false,"movie-search":false,"max_limit":100,"categories":[]});
    loop {
        match reader
            .read_event()
            .map_err(|_| bad("invalid indexer XML"))?
        {
            Event::DocType(_) => return Err(bad("indexer XML declarations are unsupported")),
            Event::Start(ref e) | Event::Empty(ref e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                let mut attrs = std::collections::HashMap::new();
                for attr in e.attributes() {
                    let attr = attr.map_err(|_| bad("invalid XML attributes"))?;
                    attrs.insert(
                        String::from_utf8_lossy(attr.key.as_ref()).into_owned(),
                        attr.decode_and_unescape_value(reader.decoder())
                            .map_err(|_| bad("invalid XML value"))?
                            .into_owned(),
                    );
                }
                if name == "error" {
                    return Err(bad(
                        "indexer rejected the request; check credentials and limits",
                    ));
                }
                if caps {
                    if matches!(name.as_str(), "search" | "tv-search" | "movie-search") {
                        capabilities[&name] =
                            json!(attrs.get("available").is_some_and(|v| v == "yes"));
                        capabilities[format!("{name}_params")] =
                            json!(attrs.get("supportedParams"));
                    }
                    if name == "limits" {
                        capabilities["max_limit"] = json!(
                            attrs
                                .get("max")
                                .and_then(|v| v.parse::<u32>().ok())
                                .unwrap_or(100)
                                .clamp(1, 100)
                        );
                    }
                    if name == "category"
                        && let Some(id) = attrs.get("id")
                    {
                        capabilities["categories"]
                            .as_array_mut()
                            .ok_or_else(|| bad("invalid categories"))?
                            .push(json!(id));
                    }
                } else if name == "item" {
                    item = Some(json!({}));
                } else if let Some(ref mut value) = item {
                    if name == "enclosure" {
                        value["download_url"] = json!(attrs.get("url"));
                        value["size"] =
                            json!(attrs.get("length").and_then(|s| s.parse::<u64>().ok()));
                    } else if name == "attr" {
                        if let (Some(key), Some(v)) = (attrs.get("name"), attrs.get("value")) {
                            match key.as_str() {
                                "seeders" | "peers" | "size" | "tmdbid" => {
                                    value[key] = json!(v.parse::<u64>().ok())
                                }
                                "infohash" | "magneturl" | "imdb" => value[key] = json!(v),
                                _ => {}
                            }
                        }
                    } else {
                        field = name;
                        text_buffer.clear();
                    }
                }
            }
            Event::Text(ref text) => {
                if item.is_some()
                    && matches!(
                        field.as_str(),
                        "title" | "guid" | "link" | "size" | "pubDate"
                    )
                {
                    let decoded = text.decode().map_err(|_| bad("invalid XML text"))?;
                    append(&mut text_buffer, &decoded)?;
                }
            }
            Event::CData(ref text) => {
                if item.is_some()
                    && matches!(
                        field.as_str(),
                        "title" | "guid" | "link" | "size" | "pubDate"
                    )
                {
                    let text = std::str::from_utf8(text).map_err(|_| bad("invalid XML text"))?;
                    append(&mut text_buffer, text)?;
                }
            }
            Event::GeneralRef(ref entity) => {
                let resolved = match entity
                    .resolve_char_ref()
                    .map_err(|_| bad("invalid XML entity"))?
                {
                    Some(character) => character.to_string(),
                    None => match &entity[..] {
                        b"amp" => "&",
                        b"lt" => "<",
                        b"gt" => ">",
                        b"quot" => "\"",
                        b"apos" => "'",
                        _ => return Err(bad("invalid XML entity")),
                    }
                    .to_owned(),
                };
                if item.is_some()
                    && matches!(
                        field.as_str(),
                        "title" | "guid" | "link" | "size" | "pubDate"
                    )
                {
                    append(&mut text_buffer, &resolved)?;
                }
            }
            Event::End(ref e) => {
                if e.local_name().as_ref() == field.as_bytes()
                    && matches!(
                        field.as_str(),
                        "title" | "guid" | "link" | "size" | "pubDate"
                    )
                    && let Some(ref mut value) = item
                {
                    value[&field] = if field == "size" {
                        json!(text_buffer.trim().parse::<u64>().ok())
                    } else {
                        json!(text_buffer.trim())
                    };
                }
                if e.local_name().as_ref() == b"item"
                    && let Some(value) = item.take()
                    && result.len() < 100
                {
                    result.push(value);
                }
                field.clear();
                text_buffer.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if caps {
        if capabilities["search"] != true
            && capabilities["movie-search"] != true
            && capabilities["tv-search"] != true
        {
            return Err(bad("indexer does not advertise a supported search"));
        }
        Ok(capabilities)
    } else {
        Ok(json!(result))
    }
}

fn append(buffer: &mut String, text: &str) -> Result<()> {
    if buffer.len().saturating_add(text.len()) > 8192 {
        return Err(bad("indexer field too large"));
    }
    buffer.push_str(text);
    Ok(())
}
