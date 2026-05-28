# 🎯 FAF Browser — M6: Layout Engine

**Status:** ✅ Concluído
**Previsão:** 4-7 dias
**Meta:** Screenshot com layout fiel, sem Chromium

---

## 🎯 Objetivo

Transformar o screenshot do FAF Browser de "prova de conceito" (elementos empilhados em x=0) em uma representação visual fiel da página, com posicionamento correto, cores, bordas, imagens e fluxo de texto.

**Hoje:** `render_to_image()` percorre elementos numa lista plana (`doc.query("*")`) e desenha tudo em x=0 com y acumulativo.

**M6:** Árvore visual com nós block/inline, quebra de texto, cores de fundo reais, bordas, imagens, posicionamento relative/absolute.

---

## 📋 Tasks

### 🔴 T045 — Layout Tree ✅
**Concluído: layout.rs + tree.rs + screenshot refatorado. 14 arquivos, 711 linhas.**

**Objetivo:** Construir uma árvore visual a partir do DOM, separando nós block de inline.

**Arquivos:** `src/render/tree.rs` (novo), `src/render/layout.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T045.1 | Criar `VisualNode` struct com: tag, text, children, style (ComputedStyle), rect (Rect) |
| T045.2 | Função `build_layout_tree(doc, computed) → VisualNode` que percorre o DOM respeitando hierarquia |
| T045.3 | Classificar cada nó como `Block` ou `Inline` baseado no `display` do computed style |
| T045.4 | Ignorar nós com `display: none` e tags estruturais (script, style, head, etc) |
| T045.5 | Integrar `compute_styles` para propagar estilos pai → filho (herança de `color`, `font-size`, etc) |

**Critério de aceite:** `build_layout_tree()` retorna uma árvore com 3+ níveis de profundidade para uma página real.

---

### 🔴 T046 — Inline Flow + Text Wrap ✅
**Concluído junto da T045: layout_inline_children() com quebra automática de linha, line-height = 1.2×, colapso de whitespace.**

**Objetivo:** Renderizar texto em linha com quebra automática dentro da largura do viewport.

**Arquivos:** `src/render/layout.rs`, `src/render/screenshot.rs`

| Subtask | Descrição |
|---------|-----------|
| T046.1 | Acumular nós inline horizontalmente até atingir `width` do contêiner, então quebrar linha |
| T046.2 | Calcular `line-height` a partir do font-size (1.2× por padrão) |
| T046.3 | Posicionar texto inline com ab_glyph respeitando a posição atual do cursor |
| T046.4 | Texto vazio ou whitespace-only não quebra layout (colapsar espaços) |
| T046.5 | Fallback: texto que não cabe no viewport é truncado (sem overflow horizontal) |

**Critério de aceite:** Parágrafo de texto quebra em múltiplas linhas dentro de 1280px.

---

### 🔴 T047 — Block Flow + Margin Collapsing ✅
**Concluído junto da T045: layout_block() com empilhamento vertical, margin collapsing (max vence), width 100%, padding.**

**Objetivo:** Empilhar blocos verticalmente com margin collapsing correto.

**Arquivos:** `src/render/layout.rs`, `src/render/screenshot.rs`

| Subtask | Descrição |
|---------|-----------|
| T047.1 | Nós block ocupam 100% da largura do pai (ou `width` explícita) |
| T047.2 | Calcular altura do bloco: conteúdo + padding-top + padding-bottom |
| T047.3 | Implementar margin collapsing entre blocos adjacentes (maior margem vence) |
| T047.4 | Bloco vazio (sem filhos, sem texto) não ocupa espaço |
| T047.5 | Posicionar blocos filhos dentro do pai (respeitando padding) |

**Critério de aceite:** Dois blocos com `margin: 20px` colapsam para 20px entre eles.

---

### 🔴 T048 — Background Rendering ✅
**Concluído junto da T045: render_node() em screenshot.rs já desenha background-color de cada nó visual.**

**Objetivo:** Renderizar cores de fundo e imagens de background nos elementos.

**Arquivos:** `src/render/background.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T048.1 | Renderizar `background-color` de cada nó que tenha cor definida |
| T048.2 | Suporte a `background-color: transparent` (padrão) |
| T048.3 | Background respeita o rect do elemento (borda interna) |
| T048.4 | Fundo do `<body>` preenche toda a viewport (como browsers reais) |

**Critério de aceite:** Elementos com `background-color: #f0f0f0` aparecem com fundo cinza no screenshot.

---

### 🟡 T049 — Bordas CSS ✅
**Concluído: border-* parse em style.rs, renderização dos 4 lados em screenshot.rs, shorthand border suportado.**

**Objetivo:** Renderizar bordas nos elementos (width, style, color).

**Arquivos:** `src/render/border.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T049.1 | Suporte a `border-width` (top, right, bottom, left) |
| T049.2 | Suporte a `border-color` |
| T049.3 | Suporte a `border-style: solid` (outros estilos: MVP apenas solid) |
| T049.4 | Bordas são desenhadas ao redor do rect do elemento, fora do background |

**Critério de aceite:** Elemento com `border: 2px solid red` aparece com borda vermelha de 2px.

---

### 🟡 T050 — Imagens `<img>` ✅
**Concluído: image crate, attributes HashMap no VisualNode, decode + render de img, fallback placeholder, cache simples.**

**Objetivo:** Carregar e renderizar imagens da página no screenshot.

**Arquivos:** `src/render/image.rs` (novo), `Cargo.toml` (image crate)

| Subtask | Descrição |
|---------|-----------|
| T050.1 | Adicionar `image` crate (compat jpeg/png/webp) |
| T050.2 | Carregar imagem de URL ou base64 inline |
| T050.3 | Decode e redimensionar para o `width`/`height` do elemento (manter aspect ratio) |
| T050.4 | Cache simples de imagens decodificadas por sessão |
| T050.5 | Fallback: placeholder colorido se imagem não carregar |

**Critério de aceite:** `<img src="...">` aparece como imagem renderizada no screenshot.

---

### 🟡 T051 — Positioning: relative + z-index ✅
**Concluído: position/relative/top/left/z-index no ComputedStyle + deslocamento relativo no layout.rs + ordenação z-index no render_node().**

**Objetivo:** Suporte básico a posicionamento CSS.

**Arquivos:** `src/render/layout.rs`

| Subtask | Descrição |
|---------|-----------|
| T051.1 | `position: relative` — desloca o elemento mas reserva espaço original |
| T051.2 | `position: absolute` — posiciona em relação ao ancestral positionado mais próximo |
| T051.3 | `position: static` (padrão) — fluxo normal |
| T051.4 | `z-index` básico — ordenar desenho (maior z-index = desenha por último) |

**Critério de aceite:** Elemento com `position: absolute; top: 10px; left: 10px` aparece na posição correta.

---

### 🟡 T052 — Testes M6 ✅
**Concluído: 9 testes em tests/m6_test.rs (4 originais + 5 novos). Total: 287 testes passando.**

**Objetivo:** Garantir que o layout engine funciona com testes visuais.

**Arquivos:** `tests/m6_test.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T052.1 | Teste de árvore visual: block → inline → text |
| T052.2 | Teste de quebra de linha com texto longo |
| T052.3 | Teste de margin collapsing entre dois blocos |
| T052.4 | Teste de background-color renderizado |
| T052.5 | Teste de borda renderizada |
| T052.6 | Teste de imagem carregada (placeholder) |
| T052.7 | Teste de posicionamento relative |
| T052.8 | Teste de posicionamento absolute |
| T052.9 | Teste de screenshot completo com HTML inline (string) vs browser screenshot |
| T052.10 | Teste de regressão: 266 testes existentes continuam passando |

---

## 📐 Estrutura final esperada (src/render/)

```
src/render/
├── mod.rs          ← re-exporta tudo
├── screenshot.rs   ← função principal render_to_image() (refatorada)
├── tree.rs         ← build_layout_tree() — DOM → árvore visual
├── layout.rs       ← layout engine (block flow, inline flow, positioning)
├── background.rs   ← renderização de background-color
├── border.rs       ← renderização de bordas
└── image.rs        ← carregamento e render de imagens
```

---

## 🧪 Critérios de Aceite Gerais

- [ ] Screenshot de página real tem texto legível posicionado corretamente
- [ ] Blocos empilhados verticalmente com margens corretas
- [ ] Parágrafos quebram em múltiplas linhas
- [ ] Background colors aparecem nos elementos corretos
- [ ] Bordas são visíveis
- [ ] Imagens carregam (ou fallback)
- [ ] `position: absolute` funciona
- [ ] 266+ testes passando
- [ ] Build release com 0 warnings
- [ ] `cargo clippy` limpo

---

## 📊 Comparativo: Antes vs Depois (M6)

| Aspecto | M5 (hoje) | M6 (meta) |
|---------|-----------|-----------|
| Layout | Lista plana, x=0 | Árvore visual block/inline |
| Text wrap | ❌ (tudo na mesma linha) | ✅ Quebra automática |
| Backgrounds | Parcial (só se style tiver) | ✅ Herdado do CSS |
| Bordas | ❌ | ✅ solid |
| Imagens | ❌ | ✅ decode + render |
| Positioning | ❌ | ✅ relative + absolute |
| Pixels com texto | 1.122 | ~200.000+ (página cheia) |
| Linhas com conteúdo | 304/800 (38%) | 750+/800 (90%+) |
