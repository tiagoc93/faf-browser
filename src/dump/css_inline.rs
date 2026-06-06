use std::collections::HashMap;

use crate::dump::url_resolver::resolve_url;

pub fn inline_css(html: &str, base_url: &str) -> String {
    let re = regex::Regex::new(
        r#"<link\b[^>]*?\brel\s*=\s*["']stylesheet["'][^>]*?/?>"#
    ).unwrap();

    let href_re = regex::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).unwrap();

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut css_cache: HashMap<String, Option<String>> = HashMap::new();

    for cap in re.find_iter(html) {
        let link_tag = cap.as_str();

        let href = match href_re.captures(link_tag) {
            Some(c) => c.get(1).unwrap().as_str().to_string(),
            None => continue,
        };

        let resolved = resolve_url(&href, base_url);
        if resolved.is_empty() {
            continue;
        }

        let css_content = css_cache
            .entry(resolved.clone())
            .or_insert_with(|| download_css(&resolved))
            .clone();

        let replacement = match css_content {
            Some(css) => {
                let css_clean = css
                    .replace("@charset", "/* @charset */")
                    .trim()
                    .to_string();
                format!("\n<style>\n{}\n</style>\n", css_clean)
            }
            None => {
                let absolute_href = resolve_url(&href, base_url);
                let tag_with_absolute = href_re
                    .replace(link_tag, |_: &regex::Captures| {
                        format!("href=\"{}\"", absolute_href)
                    })
                    .to_string();
                tag_with_absolute
            }
        };

        replacements.push((cap.start(), cap.end(), replacement));
    }

    apply_replacements(html, &replacements)
}

fn download_css(url: &str) -> Option<String> {
    log::info!("Baixando CSS: {}", url);
    let url = url.to_string();
    tokio::task::block_in_place(|| download_css_sync(&url))
}

fn download_css_sync(url: &str) -> Option<String> {
    match reqwest::blocking::get(url) {
        Ok(resp) if resp.status().is_success() => {
            resp.text().ok().map(|text| {
                log::info!("CSS baixado: {} bytes de {}", text.len(), url);
                resolve_css_urls(&text, url)
            })
        }
        Ok(resp) => {
            log::warn!("HTTP {} ao baixar CSS: {}", resp.status(), url);
            None
        }
        Err(e) => {
            log::warn!("Erro ao baixar CSS {}: {}", url, e);
            None
        }
    }
}

fn resolve_css_urls(css: &str, css_base_url: &str) -> String {
    let re = regex::Regex::new(r#"url\(\s*["']?([^)"']+)["']?\s*\)"#).unwrap();
    re.replace_all(css, |caps: &regex::Captures| {
        let original_url = caps.get(1).unwrap().as_str();
        if original_url.starts_with("data:") {
            return caps.get(0).unwrap().as_str().to_string();
        }
        let resolved = resolve_url(original_url, css_base_url);
        if resolved.contains(' ') || resolved.contains('\'') {
            format!("url(\"{}\")", resolved)
        } else {
            format!("url({})", resolved)
        }
    })
    .to_string()
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
    fn test_no_link_tags_returns_same_html() {
        let html = "<html><head></head><body><h1>Hello</h1></body></html>";
        let result = inline_css(html, "https://example.com");
        assert_eq!(result, html);
    }

    #[test]
    fn test_resolve_css_urls_preserves_data() {
        let css = "body { background: url('data:image/png;base64,abc'); }";
        let result = resolve_css_urls(css, "https://site.com/css/style.css");
        assert!(result.contains("data:image/png;base64,abc"));
    }

    #[test]
    fn test_resolve_css_urls_resolves_relative() {
        let css = "body { background: url(../img/bg.png); }";
        let result = resolve_css_urls(css, "https://site.com/css/style.css");
        assert!(result.contains("https://site.com/img/bg.png"));
    }

    #[test]
    fn test_inline_css_keeps_existing_style_tags() {
        let html = r#"<html><head><style>.a { color: red; }</style></head><body></body></html>"#;
        let result = inline_css(html, "https://example.com");
        assert!(result.contains("<style>"));
        assert!(result.contains(".a { color: red; }"));
    }
}
