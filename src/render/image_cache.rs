use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Cache global de imagens baixadas (bytes decodificados para RGBA).
static IMAGE_CACHE: Lazy<Mutex<ImageCache>> = Lazy::new(|| Mutex::new(ImageCache::new(200)));

/// Estrutura que guarda a imagem decodificada em memória.
pub struct DecodedImage {
    pub data: Vec<u8>, // RGBA raw bytes
    pub width: u32,
    pub height: u32,
}

/// Cache de imagens com evicção FIFO simples.
pub struct ImageCache {
    cache: HashMap<String, DecodedImage>,
    max_entries: usize,
    order: Vec<String>, // ordem de inserção para FIFO
}

impl ImageCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
            order: Vec::new(),
        }
    }

    /// Retorna referência se já existe no cache.
    pub fn get(&self, url: &str) -> Option<&DecodedImage> {
        self.cache.get(url)
    }

    /// Busca no cache; se não existir, baixa, decodifica e armazena.
    pub fn get_or_fetch(&mut self, url: &str, base_url: &str) -> Option<&DecodedImage> {
        if self.cache.contains_key(url) {
            return self.cache.get(url);
        }

        let bytes = download_image(url, base_url)?;
        let decoded = decode_image(&bytes)?;

        // Evicção FIFO
        if self.order.len() >= self.max_entries {
            let oldest = self.order.remove(0);
            self.cache.remove(&oldest);
        }

        self.order.push(url.to_string());
        self.cache.insert(url.to_string(), decoded);
        self.cache.get(url)
    }

    /// Retorna o número de entradas no cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Insere explicitamente uma imagem decodificada no cache.
    pub fn insert(&mut self, url: String, decoded: DecodedImage) {
        if self.order.len() >= self.max_entries {
            let oldest = self.order.remove(0);
            self.cache.remove(&oldest);
        }
        if !self.cache.contains_key(&url) {
            self.order.push(url.clone());
        }
        self.cache.insert(url, decoded);
    }
}

/// Acesso global ao cache (faz o lock do Mutex).
pub fn image_cache() -> std::sync::MutexGuard<'static, ImageCache> {
    IMAGE_CACHE.lock().expect("image cache poisoned")
}

/// Baixa bytes brutos de uma imagem via HTTP GET com timeout de 10s.
fn download_image(url: &str, base_url: &str) -> Option<Vec<u8>> {
    let full_url = resolve_url(url, base_url);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let response = client.get(&full_url).send().ok()?;
    if response.status().is_success() {
        response.bytes().ok().map(|b| b.to_vec())
    } else {
        log::warn!("HTTP {} ao baixar imagem {}", response.status(), full_url);
        None
    }
}

/// Decodifica bytes (JPEG/PNG/WebP/etc) para RGBA8.
fn decode_image(bytes: &[u8]) -> Option<DecodedImage> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(DecodedImage {
        data: rgba.into_raw(),
        width,
        height,
    })
}

/// Resolve URL relativa para absoluta.
pub fn resolve_url(url: &str, base_url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
        return url.to_string();
    }
    if let Ok(base) = url::Url::parse(base_url) {
        if let Ok(resolved) = base.join(url) {
            return resolved.to_string();
        }
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        url.trim_start_matches('/')
    )
}
