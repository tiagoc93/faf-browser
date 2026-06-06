use std::collections::HashMap;

use base64::Engine;
use crate::dump::url_resolver::resolve_url;

pub fn inline_images(html: &str, base_url: &str) -> String {
    let img_re = regex::Regex::new(r#"<img\b[^>]*?/?>"#).unwrap();
    let src_re = regex::Regex::new(r#"src\s*=\s*"([^"]*)""#).unwrap();

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut img_cache: HashMap<String, Option<String>> = HashMap::new();

    for cap in img_re.find_iter(html) {
        let img_tag = cap.as_str();

        if let Some(src_cap) = src_re.captures(img_tag) {
            let src = src_cap.get(1).unwrap().as_str().to_string();
            if src.starts_with("data:") || src.is_empty() {
                continue;
            }

            let resolved = resolve_url(&src, base_url);
            if resolved.is_empty() {
                continue;
            }

            let data_uri = img_cache
                .entry(resolved.clone())
                .or_insert_with(|| download_and_encode(&resolved))
                .clone();

            match data_uri {
                Some(data_uri) => {
                    let new_tag = src_re
                        .replace(img_tag, |_: &regex::Captures| {
                            format!("src=\"{}\"", data_uri.replace('"', "&quot;"))
                        })
                        .to_string();
                    replacements.push((cap.start(), cap.end(), new_tag));
                }
                None => {
                    let resolved_abs = resolve_url(&src, base_url);
                    let new_tag = src_re
                        .replace(img_tag, |_: &regex::Captures| {
                            format!("src=\"{}\"", resolved_abs)
                        })
                        .to_string();
                    replacements.push((cap.start(), cap.end(), new_tag));
                }
            }
        }
    }

    apply_replacements(html, &replacements)
}

fn download_and_encode(url: &str) -> Option<String> {
    let url = url.to_string();
    tokio::task::block_in_place(|| download_and_encode_sync(&url))
}

fn download_and_encode_sync(url: &str) -> Option<String> {
    log::info!("Baixando imagem: {}", url);
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Falha ao criar client para imagem {}: {}", url, e);
            return None;
        }
    };

    let resp = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Falha ao baixar imagem {}: {}", url, e);
            return None;
        }
    };

    if !resp.status().is_success() {
        log::warn!("HTTP {} ao baixar imagem {}", resp.status(), url);
        return None;
    }

    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_else(|| guess_mime_from_url(url))
        .to_string();

    let bytes = match resp.bytes() {
        Ok(b) => b.to_vec(),
        Err(e) => {
            log::warn!("Falha ao ler bytes da imagem {}: {}", url, e);
            return None;
        }
    };

    if bytes.is_empty() {
        return None;
    }

    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_uri = format!("data:{};base64,{}", mime, encoded);
    log::info!(
        "Imagem convertida: {} ({} bytes) -> data URI ({} chars)",
        url,
        bytes.len(),
        data_uri.len()
    );

    Some(data_uri)
}

fn guess_mime_from_url(url: &str) -> &str {
    let lower = url.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".ico") {
        "image/x-icon"
    } else {
        "image/png"
    }
}

fn apply_replacements(html: &str, replacements: &[(usize, usize, String)]) -> String {
    let mut result = html.to_string();
    let mut sorted: Vec<_> = replacements.iter().collect();
    sorted.sort_by(|a, b| b.0.cmp(&a.0));

    for (start, end, replacement) in sorted {
        if *start < result.len() && *end <= result.len() {
            result.replace_range(*start..*end, replacement);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_images_returns_same_html() {
        let html = "<html><body><h1>No images</h1></body></html>";
        let result = inline_images(html, "https://example.com");
        assert_eq!(result, html);
    }

    #[test]
    fn test_data_uri_already_inline_skipped() {
        let html = r#"<html><body><img src="data:image/png;base64,abc123"></body></html>"#;
        let result = inline_images(html, "https://example.com");
        assert_eq!(result, html);
    }

    #[test]
    fn test_guess_mime_from_url() {
        assert_eq!(guess_mime_from_url("photo.png"), "image/png");
        assert_eq!(guess_mime_from_url("photo.jpg"), "image/jpeg");
        assert_eq!(guess_mime_from_url("photo.jpeg"), "image/jpeg");
        assert_eq!(guess_mime_from_url("photo.webp"), "image/webp");
        assert_eq!(guess_mime_from_url("photo.gif"), "image/gif");
        assert_eq!(guess_mime_from_url("photo.svg"), "image/svg+xml");
    }
}
