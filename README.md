<p align="center">
  <img src="logo.png" alt="FAF Browser" width="600">
</p>

<p align="center">
  <a href="https://github.com/tiagoc93/faf-browser/actions"><img src="https://img.shields.io/badge/status-active-2ea043?style=flat-square" alt="Status"></a>
  <a href="#"><img src="https://img.shields.io/badge/Rust-Edition%202024-orange?style=flat-square&logo=rust" alt="Rust 2024"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-332%20passed-green?style=flat-square" alt="Tests"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/binary-~8MB-lightgrey?style=flat-square" alt="Binary"></a>
</p>

# faf — scraping and page archiving for the terminal

> *A single binary that fetches, parses, executes JS, and outputs structured data. Optimized for feeding LLMs. No Electron. No Chromium. Just Rust.*

**FAF** is a high-performance CLI scraper. It speaks CSS selectors, runs JavaScript via QuickJS, handles cookies, retries, proxies, and outputs data as JSON, CSV, JSONL, Markdown, or self-contained HTML. Not a browser — a smarter curl.

---

## Quick install

```bash
cargo install --git https://github.com/tiagoc93/faf-browser.git
# or
git clone https://github.com/tiagoc93/faf-browser.git && cd faf-browser && cargo build --release
```

Requires **Rust** (edition 2024) via [rustup](https://rustup.rs/). Linux x86_64.

---

## Using the CLI

After installation, the binary is **`faf-browser`** (not just `faf`). Call it directly:

```bash
faf-browser --help
faf-browser https://books.toscrape.com/
```

If you get `command not found`, add Cargo's bin directory to your `PATH`:

```bash
# Add this to your ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"

# Then reload
source ~/.bashrc   # or: exec bash
```

Verify it works:

```bash
which faf-browser   # → /home/USER/.cargo/bin/faf-browser
faf-browser --version
```

If you built manually with `cargo build --release`, the binary is at `target/release/faf-browser` — use the full path or add `target/release` to PATH.

---

## Features

| | | |
|---|---|---|
| **HTTP** | proxy (HTTP/SOCKS5), timeout, custom user-agent, retry with exponential backoff | ✅ |
| **HTML** | full DOM tree via html5ever, CSS selectors (`h1`, `.class`, `#id`, combinators) | ✅ |
| **CSS** | parser, cascade, specificity, computed styles, inline + external stylesheets | ✅ |
| **JavaScript** | QuickJS (ES2020) runtime, DOM bridge (querySelector, fetch, timers), page scripts | ✅ |
| **Dump** | self-contained HTML, Markdown, text output, readability, structured data extraction | ✅ |
| **Crawl** | `follow` links with concurrency, rate-limiting, filters, and selective extraction | ✅ |
| **Output** | JSON, JSONL, CSV, plain text — pipe-friendly | ✅ |
| **Session** | persistent cookies (Netscape format), response caching (SHA256 + TTL) | ✅ |
| **Interaction** | click, form fill, watch for changes, scroll simulation | ✅ |
| **Performance** | ~300ms first query, ~5MB RAM per page, single binary | ✅ |

---

## Usage

### `faf <url>` — Fetch a page

```bash
faf https://books.toscrape.com/
```

```
📄 Página: https://books.toscrape.com/
📌 Título: All products | Books to Scrape - Sandbox
🔗 Links: 94  🖼️ Imagens: 20  📋 Metadados: 4
📝 Texto: Books to Scrape We love being scraped! ...
```

### `faf query` — CSS selectors with computed styles

```bash
faf https://books.toscrape.com/ query "h1"
faf https://site.com query "a" --json
faf https://site.com query "a" --get "text,href"
```

```
🔍 Query 'h1': 1 resultado(s)
  [1.] <h1> text: All products
      🎨 color: inherit | bg: transparent | font-size: 29.96px | display: block
```

`--get` fields: `tag`, `id`, `classes`, `text`, `html`, `href`, `src`, `alt`, `color`, `bg`, `font-size`, `font-family`, `display`.

### `faf query --filter` — Filter results

```bash
faf https://books.toscrape.com/ query "a" --filter "text~=Python"
faf https://site.com query "a" --filter "href^=https" --filter "text!=."
faf https://site.com query "img" --filter "alt=.+"
```

Operators: `~=` (contains), `!~=` (not contains), `==` (exact), `!=` (not exact), `^=` (starts with), `!^=`, `$=` (ends with), `!$=`, `=` (regex).

### `faf dump` — Self-contained HTML archive

Core command. Downloads a page and produces a single-file output you can open anywhere.

```bash
# Basic dump: CSS inlined, URLs absolute, HTML preserved
faf dump --url https://books.toscrape.com/ --output page.html

# Pipe to stdout (no --output = goes to stdout)
faf dump --url https://site.com --format markdown | head -50
```

**Output formats (`--format`):**

| Value | Output | Use case |
|-------|--------|----------|
| `html` | Self-contained HTML (default) | Archive, open in browser |
| `markdown` | Clean Markdown with headings, links, lists, images | Feed to LLMs, note-taking |
| `text` | Plain text with paragraph separation | Quick reading, grep, pipe |
| `json` | Structured JSON with metadata | Programmatic consumption |

```bash
faf dump --url https://books.toscrape.com/ --format markdown
faf dump --url https://site.com --format text
```

**Content extraction (`--readability`):**

Strips navigation, sidebars, footers, ads. Keeps only the main content using text-density scoring. Combine with `--format markdown` for clean LLM input:

```bash
faf dump --url https://blog.com/post --format markdown --readability
faf dump --url https://docs.rs/some-crate --format text --readability | less
```

**Structured data (`--structured-data`):**

Extracts JSON-LD, Open Graph, meta tags, and microdata as a JSON file. Useful for SEO analysis, metadata extraction:

```bash
faf dump --url https://site.com --structured-data
# → {"json_ld": [...], "open_graph": {...}, "meta": {...}}
```

**Self-contained options:**

| Flag | Effect |
|------|--------|
| `--no-scripts` | Remove all `<script>` tags and `on*` event handlers |
| `--inline-images` | Convert `<img src>` to base64 data URIs (fully offline) |
| `--no-inline-css` | Keep `<link rel="stylesheet">` instead of inlining as `<style>` |
| `--output <path>` | Save to file (omit for stdout) |

### `faf follow` — Crawl

Follows links matching a selector, visits each page, extracts data.

```bash
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3,.price_color" --max 10 --format csv

faf https://site.com follow "a" --max 5 --delay 1000 --random-delay 500 2000
```

| Flag | Default | Description |
|------|---------|-------------|
| `--extract` | – | CSS selectors to extract from each visited page |
| `--max` | `10` | Max pages to crawl |
| `--concurrency` | `3` | Parallel requests |
| `--delay` | `0` | Fixed delay (ms) between requests |
| `--random-delay` | – | Random delay range (min max in ms) |
| `--same-domain` | `true` | Restrict to same domain |

### `faf` — JavaScript engine

QuickJS (ES2020) embedded runtime. Executes page scripts with DOM bridge and fetch API. Limited compared to V8 — no ES2021+, no `await` at top level, no `localStorage`, no `MutationObserver`.

```bash
faf https://books.toscrape.com/ --js "document.title"
# → "All products | Books to Scrape - Sandbox"

faf https://site.com --js "document.querySelectorAll('h3').length"
faf https://site.com --js-file script.js
faf https://site.com --js "while(true){}" --js-timeout 2
# → "JavaScript execution timed out after 2s"

echo 'document.querySelectorAll(".price").length' | faf --stdin --url https://site.com
```

The JS bridge exposes: `document.title`, `document.querySelector`, `document.querySelectorAll`, `document.getElementById`, `fetch()`, `setTimeout`, `setInterval`, `console.log/warn/error`.

#### JavaScript API reference

FAF runs QuickJS (ES2020), not V8. The following DOM APIs are polyfilled:

| API | Support | Notes |
|-----|:-------:|-------|
| `document.title` | ✅ | Read/write |
| `document.querySelector(sel)` | ✅ | CSS selectors via scraper |
| `document.querySelectorAll(sel)` | ✅ | Returns Array-like |
| `document.getElementById(id)` | ✅ | |
| `element.value` | ✅ | Read/write on `<input>`, `<select>`, `<textarea>` |
| `element.checked` | ✅ | Read/write on checkbox/radio |
| `element.click()` | ✅ | Via `dispatchEvent(MouseEvent)` |
| `element.text` / `element.textContent` | ✅ | |
| `element.attributes` | ✅ | Key-value map |
| `element.innerHTML` | ✅ | Read-only |
| `fetch(url)` | ✅ | Calls reqwest internally (real HTTP) |
| `setTimeout(fn, ms)` | ✅ | Tokio-backed event loop |
| `setInterval(fn, ms)` | ✅ | Tokio-backed event loop |
| `clearTimeout / clearInterval` | ✅ | |
| `console.log / warn / error` | ✅ | Routed to Rust logger |
| `window.scrollTo / scrollBy` | ✅ | Simulated position |
| `element.scrollIntoView()` | ✅ | Simulated |
| `window.pageYOffset` | ✅ | |
| `new URLSearchParams()` | ✅ | Polyfill |
| `localStorage` | ❌ | No persistent storage |
| `MutationObserver` | ❌ | No live DOM observation |
| `IntersectionObserver` | ❌ | No viewport tracking |
| `WebSocket` | ❌ | |
| `addEventListener` | ❌ | Events fired via dispatchEvent only |
| `XMLHttpRequest` | ❌ | Use `fetch()` instead |
| `canvas` / `WebGL` | ❌ | |

### `faf repl` — Interactive mode

```bash
faf repl --url https://books.toscrape.com/
> document.title
"All products | Books to Scrape - Sandbox"
> document.querySelectorAll('h3').length
20
> .json
> document.querySelector("h1").text
"\"All products\""
> .exit
```

Commands: `.json` (toggle JSON output), `.exit`, `.help`, `.clear`. Pipe input via `--stdin`:

```bash
echo "document.title" | faf --stdin --url https://site.com
```

### `faf` — Page interaction

```bash
# Click an element
faf click "#btn" --url https://site.com

# Wait for element to appear
faf wait ".spinner" --url https://site.com --timeout 10

# Watch for changes
faf watch ".price" --url https://site.com --interval 30 --max-checks 5
```

### `faf` — Output formats

```bash
faf https://site.com query "a" --json                    # JSON pretty
faf https://site.com query "a" --get "text,href" --format jsonl  # JSONL
faf https://site.com follow "a" --max 3 --format csv      # CSV
```

### `faf` — Session & network

```bash
# Proxy
faf https://site.com --proxy socks5://127.0.0.1:9050

# Persistent cookies
faf https://site.com/login --cookies session.txt --cookies-jar session.txt
faf https://site.com/dashboard --cookies session.txt

# HTTP caching
faf https://site.com --cache .cache --show-headers --show-status

# Retry on failure
faf https://site.com --retries 3 --retry-delay 2000

# Custom headers and timeout
faf https://site.com --user-agent "FAF/1.0" --timeout 60
```

### `faf` — CSS manipulation

```bash
# Inject custom CSS
faf https://site.com --css "h1 { color: red; }" query "h1"

# Load CSS from file
faf https://site.com --css custom.css query ".card"

# Disable page CSS
faf https://site.com --no-page-css query "h1"
```

---

## Comparison

FAF is **not** a browser automation tool. It does not run Chromium, handle SPAs, or bypass anti-bot detection. It is a **high-performance scraper** with JS execution.

| | curl + pup + jq | BeautifulSoup | Playwright | **FAF** |
|---|---:|---:|---:|---:|
| Language | shell | Python | Node.js | **Rust** |
| Single binary | ❌ | ❌ | ❌ | **✅** |
| CSS selectors | ❌ | ✅ | ✅ | ✅ |
| JavaScript | ❌ | ❌ | ✅ (V8) | ✅ (QuickJS) |
| Self-contained dump | ❌ | ❌ | ❌ | ✅ |
| Markdown / readability | ❌ | ❌ | ❌ | ✅ |
| Built-in crawler | ❌ | ❌ | ❌ | ✅ |
| JSONL/CSV native | ❌ | ❌ | ❌ | ✅ |
| Cookies / Cache / Retry | ❌ | ❌ | ❌ | ✅ |
| SPA rendering | ❌ | ❌ | ✅ | ❌ |
| Anti-bot bypass | ❌ | ❌ | ✅ | ❌ |
| RAM (avg page) | ~5MB | ~50MB | ~150MB | **~5MB** |
| First query | ~0.1s | ~2s | ~3s | **~0.3s** |

### When to use FAF vs Playwright

| Scenario | Tool |
|----------|------|
| Simple scraping (HTTP GET → parse) | **FAF** |
| Feed page content to LLMs (markdown, readability) | **FAF** |
| Crawl with rate-limiting, concurrency | **FAF** |
| Run on low-resource VPS (5MB RAM) | **FAF** |
| Archive pages as self-contained HTML | **FAF** |
| Execute JS-dependent SPAs (React, Vue) | **Playwright** |
| Bypass anti-bot detection | **Playwright** |
| Pixel-perfect screenshots | **Playwright** |
| Automate login flows, form fills via real browser | **Playwright** |

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

### ✅ M8 — Polish & Robustness (done)

| Task | Description |
|---|---|
| **T095** | Fix 7 broken tests in `m5_test.rs` — all 19 now pass |
| **T096** | Integration test for `--inline-images` with PNG served locally |
| **T097** | Output to stdout when `--output` is omitted (pipe support) |
| **T098** | Optimize `Cargo.toml` profiles (`dev`, `release-fast`) |
| **T099** | Update badges and docs — 329 tests, 0 failures |

```
📦 5 tasks · ✅ 5 concluídas · 329 testes · 0 falhas
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
