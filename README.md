<p align="center">
  <img src="logo.png" alt="FAF Browser" width="600">
</p>

<p align="center">
  <a href="https://github.com/tiagoc93/faf-browser/actions"><img src="https://img.shields.io/badge/status-active-2ea043?style=flat-square" alt="Status"></a>
  <a href="#"><img src="https://img.shields.io/badge/Rust-Edition%202024-orange?style=flat-square&logo=rust" alt="Rust 2024"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-309%20passed-green?style=flat-square" alt="Tests"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/binary-~2MB-lightgrey?style=flat-square" alt="Binary"></a>
</p>

# faf — headless browser for the terminal

> *A ~2MB single binary that fetches, parses, executes JS, and dumps self-contained HTML. No Electron. No Chromium. Just Rust.*

**FAF** is a CLI tool for scraping, crawling, and archiving web pages. It speaks CSS selectors, runs JavaScript via QuickJS, handles cookies, retries, proxies, and outputs data as JSON, CSV, JSONL, or self-contained HTML.

---

## Quick install

```bash
cargo install --git https://github.com/tiagoc93/faf-browser.git
# or
git clone https://github.com/tiagoc93/faf-browser.git && cd faf-browser && cargo build --release
```

Requires **Rust** (edition 2024) via [rustup](https://rustup.rs/). Linux x86_64.

---

## Features

| | | |
|---|---|---|
| **HTTP** | proxy (HTTP/SOCKS5), timeout, custom user-agent, retry with exponential backoff | ✅ |
| **HTML** | full DOM tree via html5ever, CSS selectors (`h1`, `.class`, `#id`, combinators) | ✅ |
| **CSS** | parser, cascade, specificity, computed styles, inline + external stylesheets | ✅ |
| **JavaScript** | QuickJS runtime, DOM bridge, `fetch()`, `setTimeout`, `<script>` execution | ✅ |
| **Dump** | self-contained HTML, Markdown, text output, readability extraction, structured data | ✅ |
| **Crawl** | `follow` links with concurrency, rate-limiting, filters, and selective extraction | ✅ |
| **Output** | JSON, JSONL, CSV, plain text — pipe-friendly | ✅ |
| **Session** | persistent cookies (Netscape format), response caching (SHA256 + TTL) | ✅ |
| **Interaction** | click, form fill, watch for changes, scroll simulation | ✅ |
| **Performance** | ~300ms first query, ~5MB RAM per page, single binary | ✅ |

---

## Usage

### Dump a self-contained HTML

```bash
faf dump --url https://books.toscrape.com/ --output page.html
faf dump --url https://site.com --output page.html --inline-images --no-scripts
```

The output is a **single HTML file** you can open in any browser. External CSS is inlined. URLs are resolved to absolute. Scripts and event handlers are removed with `--no-scripts`. Images are converted to base64 data URIs with `--inline-images`.

### Scrape data

```bash
faf https://books.toscrape.com/                      # full page info
faf https://site.com query "h1" --json               # CSS query
faf https://site.com query "a" --get "text,href"     # selective fields
faf https://site.com query "div" --filter "class~=product" --filter "text!=."
```

### Crawl

```bash
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3,.price_color" --max 10 --format csv

faf https://site.com follow "a" --max 5 --delay 1000 --random-delay 500 2000
```

### JavaScript

```bash
faf https://site.com --js "document.title"
faf https://site.com --js-file script.js
echo 'document.querySelectorAll(".price").length' | faf --stdin --url https://site.com
```

### Interaction

```bash
faf click "#btn" --url https://site.com
faf wait ".spinner" --url https://site.com --timeout 10
faf watch ".price" --url https://site.com --interval 30 --max-checks 5
```

### Session & network

```bash
faf https://site.com --proxy socks5://127.0.0.1:9050
faf https://site.com --cookies session.txt --cookies-jar session.txt
faf https://site.com --cache .cache --show-headers --show-status
faf https://site.com --retries 3 --retry-delay 2000
```

---

## Comparison

| | BeautifulSoup | Playwright | **FAF** |
|---|---:|---:|---:|
| Language | Python | Node.js | **Rust** |
| Single binary | ❌ | ❌ | **✅** |
| CSS selectors | ✅ | ✅ | ✅ |
| Computed styles | ❌ | ✅ | ✅ |
| JavaScript | ❌ | ✅ (V8) | ✅ (QuickJS) |
| Built-in crawler | ❌ | ❌ | ✅ |
| HTML dump (self-contained) | ❌ | ❌ | ✅ |
| JSONL/CSV native | ❌ | ❌ | ✅ |
| RAM (avg page) | ~50MB | ~150MB | **~5MB** |
| First query | ~2s | ~3s | **~0.3s** |
| Cookies / Cache / Retry | ❌ | ❌ | ✅ |

FAF is **not** a browser automation tool. It won't render SPAs pixel-perfect or drive a real browser. It is a **high-performance scraping and page-archiving CLI** that beats curl+pup+jq in convenience and speed.

---

## Architecture

```
src/
├── api/           CLI (clap), commands, output formatters, filters
├── http/          reqwest client, response cache, cookie store
├── dom/           HTML parser (scraper/html5ever)
├── css/           CSS parser (cssparser), selectors, computed styles
├── js/            QuickJS runtime (rquickjs), DOM bridge, fetch bridge
├── dump/          self-contained HTML generation, CSS/image inlining
└── utils/         config, error types
```

| Component | Crate |
|---|---|
| HTTP | `reqwest` (rustls), `tokio` |
| HTML | `scraper` (html5ever) |
| CSS | `cssparser`, `selectors` |
| JS | `rquickjs` (QuickJS) |
| CLI | `clap`, `serde`, `serde_json` |
| Encoding | `base64`, `sha2`, `hex`, `regex` |

---

## Roadmap

### ✅ M1 — Core Engine
HTTP client, HTML parsing, CLI, JSON output

### ✅ M2 — CSS Engine
Parser, selector matching, specificity, computed styles, box model

### ✅ M2.5 — Advanced Extraction
`--filter`, `--get`, `follow` crawler, CSV/JSONL output

### ✅ M3 — JavaScript Engine
QuickJS runtime, DOM bridge, `fetch()`, `setTimeout`, page scripts

### ✅ M4 — Professional Crawler Tools
Cookies, wait, REPL, rate-limiting, retry, cache, HTTP info

### ✅ M5 — Page Interaction
Click, forms, watch mode, scroll simulation

### ✅ M6 — Self-Contained HTML Dump
CSS inlining, image-to-base64, URL resolution, script removal

---

### ✅ M7 — LLM-Ready Output (done)

| Task | Description |
|---|---|
| **T090** | `dump --format markdown` — convert HTML to clean Markdown |
| **T091** | `dump --readability` — extract main content, strip navigation/ads/footers |
| **T092** | Structured data extraction — JSON-LD, microdata, schema.org, Open Graph |
| **T093** | `dump --format text` — clean visible text extraction, paragraph-preserving |
| **T094** | Tests — 12 integration tests + 24 unit tests |

```
📦 5 tasks · ✅ 5 concluídas · 309 testes · 0 falhas
```

---

## Contributing

```bash
cargo test
cargo clippy
cargo fmt
```

Conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`.

MIT © [Tiago Coelho](https://github.com/tiagoc93) — Recife, Brasil.
