use serde_json::{json, Value};

pub fn extract_structured_data(html: &str) -> Value {
    let document = scraper::Html::parse_document(html);

    let json_ld = extract_json_ld(&document);
    let open_graph = extract_open_graph(&document);
    let meta = extract_meta_tags(&document);

    json!({
        "json_ld": json_ld,
        "open_graph": open_graph,
        "meta": meta,
    })
}

fn extract_json_ld(document: &scraper::Html) -> Vec<Value> {
    let selector = match scraper::Selector::parse("script[type=\"application/ld+json\"]") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
    for el in document.select(&selector) {
        let text: String = el.text().collect();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => items.push(value),
            Err(e) => {
                log::warn!("Falha ao parsear JSON-LD: {}", e);
            }
        }
    }
    items
}

fn extract_open_graph(document: &scraper::Html) -> Value {
    let selector = match scraper::Selector::parse("meta[property^=\"og:\"]") {
        Ok(s) => s,
        Err(_) => return json!({}),
    };

    let mut og = serde_json::Map::new();
    for el in document.select(&selector) {
        if let Some(property) = el.value().attr("property") {
            let key = property.strip_prefix("og:").unwrap_or(property);
            let content = el.value().attr("content").unwrap_or("");
            if !content.is_empty() {
                og.insert(key.to_string(), json!(content));
            }
        }
    }

    if let Some(selector) = scraper::Selector::parse("meta[name^=\"twitter:\"]").ok() {
        for el in document.select(&selector) {
            if let Some(name) = el.value().attr("name") {
                let key = name.strip_prefix("twitter:").unwrap_or(name);
                let content = el.value().attr("content").unwrap_or("");
                if !content.is_empty() {
                    og.insert(format!("twitter_{}", key), json!(content));
                }
            }
        }
    }

    Value::Object(og)
}

fn extract_meta_tags(document: &scraper::Html) -> Value {
    let selector = match scraper::Selector::parse("meta[name]") {
        Ok(s) => s,
        Err(_) => return json!({}),
    };

    let relevant = [
        "description",
        "keywords",
        "author",
        "robots",
        "viewport",
        "generator",
        "theme-color",
    ];

    let mut meta = serde_json::Map::new();
    for el in document.select(&selector) {
        if let Some(name) = el.value().attr("name") {
            let name_lower = name.to_lowercase();
            if relevant.contains(&name_lower.as_str()) {
                let content = el.value().attr("content").unwrap_or("");
                if !content.is_empty() {
                    meta.insert(name_lower, json!(content));
                }
            }
        }
    }

    if let Some(title) = document
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
    {
        let text: String = title.text().collect();
        let trimmed = text.trim();
        if !trimmed.is_empty() && !meta.contains_key("title") {
            meta.insert("title".to_string(), json!(trimmed));
        }
    }

    Value::Object(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_ld() {
        let html = r#"<html><head><script type="application/ld+json">{"@type": "Product", "name": "Widget"}</script></head></html>"#;
        let result = extract_structured_data(html);
        let ld = &result["json_ld"];
        assert!(ld.is_array());
        assert_eq!(ld[0]["@type"], "Product");
        assert_eq!(ld[0]["name"], "Widget");
    }

    #[test]
    fn test_open_graph() {
        let html = r#"<html><head><meta property="og:title" content="My Page"><meta property="og:description" content="A description"></head></html>"#;
        let result = extract_structured_data(html);
        let og = &result["open_graph"];
        assert_eq!(og["title"], "My Page");
        assert_eq!(og["description"], "A description");
    }

    #[test]
    fn test_meta_tags() {
        let html = r#"<html><head><meta name="description" content="Meta desc"><meta name="keywords" content="rust, cli"></head></html>"#;
        let result = extract_structured_data(html);
        let meta = &result["meta"];
        assert_eq!(meta["description"], "Meta desc");
        assert_eq!(meta["keywords"], "rust, cli");
    }

    #[test]
    fn test_no_data() {
        let html = "<html><head></head><body><p>No metadata</p></body></html>";
        let result = extract_structured_data(html);
        assert!(result.is_object());
        assert_eq!(result["json_ld"], json!([]));
        assert_eq!(result["open_graph"], json!({}));
        assert_eq!(result["meta"], json!({}));
    }

    #[test]
    fn test_malformed_json_ld() {
        let html = r#"<html><head><script type="application/ld+json">{invalid json}</script></head></html>"#;
        let result = extract_structured_data(html);
        assert!(result.is_object());
        assert_eq!(result["json_ld"], json!([]));
    }
}
