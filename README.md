<p align="center">
  <img src="logo.png" alt="FAF Browser" width="600">
</p>

<p align="center">
  <a href="https://github.com/tiagoc93/faf-browser/actions"><img src="https://img.shields.io/badge/status-active-2ea043?style=flat-square" alt="Status"></a>
  <a href="#"><img src="https://img.shields.io/badge/Rust-Edition%202024-orange?style=flat-square&logo=rust" alt="Rust 2024"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-360%20passed-green?style=flat-square" alt="Tests"></a>
  <a href="#"><img src="https://img.shields.io/badge/binary-8MB-lightgrey?style=flat-square" alt="Binary"></a>
  <a href="#"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT"></a>
</p>

---

# faf — the scraper that talks to machines

> *Fetch, parse, execute JavaScript, and output structured data. One binary. Zero Chromium. Pure Rust.*

**FAF** is not a browser. It's a **headless web processor** — a single 8MB binary that does what `curl + pup + jq + pandoc + puppeteer` do together, except it runs in 5MB of RAM and finishes before Playwright even boots.

It speaks CSS selectors, runs real JavaScript via QuickJS, handles cookies/caching/retries/proxies, and outputs **JSON, CSV, JSONL, Markdown, text, or self-contained HTML**. Built for pipelines, agents, and LLMs — not for screenshots.

---

## Table of Contents

- [See it in action](#see-it-in-action)
- [Built for AI pipelines](#built-for-ai-pipelines)
- [Why FAF](#why-faf)
- [Comparison](#comparison)
- [Quick install](#quick-install)
- [Features](#features)
- [Limitations](#limitations)
- [Usage](#usage)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [Support](#support)

---

## See it in action

```bash
# One command → clean Markdown with metadata, ready for any LLM
$ faf dump --url https://example.com --format markdown

---
title: Example Domain
url: "https://example.com"
---

# Example Domain

This domain is for use in documentation examples without needing
permission. Avoid use in operations.

[Learn more](https://iana.org/domains/example)
```

```bash
# Readability extraction + chunking for context windows
$ faf dump --url https://doc.rust-lang.org/book/ch01-01-installation.html \
    --format markdown --readability --chunk-size 100

{
  "chunks": [
    { "index": 0, "tokens_est": 58, "content": "## Installation\n\nThe first step..." },
    { "index": 1, "tokens_est": 88, "content": "Note: If you prefer not to use..." },
    { "index": 2, "tokens_est": 89, "content": "compile will continue to..." }
  ]
}
```

No BeautifulSoup. No Selenium. No Node.js. Just Rust.

---

## Built for AI pipelines

FAF was designed from the ground up to feed clean data into LLMs and RAG pipelines:

- **YAML frontmatter** — OpenGraph, meta tags, JSON-LD types extracted automatically. Every markdown output starts with structured metadata the LLM can use for context — title, description, author, date, URL.
- **Readability extraction** — strips navigation, sidebars, footers, ads. Keeps only the main content using text-density scoring. Less tokens, more signal.
- **Whitespace collapse** — always on. 3+ blank lines collapse to 1, trailing whitespace stripped, navigation-only URL lines dropped. Every byte counts when you're paying per token.
- **Token-aware chunking** — `--chunk-size N` splits long pages into token-bounded chunks (heuristic: 4 chars/token). Divides by section, then paragraph, then line — never cuts mid-line. Each chunk carries the frontmatter for context continuity.

```bash
# The RAG pipeline in one command
faf dump --url https://long-article.com \
  --format markdown --readability --chunk-size 500 \
  --output article.md
# → article_01.md, article_02.md, ... (each with frontmatter)
```

---

## Why FAF

### The problem

Modern scraping stacks are heavy and fragmented:

- **Playwright/Puppeteer** spin up a full Chromium (~150MB RAM, ~3s startup) to render one page
- **BeautifulSoup** is pure Python — slow, no JS, no built-in output formats
- **curl + pup + jq** is a shell glue nightmare — different tools, different syntaxes, no session state
- **Firecrawl / Jina Reader** are SaaS — rate-limited, privacy concerns, require API keys, latency for every request
- **None of them** output Markdown or do readability extraction natively

You end up with 4 tools, 3 languages, and 200MB of RAM to scrape one table.

### The FAF way

FAF replaces the scraping stack with a single binary that runs anywhere:

- **Privacy:** No SaaS, no API keys, no data leaving your machine. Every request goes from your process.
- **Cost:** Zero per-request fees. No rate limits. No quota.
- **Latency:** ~300ms first query, ~5MB RAM per page. Same machine, same process, same binary.
- **Simplicity:** One tool, one syntax, one output format per command. Pipes work.

---

## Comparison

| | Playwright | BeautifulSoup | curl+pup+jq | Firecrawl | Jina Reader | **FAF** |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| Binary size | 150MB+ (Chromium) | Python runtime | 3 separate bins | SaaS | SaaS | **8MB** |
| RAM per page | ~150MB | ~50MB | ~5MB | N/A | N/A | **~5MB** |
| First query | ~3s | ~2s | ~0.1s | ~1-2s (network) | ~1-2s (network) | **~0.3s** |
| Privacy | ✅ local | ✅ local | ✅ local | ❌ SaaS | ❌ SaaS | **✅ local** |
| Per-request cost | free | free | free | paid | free tier | **free** |
| CSS selectors | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ |
| JavaScript | ✅ (V8) | ❌ | ❌ | ✅ | ❌ | ✅ (QuickJS) |
| Markdown output | ❌ | ❌ | ❌ | ✅ | ✅ | **✅** |
| YAML frontmatter | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Readability extraction | ❌ | ❌ | ❌ | ✅ | ✅ | **✅** |
| Token-aware chunking | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Self-contained HTML dump | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Built-in crawler | ❌ | ❌ | ❌ | ✅ | ❌ | **✅** |
| Cookies / Cache / Retry | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| JSONL / CSV native | ❌ | ❌ | ✅ | ❌ | ❌ | **✅** |
| SPA rendering | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Anti-bot bypass | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Offline / air-gapped | ❌ | ✅ | ✅ | ❌ | ❌ | **✅** |

**FAF trades SPA rendering and anti-bot bypass for 30× less RAM, 10× faster startup, full privacy, and LLM-native output.** If you're feeding data to LLMs, building RAG pipelines, crawling APIs, or archiving content — that's the trade you want.

---

## Quick install

```bash
cargo install --git https://github.com/tiagoc93/faf-browser.git
# or
git clone https://github.com/tiagoc93/faf-browser.git && cd faf-browser && cargo build --release
```

Requires **Rust** (edition 2024) via [rustup](https://rustup.rs/). Linux x86_64.

```bash
# Verify
which faf           # → /home/USER/.cargo/bin/faf
faf --version
```

If `command not found`, add Cargo's bin to your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bashrc
```

---

## Features

**HTTP** — proxy (HTTP/SOCKS5), timeout, custom user-agent, retry with exponential backoff

**HTML** — full DOM tree via html5ever, CSS selectors (`h1`, `.class`, `#id`, combinators)

**CSS** — parser, cascade, specificity, computed styles, inline + external stylesheets

**JavaScript** — QuickJS (ES2020) runtime, DOM bridge (querySelector, fetch, timers), page scripts

**Dump** — self-contained HTML, Markdown, text output, readability, structured data extraction

**LLM-Ready Markdown** — YAML frontmatter with OpenGraph metadata, semantic whitespace compression, token-aware chunking, GFM tables with column separators

**Crawl** — `follow` links with concurrency, rate-limiting, filters, and selective extraction

**Output** — JSON, JSONL, CSV, plain text — pipe-friendly

**Session** — persistent cookies (Netscape format), response caching (SHA256 + TTL)

**Interaction** — click, form fill, watch for changes, scroll simulation

**Performance** — ~300ms first query, ~5MB RAM per page, single binary

---

## Limitations

FAF is honest about what it doesn't do:

- **No SPA rendering** — React, Vue, Angular apps that require a real browser won't fully render. QuickJS executes page scripts, but there's no layout engine or viewport.
- **No anti-bot bypass** — Cloudflare challenges, CAPTCHAs, and fingerprinting detection are not supported. FAF sends a normal HTTP request, not a real browser.
- **No screenshots** — FAF doesn't render pixels. Use Playwright for visual testing.
- **JS is QuickJS, not V8** — ES2020 only. Some modern APIs (`MutationObserver`, `IntersectionObserver`, `WebSocket`, `canvas`, `WebGL`) are not available. See the [JS API reference](#javascript-engine) below.
- **Linux x86_64 only** — for now. Cross-platform binaries are on the roadmap.

If you need any of the above, use Playwright. FAF and Playwright are complementary, not competing.

---

## Usage

### Fetch a page

```bash
faf https://books.toscrape.com/
```

```
📄 Página: https://books.toscrape.com/
📌 Título: All products | Books to Scrape - Sandbox
🔗 Links: 94  🖼️ Imagens: 20  📋 Metadados: 4
📝 Texto: Books to Scrape We love being scraped! ...
```

### Query with CSS selectors

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

### Filter results

```bash
faf https://books.toscrape.com/ query "a" --filter "text~=Python"
faf https://site.com query "a" --filter "href^=https" --filter "text!=."
faf https://site.com query "img" --filter "alt=.+"
```

Operators: `~=` (contains), `!~=` (not contains), `==` (exact), `!=` (not exact), `^=` (starts with), `!^=`, `$=` (ends with), `!$=`, `=` (regex).

### Dump — self-contained archiving

Core command. Downloads a page and produces a single-file output you can open anywhere.

```bash
# Self-contained HTML archive
faf dump --url https://books.toscrape.com/ --output page.html

# Clean Markdown for LLMs
faf dump --url https://blog.com/post --format markdown --readability

# Plain text for grep / quick reading
faf dump --url https://docs.rs/some-crate --format text --readability | less
```

**Output formats (`--format`):**

- **`html`** — Self-contained HTML (default). Archive, open in browser.
- **`markdown`** — Clean Markdown with headings, links, lists, images. Feed to LLMs, note-taking.
- **`text`** — Plain text with paragraph separation. Quick reading, grep, pipe.
- **`json`** — Structured JSON with metadata. Programmatic consumption.

**Content extraction (`--readability`):**

Strips navigation, sidebars, footers, ads. Keeps only the main content using text-density scoring. Combine with `--format markdown` for clean LLM input.

**Structured data (`--structured-data`):**

Extracts JSON-LD, Open Graph, meta tags, and microdata as JSON.

```bash
faf dump --url https://site.com --structured-data
# → {"json_ld": [...], "open_graph": {...}, "meta": {...}}
```

**Markdown for AI (`--frontmatter`, `--chunk-size`):**

When using `--format markdown`, three optimizations make output pipeline-ready for LLMs:

- `--frontmatter on|off` — Prepend YAML frontmatter with OpenGraph/meta metadata (default: `on` for markdown).
- `--chunk-size N` — Split long pages into token-bounded chunks (heuristic 4 chars/token). Divides by section, then paragraph, then line — never mid-line.

```bash
# Markdown with frontmatter metadata
faf dump --url https://blog.com/article --format markdown --frontmatter true
# → ---
  title: ...
  description: ...
  site_name: ...
  url: ...
  ---
  # Title...

# Chunking for context windows
faf dump --url https://long-article.com --format markdown --chunk-size 500 --output article.md
# → article_01.md, article_02.md, ... (each with frontmatter)

# Chunks to stdout (JSON envelope)
faf dump --url https://long-article.com --format markdown --chunk-size 500
# → { "frontmatter": "...", "chunks": [{ "index": 0, "tokens_est": 480, "content": "..." }] }
```

If a page has no `og:url` tag, FAF falls back to the request URL — so frontmatter always contains the source URL.

Whitespace collapse is always on: 3+ blank lines collapse to 1, trailing whitespace is stripped, and navigation-only URL lines with <3 useful chars are removed.

**Self-contained options:**

- `--no-scripts` — Remove all `<script>` tags and `on*` event handlers
- `--inline-images` — Convert `<img src>` to base64 data URIs (fully offline)
- `--no-inline-css` — Keep `<link rel="stylesheet">` instead of inlining as `<style>`
- `--output <path>` — Save to file (omit for stdout)

### Follow — crawl with style

```bash
faf https://books.toscrape.com/ follow ".product_pod h3 a" \
  --extract "h3,.price_color" --max 10 --format csv
```

```
title,price
A Light in the Attic,£51.77
Tipping the Velvet,£53.74
...
```

- `--extract` — CSS selectors to extract from each visited page
- `--max` — Max pages to crawl (default: 10)
- `--concurrency` — Parallel requests (default: 3)
- `--delay` / `--random-delay` — Rate limiting between requests
- `--same-domain` — Restrict to same domain (default: true)

### JavaScript engine

QuickJS (ES2020) embedded runtime. Executes page scripts with DOM bridge and fetch API.

```bash
faf https://books.toscrape.com/ --js "document.title"
faf https://site.com --js "document.querySelectorAll('h3').length"
faf https://site.com --js-file script.js
faf https://site.com --js "while(true){}" --js-timeout 2
echo 'document.querySelectorAll(".price").length' | faf --stdin --url https://site.com
```

The JS bridge exposes: `document.title`, `document.querySelector`, `document.querySelectorAll`, `document.getElementById`, `fetch()`, `setTimeout`, `setInterval`, `console.log/warn/error`.

<details>
<summary><b>JavaScript API reference</b></summary>

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

</details>

### REPL — interactive mode

```bash
faf repl --url https://books.toscrape.com/
> document.title
"All products | Books to Scrape - Sandbox"
> document.querySelectorAll('h3').length
20
> .exit
```

Commands: `.json` (toggle JSON output), `.exit`, `.help`, `.clear`.

### Page interaction

```bash
faf click "#btn" --url https://site.com
faf wait ".spinner" --url https://site.com --timeout 10
faf watch ".price" --url https://site.com --interval 30 --max-checks 5
```

### Session & network

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

### CSS manipulation

```bash
faf https://site.com --css "h1 { color: red; }" query "h1"
faf https://site.com --css custom.css query ".card"
faf https://site.com --no-page-css query "h1"
```

---

## Architecture

```
src/
├── api/           CLI (clap), commands, output formatters, filters
├── http/          reqwest client, response cache, cookie store
├── dom/           HTML parser (scraper/html5ever)
├── css/           CSS parser (cssparser), selectors, computed styles
├── js/            QuickJS runtime (rquickjs), DOM bridge, fetch bridge
├── dump/          self-contained HTML, markdown, text, readability, structured data
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

**~12K LOC. 40 files. Zero non-Rust code.** The entire codebase fits in your head.

---

## Roadmap

### ✅ Shipped

| Milestone | What |
|---|---|
| **M1** — Core Engine | HTTP client, HTML parsing, CLI, JSON output |
| **M2** — CSS Engine | Parser, selector matching, specificity, computed styles, box model |
| **M2.5** — Advanced Extraction | `--filter`, `--get`, `follow` crawler, CSV/JSONL output |
| **M3** — JavaScript Engine | QuickJS runtime, DOM bridge, `fetch()`, timers, page scripts |
| **M4** — Professional Crawler | Cookies, wait, REPL, rate-limiting, retry, cache, HTTP info |
| **M5** — Page Interaction | Click, forms, watch mode, scroll simulation |
| **M6** — Self-Contained Dump | CSS inlining, image-to-base64, URL resolution, script removal |
| **M7** — LLM-Ready Output | Markdown conversion, readability extraction, structured data, text output |
| **T001** — Markdown Optimization for AI | YAML frontmatter with OpenGraph, semantic whitespace compression, token-aware chunking, GFM table separators |
| **T002** — Post-M7 Fixes | Double-parse elimination, LazyLock regexes, URL fallback in frontmatter |

**360 tests. 0 failures.**

### 🔮 What's next

**MCP Server Native** ⚡ — Expose FAF as an MCP (Model Context Protocol) server. Claude Desktop, Cursor, and other AI agents call `faf fetch`, `faf dump`, and structured extraction directly from chat — no shell, no subprocess. One `faf-mcp` binary, thousands of devs already using AI tooling.

**Daemon Mode + REST API** 🌐 — Run `faf serve` as a background daemon exposing REST endpoints (`POST /fetch`, `POST /crawl`, `GET /dump`). Any language — Python, TypeScript, Go — consumes FAF without Rust bindings. Turns FAF from a CLI into local infrastructure.

**Cross-Platform Binaries** 📦 — GitHub Actions matrix build: Linux x86_64 + ARM64, macOS Intel + Apple Silicon, Windows. Homebrew tap and Winget. One-command install on any OS.

---

## When to use FAF vs Playwright

| Scenario | Use |
|----------|-----|
| Simple scraping (HTTP GET → parse) | **FAF** |
| Feed page content to LLMs (markdown, readability, frontmatter) | **FAF** |
| Chunk long pages for context windows | **FAF** |
| Crawl with rate-limiting and concurrency | **FAF** |
| Run on low-resource VPS (5MB RAM) | **FAF** |
| Offline / air-gapped environments | **FAF** |
| Archive pages as self-contained HTML | **FAF** |
| Privacy-sensitive scraping (no SaaS) | **FAF** |
| Execute JS-dependent SPAs (React, Vue) | Playwright |
| Bypass anti-bot detection | Playwright |
| Pixel-perfect screenshots | Playwright |
| Automate login flows via real browser | Playwright |

**FAF is not a browser automation tool.** It does not run Chromium, handle SPAs, or bypass anti-bot detection. It is a high-performance scraper with JS execution — and it's damn good at it.

---

## Contributing

```bash
cargo test    # 360 tests
cargo clippy
cargo fmt
```

Conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`.

---

## Support

<details>
<summary><b>☕ Buy me a coffee (entirely optional)</b></summary>

FAF is free, open-source, and will always be. If it saved you time or RAM, consider buying me a coffee — no expectations.

**Pix (any amount):**

```
e4636446-8087-48b7-bca0-61481687fe27
```



</details>

---

MIT © [Tiago Coelho](https://github.com/tiagoc93) — Recife, Brasil.
