/// CSS Font representation.
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub family: String,
    pub size_pt: f32,
    pub weight: u16,
    pub style: FontStyle,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            family: "sans-serif".to_string(),
            size_pt: 14.0,
            weight: 400,
            style: FontStyle::Normal,
        }
    }
}

/// Font style variants.
#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Parse a CSS `font-family` value into a list of family names.
///
/// Handles quoted names and comma-separated lists.
///
/// # Examples
///
/// * `"Arial, sans-serif"` → `["Arial", "sans-serif"]`
/// * `"'Times New Roman', serif"` → `["Times New Roman", "serif"]`
/// * `"\"Helvetica Neue\", Arial, sans-serif"` → `["Helvetica Neue", "Arial", "sans-serif"]`
pub fn parse_font_family(value: &str) -> Vec<String> {
    let mut families = Vec::new();
    let mut current = String::new();
    let mut in_quotes: Option<char> = None;
    let chars: Vec<char> = value.chars().collect();

    for &c in &chars {
        if let Some(quote) = in_quotes {
            if c == quote {
                in_quotes = None;
            } else {
                current.push(c);
            }
        } else if c == '\'' || c == '"' {
            in_quotes = Some(c);
        } else if c == ',' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                families.push(trimmed.to_string());
            }
            current.clear();
        } else {
            current.push(c);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        families.push(trimmed.to_string());
    }

    families
}

/// Resolve a list of font families to the first available family.
///
/// Known generic families: serif, sans-serif, monospace, cursive, fantasy.
/// Falls back to `"sans-serif"` if no match is found.
pub fn resolve_font_family(families: &[String]) -> String {
    let generics = ["serif", "sans-serif", "monospace", "cursive", "fantasy"];

    for family in families {
        // In a real browser we'd check installed fonts. For the MVP we accept
        // the first concrete name and fall back to generics.
        let lower = family.to_ascii_lowercase();
        if generics.contains(&lower.as_str()) {
            return lower;
        }
        // Accept any non-empty concrete family name (rendering backend will map)
        if !family.is_empty() {
            return family.clone();
        }
    }

    "sans-serif".to_string()
}

/// Parse a CSS `font-size` value.
///
/// Returns the size in pixels (or pt-equivalent) when possible:
/// * `"16px"` → `16.0`
/// * `"small"` → `10.0`, `"medium"` → `14.0`, etc.
/// * `"smaller"` → `-2.0` (caller should add to parent size)
/// * `"larger"` → `2.0` (caller should add to parent size)
/// * `"1.5em"` → `1.5` (caller should multiply by base font size)
///
/// Returns `None` for unparseable values.
pub fn parse_font_size(value: &str) -> Option<f32> {
    let trimmed = value.trim().to_ascii_lowercase();

    match trimmed.as_str() {
        "xx-small" => Some(8.0),
        "x-small" => Some(10.0),
        "small" => Some(12.0),
        "medium" => Some(14.0),
        "large" => Some(18.0),
        "x-large" => Some(24.0),
        "xx-large" => Some(32.0),
        "smaller" => Some(-2.0),
        "larger" => Some(2.0),
        _ => {
            if let Some(px) = trimmed.strip_suffix("px") {
                px.parse::<f32>().ok()
            } else if let Some(pt) = trimmed.strip_suffix("pt") {
                pt.parse::<f32>().ok()
            } else if let Some(rem) = trimmed.strip_suffix("rem") {
                rem.parse::<f32>().ok()
            } else if let Some(em) = trimmed.strip_suffix("em") {
                em.parse::<f32>().ok()
            } else if let Some(pc) = trimmed.strip_suffix('%') {
                pc.parse::<f32>().ok().map(|v| v / 100.0)
            } else {
                trimmed.parse::<f32>().ok()
            }
        }
    }
}

/// Parse a CSS `font-weight` value.
///
/// * `"normal"` → `400`
/// * `"bold"` → `700`
/// * `"100"`..`"900"` → parsed integer
/// * `"lighter"` → `300`
/// * `"bolder"` → `800`
///
/// Defaults to `400` for unknown values.
pub fn parse_font_weight(value: &str) -> u16 {
    let trimmed = value.trim().to_ascii_lowercase();

    match trimmed.as_str() {
        "normal" => 400,
        "bold" => 700,
        "lighter" => 300,
        "bolder" => 800,
        _ => trimmed.parse::<u16>().unwrap_or(400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_font_family ─────────────────────────────────────────────
    #[test]
    fn test_parse_font_family_simple() {
        let families = parse_font_family("Arial, sans-serif");
        assert_eq!(families, vec!["Arial", "sans-serif"]);
    }

    #[test]
    fn test_parse_font_family_single() {
        let families = parse_font_family("serif");
        assert_eq!(families, vec!["serif"]);
    }

    #[test]
    fn test_parse_font_family_quoted_single() {
        let families = parse_font_family("'Times New Roman'");
        assert_eq!(families, vec!["Times New Roman"]);
    }

    #[test]
    fn test_parse_font_family_quoted_double() {
        let families = parse_font_family("\"Helvetica Neue\"");
        assert_eq!(families, vec!["Helvetica Neue"]);
    }

    #[test]
    fn test_parse_font_family_mixed_quoted_unquoted() {
        let families = parse_font_family("'Times New Roman', Arial, sans-serif");
        assert_eq!(families, vec!["Times New Roman", "Arial", "sans-serif"]);
    }

    #[test]
    fn test_parse_font_family_multiple_quotes() {
        let families = parse_font_family("'Arial Black', \"Comic Sans MS\", cursive");
        assert_eq!(families, vec!["Arial Black", "Comic Sans MS", "cursive"]);
    }

    #[test]
    fn test_parse_font_family_extra_whitespace() {
        let families = parse_font_family("  Arial  ,   sans-serif  ");
        assert_eq!(families, vec!["Arial", "sans-serif"]);
    }

    #[test]
    fn test_parse_font_family_empty() {
        let families = parse_font_family("");
        assert!(families.is_empty());
    }

    // ── resolve_font_family ───────────────────────────────────────────
    #[test]
    fn test_resolve_first_concrete() {
        let families = vec!["Arial".to_string(), "sans-serif".to_string()];
        assert_eq!(resolve_font_family(&families), "Arial");
    }

    #[test]
    fn test_resolve_generic_sans_serif() {
        let families = vec!["sans-serif".to_string()];
        assert_eq!(resolve_font_family(&families), "sans-serif");
    }

    #[test]
    fn test_resolve_generic_serif() {
        let families = vec!["serif".to_string()];
        assert_eq!(resolve_font_family(&families), "serif");
    }

    #[test]
    fn test_resolve_generic_monospace() {
        let families = vec!["monospace".to_string()];
        assert_eq!(resolve_font_family(&families), "monospace");
    }

    #[test]
    fn test_resolve_fallback_chain() {
        let families = vec![
            "Helvetica Neue".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];
        // First non-generic is accepted in MVP
        assert_eq!(resolve_font_family(&families), "Helvetica Neue");
    }

    #[test]
    fn test_resolve_empty_list() {
        let families: Vec<String> = vec![];
        assert_eq!(resolve_font_family(&families), "sans-serif");
    }

    #[test]
    fn test_resolve_all_empty_strings() {
        let families = vec!["".to_string(), "".to_string()];
        assert_eq!(resolve_font_family(&families), "sans-serif");
    }

    // ── parse_font_size ───────────────────────────────────────────────
    #[test]
    fn test_parse_font_size_px() {
        assert_eq!(parse_font_size("16px"), Some(16.0));
        assert_eq!(parse_font_size("12.5px"), Some(12.5));
    }

    #[test]
    fn test_parse_font_size_pt() {
        assert_eq!(parse_font_size("12pt"), Some(12.0));
    }

    #[test]
    fn test_parse_font_size_em() {
        assert_eq!(parse_font_size("1.5em"), Some(1.5));
        assert_eq!(parse_font_size("2em"), Some(2.0));
    }

    #[test]
    fn test_parse_font_size_rem() {
        assert_eq!(parse_font_size("1.2rem"), Some(1.2));
    }

    #[test]
    fn test_parse_font_size_percent() {
        assert_eq!(parse_font_size("150%"), Some(1.5));
        assert_eq!(parse_font_size("100%"), Some(1.0));
    }

    #[test]
    fn test_parse_font_size_keywords() {
        assert_eq!(parse_font_size("xx-small"), Some(8.0));
        assert_eq!(parse_font_size("x-small"), Some(10.0));
        assert_eq!(parse_font_size("small"), Some(12.0));
        assert_eq!(parse_font_size("medium"), Some(14.0));
        assert_eq!(parse_font_size("large"), Some(18.0));
        assert_eq!(parse_font_size("x-large"), Some(24.0));
        assert_eq!(parse_font_size("xx-large"), Some(32.0));
    }

    #[test]
    fn test_parse_font_size_relative_keywords() {
        assert_eq!(parse_font_size("smaller"), Some(-2.0));
        assert_eq!(parse_font_size("larger"), Some(2.0));
    }

    #[test]
    fn test_parse_font_size_bare_number() {
        assert_eq!(parse_font_size("16"), Some(16.0));
    }

    #[test]
    fn test_parse_font_size_whitespace() {
        assert_eq!(parse_font_size("  16px  "), Some(16.0));
    }

    #[test]
    fn test_parse_font_size_invalid() {
        assert_eq!(parse_font_size("abc"), None);
        assert_eq!(parse_font_size("px"), None);
    }

    // ── parse_font_weight ─────────────────────────────────────────────
    #[test]
    fn test_parse_font_weight_normal() {
        assert_eq!(parse_font_weight("normal"), 400);
    }

    #[test]
    fn test_parse_font_weight_bold() {
        assert_eq!(parse_font_weight("bold"), 700);
    }

    #[test]
    fn test_parse_font_weight_numeric() {
        assert_eq!(parse_font_weight("100"), 100);
        assert_eq!(parse_font_weight("400"), 400);
        assert_eq!(parse_font_weight("700"), 700);
        assert_eq!(parse_font_weight("900"), 900);
    }

    #[test]
    fn test_parse_font_weight_lighter() {
        assert_eq!(parse_font_weight("lighter"), 300);
    }

    #[test]
    fn test_parse_font_weight_bolder() {
        assert_eq!(parse_font_weight("bolder"), 800);
    }

    #[test]
    fn test_parse_font_weight_unknown() {
        assert_eq!(parse_font_weight("heavy"), 400);
    }

    #[test]
    fn test_parse_font_weight_whitespace() {
        assert_eq!(parse_font_weight("  bold  "), 700);
    }

    #[test]
    fn test_parse_font_weight_case_insensitive() {
        assert_eq!(parse_font_weight("NORMAL"), 400);
        assert_eq!(parse_font_weight("Bold"), 700);
    }

    // ── Font / FontStyle defaults ─────────────────────────────────────
    #[test]
    fn test_font_default() {
        let font = Font::default();
        assert_eq!(font.family, "sans-serif");
        assert_eq!(font.size_pt, 14.0);
        assert_eq!(font.weight, 400);
        assert_eq!(font.style, FontStyle::Normal);
    }

    #[test]
    fn test_font_style_eq() {
        assert_eq!(FontStyle::Normal, FontStyle::Normal);
        assert_eq!(FontStyle::Italic, FontStyle::Italic);
        assert_eq!(FontStyle::Oblique, FontStyle::Oblique);
        assert_ne!(FontStyle::Normal, FontStyle::Italic);
    }
}
