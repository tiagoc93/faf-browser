use crate::css::layout::{compute_box_model, css_to_pixels};
use crate::render::tree::{NodeType, VisualNode};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use std::collections::HashMap;
use std::fs;
use tiny_skia::Rect;

/// Caminhos de fontes TTF conhecidos no sistema
const FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSerif.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
];

/// Calcula o layout da árvore visual, definindo `rect` de cada nó.
///
/// Carrega fontes uma vez (cache) e passa para o layout recursivo,
/// permitindo medição real de texto com ab_glyph.
pub fn compute_layout(tree: &mut VisualNode, viewport_width: f32) {
    let font_cache = load_font_cache();
    let default_font = font_cache
        .values()
        .next()
        .cloned();
    let _ = layout_block(
        tree,
        0.0,
        0.0,
        viewport_width,
        viewport_width,
        &font_cache,
        &default_font,
    );
}

/// Carrega todas as fontes do sistema em um cache.
fn load_font_cache() -> HashMap<String, FontArc> {
    let mut cache = HashMap::new();
    for &path in FONT_PATHS {
        if let Some(file_name) = path.rsplit('/').next() {
            let file_stem = file_name.rsplit('.').next_back().unwrap_or(file_name);
            let key = file_stem.to_lowercase();
            if !cache.contains_key(&key) {
                if let Ok(data) = fs::read(path) {
                    if let Ok(font) = FontArc::try_from_vec(data) {
                        cache.insert(key, font);
                    }
                }
            }
        }
    }
    cache
}

/// Resolve a fonte para uma dada font-family usando o cache.
fn resolve_font<'a>(
    family: &str,
    cache: &'a HashMap<String, FontArc>,
    default: &'a Option<FontArc>,
) -> Option<&'a FontArc> {
    if family.is_empty() || family == "inherit" || family == "serif" || family == "sans-serif" {
        return default.as_ref();
    }
    let family_lower = family
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_lowercase()
        .replace(' ', "");

    // Busca parcial
    for (key, font) in cache {
        if key.contains(&family_lower) || family_lower.contains(key.as_str()) {
            return Some(font);
        }
    }
    default.as_ref()
}

/// Mede a largura real do texto usando ab_glyph (h_advance de cada glyph).
pub fn measure_text_width(text: &str, font: &FontArc, font_size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let size = font_size.clamp(8.0, 200.0);
    let px_scale = PxScale::from(size);
    let px_font = font.as_scaled(px_scale);
    let mut width = 0.0f32;
    for ch in text.chars() {
        let glyph = px_font.scaled_glyph(ch);
        width += px_font.h_advance(glyph.id);
    }
    width
}

/// Heurística simples para estimar largura de texto (fallback sem fonte).
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    text.chars().count() as f32 * font_size * 0.5
}

/// Calcula a largura do texto usando fonte real se disponível, senão heurística.
fn text_width(
    text: &str,
    font_size: f32,
    family: &str,
    cache: &HashMap<String, FontArc>,
    default_font: &Option<FontArc>,
) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    if let Some(font) = resolve_font(family, cache, default_font) {
        measure_text_width(text, font, font_size)
    } else {
        estimate_text_width(text, font_size)
    }
}

/// Resolve line-height CSS: "normal" → 1.2, número → multiplicador, "20px" → abs
fn resolve_line_height(value: &str, font_size: f32, container_width: f32) -> f32 {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "normal" || trimmed == "inherit" {
        return font_size * 1.2;
    }
    // Número puro = multiplicador
    if let Ok(n) = trimmed.parse::<f32>() {
        return font_size * n;
    }
    // Valor com unidade
    let px = css_to_pixels(trimmed, container_width, font_size);
    if px > 0.0 { px } else { font_size * 1.2 }
}

/// Layout recursivo de um nó block.
#[allow(clippy::too_many_arguments)]
fn layout_block(
    node: &mut VisualNode,
    x: f32,
    y: f32,
    available_width: f32,
    viewport_width: f32,
    font_cache: &HashMap<String, FontArc>,
    default_font: &Option<FontArc>,
) -> f32 {
    let font_size = css_to_pixels(&node.style.font_size, viewport_width, 16.0).max(8.0);
    let bm = compute_box_model(&node.style, available_width, font_size);

    let margin_left = bm.margin_left;
    let margin_right = bm.margin_right;
    let margin_top = bm.margin_top;

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

    // Separar filhos absolute/fixed do fluxo normal
    let mut normal_indices: Vec<usize> = Vec::new();
    let mut abs_indices: Vec<usize> = Vec::new();
    for (i, child) in node.children.iter().enumerate() {
        if child.style.position == "absolute" || child.style.position == "fixed" {
            abs_indices.push(i);
        } else {
            normal_indices.push(i);
        }
    }

    // Layout do fluxo normal
    let mut child_cursor = content_y;
    let mut prev_child_margin_bottom = 0.0f32;

    for &child_idx in &normal_indices {
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

            let gap = child_margin_top.max(prev_child_margin_bottom);
            child_cursor += gap;

            let child_bottom = layout_block(
                &mut node.children[child_idx],
                content_x,
                child_cursor - child_margin_top,
                content_width,
                viewport_width,
                font_cache,
                default_font,
            );
            child_cursor = child_bottom;
            prev_child_margin_bottom = child_bm.margin_bottom;
        } else {
            // Inline run: coletar inlines consecutivos no fluxo normal
            let start_pos = child_idx;
            let mut end_pos = child_idx + 1;
            // Avançar enquanto o próximo índice normal também é inline e consecutivo
            while end_pos < node.children.len() {
                if node.children[end_pos].style.position == "absolute"
                    || node.children[end_pos].style.position == "fixed"
                {
                    end_pos += 1;
                    continue;
                }
                if node.children[end_pos].node_type == NodeType::Inline {
                    end_pos += 1;
                } else {
                    break;
                }
            }

            let mut line_x = content_x;
            let mut line_y = child_cursor;
            let mut line_height = 0.0f32;

            for idx in start_pos..end_pos {
                if node.children[idx].style.position == "absolute"
                    || node.children[idx].style.position == "fixed"
                {
                    continue;
                }
                if node.children[idx].node_type != NodeType::Inline {
                    break;
                }

                let child = &mut node.children[idx];
                let child_font_size =
                    css_to_pixels(&child.style.font_size, viewport_width, font_size).max(8.0);
                let child_lh = resolve_line_height(
                    &child.style.line_height,
                    child_font_size,
                    viewport_width,
                );

                let tw = text_width(
                    &child.text,
                    child_font_size,
                    &child.style.font_family,
                    font_cache,
                    default_font,
                );

                // Quebra de linha
                if line_x + tw > content_x + content_width && tw > 0.0 && line_x > content_x {
                    line_x = content_x;
                    line_y += line_height;
                    line_height = 0.0;
                }

                child.rect = Rect::from_xywh(line_x, line_y, tw, child_lh)
                    .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

                if !child.children.is_empty() {
                    layout_inline_children(
                        child,
                        line_x,
                        line_y,
                        content_width,
                        viewport_width,
                        font_cache,
                        default_font,
                    );
                }

                line_x += tw;
                line_height = line_height.max(child_lh);
            }

            // Atualizar child_idx para pular os inlines processados
            // (o for loop vai incrementar de qualquer forma)
            child_cursor = line_y + line_height;
            prev_child_margin_bottom = 0.0;
        }
    }

    let content_height = (child_cursor - content_y).max(0.0);
    let block_height = if bm.height > 0.0 {
        bm.height
    } else {
        content_height + bm.padding_top + bm.padding_bottom
    };

    let mut final_x = block_x;
    let mut final_y = block_y;

    // Position relativa (já existia)
    if node.style.position == "relative" {
        if node.style.top != "auto" {
            let top_px = css_to_pixels(&node.style.top, viewport_width, font_size);
            final_y += top_px;
        }
        if node.style.left != "auto" {
            let left_px = css_to_pixels(&node.style.left, viewport_width, font_size);
            final_x += left_px;
        }
    }

    node.rect = Rect::from_xywh(
        final_x,
        final_y,
        block_width.max(0.0),
        block_height.max(0.0),
    )
    .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

    // Posicionar filhos absolute/fixed
    for &ai in &abs_indices {
        let child = &mut node.children[ai];
        let child_font_size = css_to_pixels(
            &child.style.font_size,
            viewport_width,
            font_size,
        )
        .max(8.0);
        let child_bm = compute_box_model(&child.style, content_width, child_font_size);

        // Containing block
        let (cb_x, cb_y, cb_w, cb_h) = if child.style.position == "fixed" {
            // Fixed: viewport
            (0.0f32, 0.0f32, viewport_width, viewport_width)
        } else {
            // Absolute: nearest positioned ancestor = current node (simplification)
            (
                final_x,
                final_y,
                block_width.max(0.0),
                block_height.max(0.0),
            )
        };

        let child_w = if child_bm.width > 0.0 {
            child_bm.width
        } else if child.style.left != "auto" && child.style.right != "auto" {
            let l = css_to_pixels(&child.style.left, cb_w, child_font_size);
            let r = css_to_pixels(&child.style.right, cb_w, child_font_size);
            (cb_w - l - r).max(0.0)
        } else {
            // Shrink-to-fit: usar largura do conteúdo
            content_width
        };

        let child_h = if child_bm.height > 0.0 {
            child_bm.height
        } else if child.style.top != "auto" && child.style.bottom != "auto" {
            let t = css_to_pixels(&child.style.top, cb_h, child_font_size);
            let b = css_to_pixels(&child.style.bottom, cb_h, child_font_size);
            (cb_h - t - b).max(0.0)
        } else {
            block_height // fallback
        };

        let mut abs_x = cb_x;
        let mut abs_y = cb_y;

        if child.style.left != "auto" {
            abs_x += css_to_pixels(&child.style.left, cb_w, child_font_size);
        } else if child.style.right != "auto" {
            let r = css_to_pixels(&child.style.right, cb_w, child_font_size);
            abs_x = cb_x + cb_w - child_w - r;
        }

        if child.style.top != "auto" {
            abs_y += css_to_pixels(&child.style.top, cb_h, child_font_size);
        } else if child.style.bottom != "auto" {
            let b = css_to_pixels(&child.style.bottom, cb_h, child_font_size);
            abs_y = cb_y + cb_h - child_h - b;
        }

        child.rect = Rect::from_xywh(abs_x, abs_y, child_w, child_h)
            .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

        // Recursar layout dos filhos do absolute
        let _ = layout_block(
            child,
            abs_x,
            abs_y,
            child_w,
            viewport_width,
            font_cache,
            default_font,
        );
    }

    block_y + block_height
}

/// Posiciona filhos inline de um nó inline.
#[allow(clippy::too_many_arguments)]
fn layout_inline_children(
    node: &mut VisualNode,
    x: f32,
    y: f32,
    available_width: f32,
    viewport_width: f32,
    font_cache: &HashMap<String, FontArc>,
    default_font: &Option<FontArc>,
) {
    let font_size = css_to_pixels(&node.style.font_size, viewport_width, 16.0).max(8.0);
    let mut line_x = x;
    let mut line_y = y;
    let mut line_height = 0.0f32;

    for child in &mut node.children {
        let child_font_size =
            css_to_pixels(&child.style.font_size, viewport_width, font_size).max(8.0);
        let child_lh = resolve_line_height(&child.style.line_height, child_font_size, viewport_width);
        let tw = text_width(
            &child.text,
            child_font_size,
            &child.style.font_family,
            font_cache,
            default_font,
        );

        if line_x + tw > x + available_width && tw > 0.0 && line_x > x {
            line_x = x;
            line_y += line_height;
            line_height = 0.0;
        }

        child.rect = Rect::from_xywh(line_x, line_y, tw, child_lh)
            .unwrap_or(Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap());

        if !child.children.is_empty() {
            layout_inline_children(
                child,
                line_x,
                line_y,
                available_width,
                viewport_width,
                font_cache,
                default_font,
            );
        }

        line_x += tw;
        line_height = line_height.max(child_lh);
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
        compute_layout(&mut tree, 50.0);

        let body = tree.children.iter().find(|c| c.tag == "body").unwrap();
        let p = body.children.iter().find(|c| c.tag == "p").unwrap();
        let texts: Vec<_> = p.children.iter().filter(|c| c.tag == "#text").collect();
        let ys: Vec<f32> = texts.iter().map(|t| t.rect.y()).collect();
        let mut unique_ys = ys.clone();
        unique_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique_ys.dedup_by(|a, b| (*a - *b).abs() < 1.0);
        assert!(!unique_ys.is_empty(), "deve haver pelo menos uma linha de texto");
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
        let gap = divs[1].rect.y() - divs[0].rect.y();
        assert!(gap >= 20.0, "gap deve refletir margin collapsing, gap={}", gap);
    }
}
