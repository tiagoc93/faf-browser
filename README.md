<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/status-active-2ea043?style=for-the-badge">
  <img alt="Status" src="https://img.shields.io/badge/status-active-2ea043?style=for-the-badge">
</picture>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/Rust-Edition_2024-orange?style=for-the-badge&logo=rust">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Edition_2024-orange?style=for-the-badge&logo=rust">
</picture>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/test-287_passed-green?style=for-the-badge">
  <img alt="Tests" src="https://img.shields.io/badge/test-297_passed-green?style=for-the-badge">
</picture>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge">
</picture>

<br>

# ⚡ FAF BROWSER

### Fast As Fuck — Navegador Headless 100% Rust

> *"O Chrome pesa 500MB de RAM? A gente faz em 50MB."*

FAF Browser é um navegador headless minimalista e agressivamente rápido, construído do zero em Rust. Sem Electron, sem Chromium embutido, sem overhead. Um binário único que faz scraping, crawleamento, **execução de JavaScript** e inspeção de páginas web com performance nativa.

Renderizado pelo CSS Engine próprio. Runtime **QuickJS** embarcado para execução de JavaScript. Crawleador multithread. Tudo em **menos de 1MB** de binário.

---

## ✨ Features

| Feature | Status |
|---------|--------|
| 🌐 **HTTP Client** — fetch páginas com proxy, timeout, headers customizáveis | ✅ |
| 📄 **Parser HTML** — DOM tree completa com html5ever | ✅ |
| 🔍 **Query CSS** — seletores `h1`, `.class`, `#id`, `div span`, combinadores | ✅ |
| 🎨 **CSS Engine** — parser, cascata, especificidade (inline > ID > class > tag) | ✅ |
| 📐 **Box Model** — width, height, margin, padding computados | ✅ |
| 🖌️ **Cores** — hex, rgb, rgba, 19 cores nomeadas | ✅ |
| 🔤 **Fontes** — family, size (px/em/rem/%), weight (100-900) | ✅ |
| 📊 **Computed Styles** — estilos reais da página (parseia `<style>` e `<link>` automagicamente) | ✅ |
| 📸 **Screenshot** — render PNG com texto real e CSS via `ab_glyph` + `tiny-skia` | ✅ **M7** |
| 📝 **Texto Real** — renderização com ab_glyph (100% Rust, sem freetype) | ✅ **M7** |
| 📐 **position: absolute + fixed** — containing block, viewport fallback | ✅ **M7** |
| 📏 **Text Rendering** — ab_glyph real (não heurística), text-align, line-height | ✅ **M7** |
| 🫧 **Overflow** — hidden com clip, visible, scroll | ✅ **M7** |
| ⚡ **Font Cache** — carrega uma vez, reusa em todos os nós | ✅ **M7** |
| 🌐 **CSS Externo** — baixa `<link rel="stylesheet">` automaticamente | ✅ **M7** |
| 🔗 **URLs Relativas** — resolve imagens e CSS contra base URL | ✅ **M7** |
| 🌳 **Layout Tree** — DOM → árvore visual block/inline com herança de estilo | ✅ **M6** |
| 📐 **Layout Engine** — block flow com margin collapsing, inline flow com text wrap | ✅ **M6** |
| 🎨 **Backgrounds** — cores de fundo nos elementos + body viewport | ✅ **M6** |
| 🧱 **Bordas CSS** — border-width/color/style com shorthand | ✅ **M6** |
| 🖼️ **Imagens `<img>`** — decode JPEG/PNG/WebP + fallback placeholder | ✅ **M6** |
| 📌 **Positioning** — relative + z-index ordering | ✅ **M6** |
| 🔎 **Filtros** — `--filter "text~=Python"`, `--filter "href^=https"`, regex | ✅ |
| 🎯 **Extração seletiva** — `--get "text, href"` — só os campos que importam | ✅ |
| 🕷️ **Crawler** — `faf follow` — siga links, visite páginas, extraia dados | ✅ |
| 📋 **Múltiplos formatos** — texto, JSON, JSONL, CSV | ✅ |
| 🔌 **Proxy** — HTTP e SOCKS5 | ✅ |
| 🟨 **JavaScript Engine** — QuickJS `rquickjs` com DOM bridge, timers, fetch | ✅ **NOVO** |
| 🍪 **Cookies** — `--cookies` e `--cookies-jar` — sessão persistente Netscape | ✅ **NOVO** |
| ⏳ **Wait** — `faf wait \".produto\"` — aguarda elemento carregar no DOM | ✅ **NOVO** |
| 💻 **REPL** — `faf repl --url <url>` e `--stdin` para pipes | ✅ **NOVO** |
| 🐌 **Delay** — `--delay` e `--random-delay` entre requests | ✅ **NOVO** |
| 🔁 **Retry** — `--retries N` com exponential backoff + 429 handling | ✅ **NOVO** |
| 📋 **HTTP Info** — `--show-headers` e `--show-status` na resposta | ✅ **NOVO** |
| 💾 **Cache** — `--cache .cache` com TTL configurável e SHA256 | ✅ **NOVO** |
| 🔗 **Fetch API bridge** — `fetch()` no JS chama o reqwest do Rust | ✅ **NOVO** |
| ⏱️ **setTimeout / setInterval** — timers integrados com event loop tokio | ✅ **NOVO** |
| 📜 **Script tags** — execução automática de `<script>` inline e externo | ✅ **NOVO** |
| 📦 **Binário único** — ~1MB, zero dependências runtime | ✅ |
| 🧪 **297 testes** — unitários + integração em sites reais | ✅ |

---

## 🚀 Quick Start

### Instalação

```bash
# Clone e compile
git clone https://github.com/tiagoc93/faf-browser.git
cd faf-browser
cargo build --release

# O binário estará em ./target/release/faf-browser
sudo cp ./target/release/faf-browser /usr/local/bin/faf
```

### Pré-requisitos

- Rust (edition 2024) — instale via [rustup](https://rustup.rs/)
- Linux x86_64 (suporte a mais plataformas em breve)

---

## 📖 Uso

### 🔥 Novidades M4 — Crawler Profissional

```bash
# Esperar elemento carregar (útil para SPAs)
faf wait ".produto" --url https://loja.com --timeout 10

# REPL interativo
faf repl --url https://books.toscrape.com/
> document.title
> document.querySelectorAll(".price_color").length

# Pipe de JS via --stdin
echo 'document.querySelector("h1").textContent' | faf --stdin --url https://books.toscrape.com/

# Crawler com delay entre requests
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3, .price_color" \
  --max 10 --delay 1000 --random-delay 500 1500

# Retry automático em falhas
faf https://site-instavel.com --retries 3 --retry-delay 2000 --show-status

# Cache em disco
faf https://site.com --cache .faf-cache --show-status
# → Primeira: cache MISS, Segunda: cache HIT

# Headers e status na resposta
faf https://httpbin.org/get --show-headers --show-status --json

# Sessão com cookies persistente
faf https://site.com/login --cookies session.txt --cookies-jar session.txt
faf https://site.com/dados --cookies session.txt --show-status
```

### Execução de JavaScript 🔥

```bash
# Ler título da página
faf https://books.toscrape.com/ --js "document.title"
# → "All products | Books to Scrape - Sandbox"

# Query no DOM
faf https://books.toscrape.com/ --js "document.querySelectorAll('h3').length"
# → 20

# Fetch API real (chama reqwest do Rust!)
faf https://httpbin.org/get --js "fetch('https://httpbin.org/get').text()" --json
# → { "args": {}, "headers": {...}, "origin": "...", "url": "..." }

# Executar arquivo .js
faf https://site.com --js-file script.js

# Desabilitar scripts da página
faf https://site.com --js "document.title" --no-scripts

# Timeout customizado para JS
faf https://site.com --js "while(true){}" --js-timeout 2
# → "JavaScript execution timed out after 2s"
```

### Extração completa de página

```bash
faf https://books.toscrape.com/
```

```
📄 Página: https://books.toscrape.com/
📌 Título: All products | Books to Scrape - Sandbox
🔗 Links: 94
🖼️ Imagens: 20
📋 Metadados: 4

📝 Texto:
Books to Scrape We love being scraped! Home All products ...
```

### Query CSS com estilos computados

```bash
faf https://books.toscrape.com/ query "h1"
```

```
🔍 Query 'h1': 1 resultado(s)
  [1.] <h1> texto: All products
      🎨 color: inherit | bg: transparent | font-size: 29.96px | font-family: serif | display: block
```

> O CSS é extraído automaticamente da página (`<style>` + `<link rel="stylesheet">`). Use `--no-page-css` para desabilitar.

### Subcomandos

```bash
faf https://site.com links           # Todos os links da página
faf https://site.com images          # Todas as imagens
faf https://site.com metadata        # Open Graph, title, description
faf https://site.com query "h2"      # Query CSS customizada
```

### CSS customizado

```bash
# CSS inline
faf https://site.com --css "h1 { color: red; font-size: 24px; }" query "h1"

# Arquivo CSS
faf https://site.com --css style.css query ".card"
```

### Filtros 🔥

```bash
# Filtro por texto (substring)
faf https://books.toscrape.com/ query "a" --filter "text~=Sapiens"

# Filtro por atributo
faf https://books.toscrape.com/ query "a" --filter "href^=https" --get "text, href"

# Filtro por regex
faf https://site.com query "img" --filter "alt=.+"

# Múltiplos filtros (AND)
faf https://site.com query "div" --filter "class~=product" --filter "text!=."

# Operadores disponíveis:
#   !~=  negative substring match (case insensitive)
#   !^=  does NOT start with
#   !$=  does NOT end with
#   ~=  substring match (case insensitive)
#   ==  exact match
#   !=  negated exact match
#   ^=  starts with
#   $=  ends with
#   =   regex (auto-detect) ou substring
```

### Extração seletiva de campos

```bash
# Campos do elemento
faf https://site.com query "a" --get "text, href"

# Campos do computed style
faf https://site.com --css "h1 { color: red; }" query "h1" --get "text, color, font-size"

# Campos disponíveis: tag, id, classes, text, html, href, src, alt,
#                     color, bg, font-size, font-family, display
```

### Crawleamento com `follow` 🕷️

```bash
# Crawlear produtos
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3, .price_color" \
  --max 5 \
  --json

# Crawlear com filtro + campos específicos
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3, .price_color" \
  --max 10 \
  --concurrency 5 \
  --get "text" \
  --format csv
```

### Formatos de saída

```bash
# JSON pretty
faf https://site.com query "h1" --json

# JSONL (1 objeto por linha — pipe-friendly)
faf https://site.com query "a" --get "text, href" --format jsonl | head -5

# CSV
faf https://site.com query "a" --get "text, href" --format csv

# CSV com follow
faf https://site.com follow "a" --max 3 --format csv
```

### JSON

```bash
faf https://books.toscrape.com/ --json
```

```json
{
  "url": "https://books.toscrape.com/",
  "title": "All products | Books to Scrape - Sandbox",
  "links": [
    ["Books to Scrape", "index.html"],
    ["Travel", "catalogue/category/books/travel_2/index.html"]
  ],
  "images": [
    ["A Light in the Attic", "media/cache/2c/da/...jpg"]
  ],
  "metadata": {
    "description": "",
    "viewport": "width=device-width"
  },
  "text": "Books to Scrape We love being scraped!..."
}
```

### Outras opções

```bash
# Proxy (HTTP ou SOCKS5)
faf https://site.com --proxy socks5://localhost:9050

# Timeout customizado
faf https://site.com --timeout 60

# User-Agent customizado
faf https://site.com --user-agent "FAFBrowser/1.0"

# Modo verboso (logs)
faf https://site.com -v

# Desabilitar CSS da página
faf https://site.com --no-page-css query "h1"

# Desabilitar execução de scripts da página
faf https://site.com --js "document.title" --no-scripts
```

---

## 🏗️ Arquitetura

```
faf-browser/
├── src/
│   ├── main.rs              # Entry point (#[tokio::main])
│   ├── lib.rs               # Módulos públicos
│   ├── api/
│   │   ├── mod.rs
│   │   ├── commands.rs      # CLI (clap), execução de comandos
│   │   ├── output.rs        # Formatadores: JSON, CSV, JSONL, texto
│   │   └── filter.rs        # Sistema de filtros (text match, regex, attr)
│   ├── http/
│   │   ├── mod.rs
│   │   └── client.rs        # HTTP client (reqwest, proxy, timeout)
│   ├── dom/
│   │   ├── mod.rs
│   │   └── parser.rs        # DOM tree (scraper/html5ever)
│   ├── css/
│   │   ├── mod.rs
│   │   ├── parser.rs        # Parser CSS (cssparser)
│   │   ├── selector.rs      # Selector matching + especificidade
│   │   ├── style.rs         # Computed styles + cascata
│   │   ├── color.rs         # Cores (hex, rgb, rgba, named)
│   │   ├── font.rs          # Fontes (family, size, weight)
│   │   └── layout.rs        # Box model (margin, padding, width, height)
│   ├── js/
│   │   ├── mod.rs
│   │   ├── engine.rs        # Runtime QuickJS (rquickjs)
│   │   ├── dom_bridge.rs    # Ponte DOM ↔ JS (document.querySelector, etc)
│   │   └── fetch_bridge.rs  # Ponte fetch ↔ reqwest (HTTP do JS)
│   ├── render/
│   │   ├── mod.rs
│   │   └── screenshot.rs    # Renderização PNG (tiny-skia + ab_glyph)
│   └── utils/
│       ├── mod.rs
│       ├── config.rs        # Configurações (retry, cache, cookies, etc)
│       └── error.rs         # Tipos de erro
├── tests/
│   ├── fixtures/            # Dados de teste
│   ├── m2_test.rs           # Testes de integração M2 (CSS)
│   ├── m3_test.rs           # Testes de integração M3 (JS)
│   └── m4_test.rs           # Testes de integração M4 (cookies, wait, cache, retry, etc)
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
| CSS Layout | `tiny-skia` | Box model, formas e render 2D |
| **Renderização** | **`ab_glyph`** | **Rasterização de texto (Rust puro, sem freetype)** |
| CLI | `clap` (derive) | Interface de linha de comando |
| Serialização | `serde` + `serde_json` | Output JSON, CSV, JSONL |
| JS Runtime | `rquickjs` (QuickJS) | Runtime JavaScript embarcado |
| Regex | `regex` | Filtros com expressões regulares |

---

## 📊 Comparação

| Feature | BeautifulSoup | Playwright | FAF Browser |
|---------|:---:|:---:|:---:|
| **Velocidade** | 🐌 Python | 🟡 Node.js | 🚀 Rust nativo |
| **Binário único** | ❌ | ❌ | ✅ ~1MB |
| **CLI nativa** | ❌ precisa script | ❌ precisa script | ✅ 1 comando |
| **CSS Selectors** | ✅ `select()` | ✅ `page.$()` | ✅ `query()` |
| **Computed Styles** | ❌ | ✅ | ✅ próprio engine |
| **CSS da página** | ❌ | ✅ | ✅ automático |
| **Filtros** | ✅ `re.compile` | ✅ `filter()` | ✅ `--filter` |
| **Crawler embutido** | ❌ | ❌ | ✅ `follow` |
| **JSONL/CSV** | ❌ manual | ❌ manual | ✅ nativo |
| **JavaScript** | ❌ | ✅ | ✅ **QuickJS** |
| **DOM bridge JS** | ❌ | ✅ | ✅ `document.querySelector` |
| **Fetch via JS** | ❌ | ✅ | ✅ `fetch()` → reqwest |
| **Scripts da página** | ❌ | ✅ | ✅ `<script>` execução |
| **Screenshots** | ❌ | ✅ | ✅ **ab_glyph + tiny-skia** |
| **RAM (página média)** | ~50MB | ~150MB | ~5MB |
| **Tempo 1ª query** | ~2s | ~3s | ~0.3s |
| **Cookies persistente** | ✅ `requests.Session` | ✅ `context.cookies()` | ✅ `--cookies` + `--cookies-jar` |
| **Wait/Timeout** | ❌ `time.sleep()` | ✅ `page.wait_for_selector()` | ✅ `faf wait \".sel\"` |
| **Retry automático** | ❌ manual | ❌ manual | ✅ `--retries N` + backoff |
| **Cache em disco** | ❌ manual | ❌ manual | ✅ `--cache .cache` |
| **REPL/Pipe** | ❌ | ❌ | ✅ `faf repl` + `--stdin` |

---

## 🛣️ Roadmap

### ✅ M1 — Core Engine (Concluído)
- [x] HTTP Client com proxy e timeout
- [x] Parser HTML → DOM tree
- [x] CLI com clap
- [x] Output JSON

### ✅ M2 — CSS Engine (Concluído)
- [x] Parser CSS (cssparser)
- [x] Selector matching + especificidade
- [x] Computed styles + cascata (inline > ID > class > tag)
- [x] Box model, cores, fontes
- [x] Estilos automáticos da página (`<style>` + `<link>`)

### ✅ M2.5 — Extração Avançada (Concluído)
- [x] `--filter` com regex e attribute match
- [x] `--get` para campos específicos
- [x] `follow` subcomando — crawler multithread
- [x] `--format csv|jsonl`
- [x] 180 testes, 0 falhas

### ✅ M3 — JavaScript Engine (Concluído)
- [x] Runtime QuickJS embarcado (`rquickjs`)
- [x] Bridge DOM ↔ JS: `document.getElementById`, `querySelector`
- [x] `setTimeout` / `setInterval` com event loop tokio
- [x] `fetch()` API — chamadas HTTP reais via reqwest
- [x] Timeout de execução JS (proteção contra loop infinito)
- [x] `console.log/warn/error` → Rust logger
- [x] Error handling com stack traces legíveis
- [x] Execução de `<script>` tags inline + externas
- [x] CLI: `faf --js "document.title"` e `--js-file`
- [x] 266 testes, 0 falhas

### ✅ M4 — Ferramentas de Crawler Profissional (Concluído)
- [x] 🍪 **Cookies** — `--cookies` e `--cookies-jar` com formato Netscape, sessão persistente
- [x] ⏳ **Wait** — `faf wait \".produto\"` aguarda elemento carregar com timeout e polling
- [x] 💻 **REPL** — `faf repl --url <url>` modo interativo + `--stdin` para pipes
- [x] 🐌 **Delay** — `--delay N` e `--random-delay MIN MAX` no `follow`
- [x] 🔁 **Retry** — `--retries N` com exponential backoff (500, 429, timeout)
- [x] 📋 **HTTP Info** — `--show-headers` e `--show-status` na resposta
- [x] 💾 **Cache** — `--cache .faf-cache` com SHA256 + TTL configurável
- [x] 🧪 266 testes, 0 falhas

### ✅ M5 — Interação com Páginas (Concluído 🎉)

| Task | Feature | Status |
|------|---------|--------|
| **T039** | **Click** — `dispatchEvent` via JS | ✅ |
| **T040** | **Formulários** — fill + select + submit | ✅ |
| **T041** | **Screenshot** — render PNG via tiny-skia + ab_glyph | ✅ |
| **T042** | **Watch** — monitorar mudanças com polling | ✅ |
| **T043** | **Scroll** — scrollTo/scrollBy/scrollIntoView | ✅ |
| **T044** | **Testes M5** — 9 testes de integração | ✅ |

#### 🔧 Fix Crítico no M5 — Screenshot com Texto Real

O screenshot do M5 estava gerando **tela em branco** devido a dois bugs:

| Bug | Causa | Solução |
|-----|-------|---------|
| **font-kit bug no Linux** | Loader freetype tem `// TODO: woefully incomplete` — glifos retornam alpha=0 | Migrado p/ `ab_glyph` (100% Rust, zero C) |
| **Coordenadas relativas no draw()** | `outline.draw()` fornece coordenadas RELATIVAS ao bbox, não absolutas | Soma de `px_bounds.min` |

**Antes:** 172 pixels de texto, apenas 14 linhas (y=0-13)
**Agora:** 1.122 pixels de texto preto, **304 linhas (y=3 a 799) — 38% da página**

```
📦 61 tasks · ✅ 61 concluídas · MVP Completo! 🎉
🧪 287 testes · 0 falhas · clippy limpo · build 0 warnings
```

|   ✅ Concluído 8/8 tasks  •  287 testes  •  clippy limpo  •  ~300KB binário  |
|:--------------------------------------------------------------------------:|

### ✅ M6 — Layout Engine (Concluído 🎉)

O FAF Browser agora tem um motor de layout PRÓPRIO com árvore visual, fluxo block/inline, quebra de texto, margin collapsing, cores de fundo, bordas, imagens e posicionamento.

#### Antes e Depois

| Aspecto | M5 (lista plana) | M6 (árvore visual) |
|---------|:----------------:|:------------------:|
| Layout | Lista plana, x=0 | Árvore block/inline |
| Text wrap | ❌ Tudo na mesma linha | ✅ Quebra automática |
| Backgrounds | Parcial | ✅ Herdado do CSS |
| Bordas | ❌ | ✅ solid |
| Imagens | ❌ | ✅ decode + render |
| Positioning | ❌ | ✅ relative + z-index |

| Task | Feature | Status |
|------|---------|:------:|
| **T045** | **Layout Tree** — DOM → árvore visual block/inline via scraper | ✅ |
| **T046** | **Inline Flow** — texto em linha com quebra automática (text-wrap) | ✅ |
| **T047** | **Block Flow** — empilhamento vertical com margin collapsing | ✅ |
| **T048** | **Background rendering** — cores de fundo reais | ✅ |
| **T049** | **Bordas CSS** — border-width, border-color, solid | ✅ |
| **T050** | **Imagens `<img>`** — decode (image crate) + fallback | ✅ |
| **T051** | **Positioning** — relative + z-index | ✅ |
| **T052** | **Testes M6** — 9 testes de integração visual | ✅ |

#### Commits M6

| Commit | O que fez |
|--------|-----------|
| `71db35f` | T045 — layout tree (VisualNode, build_layout_tree) |
| `d4b4193` | T045-T048 — layout engine + screenshot árvore + 711 linhas |
| `908c616` | T049-T050 — bordas CSS + imagens + 585 linhas |
| `1f15905` | T051-T052 — relative positioning + z-index + testes |

#### Stack M6 (adicionado)

| Crate | Versão | Função |
|-------|--------|--------|
| `image` | 0.25 | Decodificação JPEG/PNG/WebP |
| — | — | Layout engine 100% std (algoritmos próprios) |

---

### ✅ M7 — Playwright Parity + Polish (Concluído 🎉)

Fechar as lacunas restantes pra screenshot fiel: absolute positioning, text rendering real, overflow, performance.

| Task | Feature | Status |
|------|---------|:------:|
| **T053** | **position: absolute + fixed** — containing block, viewport | ✅ |
| **T054** | **Text Rendering Accuracy** — ab_glyph real, text-align, line-height, font-weight | ✅ |
| **T055** | **Overflow Handling** — hidden com clip, visible | ✅ |
| **T056** | **Performance & Font Cache** — cache de fontes, skip viewport, benchmark | ✅ |
| **T057** | **Testes M7** — 10 testes de integração | ✅ |
| **T058** | **CSS Externo + URLs Relativas** — `<link>` stylesheets + base URL | ✅ |

```bash
📦 6 tasks · ✅ 6 concluídas · 297 testes · 0 falhas
🎯 Screenshot fiel, < 500ms, 227KB CSS externo carregado
```

Veja `TASKS_M7.md` para specs detalhadas.

---

### 📋 M8 — Layout Parity (float, inline-block, flex) (Planejado)

Fechar o gap de layout engine que impede renderização de páginas com grid/flex/float.

| Task | Feature | Prioridade |
|------|---------|:----------:|
| **T059** | **display: inline-block** — participates in inline flow com width/height | 🔴 |
| **T060** | **float: left/right** — sai do fluxo normal, posiciona lateralmente | 🔴 |
| **T061** | **display: flex** — direction row/column, justify-content, align-items | 🟡 |
| **T062** | **CSS Matching melhorado** — ancestor selectors, path-based matching | 🟡 |
| **T063** | **background-image** — url(), download, render | 🟡 |
| **T064** | **Testes M8** — 13+ testes de integração | 🔴 |

```bash
📦 6 tasks · 📅 Previsão: 5-7 dias
🎯 Meta: books.toscrape.com com produtos lado-a-lado, 310+ testes
```

Veja `TASKS_M8.md` para specs detalhadas.

## 🧪 Testes

```bash
# Rodar todos os testes
cargo test

# Verificar lint
cargo clippy

# Build release
cargo build --release

# 266 testes, 0 falhas, clippy limpo
```

---

## 📈 Performance

FAF Browser é construído com foco obsessivo em performance:

- **Binário release:** ~1MB (com LTO + strip)
- **Primeira query:** ~300ms (vs ~2s Playwright)
- **RAM por página:** ~5MB (vs ~150MB Chrome headless)
- **Concorrência nativa:** tokio async + semaphore no crawler
- **CSS Engine:** cascata O(n) com especificidade em memória
- **JS Engine:** QuickJS inicializa em ~10ms

```bash
# Crawlear 10 páginas em paralelo em < 5s
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3, .price_color" \
  --max 10 \
  --concurrency 5 \
  --format csv > produtos.csv

# Executar JS e extrair dados estruturados
faf https://books.toscrape.com/ \
  --js "document.querySelectorAll('.product_pod').length" \
  --json
# → 20
```

---

## 🤝 Contribuindo

1. Fork o projeto
2. Crie sua branch (`git checkout -b feat/feature`)
3. Commit suas mudanças (`git commit -m 'feat: adiciona feature'`)
4. Push (`git push origin feat/feature`)
5. Abra um Pull Request

### Padrão de commits

Usamos [conventional commits](https://www.conventionalcommits.org/):
- `feat:` — nova feature
- `fix:` — correção de bug
- `docs:` — documentação
- `refactor:` — refatoração
- `chore:` — manutenção

---

## 📄 Licença

MIT © [Tiago Coelho](https://github.com/tiagoc93)

---

<p align="center">
  <b>Fast As Fuck</b> — porque navegador não precisa ser pesado.<br>
  Feito com 🦀 e ☕ em Recife, Brasil.
</p>
