<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/status-active-2ea043?style=for-the-badge">
  <img alt="Status" src="https://img.shields.io/badge/status-active-2ea043?style=for-the-badge">
</picture>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/Rust-Edition_2024-orange?style=for-the-badge&logo=rust">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Edition_2024-orange?style=for-the-badge&logo=rust">
</picture>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/test-180_passed-green?style=for-the-badge">
  <img alt="Tests" src="https://img.shields.io/badge/test-180_passed-green?style=for-the-badge">
</picture>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge">
</picture>

<br>

# ⚡ FAF BROWSER

### Fast As Fuck — Navegador Headless 100% Rust

> *"O Chrome pesa 500MB de RAM? A gente faz em 50MB."*

FAF Browser é um navegador headless minimalista e agressivamente rápido, construído do zero em Rust. Sem Electron, sem Chromium embutido, sem overhead. Um binário único que faz scraping, crawleamento e inspeção de páginas web com performance nativa.

Renderizado pelo CSS Engine próprio. Crawleador multithread embutido. Tudo em **menos de 1MB** de binário.

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
| 🔤 **Fontes** — family, size (px/em/rem/%), weight | ✅ |
| 📊 **Computed Styles** — estilos reais da página (parseia `<style>` e `<link>` automagicamente) | ✅ |
| 🔎 **Filtros** — `--filter "text~=Python"`, `--filter "href^=https"`, regex | ✅ |
| 🎯 **Extração seletiva** — `--get "text, href"` — só os campos que importam | ✅ |
| 🕷️ **Crawler** — `faf follow` — siga links, visite páginas, extraia dados | ✅ |
| 📋 **Múltiplos formatos** — texto, JSON, JSONL, CSV | ✅ |
| 🔌 **Proxy** — HTTP e SOCKS5 | ✅ |
| 📦 **Binário único** — ~1MB, zero dependências runtime | ✅ |
| 🧪 **180 testes** — unitários + integração em sites reais | ✅ |
| 🔮 **JavaScript Engine** — QuickJS (em desenvolvimento) | 🔜 M3 |

---

## 🚀 Quick Start

### Instalação

```bash
# Clone e compile
git clone https://github.com/tiagoc93/faf-browser.git
cd faf-browser
cargo build --release

# O binário estará em ./target/release/faf-browser
# (ou instale via cargo)
cargo install --path .
```

### Pré-requisitos

- Rust (edition 2024) — instale via [rustup](https://rustup.rs/)
- Linux x86_64 (suporte a mais plataformas em breve)

---

## 📖 Uso

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
```

---

## 🏗️ Arquitetura

```
faf-browser/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Modulos públicos
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
│   │   └── engine.rs        # QuickJS (em desenvolvimento)
│   └── utils/
│       ├── mod.rs
│       ├── config.rs        # Configurações
│       └── error.rs         # Tipos de erro
├── tests/
│   └── m2_test.rs           # Testes de integração
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
| CSS Layout | `tiny-skia` | Box model e dimensões |
| CLI | `clap` (derive) | Interface de linha de comando |
| Serialização | `serde` + `serde_json` | Output JSON, CSV, JSONL |
| JS Runtime | `quick-js` | QuickJS embarcado (M3) |

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
| **JavaScript** | ❌ | ✅ | 🔜 M3 |
| **Screenshots** | ❌ | ✅ | ❌ (planejado) |
| **RAM (página média)** | ~50MB | ~150MB | ~5MB |
| **Tempo 1ª query** | ~2s | ~3s | ~0.3s |

---

## 🛣️ Roadmap

### ✅ M1 — Core Engine (Concluído)
- [x] HTTP Client com proxy e timeout
- [x] Parser HTML → DOM tree
- [x] CLI com clap
- [x] Output JSON

### ✅ M2 — CSS Engine (Concluído)
- [x] Parser CSS
- [x] Selector matching + especificidade
- [x] Computed styles + cascata
- [x] Box model, cores, fontes
- [x] Estilos automáticos da página `<style>` + `<link>`

### ✅ M2.5 — Extração Avançada (Concluído)
- [x] `--filter` com regex e attribute match
- [x] `--get` para campos específicos
- [x] `follow` subcomando — crawler multithread
- [x] `--format csv|jsonl`
- [x] 180 testes, 0 falhas

### 🔜 M3 — JavaScript Engine
- [ ] Embed QuickJS: runtime JS
- [ ] Bridge DOM ↔ JS: `document.getElementById`, `querySelector`
- [ ] `setTimeout` / `setInterval` com event loop tokio
- [ ] Fetch API via JS → reqwest
- [ ] Scripts inline + `<script src=>` externos
- [ ] CLI: `faf --js "document.title" --url <url>`

### 🔮 M4+ (Futuro)
- [ ] Cookies e sessão persistente
- [ ] WaitForSelector com timeout
- [ ] Screenshot (renderização via tiny-skia)
- [ ] Modo interativo (stdin → eval → stdout)
- [ ] Documentação completa com exemplos
- [ ] Suporte a Windows/macOS
- [ ] CLI: `faf --js "document.title" --url <url>` com QuickJS

---

## 🧪 Testes

```bash
# Rodar todos os testes
cargo test

# Verificar lint
cargo clippy

# Build release
cargo build --release

# 180 testes, 0 falhas, clippy limpo
```

---

## 📈 Performance

FAF Browser é construído com foco obsessivo em performance:

- **Binário release:** ~1MB (com LTO + strip)
- **Primeira query:** ~300ms (vs ~2s Playwright)
- **RAM por página:** ~5MB (vs ~150MB Chrome headless)
- **Concorrência nativa:** tokio async + semaphore no crawler
- **CSS Engine:** cascata O(n) com especificidade em memória

```bash
# Crawlear 10 páginas em paralelo em < 5s
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3, .price_color" \
  --max 10 \
  --concurrency 5 \
  --format csv > produtos.csv
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
