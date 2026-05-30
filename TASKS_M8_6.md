# 🔧 M8.6 — Imagens: Pseudo-Classes, Dimensões Intrínsecas e Layout

**Status:** A fazer
**Meta:** Screenshot de books.toscrape.com visualmente comparável ao Playwright
**Teste de referência:** `faf screenshot --url https://books.toscrape.com/`
**Site de referência:** `https://browsershots.org/` (verificar screenshot real do Chromium)

---

## 🎯 Problema Atual

As imagens dos livros na books.toscrape.com **não aparecem** no screenshot do FAF. Causas:

1. **267 regras CSS** com seletor `>` (ex: `.thumbnail > img`)
2. Dessas, apenas **6 casam** — as outras 261 contêm pseudo-classes (`:hover`, `:focus`, etc.) que fazem o `scraper::Selector::parse()` rejeitar a regra **inteira**
3. Mesmo quando funcionam, imagens sem `width`/`height` CSS têm fallback 100×100 (quadrado genérico)
4. Dimensões reais das imagens não são baixadas/usadas

---

## 🔴 T075 — Filtrar Pseudo-Classes nos Seletores CSS

**Impacto:** Crítico — libera 261 regras CSS que hoje são descartadas
**Arquivos:** `src/css/style.rs`, `src/css/selector.rs`
**Dependências:** Nenhuma (primeira task a fazer)

### Diagnóstico

Exemplo real do CSS da books.toscrape.com:
```css
.thumbnail > img,
.thumbnail a > img:hover,
.product_pod:hover .product_price {
  /* estilos visuais importantes */
}
```

O `scraper::Selector::parse(".thumbnail > img, .thumbnail a > img:hover")` retorna **erro** porque a segunda parte tem `:hover`. Resultado: a regra **inteira** é descartada, incluindo `.thumbnail > img` que seria válida.

No T067 (M8.5), foi adicionado filtro para `::before`/`::after` que **remove toda a lista** quando encontra pseudo-elemento. Agora precisamos de algo mais sofisticado.

### Solução

Modificar `select_elements()` (ou `find_style_for_element()`) para:

1. **Split por vírgula** na lista de seletores: `".thumbnail > img, .thumbnail a > img:hover"` → 2 partes
2. **Para cada parte**, tentar `scraper::Selector::parse()`
3. Se uma parte falha por causa de pseudo-classe (`:` que não seja `::`), **descartar só essa parte**, não a regra inteira
4. Aplicar estilos de partes válidas aos elementos

### Implementação Detalhada

**Arquivo:** `src/css/style.rs` (ou `selector.rs`)

```rust
// ANTES (descarta tudo se qualquer parte falhar):
pub fn parse_selector_list(css_selector: &str) -> Vec<Selector> {
    // scraper::Selector::parse() é chamado na lista INTEIRA → falha total
}

// DEPOIS (tenta cada seletor individualmente):
pub fn parse_selector_list(css_selector: &str) -> Vec<Selector> {
    // 1. Split por vírgula
    // 2. Para cada parte:
    //    a. Tentar scraper::Selector::parse(parte)
    //    b. Se falhar E o erro contém ":" (pseudo-classe):
    //       - Logar warning: "Ignorando seletor com pseudo-classe: {parte}"
    //       - Continuar (não quebrar)
    //    c. Se_OK, adicionar à lista de seletores válidos
    // 3. Retornar lista de seletores válidos (pode estar vazia)
}
```

### Pseudo-classes a Filtrar

Filtrar seletores que contenham qualquer uma destas (mas aplicar as partes válidas):

| Pseudo-classe | Exemplo que falha | Parte válida |
|---------------|-----------------|--------------|
| `:hover` | `.thumbnail a > img:hover` | `.thumbnail a > img` |
| `:focus` | `input:focus` | `input` |
| `:active` | `a:active` | `a` |
| `:first-child` | `li:first-child` | `li` |
| `:last-child` | `div:last-child` | `div` |
| `:nth-child()` | `tr:nth-child(2n)` | `tr` |
| `:not()` | `.a:not(.b)` | `.a` |
| `:before` | (já filtrado no T067) | — |
| `:after` | (já filtrado no T067) | — |

> ⚠️ **Regra importante:** Só filtrar a **parte inválida** da lista. Se `.thumbnail > img, .thumbnail a > img:hover` → `.thumbnail > img` **deve ser aplicada**.

### Critério de Aceite

```bash
# DEPOIS:
# 1. cargo test --lib → todos passando
# 2. cargo clippy → 0 warnings
# 3. Verificar que seletores como ".thumbnail > img" são aplicados mesmo quando
#    fazem parte de uma lista com pseudo-classe em outra parte
# 4. Teste: criar HTML com <div class="a"><img class="b"></div> e CSS:
#    ".a > img, .a > img:hover { width: 100px }"
#    → a regra deve ser aplicada ao <img> (não rejeitada inteira)
```

### Teste Específico

Criar teste em `tests/m8_6_test.rs`:

```rust
#[test]
fn test_pseudo_class_selector_list_does_not_reject_valid_parts() {
    // HTML: <div class="parent"><img class="child"></div>
    // CSS: ".parent > img, .parent > img:hover { width: 100px }"
    // O <img> deve receber width: 100px (primeira parte é válida)
}
```

---

## 🔴 T076 — Dimensões Intrínsecas Reais das Imagens

**Impacto:** Crítico — substitui fallback 100×100 burdo por dimensões reais
**Arquivos:** `src/render/tree.rs`, `src/render/layout.rs`, `src/css/style.rs`
**Dependências:** T075 (precisa do CSS correto aplicado para encontrar `<img>` com classes certas)

### Diagnóstico

Hoje:
- `<img>` sem `width`/`height` CSS → `compute_box_model()` retorna 0×0
- T073 adicionou fallback 100×100 como **hack** temporário
- Isso gera thumbnails quadrados de 100×100 para todas as imagens (incluindo livros de outros formatos)

O CSS correto (T075) deveria trazer `.thumbnail img { width: ...; height: ... }` mas a books.toscrape.com **não define dimensões explícitas** no CSS para as imagens — depende das **dimensões intrínsecas** da imagem (largura × altura real do JPEG).

### Solução

Durante o **build da layout tree**, para cada nó `<img>`:

1. **Detectar** nó `<img>` com atributo `src`
2. **Baixar** a imagem (HTTP GET no `src`)
3. **Decodificar** headers de dimensão (basta o HTTP HEAD ou os primeiros bytes do JPEG/PNG/WebP)
4. **Armazenar** `intrinsic_width × intrinsic_height` no `VisualNode` ou `ComputedStyle`
5. **Usar** no `layout_block` como último recurso (depois de CSS > attributes)

### Implementação Detalhada

**Passo 1 — Adicionar campo ao ComputedStyle ou VisualNode**

```rust
// Em src/css/style.rs ou src/render/tree.rs
pub struct ComputedStyle {
    // ... campos existentes ...
    pub intrinsic_width: Option<u32>,   // NOVO
    pub intrinsic_height: Option<u32>,  // NOVO
}
```

**Passo 2 — Baixar e decodificar imagem**

```rust
// Em src/render/tree.rs — nova função
async fn fetch_image_dimensions(src: &str, base_url: &Url) -> Option<(u32, u32)> {
    // 1. Resolver URL relativa contra base_url
    let full_url = base_url.join(src).ok()?;
    
    // 2. HTTP GET (HEAD preferível, mas GET com range=0-1000 também funciona)
    //    Reutilizar HttpClient do projeto (já existe)
    let resp = http_client.head(&full_url).await.ok()?;
    
    // 3. Para JPEG:extrair w/h dos bytes 165-172 do SOF0 marker
    //   Para PNG: extrair do IHDR chunk (bytes 16-24)
    //   Para WebP: extrair do chunk VP8 (bytes 26-30)
    // 4. Retornar (width, height)
}
```

**Passo 3 — Integrar no build da layout tree**

```rust
// Em src/render/tree.rs — find_img_nodes()
async fn build_layout_tree_async(/* ... */) {
    // Para cada nó <img>:
    if let Some(img_src) = element.value().attr("src") {
        // Buscar dimensões intrínsecas (com cache!)
        if let Some((w, h)) = fetch_image_dimensions_cached(img_src, &base_url).await {
            node.intrinsic_width = Some(w);
            node.intrinsic_height = Some(h);
        }
    }
}
```

**Passo 4 — Usar dimensões no layout_block**

```rust
// Em src/render/layout.rs — layout_block()
fn layout_block(node: &VisualNode, /* ... */) -> LayoutBox {
    let width = style.width
        .or(style.intrinsic_width)   // NOVO: usa dimensão real se não houver CSS
        .unwrap_or(100);             // fallback final
    
    let height = style.height
        .or(style.intrinsic_height)  // NOVO
        .unwrap_or(100);
}
```

### ⚠️ Performance — Cache de Imagens

Baixar imagem para **cada** `<img>` é lento. Implementar cache em memória:

```rust
// Cache LRU: no máximo 50 imagens em memória
// Key: (url, dimensions)
// Valor: (width, height)
// Limpar quando exceder limite
```

### Critério de Aceite

```bash
# DEPOIS:
# 1. cargo test --lib → todos passando
# 2. cargo clippy → 0 warnings
# 3. Screenshot de books.toscrape.com mostra imagens com proporções reais
#    (não 100×100 quadradas — livros têm aspect ratio ~2:3)
# 4. Imagens diferentes têm tamanhos diferentes (não todas 100×100)
# 5. Teste: verificar que uma imagem 300×450 recebe height proporcional
```

### Teste Específico

```rust
#[test]
fn test_img_uses_intrinsic_dimensions_when_no_css() {
    // HTML: <img src="book.jpg"> (imagem 300x450)
    // CSS: (nenhum width/height definido)
    // Resultado: imagem renderiza como 300x450 (proporção real)
}
```

---

## 🟡 T074 — Corrigir Colapso de Altura com Dimensões Reais

**Impacto:** Médio — polimento final após T075+T076
**Arquivos:** `src/render/layout.rs`
**Dependências:** T076 (precisa de dimensões reais para testar)

### Diagnóstico

Com T073, o fallback 100×100 pode estar causando **colapso ou overflow** incorreto no layout block:
- Container `.product_pod` tem altura baseada em fallback 100×100
- Quando T076 trouxer dimensões reais (ex: 168×240), o container pode não expandir corretamente
- Ou imagens podem "estourar" para fora do container

### Solução

Revisar `layout_block()` para garantir que:
1. **Container expande** para acomodar imagens com dimensões reais
2. **Margins verticais** se comportam corretamente quando imagens substituem fallback
3. **`overflow: hidden`** não é afetado

### Critério de Aceite

```bash
# DEPOIS:
# 1. cargo test --lib → todos passando
# 2. Screenshot: livros aparecem bem acomodados dentro dos .product_pod
#    (não transbordando, não espremidos)
```

---

## 🧪 Testes (tests/m8_6_test.rs)

```bash
# Criar arquivo tests/m8_6_test.rs com:
# 1. test_pseudo_class_selector_list_does_not_reject_valid_parts
# 2. test_img_uses_intrinsic_dimensions_when_no_css
# 3. test_img_intrinsic_dimensions_cached
# 4. test_img_fallback_100x100_when_fetch_fails
# 5. test_layout_expands_for_real_img_dimensions
# 6. test_multiple_imgs_different_sizes
```

---

## 📊 Progresso

| Task | Status | Resultado visual |
|------|:------:|-----------------|
| **T075** — Filtrar pseudo-classes | ⬜ | - |
| **T076** — Dimensões intrínsecas reais | ⬜ | - |
| **T074** — Corrigir colapso de altura | ⬜ | - |
| **Screenshot books.toscrape.com** | ⬜ | - |

---

## 🔗 Ordem de Execução

```
T075 (pseudo-classes)
    ↓
T076 (dimensões reais) ←── T074 fica por último
    ↓
T074 (layout/height)
    ↓
Screenshot → validar → ajustar
```

---

## 📋 Protocolo de Validação Visual (OBRIGATÓRIO após cada task)

Após completar cada task:

1. **Gerar screenshot do FAF:**
   ```bash
   cd /home/hermes/faf-browser
   cargo build --release 2>&1 | tail -5
   ./target/release/faf screenshot --url https://books.toscrape.com/ --output /tmp/faf_screenshot.png
   ```

2. **Obter screenshot de referência (Chromium real):**
   ```bash
   # Via browsershots.org ou usar chromium headless se disponível:
   chromium-browser --headless --screenshot=/tmp/chromium_screenshot.png https://books.toscrape.com/
   ```

3. **Comparar lado a lado:**
   ```bash
   # Ambos em /tmp/ — enviar via Telegram para o usuário avaliar
   ```

4. **Se discrepâncias:**
   - Identificar o problema específico (cores, posições, tamanhos, textos)
   - Criar task corretiva antes de continuar para a próxima
   - Reportar ao usuário (MiniMax) para avaliação
