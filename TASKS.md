# 🎯 Tasks — FAF BROWSER (Fast As Fuck)

**Status:** M8 concluído | **Tests:** 329 | **Stack:** Rust + Cargo (Edition 2024)

---

## ✅ M1 — Core Engine (12 tasks · Concluído)

### HTTP Client & Setup
- [x] **T001** — Inicializar projeto Cargo + dependências (reqwest, tokio, scraper, rquickjs, clap, serde)
- [x] **T002** — HTTP Client: fetch página via reqwest com headers customizáveis
- [x] **T003** — Suporte a proxy: SOCKS5 + HTTP via reqwest + timeout configurável
- [x] **T004** — CLI básica com clap: `faf <url>` + flags (--proxy, --timeout, --user-agent)

### HTML Parser & DOM
- [x] **T005** — Parser HTML com scraper/html5ever: converter bytes em DOM tree
- [x] **T006** — DOM tree: HtmlDocument struct com scraper::Html interno
- [x] **T007** — Navegação na árvore: links(), images(), metadata(), visible_text()
- [x] **T008** — Search por seletor CSS: query() com scraper::Selector

### Integração & Output
- [x] **T009** — Pipeline HTTP → Parse → DOM: juntar fetch + parser no run()
- [x] **T010** — Output JSON: serializar DOM tree
- [x] **T011** — Tratamento de erros (anyhow) + logs (env_logger)
- [x] **T012** — Testes de integração M1

---

## ✅ M2 — CSS Engine (8 tasks · Concluído)

- [x] **T013** — Parser CSS com cssparser: tokenizar folhas de estilo
- [x] **T014** — Selector matching com compute_specificity()
- [x] **T015** — Computed styles + cascata: inline > ID > class > tag
- [x] **T016** — Box model: width, height, margin, padding com shorthand
- [x] **T017** — Cores: 19 named colors, hex, rgb/rgba
- [x] **T018** — Fontes: font-family + fallback, size (px/em/rem/%), weight
- [x] **T019** — CLI: `faf query "h1" --url <url>` — texto + computed style
- [x] **T020** — Testes M2

---

## ✅ M2.5 — Polimento CLI + Extração Avançada (8 tasks · Concluído)

- [x] **P01** — `--css` e `--json` flags globais com clap `global = true`
- [x] **P02** — Remover `--query` flag morto do struct Cli
- [x] **P03** — Defaults CSS reais no ComputedStyle
- [x] **P04** — Parse automático do CSS da página (`<style>` + `<link>`)
- [x] **Q01** — `--filter "campo~=valor"`: text match, regex, attribute match, múltiplos filtros AND
- [x] **Q02** — `--get "campo1, campo2"`: extração seletiva de campos
- [x] **Q03** — `faf follow <seletor>`: crawler multithread com tokio semaphore
- [x] **Q04** — `--format csv|jsonl|json|text`: múltiplos formatos de saída

---

## ✅ M3 — JavaScript Engine (10 tasks · Concluído)

- [x] **T021** — Embed QuickJS via rquickjs: Runtime + Context + eval(), eval_json()
- [x] **T022** — Bridge DOM ↔ JS: document.title, getElementById, querySelector, querySelectorAll
- [x] **T023** — setTimeout / setInterval com event loop
- [x] **T024** — Fetch API via JS → reqwest
- [x] **T025** — Timeout de execução JS: eval_with_timeout()
- [x] **T026** — Console.log/warn/error do JS → Rust logger
- [x] **T027** — Tratamento de erros JS: format_js_error()
- [x] **T028** — Suporte a scripts `<script>` inline + externos
- [x] **T029** — CLI: `--js "document.title"`, `--js-file`, `--no-scripts`, `--js-timeout`
- [x] **T030** — Testes M3

---

## ✅ M4 — Sessão, Interação & Pipeline (8 tasks · Concluído)

### T031 — Cookie Persistence ✅
- `--cookies <path>` carrega cookies formato Netscape
- `--cookies-jar <path>` salva cookies atualizados
- reqwest::Client com cookie_store(true) para gerenciar cookies entre redirects

### T032 — WaitForSelector ✅
- `faf wait ".produto" --url <url> --timeout 10`
- Loop polling a cada `--interval` ms executando `document.querySelector()` via JS
- Timeout expirado → erro claro

### T033 — Modo Interativo / REPL ✅
- `faf repl --url <url>` — prompt interativo com rustyline
- `--stdin` — pipe de JS: `echo "document.title" | faf --stdin --url <url>`
- Runtime preserva estado entre comandos

### T034 — Rate Limiting no Follow ✅
- `--delay <ms>` delay fixo entre requests
- `--random-delay MIN MAX` delay aleatório

### T035 — Retry com Exponential Backoff ✅
- `--retries N` com `--retry-delay <ms>`
- Tratamento HTTP 429 (Too Many Requests) com header Retry-After
- Delay dobra a cada tentativa: 1s, 2s, 4s

### T036 — Output com Headers HTTP ✅
- `--show-headers` exibe response headers
- `--show-status` exibe status code

### T037 — Cache de Responses ✅
- `--cache <dir>` cache em disco com SHA256
- `--cache-ttl <s>` TTL configurável (default 300s)
- `--no-cache` ignora cache

### T038 — Testes M4 ✅
- 11 testes: cookies, wait, repl, retry, cache, rate limit, show headers/status

---

## ✅ M4.5 — Refinamentos Pós-M4 (3 tasks · Concluído)

- [x] **F001** — Investigado: follow --extract NÃO vaza DOM entre páginas (falso alarme — sidebars das páginas)
- [x] **F002** — Fix: cookies-jar com cookie_store(true) do reqwest (resolve perda em redirects)
- [x] **F003** — Enhancement: `--filter` com operadores `!~=`, `!^=`, `!$=` (negative match)

---

## ✅ M5 — Interação com Páginas (5 tasks · Concluído)

### T039 — Click via dispatchEvent ✅
- Subcomando `faf click ".btn" --url <url>`
- dispatchEvent(MouseEvent) no runtime QuickJS via JS bridge
- Funciona via REPL/stdin também

### T040 — Formulários: fill, select, submit ✅
- `.value`, `.checked`, `.submit()` nos objetos Element do DOM bridge
- Sem novo subcomando — manipulado via REPL/stdin
- Polyfills: MouseEvent, URLSearchParams, HTMLFormElement

### T041 — Watch Mode ✅
- `faf watch ".preco" --url <url> --interval 30 --max-checks 5`
- Monitoramento periódico com detecção de mudanças

### T042 — Scroll e Navegação via JS ✅
- window.scrollTo(), window.scrollBy(), element.scrollIntoView()
- Polyfill com posição Y simulada no runtime JS

### T043 — Testes M5 ✅
- Testes: click, fill form, select, checkbox, watch, scroll

**M5 Resultado:** clippy limpo

---

## ✅ M6 — Dump HTML Autocontido (8 tasks · Concluído)

**Objetivo:** Substituir a funcionalidade de screenshot por um comando `dump` que gera um arquivo HTML autocontido e fiel à página original. O usuário pode abrir esse HTML em qualquer navegador e ver a página o mais próximo possível do original.

**Uso:**
```bash
faf dump --url https://books.toscrape.com/ --output pagina.html
faf dump --url https://site.com --output site.html --inline-images
faf dump --url https://site.com --output site.html --no-scripts
```

**Por que isso é melhor que screenshot:**
- HTML é universal — abre em qualquer navegador, sem dependências
- Mantém interatividade (links, formulários funcionam)
- Pode ser inspecionado, editado, compartilhado
- Tamanho de arquivo muito menor que PNG
- Preserva a semântica da página (a11y, SEO)

---

### 🔴 T082 — Criar módulo `src/dump/mod.rs` ✅

**Arquivo:** `src/dump/mod.rs`

Módulo principal com `DumpConfig` e `dump_to_file()`. Orquestra o pipeline: remove scripts → inline CSS → inline images → resolve URLs → write file.

---

### 🔴 T083 — Inline de CSS Externo ✅

**Arquivo:** `src/dump/css_inline.rs`

Baixa `<link rel="stylesheet">` externos via `reqwest::blocking::get()` e substitui por `<style>`. Resolve `url()` dentro do CSS para URLs absolutas. Cache de CSS por URL para evitar downloads duplicados.

---

### 🟡 T084 — Conversão de Imagens para Base64 ✅

**Arquivo:** `src/dump/image_inline.rs`

Baixa `<img src>` via `reqwest::blocking` com timeout 10s, converte bytes para base64 data URIs. Detecta MIME type via Content-Type header ou extensão do arquivo. Cache de imagens por URL.

---

### 🟡 T085 — Remoção de Scripts ✅

**Arquivo:** `src/dump/mod.rs` (função `remove_scripts()`)

Remove tags `<script>` (inline e externas) e atributos `on*` (onclick, onload, etc). Via regex no HTML string.

---

### 🟡 T086 — Resolução de URLs Relativas ✅

**Arquivo:** `src/dump/url_resolver.rs` + `src/dump/mod.rs`

`resolve_url()` converte URLs relativas para absolutas usando o crate `url`. `resolve_urls_in_html()` aplica resolução em atributos `href`, `src`, `action` em tags `<a>`, `<img>`, `<link>`, `<script>`, `<form>`, `<video>`, `<audio>`, `<source>`, `<iframe>`.

---

### 🟡 T087 — Reconstrução do HTML ✅

**Abordagem:** Manipulação direta da string HTML original com regex, em vez de serializar a DOM tree. Isso preserva a fidelidade do HTML original (comentários, whitespace, etc).

---

### 🟡 T088 — CLI: Subcomando `dump` ✅

**Arquivo:** `src/api/commands.rs`

```bash
faf dump --url https://site.com --output page.html
faf dump --url https://site.com --output page.html --inline-images
faf dump --url https://site.com --output page.html --no-scripts
faf dump --url https://site.com --output page.html --no-inline-css
```

---

### 🟡 T089 — Testes M6 ✅

**Arquivos:** `tests/m6_test.rs` (4 testes de integração), testes unitários nos módulos

Testes unitários: 22 (url_resolver, css_inline, image_inline, mod)
Testes de integração: 4 (dump basic, relative URLs, no-scripts, existing styles)

---

## ✅ M7 — LLM-Ready Output (5 tasks · Concluído)

**Objetivo:** Tornar o FAF a ferramenta ideal para alimentar LLMs com conteúdo web.

**Resultado:** 4 novos formatos de saída integrados ao comando `dump`:
- `--format markdown` — HTML → Markdown (headings, links, listas, imagens, code blocks)
- `--format text` — texto limpo preservando parágrafos e headings
- `--readability` — extração de conteúdo principal (text-density scoring)
- `--structured-data` — extração de JSON-LD, Open Graph, meta tags

---

### 🔴 T090 — `dump --format markdown`

**Arquivo a criar:** `src/dump/markdown.rs`

**O que faz:** Converte o HTML da página para Markdown limpo, preservando estrutura semântica.

**Flags novas:**
```bash
faf dump --url https://site.com --output page.md --format markdown
```

**Implementação detalhada (passo a passo):**

**Passo 1 — Criar `src/dump/markdown.rs` com a função pública:**

```rust
pub fn html_to_markdown(html: &str) -> String
```

Assinatura: recebe a string HTML original (após processamento de CSS inline/scripts), retorna string Markdown.

**Passo 2 — Mapeamento de tags HTML → Markdown:**

Percorrer a DOM tree do scraper e converter cada elemento:

| Tag HTML | Output Markdown |
|----------|----------------|
| `<h1>` | `# text\n\n` |
| `<h2>` | `## text\n\n` |
| `<h3>` | `### text\n\n` |
| `<h4>` | `#### text\n\n` |
| `<h5>` | `##### text\n\n` |
| `<h6>` | `###### text\n\n` |
| `<p>` | `text\n\n` |
| `<a href="url">text</a>` | `[text](url)` |
| `<img src="url" alt="text">` | `![text](url)` |
| `<strong>` / `<b>` | `**text**` |
| `<em>` / `<i>` | `*text*` |
| `<code>` | `` `text` `` |
| `<pre><code>` | ```` ```\ntext\n``` ```` |
| `<blockquote>` | `> text\n\n` |
| `<ul><li>` | `- text\n` |
| `<ol><li>` | `1. text\n` |
| `<br>` | `\n` |
| `<hr>` | `---\n\n` |
| `<table>` | tabela formatada com `\|` e `-` (GFM) |
| `<script>`, `<style>`, `<nav>`, `<footer>` | REMOVIDO (não gerar output) |
| `<div>`, `<span>`, `<section>`, `<article>` | processar filhos, sem wrapper |

**Passo 3 — Algoritmo de conversão:**

```rust
fn convert_node(element: scraper::ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(el_ref) => {
                let tag = el_ref.value().name().to_lowercase();
                match tag.as_str() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = tag.chars().nth(1).unwrap() as u32 - '0' as u32;
                        let prefix = "#".repeat(level as usize);
                        let text: String = el_ref.text().collect();
                        output.push_str(&format!("{} {}\n\n", prefix, text.trim()));
                    }
                    "p" => {
                        let text: String = el_ref.text().collect();
                        output.push_str(&format!("{}\n\n", text.trim()));
                    }
                    "a" => {
                        let href = el_ref.value().attr("href").unwrap_or("");
                        let text: String = el_ref.text().collect();
                        if !href.is_empty() && !href.starts_with("javascript:") {
                            output.push_str(&format!("[{}]({})", text.trim(), href));
                        } else {
                            output.push_str(&text);
                        }
                    }
                    "strong" | "b" => {
                        let text: String = el_ref.text().collect();
                        output.push_str(&format!("**{}**", text.trim()));
                    }
                    "em" | "i" => {
                        let text: String = el_ref.text().collect();
                        output.push_str(&format!("*{}*", text.trim()));
                    }
                    "ul" | "ol" => { convert_list(el_ref, output, tag == "ol"); }
                    "li" => { /* handled by parent ul/ol */ }
                    "br" => { output.push('\n'); }
                    "hr" => { output.push_str("\n---\n\n"); }
                    "img" => {
                        let src = el_ref.value().attr("src").unwrap_or("");
                        let alt = el_ref.value().attr("alt").unwrap_or("");
                        output.push_str(&format!("![{}]({})\n\n", alt, src));
                    }
                    "table" => { convert_table(el_ref, output); }
                    "pre" => {
                        let text: String = el_ref.text().collect();
                        output.push_str(&format!("```\n{}\n```\n\n", text.trim()));
                    }
                    "blockquote" => {
                        let text: String = el_ref.text().collect();
                        output.push_str(&format!("> {}\n\n", text.trim()));
                    }
                    "script" | "style" | "nav" | "footer" | "noscript" => {
                        // skip entirely
                    }
                    _ => {
                        // container elements: recurse into children
                        for child_el in el_ref.children() {
                            if let Some(child_ref) = scraper::ElementRef::wrap(child_el) {
                                convert_node(child_ref, output);
                            }
                        }
                    }
                }
            }
            scraper::Node::Text(text_node) => {
                let text = text_node.text.trim();
                if !text.is_empty() {
                    output.push_str(text);
                    output.push(' ');
                }
            }
            _ => {}
        }
    }
}
```

**Passo 4 — Remover tags ignoradas ANTES da conversão:**

Usar regex para remover `<script>`, `<style>`, `<nav>`, `<footer>`, `<noscript>` e seus conteúdos do HTML antes de passar para o conversor. Isso evita que o conteúdo dessas tags apareça no Markdown.

**Passo 5 — Integrar no `dump_to_file()`:**

Em `src/dump/mod.rs`, adicionar no pipeline:

```rust
if config.format == "markdown" {
    result = html_to_markdown(&result);
}
```

Adicionar campo `pub format: String` ao `DumpConfig` (default: `"html"`).

**Passo 6 — Adicionar flag `--format` ao DumpArgs:**

```rust
#[arg(long = "format", default_value = "html")]
pub format: String,
```

**Critério de aceite:**
```bash
faf dump --url https://books.toscrape.com/ --output page.md --format markdown
# → page.md contém "# All products | Books to Scrape"
# → Links convertidos para [text](url)
# → Listas convertidas para - item
# → Scripts e nav removidos
# → Arquivo abre em qualquer renderizador Markdown
```

**Testes (adicionar em tests/m7_test.rs):**
1. `test_markdown_h1` — `<h1>Title</h1>` → `"# Title\n\n"`
2. `test_markdown_link` — `<a href="url">text</a>` → `"[text](url)"`
3. `test_markdown_list` — `<ul><li>A</li><li>B</li></ul>` → `"- A\n- B\n"`
4. `test_markdown_skips_script` — `<script>code</script>` → não aparece no output
5. `test_markdown_image` — `<img src="x.png" alt="X">` → `"![X](x.png)"`

---

### 🔴 T091 — `dump --readability`

**Arquivo a criar:** `src/dump/readability.rs`

**O que faz:** Extrai APENAS o conteúdo principal da página, removendo navegação, sidebars, footers, anúncios e outros elementos não-conteúdo. Similar ao Firefox Reader View.

**Flags novas:**
```bash
faf dump --url https://site.com --output article.md --format markdown --readability
```

**Implementação detalhada (passo a passo):**

**Passo 1 — Criar `src/dump/readability.rs` com a função pública:**

```rust
pub fn extract_main_content(html: &str) -> String
```

Recebe HTML string, retorna HTML string contendo apenas o conteúdo principal.

**Passo 2 — Algoritmo de extração (heurístico, sem ML):**

Usar o algoritmo baseado em **densidade de texto** (text-to-tag ratio):

1. Parsear o HTML com `scraper::Html::parse_document()`
2. Para cada elemento block-level (`<div>`, `<article>`, `<section>`, `<main>`):
   a. Calcular: `score = text_length / (tag_count + 1)`
   b. Penalizar elementos com classes/id comuns de ruído:
      - Classes: `nav`, `menu`, `sidebar`, `footer`, `header`, `ad`, `advertisement`, `banner`, `widget`, `comment`, `related`, `social`, `share`
      - IDs: mesmo padrão
      - Tags: `<nav>`, `<footer>`, `<aside>`, `<header>`
   c. Penalizar elementos com muitos links por parágrafo (> 30% de links = lista de navegação)
   d. Bônus para elementos com tags de conteúdo: `<article>`, `<main>`, `role="main"`, `<p>`, `<h1>`-`<h6>`
3. Selecionar o elemento com maior score como "conteúdo principal"
4. Retornar o inner HTML desse elemento (ou HTML completo se nenhum candidato claro)

**Passo 3 — Estrutura de scoring:**

```rust
struct ContentScore {
    text_len: usize,       // total de caracteres de texto
    tag_count: usize,      // número de tags filhas
    link_count: usize,     // número de links <a>
    paragraph_count: usize, // número de <p>
    heading_count: usize,   // número de <h1>-<h6>
}

fn score_element(element: scraper::ElementRef) -> f64 {
    let mut score = ContentScore::default();
    count_stats(element, &mut score);

    let density = score.text_len as f64 / (score.tag_count as f64 + 1.0);
    let link_ratio = score.link_count as f64 / (score.tag_count as f64 + 1.0);

    let mut final_score = density;

    // Penalizar muitos links (provavelmente nav)
    if link_ratio > 0.3 {
        final_score *= 0.3;
    }

    // Bônus para elementos de conteúdo
    if score.heading_count > 0 {
        final_score *= 1.5;
    }
    if score.paragraph_count > 3 {
        final_score *= 1.3;
    }

    // Bônus para tags semânticas
    let tag = element.value().name().to_lowercase();
    if tag == "article" || tag == "main" {
        final_score *= 2.0;
    }

    // Penalizar classes/id de ruído
    let noise_patterns = ["nav", "menu", "sidebar", "footer", "header",
                          "ad", "advertisement", "banner", "widget",
                          "comment", "related", "social", "share"];
    if let Some(id) = element.value().id() {
        for pattern in &noise_patterns {
            if id.to_lowercase().contains(pattern) {
                final_score *= 0.2;
                break;
            }
        }
    }
    // Mesma lógica para classes...

    final_score
}
```

**Passo 4 — Integrar no pipeline:**

Em `src/dump/mod.rs`, adicionar ANTES da conversão para Markdown:

```rust
if config.readability {
    result = extract_main_content(&result);
}
```

Adicionar campo `pub readability: bool` ao `DumpConfig`.

**Passo 5 — Adicionar flag `--readability` ao DumpArgs:**

```rust
#[arg(long = "readability")]
pub readability: bool,
```

**Critério de aceite:**
```bash
faf dump --url https://books.toscrape.com/ --output article.md --format markdown --readability
# → article.md contém APENAS os livros, sem navbar/sidebar/footer
# → Menos de 50% do tamanho do dump sem --readability
# → Estrutura de parágrafos preservada
```

**Testes (adicionar em tests/m7_test.rs):**
1. `test_readability_removes_nav` — HTML com `<nav>` → output sem conteúdo do nav
2. `test_readability_keeps_article` — HTML com `<article>` → conteúdo preservado
3. `test_readability_scores_text_density` — `<div>` com muito texto vence `<div>` com muitos links
4. `test_readability_finds_main_content` — página com sidebar + conteúdo → retorna só conteúdo

---

### 🟡 T092 — Extração de Dados Estruturados

**Arquivo a criar:** `src/dump/structured_data.rs`

**O que faz:** Extrai dados estruturados da página (JSON-LD, microdata, schema.org, Open Graph, meta tags) e os retorna como JSON.

**Flags novas:**
```bash
faf dump --url https://site.com --output data.json --format json --structured-data
```

**Implementação detalhada (passo a passo):**

**Passo 1 — Criar `src/dump/structured_data.rs`:**

```rust
use serde_json::{json, Value};

pub fn extract_structured_data(html: &str) -> Value
```

Retorna um objeto JSON com todos os dados estruturados encontrados.

**Passo 2 — Extrair de 4 fontes:**

**2a. JSON-LD (`<script type="application/ld+json">`)**

Usar `scraper::Selector::parse("script[type=\"application/ld+json\"]")` para encontrar elementos. Para cada um, extrair o texto e fazer `serde_json::from_str()`. Se falhar o parse, logar warning e pular. Acumular em um array `json_ld`.

```rust
let selector = scraper::Selector::parse("script[type=\"application/ld+json\"]").unwrap();
let mut json_ld_items = Vec::new();
for el in document.select(&selector) {
    let text: String = el.text().collect();
    if let Ok(value) = serde_json::from_str::<Value>(&text) {
        json_ld_items.push(value);
    }
}
```

**2b. Open Graph (`<meta property="og:*">`)**

Usar `scraper::Selector::parse("meta[property^=\"og:\"]")` para encontrar. Extrair `property` (remover prefixo "og:") e `content`. Montar objeto `{title, description, image, url, type, site_name, locale}`.

**2c. Meta tags básicas**

Usar `scraper::Selector::parse("meta[name]")` para encontrar. Campos: `description`, `keywords`, `author`, `robots`, `viewport`, `generator`.

**2d. Microdata (schema.org via HTML attributes)**

Elementos com `itemscope`, `itemtype`, `itemprop`. Navegar a árvore e construir JSON-LD equivalente.

```rust
// Encontrar elementos com itemscope
let selector = scraper::Selector::parse("[itemscope]").unwrap();
for el in document.select(&selector) {
    let itemtype = el.value().attr("itemtype").unwrap_or("");
    // Extrair itemprop recursivamente
}
```

**Passo 3 — Estrutura do JSON de saída:**

```json
{
  "json_ld": [ /* array de objetos JSON-LD */ ],
  "open_graph": {
    "title": "...",
    "description": "...",
    "image": "...",
    "url": "...",
    "type": "website",
    "site_name": "..."
  },
  "meta": {
    "description": "...",
    "keywords": "...",
    "author": "..."
  },
  "microdata": [ /* array de objetos schema.org */ ]
}
```

**Passo 4 — Integrar no dump_to_file():**

Se `config.structured_data` é true, gerar `result = extract_structured_data(&html).to_string()` e pular o pipeline HTML.

Adicionar campo `pub structured_data: bool` ao `DumpConfig`.

**Passo 5 — Adicionar flag `--structured-data` ao DumpArgs:**

```rust
#[arg(long = "structured-data")]
pub structured_data: bool,
```

**Critério de aceite:**
```bash
faf dump --url https://books.toscrape.com/ --output data.json --format json --structured-data
# → data.json contém open_graph com title e description
# → Campos vazios/null são omitidos (não serializar None)
# → JSON-LD válido se presente na página
```

**Testes (adicionar em tests/m7_test.rs):**
1. `test_structured_data_json_ld` — script[type=ld+json] é extraído
2. `test_structured_data_open_graph` — meta[property="og:title"] → og.title
3. `test_structured_data_meta_tags` — meta[name="description"] → meta.description
4. `test_structured_data_no_data` — página sem dados estruturados → objeto vazio mas válido

---

### 🟡 T093 — `dump --format text`

**Arquivo a modificar:** `src/dump/mod.rs` (função já parcialmente existente via `remove_scripts`)

**O que faz:** Extrai texto visível limpo da página, preservando parágrafos e headings.

**Flags novas:**
```bash
faf dump --url https://site.com --output page.txt --format text
```

**Implementação detalhada (passo a passo):**

**Passo 1 — Criar função `html_to_text()` em `src/dump/mod.rs`:**

```rust
pub fn html_to_text(html: &str) -> String
```

**Passo 2 — Lógica de extração:**

1. Parsear HTML com scraper
2. Percorrer elementos do `<body>` recursivamente
3. Para cada elemento:
   - `<p>`, `<div>`, `<section>`, `<article>`: extrair texto + `\n\n`
   - `<h1>`-`<h6>`: extrair texto + `\n\n` (sem #, texto puro)
   - `<br>`: `\n`
   - `<li>`: `- text\n`
   - `<a>`: `text (url)` — incluir URL entre parênteses
   - `<script>`, `<style>`, `<nav>`, `<footer>`: pular
4. Remover linhas em branco consecutivas (> 1)
5. Remover whitespace no início/fim de cada linha
6. Limitar largura de linha a 80 caracteres (opcional, com flag `--text-width`)

**Passo 3 — Integrar no dump_to_file():**

```rust
if config.format == "text" {
    result = html_to_text(&result);
}
```

**Critério de aceite:**
```bash
faf dump --url https://books.toscrape.com/ --output page.txt --format text
# → page.txt contém texto limpo, parágrafos separados por \n\n
# → Não contém tags HTML
# → Não contém conteúdo de nav/footer
```

**Testes (adicionar em tests/m7_test.rs):**
1. `test_text_output_paragraphs` — `<p>A</p><p>B</p>` → `"A\n\nB\n\n"`
2. `test_text_output_skips_nav` — `<nav>link</nav><p>content</p>` → `"content\n\n"`
3. `test_text_output_headings` — `<h1>Title</h1>` → `"Title\n\n"` (sem #)

---

### 🔴 T094 — Testes M7 + Integração

**Arquivo a criar:** `tests/m7_test.rs`

**Testes de integração (mínimo 12 testes):**

```rust
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use clap::Parser;
use faf_browser::api::commands::{run, Cli};

fn start_server(html: &'static str) -> u16 {
    // servidor TcpListener padrão (copiar de m6_test.rs)
}

// ── T090: Markdown ──
#[tokio::test] async fn test_markdown_h1_conversion() { /* ... */ }
#[tokio::test] async fn test_markdown_link_conversion() { /* ... */ }
#[tokio::test] async fn test_markdown_list_conversion() { /* ... */ }
#[tokio::test] async fn test_markdown_skips_script() { /* ... */ }
#[tokio::test] async fn test_markdown_image() { /* ... */ }

// ── T091: Readability ──
#[tokio::test] async fn test_readability_removes_nav() { /* ... */ }
#[tokio::test] async fn test_readability_preserves_article() { /* ... */ }

// ── T092: Structured Data ──
#[tokio::test] async fn test_structured_data_json_ld() { /* ... */ }
#[tokio::test] async fn test_structured_data_open_graph() { /* ... */ }
#[tokio::test] async fn test_structured_data_meta() { /* ... */ }

// ── T093: Text ──
#[tokio::test] async fn test_text_output_paragraphs() { /* ... */ }
#[tokio::test] async fn test_text_output_skips_nav() { /* ... */ }
```

**Estrutura de cada teste (template EXATO — não desviar):**

```rust
#[tokio::test]
async fn test_markdown_h1_conversion() {
    let html = "<html><body><h1>Hello World</h1></body></html>";
    let port = start_server(html);
    let output = format!("/tmp/faf_test_md_h1_{}.md", std::process::id());

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output", &output,
        "--format", "markdown",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump markdown should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("# Hello World"));
    let _ = std::fs::remove_file(&output);
}
```

**Critério de aceite:**
```bash
cargo test --test m7_test    # 12+ testes passando
cargo test                    # todos os 285+ testes passando
cargo clippy                  # limpo
```

---

## ✅ M8 — Polimento & Robustez (5 tasks · Concluído)

**Objetivo:** Refinar o projeto para padrão profissional: testes confiáveis, build otimizado, experiência de uso fluida. Nenhuma feature nova — apenas qualidade.

---

### 🔴 T095 — Corrigir 7 testes quebrados em `m5_test.rs`

**Problema:** 7 testes em `tests/m5_test.rs` spawnam `cargo run` como subprocesso (`Command::new("cargo").args(["run", ...])`). Isso falha porque o binário compilado não está no PATH e o `cargo run` depende do working directory. Resultado: `failed to spawn cargo run: No such file or directory`.

**Testes afetados:**
- `test_stdin_mode`
- `test_repl_mode`
- `test_repl_json_toggle`
- `test_click_via_stdin`
- `test_scroll_to`
- `test_scroll_by`
- `test_scroll_clamp_negative`

**Solução:** Converter todos de `Command::new("cargo").args(["run", ...])` para chamar `run(Cli::parse_from([...])).await` diretamente, igual aos outros testes já fazem (m4_test, m6_test, m7_test).

**Template de refatoração (ANTES → DEPOIS):**

ANTES:
```rust
#[test]
fn test_scroll_to() {
    let port = start_basic_server();
    let mut child = Command::new("cargo")
        .args(["run", "--", &format!("http://127.0.0.1:{}/", port),
                "--js", "window.scrollTo(0, 500); window.pageYOffset",
                "--no-scripts"])
        .current_dir("/home/hermes/faf-browser")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo run");
    // ... read stdout, assert contains "500"
}
```

DEPOIS:
```rust
#[tokio::test]
async fn test_scroll_to() {
    let port = start_basic_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--js", "window.scrollTo(0, 500); window.pageYOffset",
        "--no-scripts",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "scroll should succeed: {:?}", result);
}
```

**Importante:** Mudar `#[test]` para `#[tokio::test]` nos que usam `async`. Remover imports de `Command` e `Stdio` se não forem mais usados. Verificar se `start_basic_server()` suporta múltiplos requests (alguns testes usam stdin pipe — esses precisam de abordagem diferente).

**Testes com stdin (PRECISAM de abordagem especial):**
- `test_stdin_mode` — usa `child.stdin.write_all(b"...")` + pipe → NÃO pode usar `run()` direto
- `test_repl_mode` — usa `child.stdin.write_all(b"...")` + pipe → NÃO pode usar `run()` direto
- `test_repl_json_toggle` — usa `child.stdin.write_all(b"...")` → NÃO pode usar `run()` direto
- `test_click_via_stdin` — usa `child.stdin.write_all(b"...")` → NÃO pode usar `run()` direto

Para esses 4, a refatoração é diferente: em vez de spawnar `cargo run`, usar o binário compilado diretamente:

```rust
let binary = std::env::current_exe().unwrap(); // usa o binário de teste atual
let mut child = Command::new(binary)
    .args([&format!("http://127.0.0.1:{}/", port), "--stdin", "--no-scripts"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("failed to spawn faf");
```

Isso resolve o problema porque o binário de teste (`cargo test`) é o próprio `faf-browser`.

**Critério de aceite:**
```bash
cargo test --test m5_test   # 19/19 passando (atualmente 12 passam, 7 falham)
cargo test                   # todos os 316+ passando
```

---

### 🟡 T096 — Teste de integração para `--inline-images`

**Problema:** A funcionalidade `--inline-images` tem testes unitários em `src/dump/image_inline.rs` (lógica de download, MIME detection), mas nunca foi validada em teste de integração ponta-a-ponta com um servidor local servindo uma imagem real.

**Solução:** Criar um teste em `tests/m7_test.rs` que:
1. Inicia um servidor TcpListener que serve uma página HTML com `<img src="/img/photo.png">`
2. O mesmo servidor, quando recebe request para `/img/photo.png`, retorna uma imagem PNG real (pode ser um PNG mínimo 1×1 gerado programaticamente)
3. Roda `faf dump --inline-images --output /tmp/test_inline.html`
4. Verifica que o arquivo de saída contém `data:image/png;base64,` (indicando que o src foi substituído por data URI)

**Implementação detalhada:**

**Passo 1 — Criar um PNG válido mínimo programaticamente:**

```rust
fn minimal_png_bytes() -> Vec<u8> {
    // PNG 1x1 pixel vermelho (menor PNG válido possível)
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 pixels
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x00, 0x03, 0x00, 0x01, 0x1A, 0x72, 0x5C,
        0xD4, 0x74, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, // IEND chunk
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}
```

**Passo 2 — Servidor que serve HTML + imagem:**

```rust
fn start_server_with_image() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            
            if request.contains("GET /img/photo.png") {
                let png = minimal_png_bytes();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    png.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
                stream.write_all(&png).unwrap();
            } else {
                let html = r##"<html><body><img src="/img/photo.png" alt="Test"></body></html>"##;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html.len(), html
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    });
    port
}
```

**Passo 3 — Teste:**

```rust
#[tokio::test]
async fn test_inline_images_replaces_src_with_data_uri() {
    let port = start_server_with_image();
    let output = format!("/tmp/faf_test_inline_img_{}.html", std::process::id());

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output", &output,
        "--inline-images",
        "--no-inline-css",
        "--no-scripts",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump with inline-images should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(
        saved.contains("data:image/png;base64,"),
        "Expected data URI for image, got: {}",
        if saved.len() > 500 { &saved[..500] } else { &saved }
    );
    assert!(
        !saved.contains("/img/photo.png"),
        "Original src should be replaced"
    );
    let _ = std::fs::remove_file(&output);
}
```

**Critério de aceite:**
```bash
cargo test --test m7_test test_inline_images   # 1 novo teste passando
```

---

### 🟡 T097 — Output para stdout quando `--output` não especificado

**Problema:** Atualmente `--output` tem default `page.html`. Se o usuário quiser pipe, precisa de um arquivo temporário.

**Solução:** Quando `--output` NÃO é passado explicitamente (ou é string vazia), escrever o resultado em stdout em vez de arquivo. Quando `--output` É passado, comportamento atual (salva em arquivo).

**Implementação detalhada:**

**Passo 1 — Alterar DumpArgs (commands.rs):**

Remover o `default_value = "page.html"` e usar `Option<String>`:

```rust
#[derive(clap::Args, Debug)]
pub struct DumpArgs {
    /// Caminho do arquivo HTML de saída (stdout se omitido)
    #[arg(long = "output")]
    pub output: Option<String>,
    // ... resto igual
}
```

**Passo 2 — Alterar handler (commands.rs, no match Command::Dump):**

```rust
Some(Command::Dump(args)) => {
    let config = crate::dump::DumpConfig {
        // ... igual ...
    };

    let result_html = crate::dump::dump_to_string(&html, &config)?;

    if let Some(ref output_path) = args.output {
        // Salvar em arquivo (comportamento atual)
        std::fs::write(output_path, &result_html)?;
        if format == "json" {
            println!("{}", serde_json::json!({"dump": output_path, "url": url}));
        } else {
            println!("💾 HTML salvo em: {}", output_path);
        }
    } else {
        // Escrever em stdout
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(result_html.as_bytes())?;
        handle.flush()?;
    }
}
```

**Passo 3 — Criar `dump_to_string()` em `src/dump/mod.rs`:**

Mesma lógica de `dump_to_file()` mas retorna a String em vez de escrever em arquivo:

```rust
pub fn dump_to_string(html: &str, config: &DumpConfig) -> anyhow::Result<String> {
    // ... mesmo pipeline de processamento ...
    // return result; em vez de fs::write
}
```

Refatorar `dump_to_file()` para chamar `dump_to_string()` internamente:

```rust
pub fn dump_to_file(html: &str, config: &DumpConfig, output_path: &str) -> anyhow::Result<()> {
    let result = dump_to_string(html, config)?;
    // ... create parent dirs, write file ...
    Ok(())
}
```

**Critério de aceite:**
```bash
faf dump --url https://httpbin.org/html --format markdown | head -5
# → saída direta no terminal (sem arquivo)

faf dump --url https://httpbin.org/html --output page.html
# → comportamento atual mantido (salva em arquivo)

cargo test  # todos passando
```

---

### 🟡 T098 — Otimizar profiles de build no `Cargo.toml`

**Problema:** O debug build (`cargo build` ou `cargo test`) demora ~20s+ porque compila todas as dependências com debug info completo e sem otimizações. O release build demora ~1min com LTO.

**Solução:** Adicionar profiles customizados no `Cargo.toml` para acelerar o ciclo de desenvolvimento:

**Passo 1 — Adicionar profile `dev` otimizado:**

```toml
[profile.dev]
opt-level = 1           # otimizações leves (melhora ~30% performance de teste)
debug = 1               # debug info reduzido (line tables only)
incremental = true      # compilação incremental

# Override para crates pesados: compilar com mais otimização
[profile.dev.package."*"]
opt-level = 1

# Compilar dependências com opt-level 2 pra reduzir tempo de link
[profile.dev.package.rquickjs]
opt-level = 2
[profile.dev.package.reqwest]
opt-level = 2
[profile.dev.package.tokio]
opt-level = 2
```

**Passo 2 — Manter profile `release` atual:**

```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

**Passo 3 — (Opcional) Adicionar profile `release-fast` sem LTO:**

```toml
[profile.release-fast]
inherits = "release"
lto = false             # sem LTO → build ~2x mais rápido
codegen-units = 16      # paralelizar codegen
strip = true
```

Uso: `cargo build --profile release-fast`

**Critério de aceite:**
```bash
cargo clean && time cargo build          # mais rápido que antes
cargo clean && time cargo test --no-run  # mais rápido que antes
cargo build --release                    # ainda funciona, mesmo tempo (~1min)
```

---

### 🟡 T099 — Atualizar badges e documentação final

**O que fazer:** Após T095-T098 concluídos, atualizar:

1. **README.md badges:** número de testes atualizado (316+)
2. **README.md roadmap:** marcar M8 como concluído
3. **TASKS.md header:** status atualizado
4. **TASKS.md summary table:** M8 marcado como concluído

---

## 📊 Resumo de Milestones

| Milestone | Tasks | Status |
|-----------|-------|--------|
| M1 — Core Engine | 12 | ✅ Concluído |
| M2 — CSS Engine | 8 | ✅ Concluído |
| M2.5 — Polimento CLI | 8 | ✅ Concluído |
| M3 — JavaScript Engine | 10 | ✅ Concluído |
| M4 — Sessão, Interação & Pipeline | 8 | ✅ Concluído |
| M4.5 — Refinamentos Pós-M4 | 3 | ✅ Concluído |
| M5 — Interação com Páginas | 5 | ✅ Concluído |
| M6 — Dump HTML Autocontido | 8 | ✅ Concluído |
| M7 — LLM-Ready Output | 5 | ✅ Concluído |
| **M8 — Polimento & Robustez** | **5** | **✅ Concluído** |
| **Total** | **72** | **72 concluídas · 0 pendentes** |

---

## 📋 Como Continuar (M7)

**Arquivos a criar:**
- `src/dump/markdown.rs` — T090: conversor HTML → Markdown
- `src/dump/readability.rs` — T091: extração de conteúdo principal
- `src/dump/structured_data.rs` — T092: extração JSON-LD/OG/microdata
- `tests/m7_test.rs` — T094: 12+ testes de integração

**Arquivos a modificar:**
- `src/dump/mod.rs` — adicionar campos `format`, `readability`, `structured_data` ao `DumpConfig`; integrar novos conversores no pipeline
- `src/api/commands.rs` — adicionar flags `--format`, `--readability`, `--structured-data` ao `DumpArgs`

**Ordem de execução recomendada:**
```
T093 (text) → T090 (markdown) → T091 (readability) → T092 (structured data) → T094 (tests)
```

**Critério de sucesso:**
```bash
faf dump --url https://books.toscrape.com/ --output page.md --format markdown --readability
# → page.md com conteúdo limpo, sem nav/sidebar, Markdown válido

faf dump --url https://site.com --output data.json --structured-data
# → JSON com JSON-LD, Open Graph, meta tags extraídos

cargo test   # 285+ passando
cargo clippy # limpo
```

---

## 🧪 Testes

```bash
cargo test           # Rodar todos os testes
cargo clippy         # Verificar lint
cargo build --release # Build release
```

---

## 🏗️ Arquitetura

```
faf-browser/
├── src/
│   ├── main.rs              # Entry point (#[tokio::main])
│   ├── lib.rs               # Módulos públicos
│   ├── api/
│   │   ├── commands.rs      # CLI (clap), execução de comandos
│   │   ├── output.rs        # Formatadores: JSON, CSV, JSONL, texto
│   │   └── filter.rs        # Sistema de filtros
│   ├── http/
│   │   ├── client.rs        # HTTP client (reqwest, proxy, timeout)
│   │   ├── cache.rs         # Cache de responses em disco
│   │   └── cookies.rs       # Cookies formato Netscape
│   ├── dom/
│   │   └── parser.rs        # DOM tree (scraper/html5ever)
│   ├── css/
│   │   ├── parser.rs        # Parser CSS (cssparser)
│   │   ├── selector.rs      # Selector matching + especificidade
│   │   ├── style.rs         # Computed styles + cascata
│   │   ├── color.rs         # Cores (hex, rgb, rgba, named)
│   │   ├── font.rs          # Fontes (family, size, weight)
│   │   └── layout.rs        # Box model (margin, padding)
│   ├── js/
│   │   ├── engine.rs        # Runtime QuickJS (rquickjs)
│   │   ├── dom_bridge.rs    # Ponte DOM ↔ JS
│   │   └── fetch_bridge.rs  # Ponte fetch ↔ reqwest
│   ├── dump/                # NOVO — M6
│   │   ├── mod.rs           # DumpConfig + dump_to_file()
│   │   ├── css_inline.rs    # Inline de CSS externo
│   │   ├── image_inline.rs  # Imagens para base64
│   │   ├── url_resolver.rs  # URLs relativas → absolutas
│   │   └── html_writer.rs   # Serialização DOM → HTML
│   └── utils/
│       ├── config.rs        # Configurações
│       └── error.rs         # Tipos de erro
├── tests/
│   ├── m2_test.rs
│   ├── m3_test.rs
│   ├── m4_test.rs
│   ├── m5_test.rs
│   └── m6_test.rs           # NOVO — testes de dump
├── Cargo.toml
└── README.md
```

### Stack

| Componente | Crate | Função |
|---|---|---|
| HTTP | `reqwest` + `tokio` | Requests assíncronos, proxy SOCKS5, cookies |
| HTML/DOM | `scraper` (html5ever) | Parse e query na DOM tree |
| CSS Parser | `cssparser` | Tokenização e parsing de folhas de estilo |
| CSS Selectors | `selectors` | Matching e especificidade |
| CLI | `clap` (derive) | Interface de linha de comando |
| Serialização | `serde` + `serde_json` | Output JSON, CSV, JSONL |
| JS Runtime | `rquickjs` (QuickJS) | Runtime JavaScript embarcado |
| Regex | `regex` | Filtros com expressões regulares |
| Cache | `sha2`, `hex` | SHA256 para cache de responses |
