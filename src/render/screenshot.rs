use crate::css::layout::{compute_box_model, css_to_pixels};
use crate::css::selector::ElementMatch;
use crate::css::style::ComputedStyle;
use crate::dom::{HtmlDocument, QueryResult};
use std::collections::HashSet;
use std::path::Path;
use tiny_skia::{Paint, Pixmap, Rect, Transform};

/// Configuração da renderização
pub struct ScreenshotConfig {
    pub width: u32,
    pub height: u32,
}

/// Renderiza um HtmlDocument em um PNG, salvando no caminho especificado.
pub fn render_to_image(
    doc: &HtmlDocument,
    config: &ScreenshotConfig,
    output_path: &str,
) -> anyhow::Result<()> {
    // 1. Parse CSS da página (inline <style> tags)
    let css_text = doc.extract_css().unwrap_or_default();
    let stylesheet = crate::css::parser::parse_css(&css_text)
        .unwrap_or(crate::css::parser::Stylesheet { rules: Vec::new() });
    let computed = crate::css::style::compute_styles(doc, &stylesheet);

    // 2. Criar canvas
    let width = config.width;
    let height = if config.height > 0 {
        config.height
    } else {
        compute_document_height(doc, &computed, width)
    };

    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("Falha ao criar pixmap {}x{}", width, height))?;

    // 3. Fundo branco
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, width as f32, height as f32).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );

    // 4. Renderizar elementos visíveis do body (flat list)
    let elements = doc.query("*").unwrap_or_default();
    let mut y = 0.0f32;

    // Tags estruturais/não-visíveis a pular
    let skip_tags: HashSet<&str> = ["html", "head", "script", "style", "meta", "link", "title"]
        .iter()
        .copied()
        .collect();

    for element in &elements {
        if skip_tags.contains(element.tag.as_str()) {
            continue;
        }

        let style = find_style_for_element(element, &computed);

        // Pular display: none
        if style.display == "none" {
            continue;
        }

        let font_size = css_to_pixels(&style.font_size, width as f32, 16.0).max(8.0);
        let box_model = compute_box_model(&style, width as f32, font_size);

        let x = box_model.margin_left;
        y += box_model.margin_top;

        let w = if box_model.width > 0.0 {
            box_model.width
        } else {
            (width as f32) - x - box_model.margin_right
        };
        let h = if box_model.height > 0.0 {
            box_model.height
        } else {
            font_size * 1.5
        };

        // Limitar ao bounds do pixmap
        let draw_w = w.min(width as f32 - x);
        let draw_h = h.min(height as f32 - y);
        if draw_w <= 0.0 || draw_h <= 0.0 || y >= height as f32 {
            y += h + box_model.margin_bottom;
            continue;
        }

        // Desenhar background-color
        if !style.background_color.is_empty() && style.background_color != "transparent" {
            if let Some(color) = crate::css::color::parse_color(&style.background_color) {
                let mut bg_paint = Paint::default();
                bg_paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
                if let Some(rect) = Rect::from_xywh(x, y, draw_w, draw_h) {
                    pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
                }
            }
        }

        // Desenhar texto (placeholder visual)
        if !element.text.is_empty() {
            let fg = if !style.color.is_empty() && style.color != "inherit" {
                &style.color
            } else {
                "#000000"
            };
            if let Some(color) = crate::css::color::parse_color(fg) {
                let mut fg_paint = Paint::default();
                fg_paint.set_color_rgba8(color.r, color.g, color.b, 255);
                let text_h = font_size.min(draw_h * 0.6);
                let text_w = (element.text.len() as f32 * font_size * 0.5).min(draw_w);
                let text_y = y + (draw_h - text_h) * 0.5;
                if let Some(rect) = Rect::from_xywh(x + 4.0, text_y, text_w, text_h) {
                    pixmap.fill_rect(rect, &fg_paint, Transform::identity(), None);
                }
            }
        }

        y += h + box_model.margin_bottom;
    }

    // 5. Salvar PNG
    pixmap
        .save_png(Path::new(output_path))
        .map_err(|e| anyhow::anyhow!("Falha ao salvar PNG: {:?}", e))?;

    Ok(())
}

fn find_style_for_element(
    element: &QueryResult,
    computed: &[(ElementMatch, ComputedStyle)],
) -> ComputedStyle {
    for (em, style) in computed {
        if em.tag == element.tag
            && em.id == element.id
            && em.classes == element.classes
            && em.text == element.text
        {
            return style.clone();
        }
    }
    ComputedStyle::default()
}

/// Calcula a altura total do documento.
fn compute_document_height(
    _doc: &HtmlDocument,
    _computed: &[(ElementMatch, ComputedStyle)],
    _width: u32,
) -> u32 {
    // Placeholder: altura fixa para MVP
    800
}
