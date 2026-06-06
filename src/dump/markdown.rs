use scraper::Html;

pub fn html_to_markdown(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut output = String::new();

    if let Some(body) = find_body(&document) {
        convert_node(body, &mut output);
    } else {
        convert_node(document.root_element(), &mut output);
    }

    collapse_spacing(&output)
}

fn find_body(document: &Html) -> Option<scraper::ElementRef<'_>> {
    let selector = scraper::Selector::parse("body").ok()?;
    document.select(&selector).next()
}

fn convert_node(element: scraper::ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(_el) => {
                let Some(el) = scraper::ElementRef::wrap(child) else { continue };
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
                            if !href.is_empty()
                                && !href.starts_with("javascript:")
                            {
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
                    "script" | "style" | "nav" | "footer" | "noscript" | "header" | "aside" | "svg" | "canvas" | "iframe" => {}
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

    for row in &rows {
        output.push_str("| ");
        for i in 0..ncols {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            output.push_str(cell);
            output.push_str(" | ");
        }
        output.push('\n');
    }
    output.push('\n');
}

fn collapse_spacing(text: &str) -> String {
    let trimmed = text
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n");
    let re = regex::Regex::new(r"\n{3,}").unwrap();
    re.replace_all(&trimmed, "\n\n").trim().to_string()
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
        let html = r##"<html><body><p>Visit <a href="/about">About Us</a> today</p></body></html>"##;
        let result = html_to_markdown(html);
        assert!(result.contains("Visit [About Us](/about) today"));
    }

    #[test]
    fn test_nested_inline_formatting() {
        let html = "<html><body><p>Text <strong>bold <em>italic</em> more</strong> end</p></body></html>";
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
}
