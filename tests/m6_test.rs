use std::fs;
use std::path::Path;

use faf_browser::dom::HtmlDocument;
use faf_browser::render::screenshot::{render_to_image, ScreenshotConfig};
use faf_browser::render::tree::build_layout_tree;
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
    let tree = build_layout_tree(&doc, &computed);

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
