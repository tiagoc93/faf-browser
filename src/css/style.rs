use crate::css::parser::{Rule, Stylesheet};
use crate::css::selector::{ElementMatch, select_elements};
use crate::dom::HtmlDocument;
use serde::Serialize;
use std::collections::HashMap;

type ElementKey = (String, Option<String>, Vec<String>, String);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComputedStyle {
    pub color: String,
    pub background_color: String,
    pub font_size: String,
    pub font_family: String,
    pub width: String,
    pub height: String,
    pub margin_top: String,
    pub margin_right: String,
    pub margin_bottom: String,
    pub margin_left: String,
    pub padding_top: String,
    pub padding_right: String,
    pub padding_bottom: String,
    pub padding_left: String,
    pub display: String,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            color: "inherit".to_string(),
            background_color: "transparent".to_string(),
            font_size: "16px".to_string(),
            font_family: "serif".to_string(),
            width: "auto".to_string(),
            height: "auto".to_string(),
            margin_top: "0".to_string(),
            margin_right: "0".to_string(),
            margin_bottom: "0".to_string(),
            margin_left: "0".to_string(),
            padding_top: "0".to_string(),
            padding_right: "0".to_string(),
            padding_bottom: "0".to_string(),
            padding_left: "0".to_string(),
            display: "block".to_string(),
        }
    }
}

/// Compute styles for all elements in a document matched by a stylesheet.
///
/// Rules are applied in ascending specificity order; for equal specificity,
/// the later rule in the stylesheet wins.  The result is a vector of
/// `(element_match, computed_style)` pairs.
#[allow(clippy::type_complexity)]
pub fn compute_styles(
    doc: &HtmlDocument,
    stylesheet: &Stylesheet,
) -> Vec<(ElementMatch, ComputedStyle)> {
    // element_key -> list of (rule, specificity, rule_index_in_stylesheet)
    let mut element_rules: HashMap<
        ElementKey,
        Vec<(&Rule, u32, usize)>,
    > = HashMap::new();
    let mut element_repr: HashMap<ElementKey, ElementMatch> =
        HashMap::new();

    for (rule_idx, rule) in stylesheet.rules.iter().enumerate() {
        match select_elements(doc, &rule.selectors) {
            Ok(matches) => {
                for m in matches {
                    let key = (
                        m.tag.clone(),
                        m.id.clone(),
                        m.classes.clone(),
                        m.text.clone(),
                    );
                    element_rules.entry(key.clone()).or_default().push((
                        rule,
                        m.selector_specificity,
                        rule_idx,
                    ));
                    element_repr.entry(key).or_insert(m);
                }
            }
            Err(_) => continue,
        }
    }

    let mut results = Vec::new();
    for (key, mut rules) in element_rules {
        // Sort by specificity ascending, then by stylesheet order for stable cascade
        rules.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2)));
        let sorted_rules: Vec<(&Rule, u32)> = rules.into_iter().map(|(r, s, _)| (r, s)).collect();
        let style = compute_element_style(&sorted_rules);
        if let Some(elem) = element_repr.remove(&key) {
            results.push((elem, style));
        }
    }

    results
}

/// Build a `ComputedStyle` by applying declarations from a slice of rules.
///
/// `rules` must already be sorted by specificity (ascending) so that later
/// declarations correctly override earlier ones.
pub fn compute_element_style(rules: &[(&Rule, u32)]) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    for (rule, _specificity) in rules {
        for decl in &rule.declarations {
            apply_declaration(&mut style, &decl.property, &decl.value);
        }
    }

    style
}

fn apply_declaration(style: &mut ComputedStyle, property: &str, value: &str) {
    match property {
        "color" => style.color = value.to_string(),
        "background-color" => style.background_color = value.to_string(),
        "font-size" => style.font_size = value.to_string(),
        "font-family" => style.font_family = value.to_string(),
        "width" => style.width = value.to_string(),
        "height" => style.height = value.to_string(),
        "margin-top" => style.margin_top = value.to_string(),
        "margin-right" => style.margin_right = value.to_string(),
        "margin-bottom" => style.margin_bottom = value.to_string(),
        "margin-left" => style.margin_left = value.to_string(),
        "padding-top" => style.padding_top = value.to_string(),
        "padding-right" => style.padding_right = value.to_string(),
        "padding-bottom" => style.padding_bottom = value.to_string(),
        "padding-left" => style.padding_left = value.to_string(),
        "display" => style.display = value.to_string(),
        "margin" => apply_shorthand_margin(style, value),
        "padding" => apply_shorthand_padding(style, value),
        _ => {}
    }
}

fn apply_shorthand_margin(style: &mut ComputedStyle, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            style.margin_top = parts[0].to_string();
            style.margin_right = parts[0].to_string();
            style.margin_bottom = parts[0].to_string();
            style.margin_left = parts[0].to_string();
        }
        2 => {
            style.margin_top = parts[0].to_string();
            style.margin_right = parts[1].to_string();
            style.margin_bottom = parts[0].to_string();
            style.margin_left = parts[1].to_string();
        }
        3 => {
            style.margin_top = parts[0].to_string();
            style.margin_right = parts[1].to_string();
            style.margin_bottom = parts[2].to_string();
            style.margin_left = parts[1].to_string();
        }
        4 => {
            style.margin_top = parts[0].to_string();
            style.margin_right = parts[1].to_string();
            style.margin_bottom = parts[2].to_string();
            style.margin_left = parts[3].to_string();
        }
        _ => {}
    }
}

fn apply_shorthand_padding(style: &mut ComputedStyle, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            style.padding_top = parts[0].to_string();
            style.padding_right = parts[0].to_string();
            style.padding_bottom = parts[0].to_string();
            style.padding_left = parts[0].to_string();
        }
        2 => {
            style.padding_top = parts[0].to_string();
            style.padding_right = parts[1].to_string();
            style.padding_bottom = parts[0].to_string();
            style.padding_left = parts[1].to_string();
        }
        3 => {
            style.padding_top = parts[0].to_string();
            style.padding_right = parts[1].to_string();
            style.padding_bottom = parts[2].to_string();
            style.padding_left = parts[1].to_string();
        }
        4 => {
            style.padding_top = parts[0].to_string();
            style.padding_right = parts[1].to_string();
            style.padding_bottom = parts[2].to_string();
            style.padding_left = parts[3].to_string();
        }
        _ => {}
    }
}

/// Parse a CSS length value into a numeric component and a unit string.
///
/// # Examples
///
/// * `"16px"`   → `(16.0, "px")`
/// * `"1.5em"`  → `(1.5, "em")`
/// * `"auto"`   → `(0.0, "auto")`
/// * `"0"`      → `(0.0, "")`
pub fn parse_css_length(value: &str) -> (f32, String) {
    if value == "auto" {
        return (0.0, "auto".to_string());
    }
    if value == "0" {
        return (0.0, "".to_string());
    }

    let chars: Vec<char> = value.chars().collect();
    let mut numeric_end = 0usize;

    // Optional leading sign
    if !chars.is_empty() && (chars[0] == '-' || chars[0] == '+') {
        numeric_end = 1;
    }

    while numeric_end < chars.len()
        && (chars[numeric_end].is_ascii_digit() || chars[numeric_end] == '.')
    {
        numeric_end += 1;
    }

    let num_part = &value[..numeric_end];
    let unit_part = &value[numeric_end..];

    match num_part.parse::<f32>() {
        Ok(num) => (num, unit_part.to_string()),
        Err(_) => (0.0, value.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_css;

    #[test]
    fn test_default_computed_style() {
        let style = ComputedStyle::default();
        assert_eq!(style.color, "inherit");
        assert_eq!(style.background_color, "transparent");
        assert_eq!(style.font_size, "16px");
        assert_eq!(style.font_family, "serif");
        assert_eq!(style.width, "auto");
        assert_eq!(style.height, "auto");
        assert_eq!(style.margin_top, "0");
        assert_eq!(style.margin_right, "0");
        assert_eq!(style.margin_bottom, "0");
        assert_eq!(style.margin_left, "0");
        assert_eq!(style.padding_top, "0");
        assert_eq!(style.padding_right, "0");
        assert_eq!(style.padding_bottom, "0");
        assert_eq!(style.padding_left, "0");
        assert_eq!(style.display, "block");
    }

    #[test]
    fn test_single_rule() {
        let html = "<html><body><h1>Title</h1></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "h1 { color: red; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (elem, style) = &results[0];
        assert_eq!(elem.tag, "h1");
        assert_eq!(style.color, "red");
    }

    #[test]
    fn test_cascade_override_later_rule_wins() {
        let html = "<html><body><h1>Title</h1></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "h1 { color: red; } h1 { color: blue; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.color, "blue");
    }

    #[test]
    fn test_specificity_class_overrides_tag() {
        let html = r#"<html><body><div class="highlight">Text</div></body></html>"#;
        let doc = HtmlDocument::parse(html);
        let css = "div { color: blue; } .highlight { color: red; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (elem, style) = &results[0];
        assert_eq!(elem.tag, "div");
        assert_eq!(style.color, "red");
    }

    #[test]
    fn test_shorthand_margin_one_value() {
        let html = "<html><body><div>Box</div></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "div { margin: 10px; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.margin_top, "10px");
        assert_eq!(style.margin_right, "10px");
        assert_eq!(style.margin_bottom, "10px");
        assert_eq!(style.margin_left, "10px");
    }

    #[test]
    fn test_shorthand_margin_two_values() {
        let html = "<html><body><div>Box</div></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "div { margin: 10px 20px; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.margin_top, "10px");
        assert_eq!(style.margin_right, "20px");
        assert_eq!(style.margin_bottom, "10px");
        assert_eq!(style.margin_left, "20px");
    }

    #[test]
    fn test_shorthand_padding_one_value() {
        let html = "<html><body><div>Box</div></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "div { padding: 5px; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.padding_top, "5px");
        assert_eq!(style.padding_right, "5px");
        assert_eq!(style.padding_bottom, "5px");
        assert_eq!(style.padding_left, "5px");
    }

    #[test]
    fn test_shorthand_padding_two_values() {
        let html = "<html><body><div>Box</div></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "div { padding: 5px 15px; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.padding_top, "5px");
        assert_eq!(style.padding_right, "15px");
        assert_eq!(style.padding_bottom, "5px");
        assert_eq!(style.padding_left, "15px");
    }

    #[test]
    fn test_parse_css_length_px() {
        assert_eq!(parse_css_length("16px"), (16.0, "px".to_string()));
    }

    #[test]
    fn test_parse_css_length_em() {
        assert_eq!(parse_css_length("1.5em"), (1.5, "em".to_string()));
    }

    #[test]
    fn test_parse_css_length_auto() {
        assert_eq!(parse_css_length("auto"), (0.0, "auto".to_string()));
    }

    #[test]
    fn test_parse_css_length_zero() {
        assert_eq!(parse_css_length("0"), (0.0, "".to_string()));
    }

    #[test]
    fn test_parse_css_length_rem() {
        assert_eq!(parse_css_length("2rem"), (2.0, "rem".to_string()));
    }

    #[test]
    fn test_parse_css_length_percent() {
        assert_eq!(parse_css_length("100%"), (100.0, "%".to_string()));
    }

    #[test]
    fn test_parse_css_length_negative() {
        assert_eq!(parse_css_length("-10px"), (-10.0, "px".to_string()));
    }

    #[test]
    fn test_multiple_elements_same_rule() {
        let html = "<html><body><p>First</p><p>Second</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "p { color: green; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 2);
        for (_, style) in &results {
            assert_eq!(style.color, "green");
        }
    }

    #[test]
    fn test_specificity_equal_index_tiebreak() {
        let html = "<html><body><h1>Title</h1></body></html>";
        let doc = HtmlDocument::parse(html);
        // Both rules have the same specificity; the later one should win.
        let css = "h1 { color: red; } h1 { color: green; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.color, "green");
    }

    #[test]
    fn test_computed_style_multiple_properties() {
        let html = "<html><body><div>Box</div></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "div { color: red; background-color: blue; font-size: 14px; display: block; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert_eq!(results.len(), 1);
        let (_, style) = &results[0];
        assert_eq!(style.color, "red");
        assert_eq!(style.background_color, "blue");
        assert_eq!(style.font_size, "14px");
        assert_eq!(style.display, "block");
    }

    #[test]
    fn test_no_matching_rules() {
        let html = "<html><body><span>Text</span></body></html>";
        let doc = HtmlDocument::parse(html);
        let css = "div { color: red; }";
        let sheet = parse_css(css).unwrap();
        let results = compute_styles(&doc, &sheet);

        assert!(results.is_empty());
    }
}
