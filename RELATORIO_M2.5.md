# 🎯 FAF Browser — M2.5 Relatório Final

**Data:** 28/05/2026
**Branch:** master
**Último commit:** d5a93a5 — feat: M2.5 — Q03 subcomando follow

---

## ✅ Status: **8/8 tasks concluídas**

| Task | Nome | Arquivos | Status | Commit |
|------|------|----------|--------|--------|
| P01 | `--css` e `--json` em qualquer posição | commands.rs | ✅ | b9072e1 |
| P02 | Remover `--query` morto | commands.rs | ✅ | b9072e1 |
| P03 | Defaults visíveis no ComputedStyle | style.rs | ✅ | b9072e1 |
| P04 | Parse CSS automático da página | parser.rs, commands.rs | ✅ | 73aa182 |
| Q01 | `--filter` para query | filter.rs (novo), commands.rs | ✅ | 193398d |
| Q02 | `--get` para extrair atributos | commands.rs, output.rs | ✅ | 2b6f952 |
| Q03 | Subcomando `follow` | commands.rs, output.rs, client.rs | ✅ | d5a93a5 |
| Q04 | `--format csv\|jsonl` | commands.rs, output.rs, Cargo.toml | ✅ | 2b6f952 |

---

## 📦 Commits da M2.5

```
b9072e1 M2.5: P03 defaults ComputedStyle, P01 --css/--json global, P02 remove --query dead flag
2b6f952 feat: M2.5 — Q02 --get e Q04 --format csv|jsonl
73aa182 P04: Parse CSS automático da página no FAF Browser
193398d Q01: implementa --filter para query CSS
d5a93a5 feat: M2.5 — Q03 subcomando follow
```

---

## 🧪 Testes

| Tipo | Resultado |
|------|-----------|
| Unit tests | 160 passed, 0 failed |
| Integration tests | 20 passed, 0 failed |
| **Total** | **180 passed, 0 failed** |
| `cargo build --release` | ✅ |
| `cargo clippy` | ✅ (sem warnings) |
| `cargo fmt` | ✅ |

---

## 🔧 Novos Recursos Implementados

### P01 — Flags globais
- `--css` e `--json` agora funcionam em **qualquer posição** na linha de comando (antes/depois de subcomandos)
- Implementado via `global = true` no clap derive

### P02 — Limpeza
- Flag `--query` removida do struct `Cli` (nunca era lida — só o subcomando `query` funcionava)

### P03 — Defaults CSS reais
- `ComputedStyle::default()` agora retorna valores CSS reais:
  - `display: "block"`, `color: "inherit"`, `background_color: "transparent"`
  - `font_size: "16px"`, `font_family: "serif"`
  - `width/height: "auto"`, `margin/padding: "0"`

### P04 — CSS automático da página
- `extract_page_stylesheets()` em `parser.rs` extrai CSS de tags `<style>` e `<link rel="stylesheet">`
- Download de CSS externo via HTTP com resolução de URLs relativas
- Flag `--no-page-css` para desabilitar
- Falha graceful em stylesheet offline (log warning)

### Q01 — Filtragem de query
- Novo módulo `src/api/filter.rs`
- `--filter "campo~=valor"`, `--filter "campo==valor"`, `--filter "campo!=valor"`, etc.
- Operadores: `~=` (substring), `==` (exact), `!=` (negado), `^=` (prefixo), `$=` (sufixo), `=` (regex auto-detect)
- Múltiplos filtros combinam com AND

### Q02 — Extração seletiva
- `--get "campo1, campo2"` limita output aos campos desejados
- Suporte a: tag, id, classes, text, html, href, src, alt, color, bg, font-size, font-family, display

### Q03 — Subcomando follow
- `faf follow <seletor> --url <base> [--extract <sel>] [--max N] [--concurrency N]`
- Crawleia links, visita páginas concorrentemente, extrai dados
- Flags: `--extract`, `--max` (10), `--concurrency` (3), `--same-domain` (true)
- Tokio semaphore para limite de concorrência
- Output em todos os formatos (text, json, jsonl, csv)

### Q04 — Formatos de saída
- `--format csv|jsonl|json|text`
- CSV com escaping de vírgulas e aspas
- JSONL: um JSON por linha (pipe-friendly)

---

## 📁 Arquivos novos

| Arquivo | Descrição |
|---------|-----------|
| `src/api/filter.rs` | Módulo de filtragem de query CSS |

## 📁 Arquivos modificados

| Arquivo | Mudanças |
|---------|----------|
| `src/api/commands.rs` | Flags globais, --get, --format, --filter, --no-page-css, Command::Follow |
| `src/api/output.rs` | filter_fields, to_jsonl, to_csv, FollowPageResult |
| `src/css/style.rs` | Default CSS reais (P03) |
| `src/css/parser.rs` | extract_page_stylesheets (P04) |
| `src/api/mod.rs` | pub mod filter |
| `src/http/client.rs` | Clone derive, inner_client() |
| `Cargo.toml` | regex dependency |

---

## 📊 Estatísticas do Projeto

```
Linhas de código: ~5300 (Rust)
Módulos: 20 arquivos .rs
Testes: 180 (160 unit + 20 integration)
```

---

## 🚀 Exemplos de uso

```bash
# Query com CSS automático da página
faf query "h1" --url https://books.toscrape.com/

# Filtrar por texto e extrair campos específicos
faf query "a" --filter "text~=Python" --get "text, href" --url https://books.toscrape.com/

# Seguir links de produtos e extrair dados
faf follow ".product a" --url https://books.toscrape.com/ --extract "h3, .price_color" --max 3 --json

# CSV pipe-friendly
faf query "a" --get "text, href" --format csv --url https://books.toscrape.com/

# JSONL para pipeline
faf query "h1" --format jsonl --url https://books.toscrape.com/ | head -5
```
