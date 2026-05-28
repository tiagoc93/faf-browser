# 🎯 FAF Browser — M8: Layout Parity (float, inline-block, flex)

**Status:** Em andamento
**Previsão:** 5-7 dias
**Meta:** Render visual comparável ao Playwright em páginas com layouts multi-coluna

---

## 🎯 Objetivo

O M7 entregou as primitivas CSS (absolute, fixed, overflow, text-align, line-height, font cache, CSS externo, URLs relativas). O M8 fecha o gap de **layout engine** que impede o FAF de renderizar páginas com grid/flex/float — responsável por 90% do visual "estranho" ao comparar com o browser real.

**Referência:** `books.toscrape.com` render deve mostrar produtos lado-a-lado (não empilhados verticalmente).

---

## 📋 Tasks

### 🔴 T059 — `display: inline-block` (2 dias)

**Objetivo:** Suporte a `inline-block` no layout engine — elementos que se comportam como inline na linha mas podem ter width/height definidos.

**Arquivos:** `src/render/layout.rs`, `src/render/tree.rs`

| Subtask | Descrição |
|---------|-----------|
| T059.1 | Classificar `display: inline-block` como NodeType que participa de inline flow |
| T059.2 | Inline-block tem width/height definidos (diferente de inline puro) |
| T059.3 | Inline-block cria "block formatting context" interno — filhos são block layout |
| T059.4 | Quebra de linha quando inline-blocks excedem largura do container |
| T059.5 | Vertical-align baseline para inline-blocks |

**Critério de aceite:** 4 divs com `display: inline-block; width: 25%` aparecem lado-a-lado em viewport 800px.

---

### 🔴 T060 — `float: left/right` (2 dias)

**Objetivo:** Implementar float básico — elementos que "flutuam" à esquerda ou direita, com conteúdo inline fluindo ao redor.

**Arquivos:** `src/render/layout.rs`, `src/render/tree.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T060.1 | Adicionar `float: String` ao ComputedStyle (default: "none") |
| T060.2 | Elements com `float: left` saem do fluxo normal e se posicionam à esquerda |
| T060.3 | Float right — posiciona à direita do container |
| T060.4 | Conteúdo inline subsequente flui ao redor do float (wrap) |
| T060.5 | `clear: both` — elemento desce abaixo de todos os floats anteriores |
| T060.6 | Múltiplos floats lado-a-lado (stack horizontal) |

**Edge cases:**
- Float sem width definido: shrink-to-fit
- Float que não cabe na linha: desce para próxima posição disponível
- `overflow: hidden` no pai cria BFC que contém floats

**Critério de aceite:** 5 elementos com `float: left; width: 20%` aparecem em uma linha.

---

### 🟡 T061 — `display: flex` básico (2 dias)

**Objetivo:** Flexbox simplificado — direction row/column, justify-content, align-items.

**Arquivos:** `src/render/layout.rs`, `src/render/tree.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T061.1 | Adicionar `flex_direction`, `justify_content`, `align_items`, `flex_wrap` ao ComputedStyle |
| T061.2 | `flex-direction: row` (default) — filhos lado-a-lado |
| T061.3 | `flex-direction: column` — filhos empilhados (similar ao block atual) |
| T061.4 | `justify-content: flex-start/center/flex-end/space-between/space-around` |
| T061.5 | `align-items: flex-start/center/flex-end/stretch` |
| T061.6 | `flex-wrap: wrap` — quebra para próxima linha quando excede |
| T061.7 | `flex: 1` shorthand — distribui espaço proporcionalmente |

**⚠️ Limitação M8:** NÃO implementar `order`, `flex-grow` complexo, `align-self`, `gap`. Foco no essencial.

**Critério de aceite:** Nav com `display: flex; justify-content: space-between` distribui itens corretamente.

---

### 🟡 T062 — CSS Matching melhorado (1 dia)

**Objetivo:** Melhorar `find_style_for_element_ref()` para considerar ancestor selectors e evitar match incorreto entre elementos iguais.

**Arquivos:** `src/render/tree.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T062.1 | Usar node index/path para identificar elementos únicos |
| T062.2 | Resolver ancestor selectors (ex: `.parent .child`) corretamente |
| T062.3 | Inline `style=""` atributos têm prioridade sobre CSS |
| T062.4 | Pseudo-classes ignoradas (não são visuais pro screenshot) |

**Critério de aceite:** Todos os `.product_pod` recebem os mesmos estilos corretamente.

---

### 🟡 T063 — `background-image` (1 dia)

**Objetivo:** Suportar `background-image: url(...)` com download e render.

**Arquivos:** `src/render/screenshot.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T063.1 | Adicionar `background_image: String` ao ComputedStyle |
| T063.2 | Parsing de `background-image: url("...")` |
| T063.3 | Download da imagem (com base_url para relativas) |
| T063.4 | Render da imagem como background (resize para fill/contain) |

**Critério de aceite:** Elemento com `background-image: url("pattern.png")` renderizado com a imagem.

---

### 🔴 T064 — Testes M8 + Regressão (1 dia)

**Arquivos:** `tests/m8_test.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T064.1 | Teste inline-block lado-a-lado |
| T064.2 | Teste float left + right |
| T064.3 | Teste flex-direction: row + justify-content |
| T064.4 | Teste flex-wrap: wrap |
| T064.5 | Teste background-image render |
| T064.6 | Regressão: 297+ testes existentes continuam passando |

**Critério de aceite:** 310+ testes passando, 0 falhas, clippy limpo.

---

## 🧪 Critérios de Aceite Gerais

- [ ] `display: inline-block` funciona com width/height
- [ ] `float: left/right` posiciona elementos corretamente
- [ ] `display: flex` com direction row/column funciona
- [ ] `justify-content` e `align-items` básicos funcionam
- [ ] `background-image: url(...)` carrega e renderiza
- [ ] CSS matching identifica elementos corretamente
- [ ] Screenshot de `books.toscrape.com` mostra produtos lado-a-lado
- [ ] Screenshot de site com flexbox mostra layout correto
- [ ] 310+ testes passando
- [ ] Build release com 0 warnings

---

## 📊 Comparativo: M7 → M8

| Aspecto | M7 (agora) | M8 (meta) |
|---------|-----------|-----------|
| Display | block + inline | **+ inline-block** |
| Float | ❌ | **left + right** |
| Flex | ❌ | **row + column + justify + align** |
| Background | Só color | **+ image** |
| CSS match | (tag, id, classes) | **Melhorado com path** |
| Inline style | Parcial | **Completo** |
| Testes | 297 | **310+** |
