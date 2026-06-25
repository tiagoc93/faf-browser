# Tasks — faf-browser

<!-- Mantenha este arquivo sincronizado com as issues do GitHub. -->
<!-- Formato: [ID] [Status] Descrição -->

---

## T002 — Correções e otimizações pós-M7

- **Status:** Pendente
- **Prioridade:** Média
- **Estimativa:** ~60 LOC total

---

### T002.1 — Separador de coluna em tabelas Markdown (~5 LOC)

- [ ] **Problema:** `convert_table()` em `src/dump/markdown.rs` gera linhas `| cell | cell |` mas não insere a linha separadora GFM `|---|---|` após o header
- [ ] **Arquivo:** `src/dump/markdown.rs` → `convert_table()`
- [ ] **Fix:** Após escrever a primeira linha (header), inserir `| --- | --- | ...` com `ncols` colunas
- [ ] **Teste:** HTML com `<table>` → output contém `|---|` após primeira linha

---

### T002.2 — Evitar double-parse de `extract_structured_data` no chunking (~10 LOC)

- [ ] **Problema:** Em `src/dump/mod.rs` → `dump_to_string()`, quando `frontmatter=true` + `chunk_size>0`, `inject_frontmatter()` é chamada 2x — linha 86 e linha 95 — re-parseando o HTML inteiro
- [ ] **Arquivo:** `src/dump/mod.rs`
- [ ] **Fix:** Extrair frontmatter uma vez, reutilizar em ambos caminhos
- [ ] **Teste:** Verificar que `extract_structured_data` é chamada uma vez só (ou comparar output antes/depois)

---

### T002.3 — Regexes de `collapse_whitespace` com LazyLock (~5 LOC)

- [ ] **Problema:** `collapse_whitespace()` em `src/dump/markdown.rs` compila 2 regexes (`url_re` e `\n{3,}`) a cada chamada
- [ ] **Arquivo:** `src/dump/markdown.rs` → `collapse_whitespace()`
- [ ] **Fix:** Usar `std::sync::LazyLock<Regex>` (Rust 2024) para compilar uma vez
- [ ] **Teste:** Existente `test_collapse_whitespace_*` continua passando

---

### T002.4 — Fallback de URL no frontmatter (~8 LOC)

- [ ] **Problema:** Campo `url` no frontmatter vem de `og:url` — muitas páginas não têm essa tag, então o campo fica ausente mesmo quando sabemos a URL
- [ ] **Arquivo:** `src/dump/markdown.rs` → `inject_frontmatter()`
- [ ] **Fix:** Se `og:url` não existir, usar `DumpConfig.base_url` como fallback
- [ ] **Impacto:** `inject_frontmatter()` precisa receber a `base_url` como parâmetro extra
- [ ] **Teste:** HTML sem `og:url` + `base_url` definida → frontmatter contém `url: <base_url>`

---

## T003 — MCP Server Native

- **Status:** Pendente
- **Prioridade:** Alta
- **Estimativa:** ~300-500 LOC
- **Arquivo novo:** `src/mcp/mod.rs` (ou binário separado `faf-mcp`)

### Contexto

Expor FAF como servidor MCP (Model Context Protocol) para que agentes como Claude Desktop, Cursor, Hermes e outros consumam `faf dump`, `faf fetch` e extração estruturada diretamente — sem subprocess, sem shell.

### Subtarefas

- [ ] Implementar MCP server (stdio transport) expondo tools:
  - `faf_fetch(url)` → HTML + metadata
  - `faf_dump(url, format, readability)` → markdown/text/json/html
  - `faf_query(url, selector)` → elementos extraídos via CSS
  - `faf_crawl(url, selector, max)` → crawl com follow
- [ ] Schema JSON para cada tool (input/output)
- [ ] Binário: `faf-mcp` ou subcomando `faf mcp serve`
- [ ] Documentação de configuração para clientes MCP
- [ ] Testes básicos de protocolo

### Referências

- Spec MCP: https://modelcontextprotocol.io/
- Já existe `src/api/commands.rs` com `run()` reutilizável