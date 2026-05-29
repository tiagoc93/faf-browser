## 🔧 M8.5 — Screenshot Parity (Fase Corretiva)

**Status:** Em andamento
**Meta:** Screenshot de books.toscrape.com visualmente comparável ao Playwright
**Estratégia:** Corrigir os 3 bugs de renderização que mais impactam, testando visualmente a cada passo

---

### Diagnóstico

O CSS da books.toscrape.com (Bootstrap 3) depende de:
1. `box-sizing: border-box` — NÃO implementado → todos os elementos têm largura errada
2. `.row::before, .row::after { display: table; content: "" }` — clearfix → pseudo-elements NÃO implementados
3. `.col-sm-* { float: left; width: X% }` — grid → float existe mas falha sem clearfix

---

### 🔴 T065 — `box-sizing: border-box`
**Impacto:** Crítico — afeta todo elemento com padding/border
**Arquivos:** `src/css/layout.rs`, `src/css/style.rs`
- Adicionar `box_sizing` ao ComputedStyle
- Se `border-box`: `content_width = width - padding - border` em vez de `width = content_width`
- Bootstrap define `*, *::before, *::after { box-sizing: border-box }`

### 🔴 T066 — Pseudo-elements `::before` e `::after`
**Impacto:** Crítico — necessário para clearfix (`.row::before, .row::after`)
**Arquivos:** `src/css/parser.rs`, `src/css/selector.rs`, `src/render/tree.rs`
- Parsear seletores com `::before`/`::after`
- Criar VisualNode sintético para pseudo-elements
- `.row::before, .row::after { content: ""; display: table; }`

### 🟡 T067 — CSS matching com seletor universal
**Impacto:** Alto — `* { box-sizing: border-box }` não funciona
**Arquivos:** `src/css/selector.rs`
- `*` seletor universal precisa matchear todos os elementos

### 🟡 T068 — `line-height` normal = 1.42857 (Bootstrap)
**Impacto:** Médio — afeta espaçamento vertical
**Arquivos:** `src/render/layout.rs`
- Bootstrap define `body { line-height: 1.42857143 }` — nosso "normal" = 1.2

---

### 📊 Progresso

| Task | Status | Resultado visual |
|------|:------:|------------------|
| T065 — box-sizing | ⬜ | - |
| T066 — Pseudo-elements | ⬜ | - |
| T067 — Seletor universal | ⬜ | - |
| T068 — line-height | ⬜ | - |
