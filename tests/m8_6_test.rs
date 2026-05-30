// M8.6 Tests: Pseudo-Class Selectors & Intrinsic Image Dimensions (T075 + T076)
use std::collections::HashMap;

use faf_browser::css::parser::parse_css;
use faf_browser::css::selector::select_elements;
use faf_browser::css::style::compute_styles;
use faf_browser::dom::HtmlDocument;
use faf_browser::render::layout::compute_layout;
use faf_browser::render::tree::build_layout_tree;

// === T075: Pseudo-class selector filtering ===

#[test]
fn test_pseudo_class_selector_list_does_not_reject_valid_parts() {
    let html = r#"<div class="parent"><img class="child"></div>"#;
    let doc = HtmlDocument::parse(html);
    // ".parent > img" is valid; ".parent > img:hover" contains a pseudo-class
    let matches = select_elements(&doc, ".parent > img, .parent > img:hover").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].tag, "img");
}

#[test]
fn test_pseudo_class_only_returns_empty() {
    let html = r#"<div class="a"></div>"#;
    let doc = HtmlDocument::parse(html);
    // All parts contain unsupported pseudo-classes
    let matches = select_elements(&doc, ".a:hover, .a:focus").unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_pseudo_element_still_filtered() {
    let html = r#"<div class="a"></div>"#;
    let doc = HtmlDocument::parse(html);
    // ::before is a pseudo-element and should be skipped
    let matches = select_elements(&doc, ".a::before, .a").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].tag, "div");
}

// === T076: Intrinsic image dimensions ===

#[test]
fn test_img_uses_intrinsic_dimensions_when_no_css() {
    let html = r#"<html><body><img src="book.jpg" alt="Book"></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let computed = vec![];

    // Simulate fetched intrinsic dimensions
    let mut image_dims = HashMap::new();
    image_dims.insert("book.jpg".to_string(), (300, 450));

    let mut tree = build_layout_tree(&doc, &computed, Some(&image_dims));
    compute_layout(&mut tree, 800.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let img = body.children.iter().find(|c| c.tag == "img").unwrap();

    // The layout should use intrinsic dimensions
    assert_eq!(img.rect.width(), 300.0, "width should use intrinsic width");
    assert_eq!(img.rect.height(), 450.0, "height should use intrinsic height");
}

#[test]
fn test_img_uses_resolved_url_intrinsic_dimensions() {
    let html = r#"<html><body><img src="media/cache/123.jpg" alt="Book"></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let computed = vec![];

    // The map may contain the resolved (absolute) URL, but build_layout_tree
    // should also find the relative src key.
    let mut image_dims = HashMap::new();
    image_dims.insert("media/cache/123.jpg".to_string(), (200, 300));

    let mut tree = build_layout_tree(&doc, &computed, Some(&image_dims));
    compute_layout(&mut tree, 800.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let img = body.children.iter().find(|c| c.tag == "img").unwrap();

    assert_eq!(img.rect.width(), 200.0, "width should use intrinsic width");
    assert_eq!(img.rect.height(), 300.0, "height should use intrinsic height");
}

#[test]
fn test_img_fallback_100x100_when_no_intrinsic_dims() {
    let html = r#"<html><body><img src="missing.jpg" alt="Missing"></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let computed = vec![];

    let mut tree = build_layout_tree(&doc, &computed, None);
    compute_layout(&mut tree, 800.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let img = body.children.iter().find(|c| c.tag == "img").unwrap();

    assert_eq!(img.rect.width(), 100.0, "width should fallback to 100");
    assert_eq!(img.rect.height(), 100.0, "height should fallback to 100");
}

#[test]
fn test_multiple_imgs_different_sizes() {
    let html = r#"<html><body>
        <img src="a.jpg" alt="A">
        <img src="b.jpg" alt="B">
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let computed = vec![];

    let mut image_dims = HashMap::new();
    image_dims.insert("a.jpg".to_string(), (150, 200));
    image_dims.insert("b.jpg".to_string(), (300, 400));

    let mut tree = build_layout_tree(&doc, &computed, Some(&image_dims));
    compute_layout(&mut tree, 800.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let imgs: Vec<_> = body.children.iter().filter(|c| c.tag == "img").collect();
    assert_eq!(imgs.len(), 2);

    assert_eq!(imgs[0].rect.width(), 150.0);
    assert_eq!(imgs[0].rect.height(), 200.0);
    assert_eq!(imgs[1].rect.width(), 300.0);
    assert_eq!(imgs[1].rect.height(), 400.0);
}

#[test]
fn test_css_explicit_dimensions_override_intrinsic() {
    let html = r#"<html><body><img src="book.jpg" alt="Book"></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = "img { width: 50px; height: 80px; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);

    let mut image_dims = HashMap::new();
    image_dims.insert("book.jpg".to_string(), (300, 450));

    let mut tree = build_layout_tree(&doc, &computed, Some(&image_dims));
    compute_layout(&mut tree, 800.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let img = body.children.iter().find(|c| c.tag == "img").unwrap();

    // CSS explicit dimensions should override intrinsic dimensions
    assert_eq!(img.rect.width(), 50.0, "CSS width should override intrinsic");
    assert_eq!(img.rect.height(), 80.0, "CSS height should override intrinsic");
}
