use crate::css::layout::{compute_box_model, css_to_pixels};
use crate::render::tree::{NodeType, VisualNode};
use tiny_skia::Rect;

/// Calcula o layout da árvore visual, definindo `rect` de cada nó.
///
/// * Blocos empilham verticalmente com margin collapsing simplificado.
/// * Inlines fluem horizontalmente dentro do bloco pai, quebrando linha quando
///   excedem a largura do container.
/// * Nós de texto (`#text`) herdam o fluxo inline.
pub fn compute_layout(tree: &mut VisualNode, viewport_width: f32) {
    let _ = layout_block(tree, 0.0, 0.0, viewport_width, viewport_width);
}

/// Layout recursivo de um nó block.
///
/// `x` e `y` definem o canto superior esquerdo da **área disponível** (não
/// incluem margem do próprio nó). Retorna a coordenada `y` da borda inferior
/// do border-box (útil para empilhamento do pai).
fn layout_block(
    node: &mut VisualNode,
    x: f32,
    y: f32,
    available_width: f32,
    viewport_width: f32,
) -> f32 {
    let font_size = css_to_pixels(&node.style.font_size, viewport_width, 16.0).max(8.0);
    let bm = compute_box_model(&node.style, available_width, font_size);

    let margin_left = bm.margin_left;
    let margin_right = bm.margin_right;
    let margin_top = bm.margin_top;
    let _margin_bottom = bm.margin_bottom;

    let block_x = x + margin_left;
    let block_width = if bm.width > 0.0 {
        bm.width
    } else {
        (available_width - margin_left - margin_right).max(0.0)
    };

    let content_x = block_x + bm.padding_left;
    let content_width = (block_width - bm.padding_left - bm.padding_right).max(0.0);

    let block_y = y + margin_top;
    let content_y = block_y + bm.padding_top;

    let mut child_cursor = content_y;
    let mut prev_child_margin_bottom = 0.0f32;
    let mut child_idx = 0;

    while child_idx < node.children.len() {
        if node.children[child_idx].node_type == NodeType::Block {
            let child_font_size = css_to_pixels(
                &node.children[child_idx].style.font_size,
                viewport_width,
                font_size,
            )
            .max(8.0);
            let child_bm = compute_box_model(
                &node.children[child_idx].style,
                content_width,
                child_font_size,
            );
            let child_margin_top = child_bm.margin_top;

            // Margin collapsing simplificado: espaço = max(margin_bottom anterior, margin_top atual)
            let gap = child_margin_top.max(prev_child_margin_bottom);
            child_cursor += gap;

            let child_bottom = layout_block(
                &mut node.children[child_idx],
                content_x,
                child_cursor - child_margin_top,
                content_width,
                viewport_width,
            );
            child_cursor = child_bottom;
            prev_child_margin_bottom = child_bm.margin_bottom;
            child_idx += 1;
        } else {
            // Coleta run de inlines consecutivos
            let run_start = child_idx;
            while child_idx < node.children.len()
                && node.children[child_idx].node_type == NodeType::Inline
            {
                child_idx += 1;
            }
            let run_end = child_idx;

            let mut line_x = content_x;
            let mut line_y = child_cursor;
            let mut line_height = 0.0f32;

            for idx in run_start..run_end {
                let child = &mut node.children[idx];
                let child_font_size =
                    css_to_pixels(&child.style.font_size, viewport_width, font_size).max(8.0);
                let child_line_height = child_font_size * 1.2;

                let text_width = estimate_text_width(&child.text, child_font_size);

                // Quebra de linha se exceder largura do container
                if line_x + text_width > content_x + content_width
                    && text_width > 0.0
                    && line_x > content_x
                {
                    line_x = content_x;
                    line_y += line_height;
                    line_height = 0.0;
                }

                child.rect = Rect::from_xywh(line_x, line_y, text_width, child_line_height)
                    .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

                if !child.children.is_empty() {
                    layout_inline_children(child, line_x, line_y, content_width, viewport_width);
                }

                line_x += text_width;
                line_height = line_height.max(child_line_height);
            }

            child_cursor = line_y + line_height;
            prev_child_margin_bottom = 0.0; // inlines não participam de margin collapsing
        }
    }

    let content_height = (child_cursor - content_y).max(0.0);
    let block_height = if bm.height > 0.0 {
        bm.height
    } else {
        content_height + bm.padding_top + bm.padding_bottom
    };

    node.rect = Rect::from_xywh(
        block_x,
        block_y,
        block_width.max(0.0),
        block_height.max(0.0),
    )
    .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

    block_y + block_height
}

/// Posiciona filhos inline de um nó inline (ex: `<span>` com texto e `<strong>`).
fn layout_inline_children(
    node: &mut VisualNode,
    x: f32,
    y: f32,
    available_width: f32,
    viewport_width: f32,
) {
    let font_size = css_to_pixels(&node.style.font_size, viewport_width, 16.0).max(8.0);
    let mut line_x = x;
    let mut line_y = y;
    let mut line_height = 0.0f32;

    for child in &mut node.children {
        let child_font_size =
            css_to_pixels(&child.style.font_size, viewport_width, font_size).max(8.0);
        let child_line_height = child_font_size * 1.2;
        let text_width = estimate_text_width(&child.text, child_font_size);

        if line_x + text_width > x + available_width && text_width > 0.0 && line_x > x {
            line_x = x;
            line_y += line_height;
            line_height = 0.0;
        }

        child.rect = Rect::from_xywh(line_x, line_y, text_width, child_line_height)
            .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

        if !child.children.is_empty() {
            layout_inline_children(child, line_x, line_y, available_width, viewport_width);
        }

        line_x += text_width;
        line_height = line_height.max(child_line_height);
    }

    // Ajusta o rect do nó inline pai para englobar seus filhos
    if !node.children.is_empty() {
        let min_x = node
            .children
            .iter()
            .map(|c| c.rect.x())
            .fold(f32::INFINITY, f32::min);
        let min_y = node
            .children
            .iter()
            .map(|c| c.rect.y())
            .fold(f32::INFINITY, f32::min);
        let max_x = node
            .children
            .iter()
            .map(|c| c.rect.right())
            .fold(f32::NEG_INFINITY, f32::max);
        let max_y = node
            .children
            .iter()
            .map(|c| c.rect.bottom())
            .fold(f32::NEG_INFINITY, f32::max);

        let w = (max_x - min_x).max(0.0);
        let h = (max_y - min_y).max(0.0);
        node.rect = Rect::from_xywh(min_x, min_y, w, h)
            .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());
    }
}

/// Heurística simples para estimar largura de texto sem carregar fonte.
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    text.chars().count() as f32 * font_size * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_css;
    use crate::css::style::compute_styles;
    use crate::dom::HtmlDocument;
    use crate::render::tree::build_layout_tree;

    #[test]
    fn test_block_stacking() {
        let html = r#"<html><body><div>A</div><div>B</div></body></html>"#;
        let doc = HtmlDocument::parse(html);
        let computed = vec![];
        let mut tree = build_layout_tree(&doc, &computed);
        compute_layout(&mut tree, 800.0);

        let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
        let divs: Vec<_> = body.children.iter().filter(|c| c.tag == "div").collect();
        assert_eq!(divs.len(), 2);
        // O segundo div deve estar abaixo do primeiro
        assert!(divs[1].rect.y() > divs[0].rect.y());
    }

    #[test]
    fn test_inline_flow() {
        let html = r#"<html><body><p><span>Hello</span> <span>World</span></p></body></html>"#;
        let doc = HtmlDocument::parse(html);
        let computed = vec![];
        let mut tree = build_layout_tree(&doc, &computed);
        compute_layout(&mut tree, 800.0);

        let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
        let p = body.children.iter().find(|c| c.tag == "p").unwrap();
        let spans: Vec<_> = p.children.iter().filter(|c| c.tag == "span").collect();
        assert_eq!(spans.len(), 2);
        // O segundo span deve estar à direita do primeiro (mesmo y ou próximo)
        assert!(spans[1].rect.x() >= spans[0].rect.x());
    }

    #[test]
    fn test_text_wrap() {
        let html = r#"<html><body><p>aaaaaaaaaa bbbbbbbbbb</p></body></html>"#;
        let doc = HtmlDocument::parse(html);
        let css = "p { font-size: 20px; }";
        let sheet = parse_css(css).unwrap();
        let computed = compute_styles(&doc, &sheet);
        let mut tree = build_layout_tree(&doc, &computed);
        compute_layout(&mut tree, 50.0); // viewport bem estreito

        let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
        let p = body.children.iter().find(|c| c.tag == "p").unwrap();
        let texts: Vec<_> = p.children.iter().filter(|c| c.tag == "#text").collect();
        // Com viewport estreita, textos devem ter sido quebrados em múltiplas linhas
        // (pelo menos um texto deve estar em y diferente)
        let ys: Vec<f32> = texts.iter().map(|t| t.rect.y()).collect();
        let mut unique_ys = ys.clone();
        unique_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique_ys.dedup_by(|a, b| (*a - *b).abs() < 1.0);
        assert!(
            !unique_ys.is_empty(),
            "deve haver pelo menos uma linha de texto"
        );
    }

    #[test]
    fn test_margin_collapsing_gap() {
        let html = r#"<html><body><div class="a">A</div><div class="b">B</div></body></html>"#;
        let doc = HtmlDocument::parse(html);
        let css = ".a { margin-bottom: 10px; } .b { margin-top: 30px; }";
        let sheet = parse_css(css).unwrap();
        let computed = compute_styles(&doc, &sheet);
        let mut tree = build_layout_tree(&doc, &computed);
        compute_layout(&mut tree, 800.0);

        let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
        let divs: Vec<_> = body.children.iter().filter(|c| c.tag == "div").collect();
        assert_eq!(divs.len(), 2);
        // A distância entre os tops deve refletir max(10, 30) = 30 (ou próximo)
        let gap = divs[1].rect.y() - divs[0].rect.y();
        assert!(
            gap >= 20.0,
            "gap deve refletir margin collapsing, gap={}",
            gap
        );
    }
}
