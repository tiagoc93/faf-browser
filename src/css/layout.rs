use crate::css::style::ComputedStyle;

#[derive(Debug, Clone, PartialEq)]
pub struct BoxModel {
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
}

/// Convert a CSS length string to pixels.
///
/// Supported units:
/// * `"auto"` → `0.0`
/// * `"0"` → `0.0`
/// * `"100px"` → `100.0`
/// * `"50%"` → `container_width * 0.50`
/// * `"2em"` → `font_size * 2.0`
/// * `"10pt"` → `10.0 * 1.333`
/// * Fallback: try to parse as a bare float, otherwise `0.0`
pub fn css_to_pixels(value: &str, container_width: f32, font_size: f32) -> f32 {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
        return 0.0;
    }
    if trimmed == "0" {
        return 0.0;
    }

    // Percentage
    if let Some(num_part) = trimmed.strip_suffix('%') {
        return num_part.parse::<f32>().unwrap_or(0.0) / 100.0 * container_width;
    }

    // em
    if let Some(num_part) = trimmed.strip_suffix("em") {
        return num_part.parse::<f32>().unwrap_or(0.0) * font_size;
    }

    // pt (1pt ≈ 1.333px)
    if let Some(num_part) = trimmed.strip_suffix("pt") {
        return num_part.parse::<f32>().unwrap_or(0.0) * 1.333;
    }

    // px (or any other unit treated as px)
    if let Some(num_part) = trimmed.strip_suffix("px") {
        return num_part.parse::<f32>().unwrap_or(0.0);
    }

    // Bare float fallback
    trimmed.parse::<f32>().unwrap_or(0.0)
}

/// Resolve a CSS shorthand value into four directional lengths.
///
/// # Patterns
///
/// * `"10px"` → `(10, 10, 10, 10)`
/// * `"10px 20px"` → `(10, 20, 10, 20)`
/// * `"10px 20px 30px"` → `(10, 20, 30, 20)`
/// * `"10px 20px 30px 40px"` → `(10, 20, 30, 40)`
pub fn resolve_margin_shorthand(
    value: &str,
    container_width: f32,
    font_size: f32,
) -> (f32, f32, f32, f32) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            let v = css_to_pixels(parts[0], container_width, font_size);
            (v, v, v, v)
        }
        2 => {
            let top_bottom = css_to_pixels(parts[0], container_width, font_size);
            let right_left = css_to_pixels(parts[1], container_width, font_size);
            (top_bottom, right_left, top_bottom, right_left)
        }
        3 => {
            let top = css_to_pixels(parts[0], container_width, font_size);
            let right_left = css_to_pixels(parts[1], container_width, font_size);
            let bottom = css_to_pixels(parts[2], container_width, font_size);
            (top, right_left, bottom, right_left)
        }
        4 => {
            let top = css_to_pixels(parts[0], container_width, font_size);
            let right = css_to_pixels(parts[1], container_width, font_size);
            let bottom = css_to_pixels(parts[2], container_width, font_size);
            let left = css_to_pixels(parts[3], container_width, font_size);
            (top, right, bottom, left)
        }
        _ => (0.0, 0.0, 0.0, 0.0),
    }
}

/// Compute a `BoxModel` from a `ComputedStyle`.
///
/// Empty or unrecognised values are treated as `0.0`.
pub fn compute_box_model(style: &ComputedStyle, container_width: f32, font_size: f32) -> BoxModel {
    BoxModel {
        width: css_to_pixels(&style.width, container_width, font_size),
        height: css_to_pixels(&style.height, container_width, font_size),
        margin_top: css_to_pixels(&style.margin_top, container_width, font_size),
        margin_right: css_to_pixels(&style.margin_right, container_width, font_size),
        margin_bottom: css_to_pixels(&style.margin_bottom, container_width, font_size),
        margin_left: css_to_pixels(&style.margin_left, container_width, font_size),
        padding_top: css_to_pixels(&style.padding_top, container_width, font_size),
        padding_right: css_to_pixels(&style.padding_right, container_width, font_size),
        padding_bottom: css_to_pixels(&style.padding_bottom, container_width, font_size),
        padding_left: css_to_pixels(&style.padding_left, container_width, font_size),
    }
}

/// Width of the content area only.
pub fn content_width(box_model: &BoxModel) -> f32 {
    box_model.width
}

/// Height of the content area only.
pub fn content_height(box_model: &BoxModel) -> f32 {
    box_model.height
}

/// Total outer width including left and right margins.
pub fn outer_width(box_model: &BoxModel) -> f32 {
    box_model.width + box_model.margin_left + box_model.margin_right
}

/// Total outer height including top and bottom margins.
pub fn outer_height(box_model: &BoxModel) -> f32 {
    box_model.height + box_model.margin_top + box_model.margin_bottom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_to_pixels_px() {
        assert_eq!(css_to_pixels("100px", 800.0, 16.0), 100.0);
    }

    #[test]
    fn test_css_to_pixels_percent() {
        assert_eq!(css_to_pixels("50%", 800.0, 16.0), 400.0);
        assert_eq!(css_to_pixels("25%", 400.0, 16.0), 100.0);
    }

    #[test]
    fn test_css_to_pixels_em() {
        assert_eq!(css_to_pixels("2em", 800.0, 16.0), 32.0);
        assert_eq!(css_to_pixels("1.5em", 800.0, 20.0), 30.0);
    }

    #[test]
    fn test_css_to_pixels_pt() {
        assert!((css_to_pixels("10pt", 800.0, 16.0) - 13.33).abs() < 0.01);
        assert!((css_to_pixels("12pt", 800.0, 16.0) - 15.996).abs() < 0.01);
    }

    #[test]
    fn test_css_to_pixels_auto() {
        assert_eq!(css_to_pixels("auto", 800.0, 16.0), 0.0);
        assert_eq!(css_to_pixels("AUTO", 800.0, 16.0), 0.0);
    }

    #[test]
    fn test_css_to_pixels_zero() {
        assert_eq!(css_to_pixels("0", 800.0, 16.0), 0.0);
    }

    #[test]
    fn test_css_to_pixels_empty() {
        assert_eq!(css_to_pixels("", 800.0, 16.0), 0.0);
        assert_eq!(css_to_pixels("   ", 800.0, 16.0), 0.0);
    }

    #[test]
    fn test_css_to_pixels_fallback_bare_float() {
        assert_eq!(css_to_pixels("42", 800.0, 16.0), 42.0);
        assert_eq!(css_to_pixels("3.14", 800.0, 16.0), 3.14);
    }

    #[test]
    fn test_css_to_pixels_unparseable() {
        assert_eq!(css_to_pixels("abc", 800.0, 16.0), 0.0);
    }

    #[test]
    fn test_resolve_margin_shorthand_one_value() {
        let result = resolve_margin_shorthand("10px", 800.0, 16.0);
        assert_eq!(result, (10.0, 10.0, 10.0, 10.0));
    }

    #[test]
    fn test_resolve_margin_shorthand_two_values() {
        let result = resolve_margin_shorthand("10px 20px", 800.0, 16.0);
        assert_eq!(result, (10.0, 20.0, 10.0, 20.0));
    }

    #[test]
    fn test_resolve_margin_shorthand_three_values() {
        let result = resolve_margin_shorthand("10px 20px 30px", 800.0, 16.0);
        assert_eq!(result, (10.0, 20.0, 30.0, 20.0));
    }

    #[test]
    fn test_resolve_margin_shorthand_four_values() {
        let result = resolve_margin_shorthand("10px 20px 30px 40px", 800.0, 16.0);
        assert_eq!(result, (10.0, 20.0, 30.0, 40.0));
    }

    #[test]
    fn test_resolve_margin_shorthand_mixed_units() {
        let result = resolve_margin_shorthand("10px 50% 2em", 800.0, 16.0);
        assert_eq!(result, (10.0, 400.0, 32.0, 400.0));
    }

    #[test]
    fn test_compute_box_model_basic() {
        let style = ComputedStyle {
            width: "200px".to_string(),
            height: "100px".to_string(),
            margin_top: "10px".to_string(),
            margin_right: "20px".to_string(),
            margin_bottom: "10px".to_string(),
            margin_left: "20px".to_string(),
            padding_top: "5px".to_string(),
            padding_right: "5px".to_string(),
            padding_bottom: "5px".to_string(),
            padding_left: "5px".to_string(),
            ..ComputedStyle::default()
        };
        let bm = compute_box_model(&style, 800.0, 16.0);
        assert_eq!(bm.width, 200.0);
        assert_eq!(bm.height, 100.0);
        assert_eq!(bm.margin_top, 10.0);
        assert_eq!(bm.margin_right, 20.0);
        assert_eq!(bm.margin_bottom, 10.0);
        assert_eq!(bm.margin_left, 20.0);
        assert_eq!(bm.padding_top, 5.0);
        assert_eq!(bm.padding_right, 5.0);
        assert_eq!(bm.padding_bottom, 5.0);
        assert_eq!(bm.padding_left, 5.0);
    }

    #[test]
    fn test_compute_box_model_percent_and_em() {
        let style = ComputedStyle {
            width: "50%".to_string(),
            height: "2em".to_string(),
            margin_top: "10%".to_string(),
            margin_right: "1em".to_string(),
            ..ComputedStyle::default()
        };
        let bm = compute_box_model(&style, 800.0, 16.0);
        assert_eq!(bm.width, 400.0);
        assert_eq!(bm.height, 32.0);
        assert_eq!(bm.margin_top, 80.0);
        assert_eq!(bm.margin_right, 16.0);
    }

    #[test]
    fn test_content_width_height() {
        let bm = BoxModel {
            width: 200.0,
            height: 100.0,
            margin_top: 10.0,
            margin_right: 20.0,
            margin_bottom: 10.0,
            margin_left: 20.0,
            padding_top: 5.0,
            padding_right: 5.0,
            padding_bottom: 5.0,
            padding_left: 5.0,
        };
        assert_eq!(content_width(&bm), 200.0);
        assert_eq!(content_height(&bm), 100.0);
    }

    #[test]
    fn test_outer_width_height() {
        let bm = BoxModel {
            width: 200.0,
            height: 100.0,
            margin_top: 10.0,
            margin_right: 20.0,
            margin_bottom: 15.0,
            margin_left: 25.0,
            padding_top: 5.0,
            padding_right: 5.0,
            padding_bottom: 5.0,
            padding_left: 5.0,
        };
        assert_eq!(outer_width(&bm), 245.0); // 200 + 25 + 20
        assert_eq!(outer_height(&bm), 125.0); // 100 + 10 + 15
    }
}
