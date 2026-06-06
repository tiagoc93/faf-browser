# FAF Browser — Bug Fix Tasks

## Contexto
FAF Browser é um scraper CLI100% Rust. Após review, foram identificados bugs em features centrais.

---

## TASK-001: Corrigir `dump --readability` retornando vazio

**Sintoma:** `faf dump --format markdown --readability` output apenas o warning HTML, sem conteúdo extraído.

**Expected:** Conteúdo principal da página (artigo, texto principal) após strip de navigation/sidebars/footers/ads.

**Reproduction:**
```bash
./target/release/faf-browser https://books.toscrape.com/ dump --format markdown --readability
# Saída atual: só o warning HTML
# Esperado: conteúdo legível da página
```

**Root cause presumida:** readability algorithm está retornando vazio ou o warning está sobrepondo o conteúdo.

**Passos:**
1. Investigar o código do readability em `src/dump/` — como funciona o text-density scoring
2. Verificar se o warning HTML está sendo tratado como conteúdo
3. Adicionar testes unitários para readability
4. Testar em páginas reais (books.toscrape.com, um blog simples)

---

## TASK-002: Corrigir `dump --structured-data` retornando incompleto

**Sintoma:** `--structured-data` retorna apenas `meta`, sem `json_ld` e `open_graph`.

**Expected:** `{ "json_ld": [...], "open_graph": {...}, "meta": {...} }` — com arrays vazios se não encontrar, não campos faltantes.

**Reproduction:**
```bash
./target/release/faf-browser https://books.toscrape.com/ dump --structured-data
# Output: só meta fields, sem json_ld/open_graph
```

**Passos:**
1. Investigar `src/dump/` para structured data extraction
2. Verificar como json_ld, open_graph, microdata são extraídos
3. Garantir que campos não-encontrados retornem como `null` ou `[]` (não omitidos)
4. Testar com site que tenha JSON-LD real (ex: um artigo Medium ou news site)

---

## TASK-003: Verificar e corrigir `dump --format markdown` standalone

**Sintoma:** `dump --format markdown` sem `--readability` funciona mas tem formatação inconsistente.

**Reproduction:**
```bash
./target/release/faf-browser https://books.toscrape.com/ dump --format markdown
# Verificar: títulos são # ou ##? Listas estão corretas? Links são [text](url) ou URLs cruas?
```

**Passos:**
1. Testar markdown output em páginas variadas
2. Verificar edge cases: tables, blockquotes, code blocks, imagens
3. Comparar com spec CommonMark
4. Corrigir formatação onde necessário

---

## TASK-004: Teste de integração em páginas reais

**Após TASK-001, TASK-002, TASK-003 estarem prontos.**

**Sites de teste:**
- https://books.toscrape.com/ (demo, sem JS pesado)
- https://example.com (HTML simples)
- Um artigo Medium oudev.to (para JSON-LD real)

**Checks:**
1. `dump --format markdown --readability` extrai conteúdo limpo
2. `dump --structured-data` retorna todos os 3 campos (json_ld, open_graph, meta)
3. `follow` com `--extract` retorna dados estruturados corretos
4. `query` com computed styles funciona
5. JS execution (`--js`) funciona
6. Todos os 329+ testes unitários passam (`cargo test`)

---

## TASK-005: Normalizar output de versionamento

**Sintoma:** `faf --version` retorna "faf 0.1.0" mas o binário se chama `faf-browser`.

**Passos:**
1. Verificar onde a string "faf 0.1.0" é definida (Cargo.toml? main.rs?)
2. Corrigir para "faf-browser 0.1.0" ou apenas "0.1.0"
3. Garantir consistência entre --version, --help header, e binary name

---

## TASK-006: Padronizar idioma do output

**Sintoma:** `--help` está em português mas README e output padrão são em inglês.

**Passos:**
1. Verificar em `src/api/` onde mensagens de help são definidas
2. Passar todas as mensagens para inglês (ou criar flag --lang)
3. Garantir consistência: todo output CLI em inglês

---

## Critério de Aceite Final

- [ ] `cargo test` passa com 0 falhas
- [ ] `dump --readability` extrai conteúdo real
- [ ] `dump --structured-data` retorna json_ld, open_graph, meta
- [ ] `dump --format markdown` gera Markdown válido
- [ ] Output em inglês consistente
- [ ] Version string consistente
