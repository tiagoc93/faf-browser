pub mod css_inline;
pub mod image_inline;
pub mod markdown;
pub mod readability;
pub mod structured_data;
pub mod text;
pub mod url_resolver;

use std::fs;
use std::path::Path;

use crate::dump::css_inline::inline_css;
use crate::dump::image_inline::inline_images;
use crate::dump::markdown::html_to_markdown;
use crate::dump::readability::extract_main_content;
use crate::dump::structured_data::extract_structured_data;
use crate::dump::text::html_to_text;
use crate::dump::url_resolver::resolve_url;

pub struct DumpConfig {
    pub inline_images: bool,
    pub inline_css: bool,
    pub remove_scripts: bool,
    pub base_url: String,
    pub format: String,
    pub readability: bool,
    pub structured_data: bool,
}

impl Default for DumpConfig {
    fn default() -> Self {
        Self {
            inline_images: false,
            inline_css: true,
            remove_scripts: false,
            base_url: String::new(),
            format: "html".to_string(),
            readability: false,
            structured_data: false,
        }
    }
}

pub fn dump_to_string(html: &str, config: &DumpConfig) -> anyhow::Result<String> {
    let mut result = html.to_string();

    if config.structured_data {
        log::info!("Extraindo dados estruturados...");
        let data = extract_structured_data(&result);
        return Ok(serde_json::to_string_pretty(&data)?);
    }

    if config.remove_scripts {
        log::info!("Removendo scripts...");
        result = remove_scripts(&result);
    }

    if config.inline_css {
        log::info!("Inlineando CSS externo...");
        result = inline_css(&result, &config.base_url);
    }

    if config.inline_images {
        log::info!("Convertendo imagens para base64...");
        result = inline_images(&result, &config.base_url);
    }

    log::info!("Resolvendo URLs relativas...");
    result = resolve_urls_in_html(&result, &config.base_url);

    if config.readability {
        log::info!("Extraindo conteúdo principal (readability)...");
        result = extract_main_content(&result);
    }

    match config.format.as_str() {
        "markdown" | "md" => {
            log::info!("Convertendo para Markdown...");
            result = html_to_markdown(&result);
        }
        "text" | "txt" => {
            log::info!("Extraindo texto puro...");
            result = html_to_text(&result);
        }
        _ => {}
    }

    Ok(result)
}

pub fn dump_to_file(html: &str, config: &DumpConfig, output_path: &str) -> anyhow::Result<()> {
    let result = dump_to_string(html, config)?;
    write_to_file(&result, output_path)
}

pub fn write_to_file(content: &str, output_path: &str) -> anyhow::Result<()> {
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, content)?;
    log::info!("Arquivo salvo em {} ({} bytes)", output_path, content.len());
    Ok(())
}

fn remove_scripts(html: &str) -> String {
    let script_re = regex::Regex::new(r#"<script\b[^>]*?/?>[\s\S]*?</script>"#).unwrap();
    let script_self_close = regex::Regex::new(r#"<script\b[^>]*?/>"#).unwrap();

    let result = script_re.replace_all(html, "").to_string();
    let result = script_self_close.replace_all(&result, "").to_string();

    let on_attr_re = regex::Regex::new(r#"\s+on\w+\s*=\s*"[^"]*""#).unwrap();
    let result = on_attr_re.replace_all(&result, "").to_string();

    let on_attr_single = regex::Regex::new(r#"\s+on\w+\s*=\s*'[^']*'"#).unwrap();
    on_attr_single.replace_all(&result, "").to_string()
}

fn resolve_urls_in_html(html: &str, base_url: &str) -> String {
    let attr_patterns: &[(&str, &str)] = &[
        ("href", r#"href\s*=\s*"([^"]*)""#),
        ("src", r#"src\s*=\s*"([^"]*)""#),
        ("action", r#"action\s*=\s*"([^"]*)""#),
    ];

    let base = base_url.to_string();
    let mut result = html.to_string();

    for (_attr_name, pattern) in attr_patterns {
        let re = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                let url_value = caps.get(1).unwrap().as_str();

                if url_value.is_empty()
                    || url_value.starts_with("http://")
                    || url_value.starts_with("https://")
                    || url_value.starts_with("data:")
                    || url_value.starts_with("#")
                    || url_value.starts_with("javascript:")
                    || url_value.starts_with("mailto:")
                {
                    return full_match.to_string();
                }

                let resolved = resolve_url(url_value, &base);
                let attr_name = caps
                    .get(0)
                    .unwrap()
                    .as_str()
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim();
                format!("{}=\"{}\"", attr_name, resolved)
            })
            .to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_scripts_removes_inline_script() {
        let html =
            r#"<html><head><script>alert('xss')</script></head><body><h1>Safe</h1></body></html>"#;
        let result = remove_scripts(html);
        assert!(!result.contains("alert"));
        assert!(result.contains("<h1>Safe</h1>"));
    }

    #[test]
    fn test_remove_scripts_removes_external_script() {
        let html = r#"<html><head><script src="app.js"></script></head><body></body></html>"#;
        let result = remove_scripts(html);
        assert!(!result.contains("script"));
        assert!(!result.contains("app.js"));
    }

    #[test]
    fn test_remove_scripts_removes_onclick() {
        let html =
            r##"<html><body><button onclick="doSomething()">Click</button></body></html>"##;
        let result = remove_scripts(html);
        assert!(!result.contains("onclick"));
        assert!(result.contains("<button"));
        assert!(result.contains("Click"));
    }

    #[test]
    fn test_resolve_urls_anchor_href() {
        let html = r##"<a href="/about">About</a>"##;
        let result = resolve_urls_in_html(html, "https://site.com");
        assert!(result.contains("https://site.com/about"));
    }

    #[test]
    fn test_resolve_urls_preserves_absolute() {
        let html = r##"<a href="https://other.com">Link</a>"##;
        let result = resolve_urls_in_html(html, "https://site.com");
        assert!(result.contains("https://other.com"));
    }

    #[test]
    fn test_resolve_urls_preserves_anchor() {
        let html = r##"<a href="#top">Top</a>"##;
        let result = resolve_urls_in_html(html, "https://site.com");
        assert!(result.contains("#top"));
    }

    #[test]
    fn test_resolve_urls_form_action() {
        let html = r##"<form action="/login" method="POST">"##;
        let result = resolve_urls_in_html(html, "https://site.com");
        assert!(result.contains("https://site.com/login"));
    }

    #[test]
    fn test_dump_to_file_basic() {
        let html = r##"<!DOCTYPE html><html><head><title>Test</title></head><body><a href="/about">About</a><img src="/img/logo.png"></body></html>"##;
        let config = DumpConfig {
            inline_images: false,
            inline_css: false,
            remove_scripts: false,
            base_url: "https://example.com".to_string(),
            ..Default::default()
        };
        let path = "/tmp/faf_test_dump_basic.html";
        dump_to_file(html, &config, path).unwrap();
        let saved = std::fs::read_to_string(path).unwrap();
        assert!(saved.contains("https://example.com/about"));
        assert!(saved.contains("https://example.com/img/logo.png"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_dump_text_format() {
        let html = r##"<html><body><h1>Title</h1><p>Body text here.</p></body></html>"##;
        let config = DumpConfig {
            format: "text".to_string(),
            ..Default::default()
        };
        let path = "/tmp/faf_test_dump_text.txt";
        dump_to_file(html, &config, path).unwrap();
        let saved = std::fs::read_to_string(path).unwrap();
        assert!(saved.contains("Title"));
        assert!(saved.contains("Body text here"));
        assert!(!saved.contains("<h1>"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_dump_markdown_format() {
        let html = r##"<html><body><h1>Title</h1><p>Body text here.</p></body></html>"##;
        let config = DumpConfig {
            format: "markdown".to_string(),
            ..Default::default()
        };
        let path = "/tmp/faf_test_dump_md.md";
        dump_to_file(html, &config, path).unwrap();
        let saved = std::fs::read_to_string(path).unwrap();
        assert!(saved.contains("# Title"));
        assert!(saved.contains("Body text here"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_dump_structured_data() {
        let html = r##"<html><head><meta property="og:title" content="Test OG"></head><body></body></html>"##;
        let config = DumpConfig {
            structured_data: true,
            ..Default::default()
        };
        let path = "/tmp/faf_test_dump_sd.json";
        dump_to_file(html, &config, path).unwrap();
        let saved = std::fs::read_to_string(path).unwrap();
        assert!(saved.contains("Test OG"));
        let _ = std::fs::remove_file(path);
    }
}
