# 📋 RELATÓRIO M2 — FAF BROWSER (CSS Engine MVP)

**Data:** 28/05/2026
**Autor:** Hermes Agent (DeepSeek V4 Pro) → Kimi K2.6

---

## ✅ Tasks Concluídas

**Todas as 8 tasks do M2 foram implementadas com sucesso.**

| # | Task | Status | Arquivos |
|---|---|---|---|
| **T013** | Parser CSS com cssparser | ✅ | `src/css/parser.rs` |
| **T014** | Selector matching com selectors | ✅ | `src/css/selector.rs` |
| **T015** | Computed styles + cascata | ✅ | `src/css/style.rs` |
| **T016** | Box model via tiny-skia | ✅ | `src/css/layout.rs` |
| **T017** | Cores e backgrounds | ✅ | `src/css/color.rs` |
| **T018** | Fontes e fallbacks | ✅ | `src/css/font.rs` |
| **T019** | CLI integração — `faf query` + `--style` | ✅ | `src/api/commands.rs`, `src/api/output.rs` |
| **T020** | Testes de integração M2 | ✅ | `tests/m2_test.rs` |

---

## 📊 Estatísticas

| Métrica | Valor |
|---|---|
| **Total de testes** | **166** (146 unit + 20 integração) |
| **Testes passando** | 166 ✅ |
| **Testes falhando** | 0 ❌ |
| **cargo clippy** | Limpo ✅ |
| **cargo build --release** | Compila ✅ |
| **Commits M2** | 5 commits |
| **Linhas de código (src/css/)** | **2.302** (7 arquivos) |
| **Novos módulos CSS** | `parser`, `selector`, `style`, `layout`, `color`, `font` |

---

## 📂 Estrutura do CSS Engine

```
src/css/
├── mod.rs          ← 6 submodulos (parser, selector, style, layout, color, font)
├── parser.rs       ← 370 linhas — Parse de CSS com cssparser (suporta @rules, comentários)
├── selector.rs     ← 304 linhas — Matching com scraper + cálculo de especificidade
├── style.rs        ← 441 linhas — ComputedStyle + cascata (inline > ID > class > tag)
├── layout.rs       ← 304 linhas — BoxModel em pixels, shorthand margin/padding
├── color.rs        ← 495 linhas — 19 cores nomeadas, hex, rgb/rgba, blending
└── font.rs         ← 382 linhas — Font family, size, weight parsing
```

---

## 🔧 Problemas Encontrados e Soluções

### 1. Timeout em delegate_task com tasks grandes
- **Problema:** T013+T014 (primeira tentativa) e T015+T016 (primeira tentativa) estouraram timeout de 600s
- **Solução:** Dividir em batches menores (T013, depois T014, T015 sozinho, etc.)

### 2. scrapper::Selector não suporta :hover
- **Problema:** Teste em `selector.rs` usava `div#app.active:hover` que `scraper::Selector::parse()` rejeita
- **Solução:** Simplificar teste para `div#app.active`

### 3. cssparser 0.34 API incompatível
- **Problema:** `Token::Whitespace` não existe; é `Token::WhiteSpace(_)`. `to_string()` substituído por `to_css_string()`
- **Solução:** Ajustar para a API real da versão 0.34

### 4. Especificidade CSS — `#nav ul li`
- **Problema:** Teste esperava 103, mas o valor correto é 102 (1 id + 2 tags)
- **Solução:** Corrigir valor esperado no teste

### 5. Shorthand de font-size: "rem" vs "em"
- **Problema:** `1.2rem` era parseado como `1.2r` + `em`
- **Solução:** Testar "rem" antes de "em" no parser

---

## 🚀 Como usar

```bash
# Query com CSS (inline)
faf query "h1" --url https://example.com --style "h1 { color: red; font-size: 24px; }"

# Query com CSS (arquivo)
echo "h1 { color: blue; }" > style.css
faf query "h1" --url https://example.com --style style.css

# Query com CSS + JSON
faf query "h1" --url https://example.com --style "h1 { color: red; }" --json
```

---

## 📈 Próximos Passos Sugeridos (M3+)

1. **M3 — JavaScript Engine (T021-T030):** Embed QuickJS, bridge DOM↔JS, setTimeout, fetch
2. **M4 — API Pública (T031-T038):** Modo comando único, cookies, WaitForSelector
3. **M5 — Extração (T039-T044):** Links, imagens, metadados, click via JS
4. **README.md:** Documentar o projeto com exemplos de uso

---

## 💻 Histórico de Commits

```
ba556fb feat: M2 — CLI com CSS (T019) + testes integracao (T020)
6dcb97b feat: M2 — parsing de cores (T017) + fontes (T018)
d866184 feat: M2 — box model com css_to_pixels e shorthand (T016)
ee4ea36 feat: M2 — computed styles com cascata (T015)
5edd26b feat: M2 — CSS parser (T013) + selector matching (T014)
386d81d feat: M1 — Core Engine (HTTP, DOM, parser, CLI, testes)
```
