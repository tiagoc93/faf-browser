use crate::css::color::Color;
use crate::css::layout::css_to_pixels;
use crate::css::selector::ElementMatch;
use crate::css::style::ComputedStyle;
use crate::dom::{HtmlDocument, QueryResult};
use crate::render::layout::compute_layout;
use crate::render::tree::VisualNode;
use crate::render::tree::build_layout_tree;
use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tiny_skia::{ColorU8, Paint, Pixmap, Rect, Transform};

/// Configuração da renderização
pub struct ScreenshotConfig {
    pub width: u32,
    pub height: u32,
}

/// Caminhos de fontes TTF conhecidos no sistema (busca em ordem)
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

    // 2. Construir árvore visual e calcular layout
    let mut tree = build_layout_tree(doc, &computed);
    compute_layout(&mut tree, config.width as f32);

    // 3. Determinar dimensões do canvas
    let width = config.width;
    let height = if config.height > 0 {
        config.height
    } else {
        let doc_bottom = tree.rect.y() + tree.rect.height();
        doc_bottom.ceil().max(100.0) as u32
    };

    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("Falha ao criar pixmap {}x{}", width, height))?;

    // 4. Fundo branco
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, width as f32, height as f32).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );

    // 5. Load font padrão (ab_glyph)
    let default_font = load_font_simple("");
    if default_font.is_some() {
        log::info!("Fonte padrão carregada com sucesso");
    } else {
        log::warn!("Fonte padrão NÃO carregada - fallback para retângulos");
    }

    // 6. Renderizar árvore visual (DFS)
    let mut text_count = 0;
    let mut font_ok_count = 0;
    let total_nodes = count_nodes(&tree);
    let mut image_cache: HashMap<String, image::DynamicImage> = HashMap::new();

    render_node(
        &mut pixmap,
        &tree,
        &default_font,
        width,
        height,
        &mut text_count,
        &mut font_ok_count,
        &mut image_cache,
    );

    log::info!(
        "Renderização: {} total nós, {} com texto, {} com fonte OK",
        total_nodes,
        text_count,
        font_ok_count
    );

    // 7. Salvar PNG
    pixmap
        .save_png(Path::new(output_path))
        .map_err(|e| anyhow::anyhow!("Falha ao salvar PNG: {:?}", e))?;

    Ok(())
}

/// Renderiza um nó visual e seus filhos recursivamente.
#[allow(clippy::too_many_arguments)]
fn render_node(
    pixmap: &mut Pixmap,
    node: &VisualNode,
    default_font: &Option<FontArc>,
    pm_width: u32,
    pm_height: u32,
    text_count: &mut usize,
    font_ok_count: &mut usize,
    image_cache: &mut HashMap<String, image::DynamicImage>,
) {
    let x = node.rect.x();
    let y = node.rect.y();
    let w = node.rect.width();
    let h = node.rect.height();

    // Limitar ao bounds do pixmap
    let draw_w = w.min(pm_width as f32 - x);
    let draw_h = h.min(pm_height as f32 - y);

    if draw_w > 0.0 && draw_h > 0.0 && y < pm_height as f32 && x < pm_width as f32 {
        // Desenhar background-color
        if !node.style.background_color.is_empty() && node.style.background_color != "transparent"
            && let Some(color) = crate::css::color::parse_color(&node.style.background_color) {
                let mut bg_paint = Paint::default();
                bg_paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
                if let Some(rect) = Rect::from_xywh(x, y, draw_w, draw_h) {
                    pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
                }
            }

        // Desenhar bordas
        let font_size = css_to_pixels(&node.style.font_size, pm_width as f32, 16.0).max(8.0);
        let border_top_w = css_to_pixels(&node.style.border_top_width, pm_width as f32, font_size);
        let border_right_w = css_to_pixels(&node.style.border_right_width, pm_width as f32, font_size);
        let border_bottom_w = css_to_pixels(&node.style.border_bottom_width, pm_width as f32, font_size);
        let border_left_w = css_to_pixels(&node.style.border_left_width, pm_width as f32, font_size);

        if border_top_w > 0.0 && node.style.border_top_style != "none" && node.style.border_top_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_top_color) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
                if let Some(rect) = Rect::from_xywh(x, y, draw_w, border_top_w) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        if border_bottom_w > 0.0 && node.style.border_bottom_style != "none" && node.style.border_bottom_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_bottom_color) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
                if let Some(rect) = Rect::from_xywh(x, y + draw_h - border_bottom_w, draw_w, border_bottom_w) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        if border_left_w > 0.0 && node.style.border_left_style != "none" && node.style.border_left_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_left_color) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
                if let Some(rect) = Rect::from_xywh(x, y, border_left_w, draw_h) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        if border_right_w > 0.0 && node.style.border_right_style != "none" && node.style.border_right_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_right_color) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
                if let Some(rect) = Rect::from_xywh(x + draw_w - border_right_w, y, border_right_w, draw_h) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }

        // Desenhar imagens <img>
        if node.tag == "img" {
            if let Some(src) = node.attributes.get("src") {
                draw_image(pixmap, node, src, image_cache, pm_width, pm_height);
            } else {
                // Fallback: placeholder cinza
                draw_image_placeholder(pixmap, x, y, draw_w, draw_h);
            }
        }

        // Desenhar texto usando ab_glyph
        if !node.text.is_empty() {
            *text_count += 1;
            let fg = if !node.style.color.is_empty() && node.style.color != "inherit" {
                &node.style.color
            } else {
                "#000000"
            };
            if let Some(color) = crate::css::color::parse_color(fg) {
                let font_size =
                    css_to_pixels(&node.style.font_size, pm_width as f32, 16.0).max(8.0);
                let font = if !node.style.font_family.is_empty() {
                    let family = node
                        .style
                        .font_family
                        .split(',')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    load_font_simple(family).or_else(|| default_font.clone())
                } else {
                    default_font.clone()
                };

                if let Some(ref font) = font {
                    *font_ok_count += 1;
                    log::info!(
                        "Renderizando texto [{:?}]: \"{}\" em x={}, y={}, size={}, cor={:?}",
                        node.tag,
                        &node.text[..node.text.len().min(30)],
                        x,
                        y,
                        font_size,
                        color
                    );
                    render_text_ab(&mut *pixmap, &node.text, x, y, font_size, color, font);
                } else {
                    // Fallback: desenhar retângulo placeholder se fonte não encontrada
                    log::warn!(
                        "Fonte não encontrada p/ {:?} (tag={}, texto=\"{}\") - fallback retângulo",
                        node.tag,
                        node.tag,
                        &node.text[..node.text.len().min(30)]
                    );
                    let mut fg_paint = Paint::default();
                    fg_paint.set_color_rgba8(color.r, color.g, color.b, 255);
                    let text_h = font_size.min(draw_h * 0.6);
                    let text_w = (node.text.len() as f32 * font_size * 0.5).min(draw_w);
                    let text_y = y + (draw_h - text_h) * 0.5;
                    if let Some(rect) = Rect::from_xywh(x + 4.0, text_y, text_w, text_h) {
                        pixmap.fill_rect(rect, &fg_paint, Transform::identity(), None);
                    }
                }
            }
        }
    }

    // Renderizar filhos
    for child in &node.children {
        render_node(
            pixmap,
            child,
            default_font,
            pm_width,
            pm_height,
            text_count,
            font_ok_count,
            image_cache,
        );
    }
}

/// Conta o total de nós na árvore visual.
fn count_nodes(node: &VisualNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// Carrega uma fonte TTF do sistema pelo nome (simplificado) ou retorna a fonte padrão.
fn load_font_simple(family: &str) -> Option<FontArc> {
    // Map de nomes de família para caminhos de fonte
    let family_lower = family.to_lowercase();

    // Tenta achar um caminho que contenha o nome da família
    for &path in FONT_PATHS {
        // Extrai o nome do arquivo sem extensão
        if let Some(file_name) = path.rsplit('/').next() {
            let file_stem = file_name.rsplit('.').next_back().unwrap_or(file_name);
            let file_lower = file_stem.to_lowercase();
            if (file_lower.contains(&family_lower.replace(' ', "")) || family_lower.is_empty())
                && let Ok(data) = fs::read(path)
                    && let Ok(font) = FontArc::try_from_vec(data) {
                        return Some(font);
                    }
        }
    }

    // Fallback: tenta todos os paths em ordem
    for &path in FONT_PATHS {
        if let Ok(data) = fs::read(path)
            && let Ok(font) = FontArc::try_from_vec(data) {
                return Some(font);
            }
    }

    None
}

/// Renderiza texto usando ab_glyph.
fn render_text_ab(
    pixmap: &mut Pixmap,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    color: Color,
    font: &FontArc,
) {
    if text.is_empty() {
        return;
    }

    let size = font_size.clamp(8.0, 200.0);
    let px_scale = PxScale::from(size);
    let px_font = font.as_scaled(px_scale);

    let pm_width = pixmap.width();
    let pm_height = pixmap.height();
    let pixels = pixmap.pixels_mut();

    // baseline_y é a linha base onde o texto se alinha
    let baseline_y = y + size * 0.85;
    let mut cursor_x = x;

    let mut chars_total = 0u32;
    let mut pixels_drawn = 0u32;

    for ch in text.chars() {
        // scaled_glyph já retorna um Glyph com glyph_id setado
        let mut glyph = px_font.scaled_glyph(ch);

        // Pega o avanço horizontal em pixels
        let advance = px_font.h_advance(glyph.id);

        // Posiciona o glyph
        glyph.position = ab_glyph::point(cursor_x, baseline_y);

        // Obtém outline para rasterizar
        let outline = match px_font.outline_glyph(glyph) {
            Some(o) => o,
            None => {
                // Sem outline (espaço, controle, etc) — avança cursor
                cursor_x += advance;
                continue;
            }
        };

        chars_total += 1;

        // draw() fornece coordenadas RELATIVAS ao bounding box do glyph
        let b = outline.px_bounds();

        outline.draw(|rx: u32, ry: u32, cover: f32| {
            if cover <= 0.0 {
                return;
            }

            let px = rx + b.min.x as u32;
            let py = ry + b.min.y as u32;

            if px >= pm_width || py >= pm_height {
                return;
            }

            let idx = (py * pm_width + px) as usize;
            let bg_premul = pixels[idx];
            let bg = bg_premul.demultiply();

            let text_alpha = cover * color.a;
            let inv_alpha = 1.0 - text_alpha;

            let r = (color.r as f32 * text_alpha + bg.red() as f32 * inv_alpha) as u8;
            let g = (color.g as f32 * text_alpha + bg.green() as f32 * inv_alpha) as u8;
            let b = (color.b as f32 * text_alpha + bg.blue() as f32 * inv_alpha) as u8;
            let a = (255.0 * text_alpha + bg.alpha() as f32 * inv_alpha) as u8;

            let blended = ColorU8::from_rgba(r, g, b, a);
            pixels[idx] = blended.premultiply();
            pixels_drawn += 1;
        });

        cursor_x += advance;
    }

    if chars_total > 0 && pixels_drawn == 0 {
        log::warn!(
            "render_text_ab: {} chars OK, mas 0 pixels desenhados! y={}, baseline_y={}, size={}",
            chars_total,
            y,
            baseline_y,
            size
        );
    }
    if pixels_drawn > 0 {
        log::info!(
            "render_text_ab: {} chars, {} pixels p/ \"{}\"",
            chars_total,
            pixels_drawn,
            &text[..text.len().min(20)]
        );
    }
}

/// Desenha uma imagem <img> no pixmap, carregando via HTTP se necessário.
fn draw_image(
    pixmap: &mut Pixmap,
    node: &VisualNode,
    src: &str,
    image_cache: &mut HashMap<String, image::DynamicImage>,
    _pm_width: u32,
    _pm_height: u32,
) {
    let x = node.rect.x();
    let y = node.rect.y();
    let w = node.rect.width();
    let h = node.rect.height();

    // Verificar cache
    let img = if let Some(cached) = image_cache.get(src) {
        Some(cached.clone())
    } else {
        // Tentar carregar via HTTP (blocking)
        match reqwest::blocking::get(src) {
            Ok(response) => {
                if response.status().is_success() {
                    match response.bytes() {
                        Ok(bytes) => {
                            match image::load_from_memory(&bytes) {
                                Ok(dynamic_img) => {
                                    image_cache.insert(src.to_string(), dynamic_img.clone());
                                    Some(dynamic_img)
                                }
                                Err(e) => {
                                    log::warn!("Falha ao decodificar imagem {}: {}", src, e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Falha ao ler bytes da imagem {}: {}", src, e);
                            None
                        }
                    }
                } else {
                    log::warn!("HTTP {} ao carregar imagem {}", response.status(), src);
                    None
                }
            }
            Err(e) => {
                log::warn!("Falha ao requisitar imagem {}: {}", src, e);
                None
            }
        }
    };

    if let Some(img) = img {
        let resized = img.resize(w as u32, h as u32, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let img_width = rgba.width();
        let img_height = rgba.height();
        let pm_w = pixmap.width();
        let pm_h = pixmap.height();
        let pixels = pixmap.pixels_mut();

        for iy in 0..img_height {
            for ix in 0..img_width {
                let px = (x as u32 + ix).min(pm_w - 1);
                let py = (y as u32 + iy).min(pm_h - 1);
                if px >= pm_w || py >= pm_h {
                    continue;
                }
                let pixel = rgba.get_pixel(ix, iy);
                let idx = (py * pm_w + px) as usize;
                let bg = pixels[idx].demultiply();
                let alpha = pixel[3] as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;
                let r = (pixel[0] as f32 * alpha + bg.red() as f32 * inv_alpha) as u8;
                let g = (pixel[1] as f32 * alpha + bg.green() as f32 * inv_alpha) as u8;
                let b = (pixel[2] as f32 * alpha + bg.blue() as f32 * inv_alpha) as u8;
                let a = (255.0 * alpha + bg.alpha() as f32 * inv_alpha) as u8;
                pixels[idx] = ColorU8::from_rgba(r, g, b, a).premultiply();
            }
        }
    } else {
        draw_image_placeholder(pixmap, x, y, w, h);
    }
}

/// Desenha um placeholder cinza para imagens que não carregaram.
fn draw_image_placeholder(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32) {
    let draw_w = w.max(0.0);
    let draw_h = h.max(0.0);
    if draw_w <= 0.0 || draw_h <= 0.0 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_color_rgba8(200, 200, 200, 255);
    if let Some(rect) = Rect::from_xywh(x, y, draw_w, draw_h) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

#[allow(dead_code)]
fn find_style_for_element(
    element: &QueryResult,
    computed: &[(ElementMatch, ComputedStyle)],
) -> ComputedStyle {
    for (em, style) in computed {
        if em.tag == element.tag && em.id == element.id && em.classes == element.classes {
            return style.clone();
        }
    }
    // Fallback: try matching only by tag
    for (em, style) in computed {
        if em.tag == element.tag {
            return style.clone();
        }
    }
    ComputedStyle::default()
}

/// Calcula a altura total do documento.
#[allow(dead_code)]
fn compute_document_height(
    _doc: &HtmlDocument,
    _computed: &[(ElementMatch, ComputedStyle)],
    _width: u32,
) -> u32 {
    // Placeholder: altura fixa para MVP
    800
}
