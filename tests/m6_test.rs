use std::fs;
use std::path::Path;

use faf_browser::dom::HtmlDocument;
use faf_browser::render::screenshot::{render_to_image, ScreenshotConfig};
use faf_browser::render::tree::build_layout_tree;
use faf_browser::render::layout::compute_layout;
use faf_browser::css::parser::parse_css;
use faf_browser::css::style::compute_styles;

// ---------------------------------------------------------------------------
// T049 — Bordas CSS
// ---------------------------------------------------------------------------

#[test]
fn test_screenshot_border_rendered() {
    let html = r#"
        <html><body>
            <div style="border: 2px solid red; width: 100px; height: 100px;"></div>
        </body></html>
    "#;
    let doc = HtmlDocument::parse(html);
    let output = "/tmp/faf_test_border.png";
    let config = ScreenshotConfig {
        width: 400,
        height: 200,
    };

    render_to_image(&doc, &config, output).expect("render should succeed");
    assert!(Path::new(output).exists(), "screenshot should exist");
    let meta = fs::metadata(output).expect("should read metadata");
    assert!(meta.len() > 0, "screenshot should not be empty");

    // Cleanup
    let _ = fs::remove_file(output);
}

#[test]
fn test_layout_tree_has_attributes() {
    let html = r#"<html><body><img src="test.jpg" alt="Test"></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let computed = vec![];
    let tree = build_layout_tree(&doc, &computed, None);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let img = body.children.iter().find(|c| c.tag == "img").unwrap();
    assert_eq!(img.attributes.get("src"), Some(&"test.jpg".to_string()));
    assert_eq!(img.attributes.get("alt"), Some(&"Test".to_string()));
}

// ---------------------------------------------------------------------------
// T050 — Imagens <img>
// ---------------------------------------------------------------------------

#[test]
fn test_screenshot_image_fallback_placeholder() {
    // URL inválida para forçar fallback
    let html = r#"
        <html><body>
            <img src="http://127.0.0.1:1/invalid.png" style="width: 50px; height: 50px;">
        </body></html>
    "#;
    let doc = HtmlDocument::parse(html);
    let output = "/tmp/faf_test_image_fallback.png";
    let config = ScreenshotConfig {
        width: 400,
        height: 200,
    };

    render_to_image(&doc, &config, output).expect("render should succeed even with broken image");
    assert!(Path::new(output).exists(), "screenshot should exist");
    let meta = fs::metadata(output).expect("should read metadata");
    assert!(meta.len() > 0, "screenshot should not be empty");

    // Cleanup
    let _ = fs::remove_file(output);
}

#[test]
fn test_border_width_zero_by_default() {
    let html = "<html><body><div>Box</div></body></html>";
    let doc = HtmlDocument::parse(html);
    let css = "div { color: black; }";
    let sheet = parse_css(css).unwrap();
    let results = compute_styles(&doc, &sheet);

    assert_eq!(results.len(), 1);
    let (_, style) = &results[0];
    assert_eq!(style.border_top_width, "0");
    assert_eq!(style.border_right_width, "0");
    assert_eq!(style.border_bottom_width, "0");
    assert_eq!(style.border_left_width, "0");
    assert_eq!(style.border_top_color, "transparent");
    assert_eq!(style.border_top_style, "none");
}

// ---------------------------------------------------------------------------
// T051 — Relative positioning
// ---------------------------------------------------------------------------

#[test]
fn test_relative_positioning_offset() {
    let html = r#"
        <html><head><style>
            .box { position: relative; top: 20px; left: 15px; width: 50px; height: 50px; }
        </style></head><body>
            <div class="box">A</div>
        </body></html>
    "#;
    let doc = HtmlDocument::parse(html);
    let css = doc.extract_css().unwrap_or_default();
    let sheet = parse_css(&css).unwrap();
    let computed = compute_styles(&doc, &sheet);
    let mut tree = build_layout_tree(&doc, &computed, None);
    compute_layout(&mut tree, 400.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let div = body.children.iter().find(|c| c.tag == "div").unwrap();

    assert!(
        (div.rect.y() - 20.0).abs() < 0.1,
        "relative top=20px should offset y, got y={}",
        div.rect.y()
    );
    assert!(
        (div.rect.x() - 15.0).abs() < 0.1,
        "relative left=15px should offset x, got x={}",
        div.rect.x()
    );
}

// ---------------------------------------------------------------------------
// T052 — z-index
// ---------------------------------------------------------------------------

#[test]
fn test_z_index_computed_and_render() {
    let html = r#"
        <html><head><style>
            .back { z-index: 1; position: relative; width: 100px; height: 100px; background-color: red; }
            .front { z-index: 2; position: relative; width: 100px; height: 100px; background-color: blue; }
        </style></head><body>
            <div class="back"></div>
            <div class="front"></div>
        </body></html>
    "#;
    let doc = HtmlDocument::parse(html);
    let output = "/tmp/faf_test_zindex.png";
    let config = ScreenshotConfig {
        width: 400,
        height: 200,
    };

    render_to_image(&doc, &config, output).expect("render should succeed with z-index");
    assert!(Path::new(output).exists(), "screenshot should exist");
    let meta = fs::metadata(output).expect("should read metadata");
    assert!(meta.len() > 0, "screenshot should not be empty");

    let css = doc.extract_css().unwrap_or_default();
    let sheet = parse_css(&css).unwrap();
    let results = compute_styles(&doc, &sheet);
    let z_values: Vec<&str> = results.iter().map(|(_, s)| s.z_index.as_str()).collect();
    assert!(z_values.contains(&"1"), "should have z-index 1");
    assert!(z_values.contains(&"2"), "should have z-index 2");

    let _ = fs::remove_file(output);
}

// ---------------------------------------------------------------------------
// T053 — Background renderizado
// ---------------------------------------------------------------------------

#[test]
fn test_screenshot_background_rendered() {
    let html = r#"
        <html><head><style>
            div { background-color: green; width: 100px; height: 100px; }
        </style></head><body>
            <div></div>
        </body></html>
    "#;
    let doc = HtmlDocument::parse(html);
    let output = "/tmp/faf_test_background.png";
    let config = ScreenshotConfig {
        width: 400,
        height: 200,
    };

    render_to_image(&doc, &config, output).expect("render should succeed");
    assert!(Path::new(output).exists(), "screenshot should exist");
    let meta = fs::metadata(output).expect("should read metadata");
    assert!(meta.len() > 0, "screenshot should not be empty");

    let _ = fs::remove_file(output);
}

// ---------------------------------------------------------------------------
// T054 — Text wrap
// ---------------------------------------------------------------------------

#[test]
fn test_text_wrap_narrow_viewport() {
    let html = r#"
        <html><head><style>
            p { width: 40px; font-size: 20px; }
            span { font-size: 20px; }
        </style></head><body>
            <p>aaaa<span></span>bbbb<span></span>cccc</p>
        </body></html>
    "#;
    let doc = HtmlDocument::parse(html);
    let css = doc.extract_css().unwrap_or_default();
    let sheet = parse_css(&css).unwrap();
    let computed = compute_styles(&doc, &sheet);
    let mut tree = build_layout_tree(&doc, &computed, None);
    compute_layout(&mut tree, 100.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let p = body.children.iter().find(|c| c.tag == "p").unwrap();
    let children: Vec<_> = p.children.iter().collect();
    assert!(children.len() >= 3, "should have multiple inline children");

    let ys: Vec<f32> = children.iter().map(|c| c.rect.y()).collect();
    let mut unique_ys = ys.clone();
    unique_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    unique_ys.dedup_by(|a, b| (*a - *b).abs() < 1.0);
    assert!(
        unique_ys.len() > 1,
        "text should wrap producing multiple lines, got ys={:?}",
        ys
    );
}

// ---------------------------------------------------------------------------
// T055 — Regressão de profundidade
// ---------------------------------------------------------------------------

#[test]
fn test_deep_nesting_regression() {
    let mut html = String::from("<html><body>");
    for i in 0..50 {
        html.push_str(&format!("<div class=\"d{}\">", i));
    }
    html.push_str("Deep content");
    for _ in 0..50 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    let doc = HtmlDocument::parse(&html);
    let computed = vec![];
    let tree = build_layout_tree(&doc, &computed, None);

    fn max_depth(node: &faf_browser::render::tree::VisualNode) -> usize {
        1 + node.children.iter().map(max_depth).max().unwrap_or(0)
    }

    let depth = max_depth(&tree);
    assert!(
        depth >= 50,
        "deep nesting should not break tree construction, depth={}",
        depth
    );
}

