/// CSS Color representation.
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        }
    }
}

/// Parse a CSS color value into a `Color`.
///
/// Supports:
/// * Named colors (red, blue, green, black, white, gray, yellow, orange,
///   purple, pink, brown, navy, teal, olive, maroon, aqua, fuchsia, lime, silver)
/// * Hex: `#ff0000`, `#f00`, `#ff0000ff` (with alpha), `#f00f`
/// * rgb/rgba: `rgb(255, 0, 0)`, `rgba(255, 0, 0, 0.5)`
/// * `transparent` → `(0, 0, 0, 0.0)`
/// * `currentcolor` → `None` (defer to inherited)
pub fn parse_color(value: &str) -> Option<Color> {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower == "currentcolor" {
        return None;
    }

    if lower == "transparent" {
        return Some(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0.0,
        });
    }

    if let Some(c) = parse_named_color(&lower) {
        return Some(c);
    }

    if let Some(c) = parse_hex_color(trimmed) {
        return Some(c);
    }

    if let Some(c) = parse_rgb_color(trimmed) {
        return Some(c);
    }

    None
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name {
        "red" => Some(Color { r: 255, g: 0, b: 0, a: 1.0 }),
        "blue" => Some(Color { r: 0, g: 0, b: 255, a: 1.0 }),
        "green" => Some(Color { r: 0, g: 128, b: 0, a: 1.0 }),
        "black" => Some(Color { r: 0, g: 0, b: 0, a: 1.0 }),
        "white" => Some(Color { r: 255, g: 255, b: 255, a: 1.0 }),
        "gray" => Some(Color { r: 128, g: 128, b: 128, a: 1.0 }),
        "yellow" => Some(Color { r: 255, g: 255, b: 0, a: 1.0 }),
        "orange" => Some(Color { r: 255, g: 165, b: 0, a: 1.0 }),
        "purple" => Some(Color { r: 128, g: 0, b: 128, a: 1.0 }),
        "pink" => Some(Color { r: 255, g: 192, b: 203, a: 1.0 }),
        "brown" => Some(Color { r: 165, g: 42, b: 42, a: 1.0 }),
        "navy" => Some(Color { r: 0, g: 0, b: 128, a: 1.0 }),
        "teal" => Some(Color { r: 0, g: 128, b: 128, a: 1.0 }),
        "olive" => Some(Color { r: 128, g: 128, b: 0, a: 1.0 }),
        "maroon" => Some(Color { r: 128, g: 0, b: 0, a: 1.0 }),
        "aqua" => Some(Color { r: 0, g: 255, b: 255, a: 1.0 }),
        "fuchsia" => Some(Color { r: 255, g: 0, b: 255, a: 1.0 }),
        "lime" => Some(Color { r: 0, g: 255, b: 0, a: 1.0 }),
        "silver" => Some(Color { r: 192, g: 192, b: 192, a: 1.0 }),
        _ => None,
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let s = value.strip_prefix('#')?;
    let len = s.len();

    if len == 3 {
        // #f00
        let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
        Some(Color { r, g, b, a: 1.0 })
    } else if len == 4 {
        // #f00f
        let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
        let a = u8::from_str_radix(&s[3..4].repeat(2), 16).ok()?;
        Some(Color {
            r,
            g,
            b,
            a: a as f32 / 255.0,
        })
    } else if len == 6 {
        // #ff0000
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color { r, g, b, a: 1.0 })
    } else if len == 8 {
        // #ff0000ff
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(Color {
            r,
            g,
            b,
            a: a as f32 / 255.0,
        })
    } else {
        None
    }
}

fn parse_rgb_color(value: &str) -> Option<Color> {
    let lower = value.to_ascii_lowercase();
    let inner = if let Some(stripped) = lower.strip_prefix("rgba(") {
        stripped.strip_suffix(')')
    } else if let Some(stripped) = lower.strip_prefix("rgb(") {
        stripped.strip_suffix(')')
    } else {
        None
    }?;

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    if parts.len() == 3 {
        let r = parse_color_channel(parts[0])?;
        let g = parse_color_channel(parts[1])?;
        let b = parse_color_channel(parts[2])?;
        Some(Color { r, g, b, a: 1.0 })
    } else if parts.len() == 4 {
        let r = parse_color_channel(parts[0])?;
        let g = parse_color_channel(parts[1])?;
        let b = parse_color_channel(parts[2])?;
        let a = parse_alpha(parts[3])?;
        Some(Color { r, g, b, a })
    } else {
        None
    }
}

fn parse_color_channel(value: &str) -> Option<u8> {
    if let Some(percent) = value.strip_suffix('%') {
        let p = percent.parse::<f32>().ok()?;
        Some((p * 255.0 / 100.0).clamp(0.0, 255.0) as u8)
    } else {
        let n = value.parse::<f32>().ok()?;
        Some(n.clamp(0.0, 255.0) as u8)
    }
}

fn parse_alpha(value: &str) -> Option<f32> {
    if let Some(percent) = value.strip_suffix('%') {
        let p = percent.parse::<f32>().ok()?;
        Some((p / 100.0).clamp(0.0, 1.0))
    } else {
        let a = value.parse::<f32>().ok()?;
        Some(a.clamp(0.0, 1.0))
    }
}

/// Convert a `Color` to a CSS color string.
///
/// Returns `rgb(r, g, b)` for opaque colors or `rgba(r, g, b, a)` for colors
/// with alpha < 1.0.
pub fn color_to_css(color: &Color) -> String {
    if (color.a - 1.0).abs() < f32::EPSILON {
        format!("rgb({}, {}, {})", color.r, color.g, color.b)
    } else {
        format!(
            "rgba({}, {}, {}, {})",
            color.r, color.g, color.b, color.a
        )
    }
}

/// Determine whether a color is "light" based on perceived luminance.
///
/// Formula: `0.299*r + 0.587*g + 0.114*b > 127`
pub fn is_light(color: &Color) -> bool {
    let luminance = 0.299 * f32::from(color.r)
        + 0.587 * f32::from(color.g)
        + 0.114 * f32::from(color.b);
    luminance > 127.0
}

/// Alpha-blend a foreground color over a background color.
pub fn blend(foreground: &Color, background: &Color) -> Color {
    let fa = foreground.a;
    let ba = background.a;

    let out_a = fa + ba * (1.0 - fa);
    if out_a.abs() < f32::EPSILON {
        return Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0.0,
        };
    }

    let r = ((f32::from(foreground.r) * fa + f32::from(background.r) * ba * (1.0 - fa))
        / out_a)
        .clamp(0.0, 255.0) as u8;
    let g = ((f32::from(foreground.g) * fa + f32::from(background.g) * ba * (1.0 - fa))
        / out_a)
        .clamp(0.0, 255.0) as u8;
    let b = ((f32::from(foreground.b) * fa + f32::from(background.b) * ba * (1.0 - fa))
        / out_a)
        .clamp(0.0, 255.0) as u8;

    Color {
        r,
        g,
        b,
        a: out_a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_color: named ─────────────────────────────────────────────
    #[test]
    fn test_named_red() {
        let c = parse_color("red").unwrap();
        assert_eq!(c, Color { r: 255, g: 0, b: 0, a: 1.0 });
    }

    #[test]
    fn test_named_blue() {
        let c = parse_color("blue").unwrap();
        assert_eq!(c, Color { r: 0, g: 0, b: 255, a: 1.0 });
    }

    #[test]
    fn test_named_black() {
        let c = parse_color("black").unwrap();
        assert_eq!(c, Color { r: 0, g: 0, b: 0, a: 1.0 });
    }

    #[test]
    fn test_named_white() {
        let c = parse_color("white").unwrap();
        assert_eq!(c, Color { r: 255, g: 255, b: 255, a: 1.0 });
    }

    #[test]
    fn test_named_all_colors() {
        let expected = [
            ("red", (255, 0, 0)),
            ("blue", (0, 0, 255)),
            ("green", (0, 128, 0)),
            ("black", (0, 0, 0)),
            ("white", (255, 255, 255)),
            ("gray", (128, 128, 128)),
            ("yellow", (255, 255, 0)),
            ("orange", (255, 165, 0)),
            ("purple", (128, 0, 128)),
            ("pink", (255, 192, 203)),
            ("brown", (165, 42, 42)),
            ("navy", (0, 0, 128)),
            ("teal", (0, 128, 128)),
            ("olive", (128, 128, 0)),
            ("maroon", (128, 0, 0)),
            ("aqua", (0, 255, 255)),
            ("fuchsia", (255, 0, 255)),
            ("lime", (0, 255, 0)),
            ("silver", (192, 192, 192)),
        ];
        for (name, (r, g, b)) in expected {
            let c = parse_color(name).unwrap();
            assert_eq!(
                c,
                Color {
                    r,
                    g,
                    b,
                    a: 1.0
                },
                "failed for {}",
                name
            );
        }
    }

    // ── parse_color: hex ───────────────────────────────────────────────
    #[test]
    fn test_hex_six() {
        let c = parse_color("#ff0000").unwrap();
        assert_eq!(c, Color { r: 255, g: 0, b: 0, a: 1.0 });
    }

    #[test]
    fn test_hex_three() {
        let c = parse_color("#f00").unwrap();
        assert_eq!(c, Color { r: 255, g: 0, b: 0, a: 1.0 });
    }

    #[test]
    fn test_hex_eight_with_alpha() {
        let c = parse_color("#ff000080").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert!((c.a - 0.50196).abs() < 0.01);
    }

    #[test]
    fn test_hex_four_with_alpha() {
        let c = parse_color("#f00f").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_hex_white() {
        let c = parse_color("#ffffff").unwrap();
        assert_eq!(c, Color { r: 255, g: 255, b: 255, a: 1.0 });
    }

    #[test]
    fn test_hex_black() {
        let c = parse_color("#000000").unwrap();
        assert_eq!(c, Color { r: 0, g: 0, b: 0, a: 1.0 });
    }

    // ── parse_color: rgb/rgba ─────────────────────────────────────────
    #[test]
    fn test_rgb_integers() {
        let c = parse_color("rgb(255, 128, 64)").unwrap();
        assert_eq!(c, Color { r: 255, g: 128, b: 64, a: 1.0 });
    }

    #[test]
    fn test_rgb_no_spaces() {
        let c = parse_color("rgb(0,0,0)").unwrap();
        assert_eq!(c, Color { r: 0, g: 0, b: 0, a: 1.0 });
    }

    #[test]
    fn test_rgb_percentages() {
        let c = parse_color("rgb(100%, 50%, 25%)").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 127);
        assert_eq!(c.b, 63);
    }

    #[test]
    fn test_rgba_with_alpha() {
        let c = parse_color("rgba(255, 0, 0, 0.5)").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 0.5);
    }

    #[test]
    fn test_rgba_alpha_percent() {
        let c = parse_color("rgba(0, 0, 255, 50%)").unwrap();
        assert_eq!(c.b, 255);
        assert_eq!(c.a, 0.5);
    }

    // ── parse_color: special ──────────────────────────────────────────
    #[test]
    fn test_transparent() {
        let c = parse_color("transparent").unwrap();
        assert_eq!(c, Color { r: 0, g: 0, b: 0, a: 0.0 });
    }

    #[test]
    fn test_currentcolor() {
        assert!(parse_color("currentcolor").is_none());
    }

    #[test]
    fn test_unknown_color() {
        assert!(parse_color("notacolor").is_none());
    }

    #[test]
    fn test_invalid_hex() {
        assert!(parse_color("#gggggg").is_none());
        assert!(parse_color("#fff00").is_none());
        assert!(parse_color("#ff000").is_none());
    }

    // ── color_to_css ──────────────────────────────────────────────────
    #[test]
    fn test_color_to_css_opaque() {
        let c = Color { r: 255, g: 0, b: 0, a: 1.0 };
        assert_eq!(color_to_css(&c), "rgb(255, 0, 0)");
    }

    #[test]
    fn test_color_to_css_alpha() {
        let c = Color { r: 255, g: 0, b: 0, a: 0.5 };
        assert_eq!(color_to_css(&c), "rgba(255, 0, 0, 0.5)");
    }

    // ── is_light ──────────────────────────────────────────────────────
    #[test]
    fn test_is_light_white() {
        assert!(is_light(&Color { r: 255, g: 255, b: 255, a: 1.0 }));
    }

    #[test]
    fn test_is_light_black() {
        assert!(!is_light(&Color { r: 0, g: 0, b: 0, a: 1.0 }));
    }

    #[test]
    fn test_is_light_gray() {
        // 128,128,128 → luminance = 128, which IS > 127
        assert!(is_light(&Color { r: 128, g: 128, b: 128, a: 1.0 }));
    }

    #[test]
    fn test_is_light_yellow() {
        assert!(is_light(&Color { r: 255, g: 255, b: 0, a: 1.0 }));
    }

    // ── blend ─────────────────────────────────────────────────────────
    #[test]
    fn test_blend_opaque_over_opaque() {
        let fg = Color { r: 255, g: 0, b: 0, a: 1.0 };
        let bg = Color { r: 0, g: 0, b: 255, a: 1.0 };
        let blended = blend(&fg, &bg);
        assert_eq!(blended.r, 255);
        assert_eq!(blended.g, 0);
        assert_eq!(blended.b, 0);
        assert_eq!(blended.a, 1.0);
    }

    #[test]
    fn test_blend_half_over_blue() {
        let fg = Color { r: 255, g: 0, b: 0, a: 0.5 };
        let bg = Color { r: 0, g: 0, b: 255, a: 1.0 };
        let blended = blend(&fg, &bg);
        assert_eq!(blended.r, 127);
        assert_eq!(blended.g, 0);
        assert_eq!(blended.b, 127);
    }

    #[test]
    fn test_blend_transparent_over_opaque() {
        let fg = Color { r: 255, g: 0, b: 0, a: 0.0 };
        let bg = Color { r: 0, g: 255, b: 0, a: 1.0 };
        let blended = blend(&fg, &bg);
        assert_eq!(blended.r, 0);
        assert_eq!(blended.g, 255);
        assert_eq!(blended.b, 0);
    }

    #[test]
    fn test_blend_over_transparent() {
        let fg = Color { r: 255, g: 0, b: 0, a: 0.5 };
        let bg = Color { r: 0, g: 0, b: 0, a: 0.0 };
        let blended = blend(&fg, &bg);
        assert_eq!(blended.r, 255);
        assert_eq!(blended.g, 0);
        assert_eq!(blended.b, 0);
        assert_eq!(blended.a, 0.5);
    }

    #[test]
    fn test_blend_both_transparent() {
        let fg = Color { r: 255, g: 0, b: 0, a: 0.0 };
        let bg = Color { r: 0, g: 0, b: 255, a: 0.0 };
        let blended = blend(&fg, &bg);
        assert_eq!(blended.a, 0.0);
    }
}
