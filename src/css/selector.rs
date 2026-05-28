use crate::dom::HtmlDocument;
use anyhow::Result;
use scraper::Selector;

#[derive(Debug, Clone, PartialEq)]
pub struct ElementMatch {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text: String,
    pub selector_specificity: u32,
}

/// Match multiple CSS selectors against a DOM document.
/// Returns a vector of `(selector_string, matched_elements)` for each selector.
pub fn match_selectors(
    doc: &HtmlDocument,
    selectors: &[&str],
) -> Result<Vec<(String, Vec<ElementMatch>)>> {
    let mut results = Vec::new();
    for &selector in selectors {
        let matches = select_elements(doc, selector)?;
        results.push((selector.to_string(), matches));
    }
    Ok(results)
}

/// Compute CSS specificity for a selector string.
///
/// Specificity formula (M2 MVP):
/// - inline style = 1000
/// - `#id`        = 100 each
/// - `.class`, `[attr]`, `:pseudo` = 10 each
/// - tag name     = 1 each
///
/// For comma-separated selector groups the maximum specificity is returned.
pub fn compute_specificity(selector: &str) -> u32 {
    let mut max_specificity = 0u32;

    for sel in selector.split(',') {
        let sel = sel.trim();
        if sel.is_empty() {
            continue;
        }

        let mut ids = 0u32;
        let mut classes = 0u32;
        let mut pseudos = 0u32;
        let mut tags = 0u32;

        let chars: Vec<char> = sel.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '#' => {
                    ids += 1;
                    i += 1;
                    while i < chars.len() && is_selector_char(chars[i]) {
                        i += 1;
                    }
                }
                '.' => {
                    classes += 1;
                    i += 1;
                    while i < chars.len() && is_selector_char(chars[i]) {
                        i += 1;
                    }
                }
                '[' => {
                    classes += 1; // attribute selector counts as class-level
                    i += 1;
                    while i < chars.len() && chars[i] != ']' {
                        i += 1;
                    }
                    i += 1; // skip ']'
                }
                ':' => {
                    pseudos += 1;
                    i += 1;
                    if i < chars.len() && chars[i] == ':' {
                        i += 1; // skip second colon for pseudo-element
                    }
                    while i < chars.len() && is_selector_char(chars[i]) {
                        i += 1;
                    }
                    // Skip pseudo-class arguments like :nth-child(2n+1)
                    if i < chars.len() && chars[i] == '(' {
                        let mut paren_depth = 1;
                        i += 1;
                        while i < chars.len() && paren_depth > 0 {
                            if chars[i] == '(' {
                                paren_depth += 1;
                            } else if chars[i] == ')' {
                                paren_depth -= 1;
                            }
                            if paren_depth > 0 {
                                i += 1;
                            }
                        }
                        i += 1; // skip ')'
                    }
                }
                ' ' | '\t' | '\n' | '\r' | '>' | '+' | '~' => {
                    i += 1;
                }
                '*' => {
                    // Universal selector counts as 0
                    i += 1;
                }
                _ => {
                    if chars[i].is_alphabetic() {
                        tags += 1;
                        while i < chars.len() && is_selector_char(chars[i]) {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
            }
        }

        let specificity = ids * 100 + (classes + pseudos) * 10 + tags;
        max_specificity = max_specificity.max(specificity);
    }

    max_specificity
}

fn is_selector_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// Select elements from an `HtmlDocument` matching a CSS selector.
///
/// Uses `scraper::Selector` and `scraper::Html` internally.
pub fn select_elements(doc: &HtmlDocument, selector_str: &str) -> Result<Vec<ElementMatch>> {
    let selector = Selector::parse(selector_str)
        .map_err(|e| anyhow::anyhow!("Invalid selector '{}': {:?}", selector_str, e))?;
    let specificity = compute_specificity(selector_str);

    let matches: Vec<ElementMatch> = doc
        .scraper_html()
        .select(&selector)
        .map(|el| {
            let tag = el.value().name().to_string();
            let id = el.value().attr("id").map(|s| s.to_string());
            let classes = el
                .value()
                .attr("class")
                .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let text: String = el.text().collect::<String>().trim().to_string();

            ElementMatch {
                tag,
                id,
                classes,
                text,
                selector_specificity: specificity,
            }
        })
        .collect();

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity_tag() {
        assert_eq!(compute_specificity("div"), 1);
        assert_eq!(compute_specificity("h1"), 1);
        assert_eq!(compute_specificity("body"), 1);
    }

    #[test]
    fn test_specificity_class() {
        assert_eq!(compute_specificity(".highlight"), 10);
        assert_eq!(compute_specificity(".a.b"), 20);
    }

    #[test]
    fn test_specificity_id() {
        assert_eq!(compute_specificity("#main"), 100);
    }

    #[test]
    fn test_specificity_combined() {
        assert_eq!(compute_specificity("div.content"), 11);
        assert_eq!(compute_specificity("#nav ul li"), 102);
        assert_eq!(compute_specificity("#header .logo"), 110);
    }

    #[test]
    fn test_specificity_pseudo() {
        assert_eq!(compute_specificity(":hover"), 10);
        assert_eq!(compute_specificity("div:hover"), 11);
        assert_eq!(compute_specificity("::before"), 10);
    }

    #[test]
    fn test_specificity_attribute() {
        assert_eq!(compute_specificity("[type='text']"), 10);
        assert_eq!(compute_specificity("input[type='text']"), 11);
    }

    #[test]
    fn test_specificity_comma_group() {
        assert_eq!(compute_specificity("h1, .class, #id"), 100);
        assert_eq!(compute_specificity("h1, h2"), 1);
    }

    #[test]
    fn test_specificity_universal() {
        assert_eq!(compute_specificity("*"), 0);
        assert_eq!(compute_specificity("*.class"), 10);
    }

    #[test]
    fn test_select_elements_by_tag() {
        let html = "<html><body><h1>Title</h1><p>Paragraph</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let matches = select_elements(&doc, "h1").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].tag, "h1");
        assert_eq!(matches[0].text, "Title");
    }

    #[test]
    fn test_select_elements_by_class() {
        let html = r#"<div class="container"><p class="highlight">Text</p><p>Normal</p></div>"#;
        let doc = HtmlDocument::parse(html);
        let matches = select_elements(&doc, ".highlight").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].tag, "p");
        assert!(matches[0].classes.contains(&"highlight".to_string()));
    }

    #[test]
    fn test_select_elements_by_id() {
        let html = r#"<div id="main"><p>Content</p></div>"#;
        let doc = HtmlDocument::parse(html);
        let matches = select_elements(&doc, "#main").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, Some("main".to_string()));
        assert_eq!(matches[0].tag, "div");
    }

    #[test]
    fn test_select_elements_descendant() {
        let html = "<div><span>Inner</span></div><span>Outer</span>";
        let doc = HtmlDocument::parse(html);
        let matches = select_elements(&doc, "div span").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "Inner");
    }

    #[test]
    fn test_select_elements_multiple() {
        let html = "<ul><li>A</li><li>B</li><li>C</li></ul>";
        let doc = HtmlDocument::parse(html);
        let matches = select_elements(&doc, "li").unwrap();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].text, "A");
        assert_eq!(matches[1].text, "B");
        assert_eq!(matches[2].text, "C");
    }

    #[test]
    fn test_match_selectors() {
        let html = r#"<div id="app"><h1 class="title">Hello</h1><p>World</p></div>"#;
        let doc = HtmlDocument::parse(html);
        let results = match_selectors(&doc, &["h1", "#app", ".title"]).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "h1");
        assert_eq!(results[0].1.len(), 1);
        assert_eq!(results[1].0, "#app");
        assert_eq!(results[1].1.len(), 1);
        assert_eq!(results[2].0, ".title");
        assert_eq!(results[2].1.len(), 1);
    }

    #[test]
    fn test_select_elements_specificity() {
        let html = r#"<div id="app" class="active">Test</div>"#;
        let doc = HtmlDocument::parse(html);
        let matches = select_elements(&doc, "div#app.active").unwrap();
        assert_eq!(matches.len(), 1);
        // Specificity = 1 (tag) + 100 (id) + 10 (class) = 111
        assert_eq!(matches[0].selector_specificity, 111);
    }

    #[test]
    fn test_invalid_selector() {
        let html = "<p>Oi</p>";
        let doc = HtmlDocument::parse(html);
        let result = select_elements(&doc, "]][[");
        assert!(result.is_err());
    }
}
