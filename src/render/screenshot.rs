use crate::css::color::Color;
use crate::css::layout::css_to_pixels;
use crate::css::selector::ElementMatch;
use crate::css::style::ComputedStyle;
use crate::dom::{HtmlDocument, QueryResult};
use crate::render::layout::compute_layout;
use crate::render::tree::build_layout_tree;
use crate::render::tree::VisualNode;
use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;
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

/// Carrega todas as fontes do sistema em cache. Chame uma vez e reutilize.
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

/// Resolve fonte por família usando cache.
fn resolve_font_cached<'a>(
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

    for (key, font) in cache {
        if key.contains(&family_lower) || family_lower.contains(key.as_str()) {
            return Some(font);
        }
    }
    default.as_ref()
}

/// Extrai TODOS os CSS da página: inline `<style>` + externo `<link rel="stylesheet">`.
/// Baixa CSS externo usando reqwest::blocking se base_url for fornecido.
fn extract_all_css(doc: &HtmlDocument, base_url: Option<&str>) -> String {
    let mut css_parts = Vec::new();

    // 1. CSS inline (<style>)
    if let Some(inline_css) = doc.extract_css() {
        css_parts.push(inline_css);
    }

    // 2. CSS externo (<link rel="stylesheet">)
    if let Some(base) = base_url {
        let scraper_html = doc.scraper_html();
        if let Ok(link_selector) = scraper::Selector::parse("link[rel~=stylesheet]") {
            for el in scraper_html.select(&link_selector) {
                if let Some(href) = el.value().attr("href") {
                    let resolved = if href.starts_with("http://") || href.starts_with("https://") {
                        href.to_string()
                    } else if let Ok(base_url) = url::Url::parse(base) {
                        match base_url.join(href) {
                            Ok(u) => u.to_string(),
                            Err(_) => continue,
                        }
                    } else {
                        format!("{}/{}", base.trim_end_matches('/'), href.trim_start_matches('/'))
                    };
                    log::info!("Baixando CSS externo: {}", resolved);
                    match reqwest::blocking::get(&resolved) {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                if let Ok(text) = resp.text() {
                                    log::info!("CSS externo baixado: {} bytes de {}", text.len(), resolved);
                                    css_parts.push(text);
                                }
                            } else {
                                log::warn!("HTTP {} ao baixar CSS: {}", resp.status(), resolved);
                            }
                        }
                        Err(e) => log::warn!("Erro ao baixar CSS {}: {}", resolved, e),
                    }
                }
            }
        }
    }

    css_parts.join("\n")
}

/// Resolve uma URL relativa contra uma base URL.
fn resolve_url(src: &str, base_url: Option<&str>) -> String {
    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
        return src.to_string();
    }
    if let Some(base) = base_url {
        if let Ok(base_url) = url::Url::parse(base) {
            if let Ok(resolved) = base_url.join(src) {
                return resolved.to_string();
            }
        }
        // Fallback manual
        return format!("{}/{}", base.trim_end_matches('/'), src.trim_start_matches('/'));
    }
    src.to_string()
}

/// Renderiza um HtmlDocument em um PNG, salvando no caminho especificado.
///
/// `base_url` é usada para:
/// 1. Resolver URLs relativas de `<link rel="stylesheet">` (baixar CSS externo)
/// 2. Resolver URLs relativas de `<img src="...">`
pub fn render_to_image(
    doc: &HtmlDocument,
    config: &ScreenshotConfig,
    output_path: &str,
) -> anyhow::Result<()> {
    render_to_image_with_base(doc, config, output_path, None)
}

/// Versão com base URL para resolver recursos externos.
pub fn render_to_image_with_base(
    doc: &HtmlDocument,
    config: &ScreenshotConfig,
    output_path: &str,
    base_url: Option<&str>,
) -> anyhow::Result<()> {
    let total_start = Instant::now();

    // 1. Extrair CSS inline + baixar CSS externo (<link rel="stylesheet">)
    let t1 = Instant::now();
    let css_text = extract_all_css(doc, base_url);
    log::info!("CSS extraido (inline + externo): {} bytes em {:?}", css_text.len(), t1.elapsed());
    let stylesheet = crate::css::parser::parse_css(&css_text)
        .unwrap_or(crate::css::parser::Stylesheet { rules: Vec::new() });
    log::info!("CSS parseado: {} regras", stylesheet.rules.len());
    let computed = crate::css::style::compute_styles(doc, &stylesheet);
    log::info!("Estilos computados: {} elementos", computed.len());
    log::info!("CSS parse + compute: {:?}", t1.elapsed());

    // 2. Baixar dimensões de imagens ANTES da árvore visual
    let t2 = Instant::now();
    let image_dims = crate::render::image_dimensions::fetch_all_image_dimensions(doc, base_url);
    log::info!("Dimensões de imagens buscadas: {} imagens", image_dims.len());

    // NOVO (T078): Baixar TODAS as imagens (pixels) para o cache global antes do layout
    if let Some(base) = base_url {
        let scraper_html = doc.scraper_html();
        if let Ok(img_selector) = scraper::Selector::parse("img[src]") {
            let img_urls: Vec<String> = scraper_html
                .select(&img_selector)
                .filter_map(|el| el.value().attr("src").map(|s| s.to_string()))
                .collect();
            log::info!("Baixando {} imagens...", img_urls.len());
            let mut cache = crate::render::image_cache::image_cache();
            for src in &img_urls {
                let resolved = crate::render::image_cache::resolve_url(src, base);
                let _ = cache.get_or_fetch(&resolved, base);
            }
            log::info!("Cache de imagens preenchido: {} entradas", cache.len());
        }
    }

    // 3. Construir árvore visual e calcular layout
    let t3 = Instant::now();
    let mut tree = build_layout_tree(doc, &computed, Some(&image_dims));
    log::info!("TREE_BEFORE_LAYOUT: img_count={}", count_img_nodes(&tree));
    compute_layout(&mut tree, config.width as f32);
    log::info!("TREE_AFTER_LAYOUT: img_count={}", count_img_nodes(&tree));
    log::info!("Layout tree + compute_layout: {:?}", t2.elapsed());

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

    // 5. Font cache (carregar UMA vez)
    let t3 = Instant::now();
    let font_cache = load_font_cache();
    let default_font = font_cache.values().next().cloned();
    log::info!(
        "Font cache loaded: {} fonts in {:?}",
        font_cache.len(),
        t3.elapsed()
    );

    // 6. Renderizar árvore visual (DFS)
    let t4 = Instant::now();
    let mut text_count = 0;
    let mut font_ok_count = 0;
    let total_nodes = count_nodes(&tree);

    render_node(
        &mut pixmap,
        &tree,
        &font_cache,
        &default_font,
        width,
        height,
        &mut text_count,
        &mut font_ok_count,
        None, // sem clip inicial
        base_url,
    );

    log::info!("Render nodes: {:?}", t4.elapsed());

    log::info!(
        "Renderização: {} total nós, {} com texto, {} com fonte OK | Total: {:?}",
        total_nodes,
        text_count,
        font_ok_count,
        total_start.elapsed()
    );

    // 7. Salvar PNG
    pixmap
        .save_png(Path::new(output_path))
        .map_err(|e| anyhow::anyhow!("Falha ao salvar PNG: {:?}", e))?;

    Ok(())
}

/// Conta nós img na árvore visual.
fn count_img_nodes(node: &VisualNode) -> usize {
    let self_count = if node.tag == "img" { 1 } else { 0 };
    self_count + node.children.iter().map(count_img_nodes).sum::<usize>()
}

/// Preenche um retângulo com segurança, clampando ao pixmap.
fn safe_fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, paint: &Paint) {
    let mut px = x.max(0.0);
    let mut py = y.max(0.0);
    let mut pw = (w - (px - x)).max(0.5);
    let mut ph = (h - (py - y)).max(0.5);
    let max_w = pixmap.width() as f32 - px;
    let max_h = pixmap.height() as f32 - py;
    pw = pw.min(max_w);
    ph = ph.min(max_h);
    if pw <= 0.0 || ph <= 0.0 {
        return;
    }
    if let Some(rect) = Rect::from_xywh(px, py, pw, ph) {
        pixmap.fill_rect(rect, paint, Transform::identity(), None);
    }
}

/// Renderiza um nó visual e seus filhos recursivamente.
#[allow(clippy::too_many_arguments)]
fn render_node(
    pixmap: &mut Pixmap,
    node: &VisualNode,
    font_cache: &HashMap<String, FontArc>,
    default_font: &Option<FontArc>,
    pm_width: u32,
    pm_height: u32,
    text_count: &mut usize,
    font_ok_count: &mut usize,
    clip_rect: Option<Rect>,
    base_url: Option<&str>,
) {
    // Log every call to track render traversal
    log::info!("RENDER_CALL: tag={:?} rect={:?} children={}", node.tag, node.rect, node.children.len());
    let x = node.rect.x();
    let y = node.rect.y();
    let w = node.rect.width();
    let h = node.rect.height();
    log::info!("NODE_DEBUG: tag={:?} rect=Rect{{ x={:.1}, y={:.1}, w={:.1}, h={:.1} }}", 
               node.tag, x, y, w, h);

    // T056: Pular nós completamente fora da viewport (early skip)
    if y + h < 0.0 || y > pm_height as f32 || x + w < 0.0 || x > pm_width as f32 {
        return;
    }

    // Clamp x/y para >= 0 e w/h para caber dentro do pixmap
    let clamped_x = x.max(0.0);
    let clamped_y = y.max(0.0);
    let draw_w = (x + w).min(pm_width as f32) - clamped_x;
    let draw_h = (y + h).min(pm_height as f32) - clamped_y;

    // Aplicar clip se houver (T055 overflow:hidden)
    let effective_clip = if let Some(clip) = clip_rect {
        // Interseção do clip atual com o rect do nó
        let cx1 = clip.x().max(x);
        let cy1 = clip.y().max(y);
        let cx2 = (clip.x() + clip.width()).min(x + w);
        let cy2 = (clip.y() + clip.height()).min(y + h);
        if cx2 <= cx1 || cy2 <= cy1 {
            // Nó fora do clip — não desenha este nó, mas filhos ainda usam o clip do pai
            log::warn!("CLIP_SKIP: tag={:?} x={:.1} w={:.1} y={:.1} h={:.1} cx1={:.1} cx2={:.1} cy1={:.1} cy2={:.1}", 
                       node.tag, x, w, y, h, cx1, cx2, cy1, cy2);
            clip_rect
        } else {
            Rect::from_xywh(cx1, cy1, cx2 - cx1, cy2 - cy1)
        }
    } else {
        None
    };

    if draw_w > 0.0 && draw_h > 0.0 {
        log::info!("RENDER_DRAW: tag={:?} w={:.1} h={:.1}", node.tag, draw_w, draw_h);

        // Desenhar background-color
        if !node.style.background_color.is_empty()
            && node.style.background_color != "transparent"
            && let Some(color) = crate::css::color::parse_color(&node.style.background_color)
        {
            let mut bg_paint = Paint::default();
            bg_paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
            safe_fill_rect(pixmap, clamped_x, clamped_y, draw_w, draw_h, &bg_paint);
        }

        // Desenhar bordas
        let font_size = css_to_pixels(&node.style.font_size, pm_width as f32, 16.0).max(8.0);
        let border_top_w =
            css_to_pixels(&node.style.border_top_width, pm_width as f32, font_size);
        let border_right_w =
            css_to_pixels(&node.style.border_right_width, pm_width as f32, font_size);
        let border_bottom_w =
            css_to_pixels(&node.style.border_bottom_width, pm_width as f32, font_size);
        let border_left_w =
            css_to_pixels(&node.style.border_left_width, pm_width as f32, font_size);

        if border_top_w > 0.0
            && node.style.border_top_style != "none"
            && node.style.border_top_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_top_color)
        {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
            safe_fill_rect(pixmap, clamped_x, clamped_y, draw_w, border_top_w, &paint);
        }
        if border_bottom_w > 0.0
            && node.style.border_bottom_style != "none"
            && node.style.border_bottom_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_bottom_color)
        {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
            safe_fill_rect(pixmap, clamped_x, clamped_y + draw_h - border_bottom_w, draw_w, border_bottom_w, &paint);
        }
        if border_left_w > 0.0
            && node.style.border_left_style != "none"
            && node.style.border_left_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_left_color)
        {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
            safe_fill_rect(pixmap, clamped_x, clamped_y, border_left_w, draw_h, &paint);
        }
        if border_right_w > 0.0
            && node.style.border_right_style != "none"
            && node.style.border_right_style != "hidden"
            && let Some(color) = crate::css::color::parse_color(&node.style.border_right_color)
        {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, (color.a * 255.0) as u8);
            safe_fill_rect(pixmap, clamped_x + draw_w - border_right_w, clamped_y, border_right_w, draw_h, &paint);
        }

        // Desenhar imagens <img>
        if node.tag == "img" {
            log::info!("RENDER_IMG: found img node, rect={:?}, attrs={:?}", node.rect, node.attributes);
            if let Some(src) = node.attributes.get("src") {
                log::info!("RENDER_IMG: calling draw_image for src={}", src);
                draw_image(pixmap, node, src, pm_width, pm_height, base_url);
            } else {
                log::warn!("RENDER_IMG: img node has no src attribute!");
                draw_image_placeholder(pixmap, clamped_x, clamped_y, draw_w, draw_h);
            }
        } else if node.tag == "div" {
            log::debug!("RENDER_DIV: tag=div rect={:?}", node.rect);
        }

        // Desenhar texto usando ab_glyph com text-align
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
                let font = resolve_font_cached(
                    &node.style.font_family,
                    font_cache,
                    default_font,
                )
                .cloned();

                if let Some(ref font) = font {
                    *font_ok_count += 1;

                    // T054: text-align offset
                    let text_w = measure_text_width_render(&node.text, font, font_size);
                    let align_offset = match node.style.text_align.as_str() {
                        "center" => ((draw_w - text_w) * 0.5).max(0.0),
                        "right" => (draw_w - text_w).max(0.0),
                        _ => 0.0,
                    };

                    render_text_ab(
                        &mut *pixmap,
                        &node.text,
                        clamped_x + align_offset,
                        clamped_y,
                        font_size,
                        color,
                        font,
                        effective_clip,
                    );
                } else {
                    // Fallback: desenhar retângulo placeholder
                    let mut fg_paint = Paint::default();
                    fg_paint.set_color_rgba8(color.r, color.g, color.b, 255);
                    let text_h = font_size.min(draw_h * 0.6);
                    let text_w = (node.text.len() as f32 * font_size * 0.5).min(draw_w);
                    let text_y = clamped_y + (draw_h - text_h) * 0.5;
                    if let Some(rect) = Rect::from_xywh(clamped_x + 4.0, text_y, text_w, text_h) {
                        pixmap.fill_rect(rect, &fg_paint, Transform::identity(), None);
                    }
                }
            }
        }
    }

    // Determinar clip para filhos (T055 overflow)
    let child_clip = if node.style.overflow == "hidden" || node.style.overflow == "scroll" || node.style.overflow == "auto" {
        // Criar clip rect = rect do nó
        let node_clip = Rect::from_xywh(x, y, w.max(0.0), h.max(0.0));
        if let Some(existing) = effective_clip {
            // Interseção com clip existente
            if let Some(nc) = node_clip {
                let cx1 = existing.x().max(nc.x());
                let cy1 = existing.y().max(nc.y());
                let cx2 = (existing.x() + existing.width()).min(nc.x() + nc.width());
                let cy2 = (existing.y() + existing.height()).min(nc.y() + nc.height());
                if cx2 > cx1 && cy2 > cy1 {
                    Rect::from_xywh(cx1, cy1, cx2 - cx1, cy2 - cy1).or(Some(existing))
                } else {
                    Some(existing)
                }
            } else {
                Some(existing)
            }
        } else {
            node_clip
        }
    } else {
        effective_clip
    };

    // Renderizar filhos ordenados por z-index
    let mut children_refs: Vec<&VisualNode> = node.children.iter().collect();
    children_refs.sort_by(|a, b| {
        let z_a = a.style.z_index.parse::<i32>().unwrap_or(0);
        let z_b = b.style.z_index.parse::<i32>().unwrap_or(0);
        z_a.cmp(&z_b)
    });
    for child in children_refs {
        render_node(
            pixmap,
            child,
            font_cache,
            default_font,
            pm_width,
            pm_height,
            text_count,
            font_ok_count,
            child_clip,
            base_url,
        );
    }
}

/// Conta o total de nós na árvore visual.
fn count_nodes(node: &VisualNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// Mede largura do texto para alinhamento (ab_glyph real).
fn measure_text_width_render(text: &str, font: &FontArc, font_size: f32) -> f32 {
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

/// Renderiza texto usando ab_glyph com suporte a clip rect.
fn render_text_ab(
    pixmap: &mut Pixmap,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    color: Color,
    font: &FontArc,
    clip: Option<Rect>,
) {
    if text.is_empty() {
        return;
    }

    let size = font_size.clamp(8.0, 200.0);
    let px_scale = PxScale::from(size);
    let px_font = font.as_scaled(px_scale);

    let pm_w = pixmap.width();
    let pm_h = pixmap.height();
    let pixels = pixmap.pixels_mut();

    let baseline_y = y + size * 0.85;
    let mut cursor_x = x;

    let mut chars_total = 0u32;
    let mut pixels_drawn = 0u32;

    for ch in text.chars() {
        let mut glyph = px_font.scaled_glyph(ch);
        let advance = px_font.h_advance(glyph.id);
        glyph.position = ab_glyph::point(cursor_x, baseline_y);

        let outline = match px_font.outline_glyph(glyph) {
            Some(o) => o,
            None => {
                cursor_x += advance;
                continue;
            }
        };

        chars_total += 1;
        let b = outline.px_bounds();

        outline.draw(|rx: u32, ry: u32, cover: f32| {
            if cover <= 0.0 {
                return;
            }

            let px = rx + b.min.x as u32;
            let py = ry + b.min.y as u32;

            if px >= pm_w || py >= pm_h {
                return;
            }

            // Aplicar clip (T055 overflow:hidden)
            if let Some(clip_rect) = clip {
                let fx = px as f32;
                let fy = py as f32;
                if fx < clip_rect.x()
                    || fy < clip_rect.y()
                    || fx >= clip_rect.x() + clip_rect.width()
                    || fy >= clip_rect.y() + clip_rect.height()
                {
                    return;
                }
            }

            let idx = (py * pm_w + px) as usize;
            let bg_premul = pixels[idx];
            let bg = bg_premul.demultiply();

            let text_alpha = cover * color.a;
            let inv_alpha = 1.0 - text_alpha;

            let r = (color.r as f32 * text_alpha + bg.red() as f32 * inv_alpha) as u8;
            let g = (color.g as f32 * text_alpha + bg.green() as f32 * inv_alpha) as u8;
            let b_val = (color.b as f32 * text_alpha + bg.blue() as f32 * inv_alpha) as u8;
            let a = (255.0 * text_alpha + bg.alpha() as f32 * inv_alpha) as u8;

            let blended = ColorU8::from_rgba(r, g, b_val, a);
            pixels[idx] = blended.premultiply();
            pixels_drawn += 1;
        });

        cursor_x += advance;
    }

    if chars_total > 0 && pixels_drawn == 0 {
        log::warn!(
            "render_text_ab: {} chars OK, mas 0 pixels desenhados! y={}, baseline_y={}, size={}",
            chars_total, y, baseline_y, size
        );
    }
}

/// Desenha uma imagem <img> no pixmap a partir do cache global.
fn draw_image(
    pixmap: &mut Pixmap,
    node: &VisualNode,
    src: &str,
    _pm_width: u32,
    _pm_height: u32,
    base_url: Option<&str>,
) {
    let x = node.rect.x();
    let y = node.rect.y();
    let w = node.rect.width();
    let h = node.rect.height();

    log::info!("DRAW_IMAGE called: src={}, x={}, y={}, w={}, h={}", src, x, y, w, h);

    let resolved_src = if let Some(base) = base_url {
        crate::render::image_cache::resolve_url(src, base)
    } else {
        src.to_string()
    };

    let mut cache = crate::render::image_cache::image_cache();
    let decoded = cache
        .get(&resolved_src)
        .or_else(|| cache.get(src));

    log::info!("  -> resolved_src={}, cache_hits={}, decoded={}", 
        resolved_src,
        cache.len(),
        if decoded.is_some() { "Some" } else { "None" });

    if let Some(decoded) = decoded {
        let img_w = decoded.width;
        let img_h = decoded.height;
        let data = &decoded.data;

        // T080: scale proporcional preservando aspect ratio (object-fit: contain)
        let scale_x = if img_w > 0 { w / img_w as f32 } else { 1.0 };
        let scale_y = if img_h > 0 { h / img_h as f32 } else { 1.0 };
        let scale = scale_x.min(scale_y);
        let draw_w = (img_w as f32 * scale).min(w);
        let draw_h = (img_h as f32 * scale).min(h);
        let offset_x = (w - draw_w) * 0.5;
        let offset_y = (h - draw_h) * 0.5;
        let start_x = x + offset_x;
        let start_y = y + offset_y;

        let pm_w = pixmap.width();
        let pm_h = pixmap.height();
        let pixels = pixmap.pixels_mut();

        for iy in 0..draw_h as u32 {
            for ix in 0..draw_w as u32 {
                let src_x = ((ix as f32 / scale) as u32).min(img_w.saturating_sub(1));
                let src_y = ((iy as f32 / scale) as u32).min(img_h.saturating_sub(1));
                let src_idx = ((src_y * img_w + src_x) * 4) as usize;
                let px = (start_x as u32 + ix).min(pm_w.saturating_sub(1));
                let py = (start_y as u32 + iy).min(pm_h.saturating_sub(1));
                if px >= pm_w || py >= pm_h {
                    continue;
                }
                let pixel_r = data[src_idx];
                let pixel_g = data[src_idx + 1];
                let pixel_b = data[src_idx + 2];
                let pixel_a = data[src_idx + 3];
                let idx = (py * pm_w + px) as usize;
                let bg = pixels[idx].demultiply();
                let alpha = pixel_a as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;
                let r = (pixel_r as f32 * alpha + bg.red() as f32 * inv_alpha) as u8;
                let g = (pixel_g as f32 * alpha + bg.green() as f32 * inv_alpha) as u8;
                let b = (pixel_b as f32 * alpha + bg.blue() as f32 * inv_alpha) as u8;
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
    let mut paint = Paint::default();
    paint.set_color_rgba8(200, 200, 200, 255);
    safe_fill_rect(pixmap, x, y, w, h, &paint);
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
    800
}
