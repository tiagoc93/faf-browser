# M8.7 — Renderização de Imagens Reais

**Status:** EM ANDAMENTO — bug identificado, aguardando correção
**Data início:** 2025-05-30
**Data última atualização:** 2025-05-30

## Contexto

Comparando screenshot FAF vs Playwright (Chromium real):

| Aspecto | FAF Browser | Playwright (real) |
|---------|-------------|-------------------|
| Capas dos livros | Retângulo cinza placeholder 100×100 | Imagens coloridas reais |
| Estrelas de rating | Ausentes | Presentes (★★★★★) |
| Altura da página | ~2044px | ~4500px+ |
| Cores gerais | Mais escuras que o original | Fidélio ao site |

**Problema identificado:** O FAF Browser baixa as **dimensões** (largura × altura) das imagens via HTTP HEAD, mas **não baixa os pixels** (dados binários). O resultado é que todas as imagens renderizam como retângulos cinza placeholder.

## Objetivo

Implementar pipeline completo de renderização de imagens:
1. **Baixar** dados binários (bytes) das imagens
2. **Decodificar** JPEG/PNG/WebP para RGBA
3. **Renderizar** pixels no tiny_skia Pixmap
4. **Corrigir** altura da página para corresponder ao conteúdo real

## Estrutura de Arquivos

```
src/
├── render/
│   ├── mod.rs
│   ├── screenshot.rs     # Não modificar (já funciona)
│   ├── layout.rs        # Layout block/inline — NÃO modificar
│   ├── tree.rs          # NÃO modificar
│   ├── image_dimensions.rs  # JÁ EXISTE — usar como base
│   └── image_cache.rs   # NOVO — cache de imagens baixadas
src/css/style.rs         # NÃO modificar
Cargo.toml               # NÃO modificar (image crate já está)
```

## Pipeline de Imagens

### 1. Fluxo de Dados

```
HTML <img src="media/cache/2c/da/...jpg">
        │
        ▼
fetch_image() — reqwest GET → bytes
        │
        ▼
decode_image() — image::load_from_memory() → DynamicImage
        │
        ▼
to_rgba8() → RgbaImage (Vec<u8> com pixels RGBA)
        │
        ▼
create tiny_skia::Pixmap from RGBA bytes
        │
        ▼
draw_pixmap() na posição correta do layout
```

### 2. Código Existente — O que usar

**image_dimensions.rs (já existe, NÃO reescrever):**
- `fetch_all_image_dimensions()` — busca dimensões de TODAS as imagens
- `ImageDimensionCache` com LRU (max 200 entries)
- `fetch_image_dimensions(url, cache)` — uma imagem

**Problema:** Esse arquivo só busca DIMENSÕES (width × height), não os BYTES.

### 3. Nova estrutura — image_cache.rs

Criar `src/render/image_cache.rs` com:

```rust
use image::{GenericImageView, DynamicImage};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Cache global de imagens baixadas (bytes decodificados)
static IMAGE_CACHE: Lazy<Mutex<ImageCache>> =
    Lazy::new(|| Mutex::new(ImageCache::new(200))); // max 200 entries

pub struct ImageCache {
    cache: HashMap<String, DecodedImage>,
    max_entries: usize,
}

pub struct DecodedImage {
    pub pixmap: tiny_skia::Pixmap,  // RGBA pixels prontos pra renderizar
    pub width: u32,
    pub height: u32,
}

impl ImageCache {
    /// Baixa, decodifica e cachea uma imagem
    pub fn get_or_fetch(&mut self, url: &str, base_url: &str) -> Option<&DecodedImage> {
        // 1. Se já está no cache, retorna
        // 2. Se não, baixa via reqwest::blocking::get(url_completo)
        // 3. Decodifica: image::load_from_memory(&bytes) → DynamicImage → to_rgba8()
        // 4. Cria tiny_skia::Pixmap::from_bytes()
        // 5. Armazena no cache
        // 6. Se cache > max_entries, remove oldest (FIFO simples)
    }
}
```

### 4. Modificar build_layout_tree

Em `src/render/tree.rs`, no ponto onde imagens são processadas:

```rust
// ANTES (apenas dimensões):
if tag == "img" {
    if let Some(dims_map) = image_dims {
        // Seta intrinsic_width/intrinsic_height no style
    }
}

// DEPOIS (dimensões + pixels):
if tag == "img" {
    if let Some(dims_map) = image_dims {
        // Seta intrinsic_width/intrinsic_height
    }
    // NOVO: buscar pixels da imagem do cache
    if let Some(src) = visual.attributes.get("src") {
        let resolved = resolve_url(src, base_url);
        if let Some(decoded) = image_cache().get_or_fetch(&resolved, base_url) {
            // Armazenar reference ao pixmap no VisualNode
            // Ou: colocar decoded.pixmap.data() no attributes como "image_pixels"
        }
    }
}
```

**IMPORTANTE:** VisualNode NÃO precisa armazenar o pixmap inteiro — só precisa de uma referência (índice no cache) que será usada no render.

### 5. Renderização em screenshot.rs

Em `src/render/screenshot.rs` na função que renderiza os nós:

```rust
// NOVO: dentro do match que renderiza elementos
NodeType::Block | NodeType::InlineBlock => {
    // ... código existente de render_background, render_borders, etc.

    // NOVO: renderizar imagem se for <img>
    if node.tag == "img" {
        if let Some(img_data) = node.attributes.get("image_pixels") {
            // img_data contém o tiny_skia::Pixmap
            let pixmap = /* extrair do cache via img_data como chave */;
            let x = node.rect.x() + bm.padding_left;
            let y = node.rect.y() + bm.padding_top;
            let w = node.rect.width() - bm.padding_left - bm.padding_right;
            let h = node.rect.height() - bm.padding_top - bm.padding_bottom;

            // Scale preserve se w/h diferente do original
            // sk_pixmap.draw_pixmap_at(x, y, &pixmap)
            // Ou: usar tiny_skia::IntRect pra clipping
        }
    }
}
```

### 6. Detalhes de Implementação

#### 6.1 Baixar imagem (reqwest)

```rust
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
        None
    }
}
```

#### 6.2 Decodificar (image crate — JÁ DEPENDENTE)

```rust
fn decode_image(bytes: &[u8]) -> Option<tiny_skia::Pixmap> {
    // image::load_from_memory supports JPEG, PNG, WebP, GIF, BMP, ICO, TIFF
    let img: DynamicImage = image::load_from_memory(bytes).ok()?;

    // Converter pra RGBA
    let rgba: image::RgbaImage = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Criar tiny_skia::Pixmap
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    pixmap.data_mut().copy_from_slice(rgba.as_raw());

    Some(pixmap)
}
```

#### 6.3 Resolver URL relativa

```rust
fn resolve_url(url: &str, base_url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        let base = url::Url::parse(base_url).unwrap_or_else(|_| url::Url::parse("https://dummy.com").unwrap());
        base.join(url).map(|u| u.to_string()).unwrap_or_else(|_| url.to_string())
    }
}
```

#### 6.4 Adicionar em Cargo.toml (se necessário)

```toml
[dependencies]
# JÁ ESTÁ:
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }

# PRECISA ADICIONAR (verificar se já existe):
once_cell = "1.19"  # Para static Lazy

# NÃO PRECISA ADICIONAR (já é dependência de outras crates):
# reqwest (já é dependência direta)
# tiny-skia (já é dependência)
```

### 7. Fluxo de Execução Completo

```
1. screenshot.rs::take_screenshot()
       │
       ▼
2. Parse HTML → DOM tree
       │
       ▼
3. Parse CSS (inline + externos)
       │
       ▼
4. Compute styles (cascata)
       │
       ▼
5. NOVO: Baixar TODAS as imagens (bytes) → IMAGE_CACHE
       │    Usa reqwest::blocking::get() pra cada <img src="...">
       │    Decodifica: image::load_from_memory() → tiny_skia::Pixmap
       │    Armazena no cache (HashMap<String, Pixmap>)
       │
       ▼
6. build_layout_tree(doc, computed, &image_dims)
       │    Para cada <img>, busca dimensões no cache
       │    Seta intrinsic_width/intrinsic_height
       │
       ▼
7. compute_layout(tree, viewport_width)
       │    Alturas CORRETAS agora (não mais 100px fallback)
       │
       ▼
8. Render PNG (tiny-skia)
       │    Para cada <img>, extrai Pixmap do cache
       │    Desenha na posição x,y com dimensões w,h
       │
       ▼
9. Output PNG
```

### 8. Critérios de Aceitação

| Critério | Como verificar |
|----------|----------------|
| Imagens reais aparecem no screenshot | Comparar visualmente com Playwright — capas coloridas |
| Estrelas de rating visíveis | ★★★★★ no lugar de retângulos cinza |
| Altura da página aumenta | screenshot deve ter > 4000px (antes: 2044px) |
| Sem crashes | cargo test + screenshot roda sem panic |
| Cache funciona | 2ª execução não baixa imagens (verificar logs RUST_LOG) |
| Fallback funciona | Imagem quebrada → placeholder 100×100 (não crash) |

### 9. Tasks Detalhadas

#### T077 — Criar image_cache.rs
- Criar `src/render/image_cache.rs`
- Implementar `ImageCache` com HashMap + max_entries
- Implementar `download_image(url, base_url)` via reqwest
- Implementar `decode_image(bytes)` via image crate → tiny_skia::Pixmap
- Implementar `get_or_fetch(url, base_url)` — busca cache ou baixa
- Criar `resolve_url(url, base_url)` — URL relativa → absoluta
- Adicionar `once_cell` em Cargo.toml se necessário
- Testar: baixar uma imagem e criar Pixmap

#### T078 — Modificar screenshot.rs para baixar imagens ANTES do layout
- Em `take_screenshot()`, ANTES de `build_layout_tree`:
  - Extrair todas as URLs de `<img src="...">`
  - Chamar `image_cache().get_or_fetch(url, base_url)` pra cada uma
  - Log: "Baixando N imagens..."
- NÃO modificar build_layout_tree (já recebe image_dims)
- Garantir que download seja sequencial (reqwest blocking, não async)
- Timeout: 10s por imagem, fallback se falhar

#### T079 — Modificar renderização de <img> em screenshot.rs
- Na função que renderiza nós visuais:
  - Detectar `node.tag == "img"`
  - Buscar Pixmap no IMAGE_CACHE via `node.attributes.get("src")`
  - Se existe Pixmap:
    - Calcular posição: x + padding_left, y + padding_top
    - Calcular dimensões: rect.width - padding_left - padding_right
    - Desenhar: `sk_pixmap.draw_pixmap(...)` com scale se necessário
  - Se NÃO existe (fallback): continuar renderizando retângulo cinza (comportamento atual)

#### T080 — Tratar scale/preserve aspect ratio
- Se `img.width` CSS é diferente da largura natural da imagem:
  - Manter aspect ratio
  - Usar `object-fit: contain` logic (default CSS)
  - Centralizar na box se smaller
- Se `img.height` CSS é explícito E diferente do natural:
  - Aplicar height explícito
  - Scale width proporcionalmente
- IMPORTANTE: isso é complexo — começar sem scale (1:1)

#### T081 — Testes de integração
- `cargo test` passa
- Screenshot books.toscrape.com mostra capas coloridas
- Altura screenshot > 4000px
- 2ª execução: logs mostram "Cache HIT" (não baixa novamente)
- Imagem quebrada: fallback 100×100 cinza (não crash)
- `cargo clippy` limpo

### 10. Possíveis Problemas e Soluções

| Problema | Solução |
|----------|--------|
| Imagem muito grande (10MB+) | Timeout 10s + não cachear |
| Tipo MIME errado (não é img) | Verificar Content-Type ou confiar no decode |
| URL relativa mal resolvida | Usar `url` crate (já no projeto) |
| tiny_skia Pixmap muito grande | Não alocar > 4096×4096 por imagem |
| Cache grows unbounded | max 200 entries, FIFO eviction |
| Sync dentro async context | `reqwest::blocking::get` é blocking, ok se called from sync context |
| Many images (50+) timeout | Timeout 10s por imagem, continue others |

## 📊 Progresso

|| Task | Status | Notas |
|------|:------:|-------|-------|
| **T077** — Criar image_cache.rs | ✅ Feito | Cache com 20 entradas funciona |
| **T078** — Baixar imagens ANTES do layout | ✅ Feito | Imagens em /tmp/image_cache/ |
| **T079** — Renderizar <img> em screenshot.rs | ✅ Corrigido | `contains("img")` aplicado |
| **T080** — Scale/preserve aspect ratio | ⬜ Pendente | Depende de T081 |
| **T081** — Testes de integração | ⬜ Pendente | — |
| **Screenshot books.toscrape.com** | 🔴 Falhando | Imagens não aparecem visualmente |

---

## 🐛 Bug Identificado: Imagens Não Aparecem Visualmente

**Arquivo:** `src/render/screenshot.rs`

### Problema 1 — count_img_nodes (mesmo bug de render_node)

```rust
// screenshot.rs:280 — AINDA USA:
let self_count = if node.tag == "img" { 1 } else { 0 };

// DEVERIA SER:
let self_count = if node.tag.contains("img") { 1 } else { 0 };
```

**Impacto:** Esta função não detecta img nodes, mas o problema real está em outro lugar.

### Problema 2 — draw_image NÃO aparece nos logs (SUSPECT: timing)

**Sintomas:**
- Cache preenchido com 20 entradas
- TREE_BEFORE_LAYOUT e TREE_AFTER_LAYOUT mostram img_count=20
- `RUST_LOG=trace` NÃO mostra DRAW_IMAGE
- Screenshot final: 697 pixels coloridos (muito pouco para 20 capas)
- Pixels coloridos existem mas NÃO correspondem às capas dos livros

**Hipótese:** O download das imagens pode ser assíncrono e o render acontece antes das imagens serem baixadas.

### Logs Coletados

```
TREE_BEFORE_LAYOUT: img_count=20
TREE_AFTER_LAYOUT: img_count=20
Cache de imagens preenchido: 20 entradas
Dimensões de imagens buscadas: 40 imagens
# DRAW_IMAGE NÃO aparece mesmo com RUST_LOG=trace
```

### Próximos Passos para Resolver

1. [ ] **CORRIGIR count_img_nodes**: `== "img"` → `contains("img")` (screenshot.rs:280)
2. [ ] **ADICIONAR logging em draw_image**: Dentro de `render_node`, adicionar log antes de chamar `draw_image`
3. [ ] **VERIFICAR timing**: Confirmar que as imagens estão no cache ANTES de renderizar
4. [ ] **VALIDAR screenshot**: Após correção, gerar screenshot e confirmar capas visíveis
5. [ ] **COMMIT**: git add + commit + push
6. [ ] **DOCUMENTAR**: Atualizar README.md e TASKS.md

---

## 📋 Protocolo de Validação Visual (OBRIGATÓRIO após cada task)

Após completar cada task:

1. **Gerar screenshot do FAF:**
   ```bash
   cd /home/hermes/faf-browser
   cargo build --release 2>&1 | tail -5
   RUST_LOG=faf_browser=info cargo run --release -- screenshot
   ```

2. **Enviar screenshot via Telegram:**
   ```
   Arquivo: /home/hermes/faf-browser/screenshot.png
   ```

3. **Avaliação visual:**
   - Capas dos livros visíveis?
   - Estrelas de rating visíveis?
   - Altura da página > 4000px?
   - Cores fiéis ao site?

4. **Se falhar:** Voltar ao debugging, não avançar para próxima task.