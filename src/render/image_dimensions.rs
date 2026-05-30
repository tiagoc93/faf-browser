use std::collections::{HashMap, VecDeque};
use std::io::Cursor;

/// Cache LRU em memória para dimensões de imagens (máx 50 entradas).
pub struct ImageDimensionCache {
    map: HashMap<String, (u32, u32)>,
    order: VecDeque<String>,
    capacity: usize,
}

impl ImageDimensionCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    /// Retorna dimensões se a URL estiver no cache (promove a entrada).
    pub fn get(&mut self, key: &str) -> Option<(u32, u32)> {
        if self.map.contains_key(key) {
            self.order.retain(|k| k != key);
            self.order.push_back(key.to_string());
            self.map.get(key).copied()
        } else {
            None
        }
    }

    /// Insere ou atualiza dimensões no cache.
    pub fn put(&mut self, key: String, value: (u32, u32)) {
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else if self.order.len() >= self.capacity
            && let Some(lru) = self.order.pop_front() {
                self.map.remove(&lru);
            }
        self.order.push_back(key.clone());
        self.map.insert(key, value);
    }

}

/// Extrai dimensões (width, height) dos primeiros bytes de uma imagem.
/// Usa a crate `image` para decodificar apenas o header.
pub fn extract_image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let reader = image::ImageReader::new(Cursor::new(bytes));
    let reader = reader.with_guessed_format().ok()?;
    reader.into_dimensions().ok()
}

/// Faz HTTP GET (com header Range para economizar banda) e extrai dimensões.
/// Usa cache LRU para evitar downloads repetidos.
pub fn fetch_image_dimensions(url: &str, cache: &mut ImageDimensionCache) -> Option<(u32, u32)> {
    if let Some(dim) = cache.get(url) {
        return Some(dim);
    }

    // Tentar Range request primeiro (até 8KB costuma cobrir o header)
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header("Range", "bytes=0-8191")
        .timeout(std::time::Duration::from_secs(10))
        .send();

    let bytes = match resp {
        Ok(r) => {
            if r.status().is_success() || r.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                match r.bytes() {
                    Ok(b) => b.to_vec(),
                    Err(e) => {
                        log::warn!("Falha ao ler bytes de imagem {}: {}", url, e);
                        return None;
                    }
                }
            } else {
                log::warn!("HTTP {} ao buscar dimensões de imagem {}", r.status(), url);
                return None;
            }
        }
        Err(e) => {
            log::warn!("Falha de rede ao buscar dimensões de imagem {}: {}", url, e);
            return None;
        }
    };

    if let Some(dim) = extract_image_dimensions(&bytes) {
        cache.put(url.to_string(), dim);
        Some(dim)
    } else {
        log::warn!("Não foi possível decodificar dimensões da imagem {}", url);
        None
    }
}

/// Resolve uma URL relativa contra uma base URL.
pub fn resolve_url(src: &str, base_url: Option<&str>) -> String {
    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
        return src.to_string();
    }
    if let Some(base) = base_url {
        if let Ok(base_url) = url::Url::parse(base)
            && let Ok(resolved) = base_url.join(src) {
                return resolved.to_string();
            }
        return format!(
            "{}/{}",
            base.trim_end_matches('/'),
            src.trim_start_matches('/')
        );
    }
    src.to_string()
}

/// Varre o DOM em busca de todas as URLs de `<img src="...">` e retorna um mapa
/// URL → (width, height) com as dimensões intrínsecas reais.
pub fn fetch_all_image_dimensions(
    doc: &crate::dom::HtmlDocument,
    base_url: Option<&str>,
) -> HashMap<String, (u32, u32)> {
    let mut cache = ImageDimensionCache::new(50);
    let mut result = HashMap::new();

    let scraper_html = doc.scraper_html();
    if let Ok(selector) = scraper::Selector::parse("img[src]") {
        for el in scraper_html.select(&selector) {
            if let Some(src) = el.value().attr("src") {
                let resolved = resolve_url(src, base_url);
                if let Some(dim) = fetch_image_dimensions(&resolved, &mut cache) {
                    result.insert(resolved.clone(), dim);
                    result.insert(src.to_string(), dim);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = ImageDimensionCache::new(2);
        cache.put("a".to_string(), (10, 20));
        cache.put("b".to_string(), (30, 40));
        assert_eq!(cache.get("a"), Some((10, 20)));
        assert_eq!(cache.get("b"), Some((30, 40)));

        // Adicionar c expulsa a (LRU)
        cache.put("c".to_string(), (50, 60));
        assert!(cache.get("a").is_none());
        assert_eq!(cache.get("b"), Some((30, 40)));
        assert_eq!(cache.get("c"), Some((50, 60)));
    }

    #[test]
    fn test_lru_promotion() {
        let mut cache = ImageDimensionCache::new(2);
        cache.put("a".to_string(), (10, 20));
        cache.put("b".to_string(), (30, 40));
        // Acessar a promove a
        assert_eq!(cache.get("a"), Some((10, 20)));
        // Adicionar c expulsa b (b agora é LRU)
        cache.put("c".to_string(), (50, 60));
        assert_eq!(cache.get("a"), Some((10, 20)));
        assert!(cache.get("b").is_none());
        assert_eq!(cache.get("c"), Some((50, 60)));
    }

    #[test]
    fn test_extract_png_dimensions() {
        // PNG 2x3 minimal válido (gerado via image crate)
        let mut buf = Vec::new();
        {
            let img = image::RgbaImage::from_raw(2, 3, vec![255; 2 * 3 * 4]).unwrap();
            let mut cursor = Cursor::new(&mut buf);
            img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
        }
        assert_eq!(extract_image_dimensions(&buf), Some((2, 3)));
    }

    #[test]
    fn test_extract_jpeg_dimensions() {
        // JPEG 4x5 minimal válido (JPEG não suporta RGBA8)
        let mut buf = Vec::new();
        {
            let img = image::RgbImage::from_raw(4, 5, vec![128; 4 * 5 * 3]).unwrap();
            let mut cursor = Cursor::new(&mut buf);
            img.write_to(&mut cursor, image::ImageFormat::Jpeg).unwrap();
        }
        assert_eq!(extract_image_dimensions(&buf), Some((4, 5)));
    }

    #[test]
    fn test_extract_webp_dimensions() {
        // WebP 6x7 minimal válido
        let mut buf = Vec::new();
        {
            let img = image::RgbaImage::from_raw(6, 7, vec![64; 6 * 7 * 4]).unwrap();
            let mut cursor = Cursor::new(&mut buf);
            img.write_to(&mut cursor, image::ImageFormat::WebP).unwrap();
        }
        assert_eq!(extract_image_dimensions(&buf), Some((6, 7)));
    }
}
