use scraper::Html;

pub fn html_to_text(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut output = String::new();
    let body = match find_body(&document) {
        Some(b) => b,
        None => return extract_all_text(&document),
    };
    convert_element(body, &mut output);
    collapse_spacing(&output)
}

fn find_body(document: &Html) -> Option<scraper::ElementRef<'_>> {
    let selector = scraper::Selector::parse("body").ok()?;
    document.select(&selector).next()
}

fn extract_all_text(document: &Html) -> String {
    let text: String = document.root_element().text().collect::<String>();
    collapse_spacing(&text)
}

fn convert_element(element: scraper::ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(_el) => {
                let Some(el) = scraper::ElementRef::wrap(child) else {
                    continue;
                };
                let tag = el.value().name().to_lowercase();
                match tag.as_str() {
                    "script" | "style" | "nav" | "footer" | "noscript" | "header" | "aside"
                    | "svg" => {}
                    "p" | "div" | "section" | "article" | "main" | "blockquote" | "figure"
                    | "figcaption" | "details" => {
                        let before = output.len();
                        convert_children(el, output);
                        if output.len() > before && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let text: String = el.text().collect();
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            output.push_str(trimmed);
                            output.push_str("\n\n");
                        }
                    }
                    "br" => {
                        output.push('\n');
                    }
                    "hr" => {
                        output.push_str("\n\n");
                    }
                    "li" => {
                        output.push_str("- ");
                        convert_children(el, output);
                        output.push('\n');
                    }
                    "a" => {
                        let href = el.value().attr("href").unwrap_or("");
                        let text: String = el.text().collect();
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            if !href.is_empty()
                                && !href.starts_with("javascript:")
                                && !href.starts_with("#")
                            {
                                output.push_str(&format!("{} ({})", trimmed, href));
                            } else {
                                output.push_str(trimmed);
                            }
                        }
                    }
                    "img" => {
                        let alt = el.value().attr("alt").unwrap_or("");
                        if !alt.is_empty() {
                            output.push_str(&format!("[{}]", alt));
                        }
                    }
                    "pre" => {
                        let text: String = el.text().collect();
                        output.push_str(text.trim());
                        output.push_str("\n\n");
                    }
                    _ => {
                        convert_children(el, output);
                    }
                }
            }
            scraper::Node::Text(text) => {
                let txt = text.text.trim();
                if !txt.is_empty() {
                    output.push_str(txt);
                    output.push(' ');
                }
            }
            _ => {}
        }
    }
}

fn convert_children(element: scraper::ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(_el) => {
                let Some(el) = scraper::ElementRef::wrap(child) else {
                    continue;
                };
                let tag = el.value().name().to_lowercase();
                match tag.as_str() {
                    "script" | "style" | "nav" | "footer" | "noscript" | "header" | "aside"
                    | "svg" => {}
                    "br" => {
                        output.push('\n');
                    }
                    "li" => {
                        output.push_str("- ");
                        convert_children(el, output);
                        output.push('\n');
                    }
                    "a" => {
                        let href = el.value().attr("href").unwrap_or("");
                        let text: String = el.text().collect();
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            if !href.is_empty()
                                && !href.starts_with("javascript:")
                                && !href.starts_with("#")
                            {
                                output.push_str(&format!("{} ({})", trimmed, href));
                            } else {
                                output.push_str(trimmed);
                            }
                        }
                    }
                    "img" => {
                        let alt = el.value().attr("alt").unwrap_or("");
                        if !alt.is_empty() {
                            output.push_str(&format!("[{}]", alt));
                        }
                    }
                    "strong" | "b" | "em" | "i" | "code" | "span" | "label" | "small" | "sub"
                    | "sup" | "mark" | "time" | "abbr" => {
                        convert_children(el, output);
                    }
                    _ => {
                        convert_children(el, output);
                    }
                }
            }
            scraper::Node::Text(text) => {
                let txt = text.text.trim();
                if !txt.is_empty() {
                    output.push_str(txt);
                    output.push(' ');
                }
            }
            _ => {}
        }
    }
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
    fn test_paragraphs() {
        let html = "<html><body><p>First paragraph.</p><p>Second paragraph.</p></body></html>";
        let result = html_to_text(html);
        assert!(result.contains("First paragraph"));
        assert!(result.contains("Second paragraph"));
        assert!(!result.contains("<p>"));
        assert!(!result.contains("<html>"));
    }

    #[test]
    fn test_skips_nav() {
        let html = "<html><body><nav>Menu</nav><p>Content</p></body></html>";
        let result = html_to_text(html);
        assert!(result.contains("Content"));
        assert!(!result.contains("Menu"));
    }

    #[test]
    fn test_headings() {
        let html = "<html><body><h1>Title</h1><p>Body</p></body></html>";
        let result = html_to_text(html);
        assert!(result.contains("Title"));
        assert!(!result.contains("#"));
    }

    #[test]
    fn test_skips_script() {
        let html = "<html><body><script>alert(1)</script><p>Safe</p></body></html>";
        let result = html_to_text(html);
        assert!(!result.contains("alert"));
        assert!(result.contains("Safe"));
    }

    #[test]
    fn test_list() {
        let html = "<html><body><ul><li>Item A</li><li>Item B</li></ul></body></html>";
        let result = html_to_text(html);
        assert!(result.contains("- Item A"));
        assert!(result.contains("- Item B"));
    }
}
