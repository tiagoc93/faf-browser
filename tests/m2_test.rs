use faf_browser::css::color::{Color, parse_color};
use faf_browser::css::font::parse_font_family;
use faf_browser::css::layout::{compute_box_model, css_to_pixels, outer_height, outer_width};
use faf_browser::css::parser::parse_css;
use faf_browser::css::selector::select_elements;
use faf_browser::css::style::{ComputedStyle, compute_styles};
use faf_browser::dom::HtmlDocument;

// ---------------------------------------------------------------------------
// 1. CSS parsing + selector matching end-to-end
// ---------------------------------------------------------------------------
#[test]
fn test_css_parsing_and_selector_matching() {
    let html = r#"<html>
        <body>
            <h1 id="title" class="main">Hello</h1>
            <p class="text">World</p>
            <div class="box"><span>Inner</span></div>
        </body>
    </html>"#;
    let doc = HtmlDocument::parse(html);
    let css = r#"
        h1 { color: red; }
        .text { font-size: 14px; }
        #title { margin: 10px; }
        div span { display: inline; }
    "#;
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 4);

    // h1 selector
    let h1_matches = select_elements(&doc, "h1").unwrap();
    assert_eq!(h1_matches.len(), 1);
    assert_eq!(h1_matches[0].tag, "h1");
    assert_eq!(h1_matches[0].text, "Hello");

    // class selector
    let text_matches = select_elements(&doc, ".text").unwrap();
    assert_eq!(text_matches.len(), 1);
    assert_eq!(text_matches[0].tag, "p");

    // id selector
    let title_matches = select_elements(&doc, "#title").unwrap();
    assert_eq!(title_matches.len(), 1);
    assert_eq!(title_matches[0].id, Some("title".to_string()));

    // descendant selector
    let span_matches = select_elements(&doc, "div span").unwrap();
    assert_eq!(span_matches.len(), 1);
    assert_eq!(span_matches[0].tag, "span");
}

// ---------------------------------------------------------------------------
// 2. Computed styles
// ---------------------------------------------------------------------------
#[test]
fn test_computed_styles_basic() {
    let html = r#"<html>
        <body>
            <h1 class="title">Hello</h1>
            <p>World</p>
        </body>
    </html>"#;
    let doc = HtmlDocument::parse(html);
    let css = r#"
        h1 { color: blue; font-size: 24px; }
        .title { background-color: gray; }
        p { color: black; font-family: Arial; }
    "#;
    let sheet = parse_css(css).unwrap();
    let results = compute_styles(&doc, &sheet);

    assert_eq!(results.len(), 2);

    let h1 = results.iter().find(|(em, _)| em.tag == "h1").unwrap();
    assert_eq!(h1.1.color, "blue");
    assert_eq!(h1.1.font_size, "24px");
    assert_eq!(h1.1.background_color, "gray");

    let p = results.iter().find(|(em, _)| em.tag == "p").unwrap();
    assert_eq!(p.1.color, "black");
    assert_eq!(p.1.font_family, "Arial");
}

#[test]
fn test_computed_styles_cascade() {
    let html = r#"<html><body><h1>Title</h1></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = "h1 { color: red; } h1 { color: green; }";
    let sheet = parse_css(css).unwrap();
    let results = compute_styles(&doc, &sheet);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.color, "green");
}

#[test]
fn test_computed_styles_specificity() {
    let html = r#"<html><body><div class="box">Text</div></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = "div { color: blue; } .box { color: red; }";
    let sheet = parse_css(css).unwrap();
    let results = compute_styles(&doc, &sheet);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.color, "red");
}

// ---------------------------------------------------------------------------
// 3. Box model calculation
// ---------------------------------------------------------------------------
#[test]
fn test_box_model_pixels() {
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
fn test_box_model_percent_and_em() {
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
fn test_box_model_outer_dimensions() {
    let style = ComputedStyle {
        width: "200px".to_string(),
        height: "100px".to_string(),
        margin_top: "10px".to_string(),
        margin_right: "20px".to_string(),
        margin_bottom: "15px".to_string(),
        margin_left: "25px".to_string(),
        ..ComputedStyle::default()
    };
    let bm = compute_box_model(&style, 800.0, 16.0);
    assert_eq!(outer_width(&bm), 245.0); // 200 + 25 + 20
    assert_eq!(outer_height(&bm), 125.0); // 100 + 10 + 15
}

#[test]
fn test_css_to_pixels_units() {
    assert_eq!(css_to_pixels("100px", 800.0, 16.0), 100.0);
    assert_eq!(css_to_pixels("50%", 800.0, 16.0), 400.0);
    assert_eq!(css_to_pixels("2em", 800.0, 16.0), 32.0);
    assert_eq!(css_to_pixels("auto", 800.0, 16.0), 0.0);
    assert_eq!(css_to_pixels("0", 800.0, 16.0), 0.0);
    assert_eq!(css_to_pixels("", 800.0, 16.0), 0.0);
}

// ---------------------------------------------------------------------------
// 4. Color parsing
// ---------------------------------------------------------------------------
#[test]
fn test_color_parsing_named() {
    let c = parse_color("red").unwrap();
    assert_eq!(
        c,
        Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0
        }
    );

    let c = parse_color("blue").unwrap();
    assert_eq!(
        c,
        Color {
            r: 0,
            g: 0,
            b: 255,
            a: 1.0
        }
    );

    let c = parse_color("white").unwrap();
    assert_eq!(
        c,
        Color {
            r: 255,
            g: 255,
            b: 255,
            a: 1.0
        }
    );

    let c = parse_color("black").unwrap();
    assert_eq!(
        c,
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0
        }
    );
}

#[test]
fn test_color_parsing_hex() {
    let c = parse_color("#ff0000").unwrap();
    assert_eq!(
        c,
        Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0
        }
    );

    let c = parse_color("#f00").unwrap();
    assert_eq!(
        c,
        Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0
        }
    );

    let c = parse_color("#ffffff").unwrap();
    assert_eq!(
        c,
        Color {
            r: 255,
            g: 255,
            b: 255,
            a: 1.0
        }
    );

    let c = parse_color("#000000").unwrap();
    assert_eq!(
        c,
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0
        }
    );
}

#[test]
fn test_color_parsing_hex_alpha() {
    let c = parse_color("#ff000080").unwrap();
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 0);
    assert_eq!(c.b, 0);
    assert!((c.a - 0.50196).abs() < 0.01);
}

#[test]
fn test_color_parsing_rgb() {
    let c = parse_color("rgb(255, 128, 64)").unwrap();
    assert_eq!(
        c,
        Color {
            r: 255,
            g: 128,
            b: 64,
            a: 1.0
        }
    );

    let c = parse_color("rgb(0,0,0)").unwrap();
    assert_eq!(
        c,
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0
        }
    );
}

#[test]
fn test_color_parsing_rgba() {
    let c = parse_color("rgba(255, 0, 0, 0.5)").unwrap();
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 0);
    assert_eq!(c.b, 0);
    assert_eq!(c.a, 0.5);
}

#[test]
fn test_color_parsing_transparent() {
    let c = parse_color("transparent").unwrap();
    assert_eq!(
        c,
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0.0
        }
    );
}

#[test]
fn test_color_parsing_invalid() {
    assert!(parse_color("notacolor").is_none());
    assert!(parse_color("#gggggg").is_none());
}

// ---------------------------------------------------------------------------
// 5. Font parsing
// ---------------------------------------------------------------------------
#[test]
fn test_font_family_parsing_simple() {
    let families = parse_font_family("Arial, sans-serif");
    assert_eq!(families, vec!["Arial", "sans-serif"]);
}

#[test]
fn test_font_family_parsing_quoted() {
    let families = parse_font_family("'Times New Roman', Arial, sans-serif");
    assert_eq!(families, vec!["Times New Roman", "Arial", "sans-serif"]);

    let families = parse_font_family("\"Helvetica Neue\", Arial, sans-serif");
    assert_eq!(families, vec!["Helvetica Neue", "Arial", "sans-serif"]);
}

#[test]
fn test_font_family_parsing_single() {
    let families = parse_font_family("serif");
    assert_eq!(families, vec!["serif"]);
}

#[test]
fn test_font_family_parsing_empty() {
    let families = parse_font_family("");
    assert!(families.is_empty());
}

#[test]
fn test_font_family_parsing_whitespace() {
    let families = parse_font_family("  Arial  ,   sans-serif  ");
    assert_eq!(families, vec!["Arial", "sans-serif"]);
}
