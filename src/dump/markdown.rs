use scraper::Html;

use crate::dump::structured_data::extract_structured_data;

pub fn html_to_markdown(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut output = String::new();

    if let Some(body) = find_body(&document) {
        convert_node(body, &mut output);
    } else {
        convert_node(document.root_element(), &mut output);
    }

    collapse_whitespace(&output)
}

fn find_body(document: &Html) -> Option<scraper::ElementRef<'_>> {
    let selector = scraper::Selector::parse("body").ok()?;
    document.select(&selector).next()
}

fn convert_node(element: scraper::ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(_el) => {
                let Some(el) = scraper::ElementRef::wrap(child) else {
                    continue;
                };
                let tag = el.value().name().to_lowercase();
                match tag.as_str() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = tag.chars().nth(1).unwrap().to_digit(10).unwrap_or(1) as usize;
                        let prefix = "#".repeat(level);
                        let mut content = String::new();
                        convert_node(el, &mut content);
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            output.push_str(&format!("{} {}\n\n", prefix, trimmed));
                        }
                    }
                    "p" => {
                        let mut content = String::new();
                        convert_node(el, &mut content);
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            output.push_str(trimmed);
                            output.push_str("\n\n");
                        }
                    }
                    "a" => {
                        let href = el.value().attr("href").unwrap_or("");
                        let mut content = String::new();
                        convert_node(el, &mut content);
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            if !href.is_empty() && !href.starts_with("javascript:") {
                                output.push_str(&format!("[{}]({})", trimmed, href));
                            } else {
                                output.push_str(trimmed);
                            }
                        }
                    }
                    "img" => {
                        let src = el.value().attr("src").unwrap_or("");
                        let alt = el.value().attr("alt").unwrap_or("image");
                        if !src.is_empty() {
                            output.push_str(&format!("![{}]({})", alt, src));
                        }
                    }
                    "strong" | "b" => {
                        let mut content = String::new();
                        convert_node(el, &mut content);
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            output.push_str(&format!("**{}**", trimmed));
                        }
                    }
                    "em" | "i" => {
                        let mut content = String::new();
                        convert_node(el, &mut content);
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            output.push_str(&format!("*{}*", trimmed));
                        }
                    }
                    "code" => {
                        let text: String = el.text().collect();
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            output.push_str(&format!("`{}`", trimmed));
                        }
                    }
                    "pre" => {
                        let text: String = el.text().collect();
                        output.push_str(&format!("```\n{}\n```\n\n", text.trim()));
                    }
                    "blockquote" => {
                        let mut content = String::new();
                        convert_node(el, &mut content);
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            for line in trimmed.lines() {
                                output.push_str(&format!("> {}\n", line));
                            }
                            output.push('\n');
                        }
                    }
                    "br" => {
                        output.push('\n');
                    }
                    "hr" => {
                        output.push_str("\n---\n\n");
                    }
                    "ul" | "ol" => {
                        let is_ordered = tag == "ol";
                        let mut idx = 1u32;
                        for li in el.children() {
                            if let Some(li_el) = scraper::ElementRef::wrap(li) {
                                if li_el.value().name().to_lowercase() == "li" {
                                    let mut content = String::new();
                                    convert_node(li_el, &mut content);
                                    let trimmed = content.trim();
                                    if !trimmed.is_empty() {
                                        if is_ordered {
                                            output.push_str(&format!("{}. {}\n", idx, trimmed));
                                            idx += 1;
                                        } else {
                                            output.push_str(&format!("- {}\n", trimmed));
                                        }
                                    }
                                }
                            }
                        }
                        output.push('\n');
                    }
                    "table" => {
                        convert_table(el, output);
                    }
                    "script" | "style" | "nav" | "footer" | "noscript" | "header" | "aside"
                    | "svg" | "canvas" | "iframe" => {}
                    _ => {
                        convert_node(el, output);
                    }
                }
            }
            scraper::Node::Text(text) => {
                let txt: &str = text.text.as_ref();
                if !txt.is_empty() {
                    if txt.chars().all(char::is_whitespace) {
                        output.push(' ');
                    } else {
                        output.push_str(txt);
                    }
                }
            }
            _ => {}
        }
    }
}

fn convert_table(element: scraper::ElementRef, output: &mut String) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for tr in element.children() {
        if let Some(tr_el) = scraper::ElementRef::wrap(tr) {
            let tag = tr_el.value().name().to_lowercase();
            if tag == "tr" || tag == "thead" || tag == "tbody" || tag == "tfoot" {
                for row_el in tr_el.children() {
                    if let Some(r_el) = scraper::ElementRef::wrap(row_el) {
                        if r_el.value().name().to_lowercase() == "tr" {
                            let mut cells = Vec::new();
                            for td in r_el.children() {
                                if let Some(td_el) = scraper::ElementRef::wrap(td) {
                                    let td_tag = td_el.value().name().to_lowercase();
                                    if td_tag == "td" || td_tag == "th" {
                                        let text: String = td_el.text().collect();
                                        cells.push(text.trim().replace('\n', " "));
                                    }
                                }
                            }
                            if !cells.is_empty() {
                                rows.push(cells);
                            }
                        }
                    }
                }
            }
        }
    }

    if rows.is_empty() {
        return;
    }

    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return;
    }

    for (ri, row) in rows.iter().enumerate() {
        output.push_str("| ");
        for i in 0..ncols {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            output.push_str(cell);
            output.push_str(" | ");
        }
        output.push('\n');
        // GFM separator after the header row (first row)
        if ri == 0 {
            output.push_str("| ");
            for _ in 0..ncols {
                output.push_str("--- | ");
            }
            output.push('\n');
        }
    }
    output.push('\n');
}

/// Regexes compiled once via LazyLock (Rust 2024).
use std::sync::LazyLock;

static URL_SHORT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^https?://\S{0,3}$").unwrap());

static BLANK_LINES_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\n{3,}").unwrap());

/// Collapse 3+ blank lines to exactly 1, strip trailing whitespace per line,
/// and drop lines that are navigation-only URLs with <3 chars of useful text.
pub fn collapse_whitespace(markdown: &str) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            out_lines.push(String::new());
            continue;
        }
        if URL_SHORT_RE.is_match(trimmed.trim()) {
            continue;
        }
        out_lines.push(trimmed.to_string());
    }
    let joined = out_lines.join("\n");
    BLANK_LINES_RE
        .replace_all(&joined, "\n\n")
        .trim()
        .to_string()
}

/// Build a YAML frontmatter block from page metadata (OpenGraph, meta tags, title).
/// `base_url` is used as fallback for the `url` field when `og:url` is absent.
/// Returns an empty string when no useful metadata is found.
pub fn inject_frontmatter(html: &str, markdown: &str, base_url: &str) -> String {
    let data = extract_structured_data(html);
    let meta = data
        .get("meta")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let og = data
        .get("open_graph")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let json_ld = data
        .get("json_ld")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut lines: Vec<String> = Vec::new();

    let title = og
        .get("title")
        .or_else(|| meta.get("title"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    if let Some(t) = title {
        lines.push(format!("title: {}", yaml_escape(&t)));
    }

    if let Some(desc) = og
        .get("description")
        .or_else(|| meta.get("description"))
        .and_then(|v| v.as_str())
    {
        if !desc.is_empty() {
            lines.push(format!("description: {}", yaml_escape(desc)));
        }
    }

    if let Some(site) = og.get("site_name").and_then(|v| v.as_str()) {
        if !site.is_empty() {
            lines.push(format!("site_name: {}", yaml_escape(site)));
        }
    }

    let url_val: Option<String> = og
        .get("url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            if !base_url.is_empty() {
                Some(base_url.to_string())
            } else {
                None
            }
        });
    if let Some(url) = url_val {
        lines.push(format!("url: {}", yaml_escape(&url)));
    }

    if let Some(author) = meta.get("author").and_then(|v| v.as_str()) {
        if !author.is_empty() {
            lines.push(format!("author: {}", yaml_escape(author)));
        }
    }

    if let Some(published) = og
        .get("published_time")
        .or_else(|| og.get("article:published_time"))
        .and_then(|v| v.as_str())
    {
        if !published.is_empty() {
            lines.push(format!("published: {}", yaml_escape(published)));
        }
    }

    if let Some(twitter) = og.get("twitter_card").and_then(|v| v.as_str()) {
        if !twitter.is_empty() {
            lines.push(format!("twitter_card: {}", yaml_escape(twitter)));
        }
    }

    if !json_ld.is_empty() {
        let types: Vec<String> = json_ld
            .iter()
            .filter_map(|v| v.get("@type"))
            .filter_map(|t| t.as_str().map(|s| yaml_escape(s)))
            .collect();
        if !types.is_empty() {
            lines.push(format!("json_ld_types: [{}]", types.join(", ")));
        }
    }

    if lines.is_empty() {
        return String::new();
    }

    format!("---\n{}\n---\n\n{}", lines.join("\n"), markdown)
}

fn yaml_escape(value: &str) -> String {
    if value.contains(':')
        || value.contains('#')
        || value.contains('\n')
        || value.contains('"')
        || value.contains('\'')
        || value.trim_start().starts_with('-')
        || value.trim_start().starts_with('[')
        || value.trim_start().starts_with('{')
    {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', " ");
        format!("\"{}\"", escaped)
    } else {
        value.replace('\n', " ").to_string()
    }
}

#[derive(serde::Serialize)]
struct Chunk {
    index: usize,
    tokens_est: usize,
    content: String,
}

/// Split markdown into chunks of approximately `max_tokens` tokens (heuristic: 4 chars/token).
/// Strategy: split by sections (##, ###), then paragraphs, then lines; never cut mid-line.
/// Returns a JSON object `{ "chunks": [ {index, tokens_est, content}, ... ] }`.
pub fn chunk_markdown(markdown: &str, max_tokens: usize) -> String {
    if max_tokens == 0 {
        return markdown.to_string();
    }
    let max_chars = max_tokens.saturating_mul(4).max(1);

    let mut sections: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in markdown.lines() {
        let is_section = line.starts_with("## ") || line.starts_with("### ");
        if is_section && !current.trim().is_empty() {
            sections.push(std::mem::take(&mut current));
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        sections.push(current);
    }

    let mut chunks: Vec<String> = Vec::new();
    for section in sections {
        if section.chars().count() <= max_chars {
            chunks.push(section);
            continue;
        }
        let paragraphs: Vec<&str> = section.split("\n\n").collect();
        let mut buf = String::new();
        for para in paragraphs {
            if para.chars().count() <= max_chars {
                if buf.chars().count() + para.chars().count() + 2 > max_chars && !buf.is_empty() {
                    chunks.push(std::mem::take(&mut buf));
                }
                buf.push_str(para);
                buf.push_str("\n\n");
                continue;
            }
            // paragraph too big: split by lines
            for line in para.lines() {
                if buf.chars().count() + line.chars().count() + 1 > max_chars && !buf.is_empty() {
                    chunks.push(std::mem::take(&mut buf));
                }
                buf.push_str(line);
                buf.push('\n');
            }
        }
        if !buf.trim().is_empty() {
            chunks.push(buf);
        }
    }

    let result: Vec<Chunk> = chunks
        .into_iter()
        .enumerate()
        .map(|(index, content)| {
            let tokens_est = content.chars().count().div_ceil(4);
            Chunk {
                index,
                tokens_est,
                content: content.trim().to_string(),
            }
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({ "chunks": result })).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_h1() {
        let html = "<html><body><h1>Title Here</h1></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("# Title Here"));
    }

    #[test]
    fn test_markdown_link() {
        let html = r##"<html><body><a href="https://example.com">Example</a></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("[Example](https://example.com)"));
    }

    #[test]
    fn test_markdown_list() {
        let html = "<html><body><ul><li>Item A</li><li>Item B</li></ul></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("- Item A"));
        assert!(result.contains("- Item B"));
    }

    #[test]
    fn test_markdown_skips_script() {
        let html = "<html><body><script>alert(1)</script><p>Safe</p></body></html>";
        let result = html_to_markdown(html);
        assert!(!result.contains("alert"));
        assert!(result.contains("Safe"));
    }

    #[test]
    fn test_markdown_image() {
        let html = r#"<html><body><img src="photo.png" alt="A photo"></body></html>"#;
        let result = html_to_markdown(html);
        assert!(result.contains("![A photo](photo.png)"));
    }

    #[test]
    fn test_markdown_bold() {
        let html = "<html><body><strong>Bold text</strong></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("**Bold text**"));
    }

    #[test]
    fn test_markdown_ordered_list() {
        let html = "<html><body><ol><li>First</li><li>Second</li></ol></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("1. First"));
        assert!(result.contains("2. Second"));
    }

    #[test]
    fn test_markdown_no_body() {
        let html = "<h1>No body tag</h1>";
        let result = html_to_markdown(html);
        assert!(result.contains("# No body tag"));
    }

    #[test]
    fn test_link_inside_heading() {
        let html = r##"<html><body><h3><a href="/book">Book Title</a></h3></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("### [Book Title](/book)"));
    }

    #[test]
    fn test_image_inside_link_inside_heading() {
        let html = r##"<html><body><h3><a href="/book"><img src="cover.jpg" alt="Cover"> Book Title</a></h3></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("### [![Cover](cover.jpg) Book Title](/book)"));
    }

    #[test]
    fn test_bold_inside_paragraph() {
        let html = "<html><body><p>Hello <strong>world</strong> text</p></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("Hello **world** text"));
    }

    #[test]
    fn test_link_inside_paragraph() {
        let html =
            r##"<html><body><p>Visit <a href="/about">About Us</a> today</p></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("Visit [About Us](/about) today"));
    }

    #[test]
    fn test_nested_inline_formatting() {
        let html =
            "<html><body><p>Text <strong>bold <em>italic</em> more</strong> end</p></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("Text **bold *italic* more** end"));
    }

    #[test]
    fn test_blockquote_with_formatting() {
        let html = r##"<html><body><blockquote><p>Quote <strong>bold</strong> text</p></blockquote></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("> Quote **bold** text"));
    }

    #[test]
    fn test_list_with_links() {
        let html = r##"<html><body><ul><li><a href="/one">First</a></li><li><a href="/two">Second</a></li></ul></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("- [First](/one)"));
        assert!(result.contains("- [Second](/two)"));
    }

    #[test]
    fn test_text_spacing_preserved() {
        let html = "<html><body><p>Hello <strong>world</strong> again</p></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("Hello **world** again"));
    }

    #[test]
    fn test_skip_javascript_link() {
        let html = r##"<html><body><a href="javascript:void(0)">Click</a></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("Click"));
        assert!(!result.contains("javascript"));
    }

    #[test]
    fn test_empty_elements_skipped() {
        let html = "<html><body><div><i></i></div><p>Content</p></body></html>";
        let result = html_to_markdown(html);
        assert_eq!(result, "Content");
    }

    #[test]
    fn test_pre_code_block() {
        let html = "<html><body><pre><code>fn main() {\n    println!(\"hello\");\n}</code></pre></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("```"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_headings_are_correct_level() {
        let html = "<html><body><h1>Title</h1><h2>Section</h2></body></html>";
        let result = html_to_markdown(html);
        assert!(result.contains("# Title"));
        assert!(result.contains("## Section"));
    }

    #[test]
    fn test_collapse_whitespace_removes_trailing_spaces() {
        let input = "line one   \nline two\t\n\n\n\n\nline three";
        let result = collapse_whitespace(input);
        assert!(!result.contains("  \n"));
        assert!(!result.contains("\t\n"));
        assert!(result.matches("\n\n\n").count() == 0);
    }

    #[test]
    fn test_collapse_whitespace_collapses_blanks() {
        let input = "para one\n\n\n\n\n\npara two";
        let result = collapse_whitespace(input);
        assert_eq!(result, "para one\n\npara two");
    }

    #[test]
    fn test_collapse_whitespace_drops_short_url_lines() {
        let input = "intro\nhttp://a\n\ntext";
        let result = collapse_whitespace(input);
        assert!(!result.contains("http://a"));
        assert!(result.contains("intro"));
        assert!(result.contains("text"));
    }

    #[test]
    fn test_collapse_whitespace_keeps_real_urls() {
        let input = "see http://example.com/page now";
        let result = collapse_whitespace(input);
        assert!(result.contains("http://example.com/page"));
    }

    #[test]
    fn test_inject_frontmatter_with_og() {
        let html = r##"<html><head><title>Page</title><meta property="og:title" content="Hello"><meta property="og:description" content="Desc"></head><body><p>body</p></body></html>"##;
        let md = "# Hello\n\nbody";
        let result = inject_frontmatter(html, md, "");
        assert!(result.starts_with("---\n"));
        assert!(result.contains("title: Hello"));
        assert!(result.contains("description: Desc"));
    }

    #[test]
    fn test_inject_frontmatter_no_metadata() {
        let html = "<html><body><p>just content</p></body></html>";
        let md = "# Just content";
        let result = inject_frontmatter(html, md, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_inject_frontmatter_partial_meta() {
        let html =
            r##"<html><head><meta name="author" content="Alice"></head><body></body></html>"##;
        let md = "content";
        let result = inject_frontmatter(html, md, "");
        assert!(result.contains("author: Alice"));
    }

    #[test]
    fn test_chunk_markdown_disabled() {
        let md = "# Title\n\nbody text";
        let result = chunk_markdown(md, 0);
        assert_eq!(result, md);
    }

    #[test]
    fn test_chunk_markdown_short_single_chunk() {
        let md = "# Title\n\nshort body";
        let result = chunk_markdown(md, 500);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let chunks = parsed["chunks"].as_array().unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0]["index"], 0);
        assert!(chunks[0]["content"].as_str().unwrap().contains("Title"));
    }

    #[test]
    fn test_chunk_markdown_multiple_sections() {
        let mut md = String::new();
        for i in 1..=5 {
            md.push_str(&format!("## Section {}\n\n", i));
            md.push_str(&"a b c d e f g h i j ".repeat(40));
            md.push_str("\n\n");
        }
        let result = chunk_markdown(&md, 50);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let chunks = parsed["chunks"].as_array().unwrap();
        assert!(
            chunks.len() >= 2,
            "expected multiple chunks, got {}",
            chunks.len()
        );
    }

    #[test]
    fn test_chunk_markdown_huge_paragraph_splits_by_line() {
        let paragraph = "word ".repeat(800);
        let md = format!("# Title\n\n{}", paragraph);
        let result = chunk_markdown(&md, 30);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let chunks = parsed["chunks"].as_array().unwrap();
        assert!(chunks.len() > 1);
    }
}
