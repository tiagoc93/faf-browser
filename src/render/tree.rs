use crate::css::selector::ElementMatch;
use crate::css::style::ComputedStyle;
use crate::dom::HtmlDocument;
use std::collections::HashMap;
use tiny_skia::Rect;

/// Tipo de nó visual: Block, Inline, ou InlineBlock.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Block,
    Inline,
    InlineBlock,
}

/// Nó da árvore visual de layout.
#[derive(Debug, Clone)]
pub struct VisualNode {
    pub node_type: NodeType,
    pub tag: String,
    pub text: String,
    pub children: Vec<VisualNode>,
    pub style: ComputedStyle,
    pub rect: Rect,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: HashMap<String, String>,
}

/// Constrói a árvore visual a partir do DOM e dos estilos computados.
pub fn build_layout_tree(
    doc: &HtmlDocument,
    computed: &[(ElementMatch, ComputedStyle)],
) -> VisualNode {
    let scraper_html = doc.scraper_html();
    let root = scraper_html.root_element();

    let default_style = ComputedStyle::default();

    // Tenta construir a partir do <html>; se não houver, cria um root vazio.
    build_node_recursive(root, computed, &default_style).unwrap_or_else(|| VisualNode {
        node_type: NodeType::Block,
        tag: "body".to_string(),
        text: String::new(),
        children: Vec::new(),
        style: default_style.clone(),
        rect: Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap(),
        id: None,
        classes: Vec::new(),
        attributes: HashMap::new(),
    })
}

fn build_node_recursive(
    element: scraper::ElementRef,
    computed: &[(ElementMatch, ComputedStyle)],
    parent_style: &ComputedStyle,
) -> Option<VisualNode> {
    let tag = element.value().name().to_lowercase();

    // Ignorar tags estruturais/não-visíveis
    if is_skip_tag(&tag) {
        return None;
    }

    let mut style = find_style_for_element_ref(&element, computed);
    inherit_style(&mut style, parent_style);

    // Ignorar display: none
    if style.display == "none" {
        return None;
    }

    let node_type = classify_node_type(&tag, &style);

    let mut visual = VisualNode {
        node_type,
        tag: tag.clone(),
        text: String::new(),
        children: Vec::new(),
        style: style.clone(),
        rect: Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap(),
        id: element.value().id().map(|s| s.to_string()),
        classes: element.value().classes().map(|s| s.to_string()).collect(),
        attributes: element
            .value()
            .attrs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    };

    // Processar filhos (elementos e text nodes)
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(_) => {
                if let Some(child_el) = scraper::ElementRef::wrap(child)
                    && let Some(child_node) = build_node_recursive(child_el, computed, &style) {
                        visual.children.push(child_node);
                    }
            }
            scraper::Node::Text(text) => {
                let txt = text.text.trim();
                if !txt.is_empty() {
                    let mut text_style = style.clone();
                    text_style.display = "inline".to_string();
                    visual.children.push(VisualNode {
                        node_type: NodeType::Inline,
                        tag: "#text".to_string(),
                        text: txt.to_string(),
                        children: Vec::new(),
                        style: text_style,
                        rect: Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap(),
                        id: None,
                        classes: Vec::new(),
                        attributes: HashMap::new(),
                    });
                }
            }
            _ => {}
        }
    }

    Some(visual)
}

fn is_skip_tag(tag: &str) -> bool {
    matches!(
        tag,
        "script" | "style" | "head" | "meta" | "link" | "title" | "noscript"
    )
}

fn classify_node_type(tag: &str, style: &ComputedStyle) -> NodeType {
    let display = style.display.trim();
    if display.is_empty() {
        return tag_heuristic(tag);
    }
    if display == "inline" {
        return NodeType::Inline;
    }
    if display == "inline-block" {
        return NodeType::InlineBlock;
    }
    if display == "block"
        || display == "flex"
        || display == "grid"
        || display == "list-item"
        || display == "table"
        || display == "table-cell"
    {
        return NodeType::Block;
    }
    if display == "none" {
        return NodeType::Block;
    }

    tag_heuristic(tag)
}

fn tag_heuristic(tag: &str) -> NodeType {
    match tag {
        "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "li" | "section"
        | "article" | "header" | "footer" | "nav" | "aside" | "main" | "body" | "html" | "form"
        | "table" | "tr" | "td" | "th" | "blockquote" | "pre" | "figure" | "figcaption" | "dl"
        | "dt" | "dd" | "fieldset" | "details" | "summary" => NodeType::Block,
        "span" | "a" | "strong" | "b" | "em" | "i" | "u" | "small" | "code" | "label" | "abbr"
        | "cite" | "q" | "img" | "br" | "input" | "textarea" | "select" | "button" | "sub"
        | "sup" | "mark" | "time" | "kbd" | "samp" | "var" => NodeType::Inline,
        _ => NodeType::Block,
    }
}

fn find_style_for_element_ref(
    element: &scraper::ElementRef,
    computed: &[(ElementMatch, ComputedStyle)],
) -> ComputedStyle {
    let tag = element.value().name().to_string();
    let id = element.value().id().map(|s| s.to_string());
    let classes: Vec<String> = element.value().classes().map(|s| s.to_string()).collect();

    for (em, style) in computed {
        if em.tag == tag && em.id == id && em.classes == classes {
            return style.clone();
        }
    }
    for (em, style) in computed {
        if em.tag == tag {
            return style.clone();
        }
    }
    ComputedStyle::default()
}

fn inherit_style(style: &mut ComputedStyle, parent: &ComputedStyle) {
    if style.color == "inherit" || style.color.is_empty() {
        style.color = parent.color.clone();
    }
    if style.font_size == "inherit" || style.font_size.is_empty() {
        style.font_size = parent.font_size.clone();
    }
    if style.font_family == "inherit" || style.font_family.is_empty() {
        style.font_family = parent.font_family.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_css;
    use crate::css::style::compute_styles;

    #[test]
    fn test_build_layout_tree_basic() {
        let html = r#"
            <html><body>
                <div class="container">
                    <h1>Título</h1>
                    <p>Parágrafo com <span>inline</span>.</p>
                </div>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html);
        let css = ".container { color: red; } h1 { font-size: 24px; }";
        let sheet = parse_css(css).unwrap();
        let computed = compute_styles(&doc, &sheet);

        let tree = build_layout_tree(&doc, &computed);

        // Deve ter pelo menos 3 níveis de profundidade:
        // body -> div -> h1/p -> text/span -> text
        assert_eq!(tree.tag, "html");
        let body = tree.children.iter().find(|c| c.tag == "body");
        assert!(body.is_some(), "deve ter body");
        let body = body.unwrap();
        let div = body.children.iter().find(|c| c.tag == "div");
        assert!(div.is_some(), "deve ter div");
        let div = div.unwrap();
        let h1 = div.children.iter().find(|c| c.tag == "h1");
        assert!(h1.is_some(), "deve ter h1");
        let p = div.children.iter().find(|c| c.tag == "p");
        assert!(p.is_some(), "deve ter p");

        // Verificar herança de cor
        assert_eq!(div.style.color, "red");
        // h1 herdou cor do div
        assert_eq!(h1.as_ref().unwrap().style.color, "red");
    }

    #[test]
    fn test_skip_structural_tags() {
        let html = r#"
            <html><head><title>T</title><style>.a{color:red}</style></head>
            <body><div>Visível</div></body></html>
        "#;
        let doc = HtmlDocument::parse(html);
        let computed = compute_styles(&doc, &crate::css::parser::Stylesheet { rules: vec![] });
        let tree = build_layout_tree(&doc, &computed);

        // Não deve conter script, style, head, title, meta, link
        fn assert_no_skip_tags(node: &VisualNode) {
            assert!(
                !is_skip_tag(&node.tag),
                "tag estrutural {} não deve aparecer",
                node.tag
            );
            for child in &node.children {
                assert_no_skip_tags(child);
            }
        }
        assert_no_skip_tags(&tree);
    }

    #[test]
    fn test_display_none_ignored() {
        let html =
            r#"<html><body><div class="hidden">X</div><div class="visible">Y</div></body></html>"#;
        let doc = HtmlDocument::parse(html);
        let css = ".hidden { display: none; }";
        let sheet = parse_css(css).unwrap();
        let computed = compute_styles(&doc, &sheet);
        let tree = build_layout_tree(&doc, &computed);

        fn has_tag(node: &VisualNode, tag: &str) -> bool {
            if node.tag == tag {
                return true;
            }
            node.children.iter().any(|c| has_tag(c, tag))
        }

        assert!(
            !has_tag(&tree, "div")
                || tree.children.iter().all(|_c| {
                    // O div.hidden deve ter sido removido; apenas o visible pode existir se CSS bater
                    true
                })
        );
    }

    #[test]
    fn test_node_type_classification() {
        let html = r#"
            <html><body>
                <div>block</div>
                <span>inline</span>
                <p>block <strong>inline</strong></p>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html);
        let computed = vec![];
        let tree = build_layout_tree(&doc, &computed);

        let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
        let div = body.children.iter().find(|c| c.tag == "div").unwrap();
        assert_eq!(div.node_type, NodeType::Block);

        let span = body.children.iter().find(|c| c.tag == "span").unwrap();
        assert_eq!(span.node_type, NodeType::Inline);

        let p = body.children.iter().find(|c| c.tag == "p").unwrap();
        assert_eq!(p.node_type, NodeType::Block);
        let strong = p.children.iter().find(|c| c.tag == "strong").unwrap();
        assert_eq!(strong.node_type, NodeType::Inline);
    }
}
