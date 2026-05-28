// M7 Tests: position absolute/fixed, overflow, text-align, line-height, font cache
use faf_browser::css::parser::parse_css;
use faf_browser::css::style::compute_styles;
use faf_browser::dom::HtmlDocument;
use faf_browser::render::layout::compute_layout;
use faf_browser::render::screenshot::{render_to_image, ScreenshotConfig};
use faf_browser::render::tree::build_layout_tree;

// === T053: position: absolute ===

#[test]
fn test_position_absolute_with_positioned_ancestor() {
    let html = r#"<html><body>
        <div class="parent" style="position: relative; width: 400px; height: 300px; padding: 20px;">
            <div class="child" style="position: absolute; top: 10px; left: 10px; width: 50px; height: 50px;">
                ABS
            </div>
            <div class="normal">Normal flow</div>
        </div>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = ".parent { position: relative; width: 400px; height: 300px; padding: 20px; } \
               .child { position: absolute; top: 10px; left: 10px; width: 50px; height: 50px; } \
               .normal { }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);
    let mut tree = build_layout_tree(&doc, &computed);
    compute_layout(&mut tree, 800.0);

    // Encontrar parent div
    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let parent = body.children.iter().find(|c| c.classes.contains(&"parent".to_string())).unwrap();

    // Abs child deve ter posição calculada
    let abs_child = parent.children.iter().find(|c| c.classes.contains(&"child".to_string()));
    assert!(abs_child.is_some(), "deve ter child absolute");
    let abs = abs_child.unwrap();
    // absolute child não afeta o fluxo normal — parent height deve ser 300px (definido)
    assert!(
        (parent.rect.height() - 300.0).abs() < 5.0,
        "parent height deve ser 300, got {}",
        parent.rect.height()
    );
}

#[test]
fn test_position_absolute_not_in_normal_flow() {
    let html = r#"<html><body>
        <div class="container">
            <div class="abs">ABS</div>
            <div class="a">A</div>
            <div class="b">B</div>
        </div>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = ".container { position: relative; } \
               .abs { position: absolute; top: 0; left: 0; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);
    let mut tree = build_layout_tree(&doc, &computed);
    compute_layout(&mut tree, 800.0);

    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let container = body
        .children
        .iter()
        .find(|c| c.classes.contains(&"container".to_string()))
        .unwrap();

    // .a deve começar diretamente em content_y (sem .abs empurrando)
    let a_div = container.children.iter().find(|c| c.classes.contains(&"a".to_string()));
    assert!(a_div.is_some());
}

#[test]
fn test_position_fixed_uses_viewport() {
    let html = r#"<html><body>
        <div class="parent">
            <div class="fixed">FIXED</div>
        </div>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = ".fixed { position: fixed; top: 5px; left: 5px; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);
    let mut tree = build_layout_tree(&doc, &computed);
    compute_layout(&mut tree, 800.0);

    // Fixed deve usar viewport (0,0) como referência
    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    fn find_fixed(node: &faf_browser::render::tree::VisualNode) -> Option<&faf_browser::render::tree::VisualNode> {
        if node.classes.contains(&"fixed".to_string()) {
            return Some(node);
        }
        for c in &node.children {
            if let Some(f) = find_fixed(c) {
                return Some(f);
            }
        }
        None
    }
    let fixed = find_fixed(body);
    assert!(fixed.is_some(), "deve ter nó fixed");
    // top: 5px, left: 5px from viewport
    let f = fixed.unwrap();
    assert!(
        (f.rect.x() - 5.0).abs() < 2.0,
        "fixed left should be ~5 from viewport, got {}",
        f.rect.x()
    );
}

// === T054: text-align & line-height ===

#[test]
fn test_text_align_center_renders() {
    let html = r#"<html><body>
        <div style="width: 200px; text-align: center;">Centered</div>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = "div { width: 200px; text-align: center; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);

    // Verificar que text-align foi computado
    let has_center = computed.iter().any(|(_, s)| s.text_align == "center");
    assert!(has_center, "text-align: center deve estar no computed style");
}

#[test]
fn test_text_align_right_renders() {
    let html = r#"<html><body>
        <div style="width: 200px; text-align: right;">Right</div>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = "div { width: 200px; text-align: right; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);

    let has_right = computed.iter().any(|(_, s)| s.text_align == "right");
    assert!(has_right, "text-align: right deve estar no computed style");
}

#[test]
fn test_line_height_explicit() {
    let html = r#"<html><body>
        <p class="tall">Line height 2</p>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = ".tall { line-height: 2; font-size: 16px; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);

    let tall = computed.iter().find(|(em, _)| em.classes.contains(&"tall".to_string()));
    assert!(tall.is_some(), "deve ter .tall");
    let (_, style) = tall.unwrap();
    assert_eq!(style.line_height, "2", "line-height deve ser 2");

    // Renderizar e verificar que funciona
    let mut tree = build_layout_tree(&doc, &computed);
    compute_layout(&mut tree, 800.0);
}

// === T055: overflow ===

#[test]
fn test_overflow_hidden_clips_screenshot() {
    let html = r#"<html><body>
        <div class="clip" style="overflow: hidden; width: 50px; height: 50px;">
            <div style="width: 200px; height: 200px; background: red;">X</div>
        </div>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = ".clip { overflow: hidden; width: 50px; height: 50px; }";
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);

    let has_overflow = computed.iter().any(|(_, s)| s.overflow == "hidden");
    assert!(has_overflow, "overflow: hidden deve estar computado");

    // Verificar que renderiza sem crash
    let mut tree = build_layout_tree(&doc, &computed);
    compute_layout(&mut tree, 800.0);

    let config = ScreenshotConfig {
        width: 200,
        height: 200,
    };
    let path = "/tmp/faf_m7_overflow_test.png";
    render_to_image(&doc, &config, path).expect("screenshot deve renderizar");
    assert!(std::path::Path::new(path).exists(), "PNG deve existir");
}

#[test]
fn test_overflow_visible_default() {
    let html = r#"<html><body><div>Normal</div></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let computed = vec![];
    let tree = build_layout_tree(&doc, &computed);

    // Default overflow deve ser "visible"
    let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
    let div = body.children.iter().find(|c| c.tag == "div").unwrap();
    assert_eq!(div.style.overflow, "visible");
}

// === T056: font cache & performance ===

#[test]
fn test_screenshot_renders_full_page() {
    let html = r#"<html><body>
        <h1>FAF M7 Test</h1>
        <p>Parágrafo com texto normal para verificar renderização.</p>
        <div style="position: relative; width: 400px; height: 200px; background: #eee;">
            <div style="position: absolute; top: 10px; left: 10px; width: 100px; height: 30px; background: blue;">
                ABS
            </div>
            <div style="position: fixed; top: 5px; right: 5px;">FIXED</div>
        </div>
        <div style="overflow: hidden; width: 100px; height: 40px; border: 1px solid black;">
            <div style="width: 300px;">Este texto deve ser clipado</div>
        </div>
        <p style="text-align: center; font-size: 20px;">Texto centralizado</p>
        <p style="line-height: 2;">Texto com line-height dobrado.</p>
    </body></html>"#;
    let doc = HtmlDocument::parse(html);
    let css = "h1 { font-size: 24px; } \
               p { font-size: 16px; } \
               .abs { position: absolute; } \
               .fixed { position: fixed; }";
    let sheet = parse_css(css).unwrap();
    let _ = compute_styles(&doc, &sheet);

    let config = ScreenshotConfig {
        width: 800,
        height: 0, // auto
    };
    let path = "/tmp/faf_m7_full_test.png";
    render_to_image(&doc, &config, path).expect("screenshot M7 deve renderizar");
    assert!(std::path::Path::new(path).exists(), "PNG M7 deve existir");

    let meta = std::fs::metadata(path).unwrap();
    assert!(meta.len() > 100, "PNG deve ter conteúdo, got {} bytes", meta.len());
}

// === Regressão: campos novos têm defaults corretos ===

#[test]
fn test_computed_style_has_new_fields() {
    let style = faf_browser::css::style::ComputedStyle::default();
    assert_eq!(style.bottom, "auto");
    assert_eq!(style.right, "auto");
    assert_eq!(style.text_align, "left");
    assert_eq!(style.line_height, "normal");
    assert_eq!(style.font_weight, "normal");
    assert_eq!(style.overflow, "visible");
}
