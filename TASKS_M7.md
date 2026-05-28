# 🎯 FAF Browser — M7: Playwright Parity + Polish

**Status:** Planejado
**Previsão:** 4-6 dias
**Meta:** Screenshot fiel à página, com qualidade visual comparável ao Playwright

---

## 🎯 Objetivo

O M6 entregou um motor de layout funcional com árvore visual, fluxo block/inline, text wrap, backgrounds, bordas, imagens e relative positioning. O M7 fecha as lacunas restantes para que o screenshot do FAF Browser seja fiel ao que é visto na página real.

A referência é a **qualidade** do Playwright, não a ferramenta em si. A verificação é visual: renderiza uma página real, olha, ajusta, repete.

**Áreas de melhoria identificadas no M6:**
1. `position: absolute` — implementação incompleta (timeout no Kimi)
2. `position: fixed` — não implementado
3. Text rendering — `estimate_text_width()` é heurística (chars × font_size × 0.5), não real
4. Line-height — fixo em 1.2×, sem suporte a line-height explícito
5. Semântica visual — `text-align`, `font-weight` sem efeito real
6. Overflow — conteúdo que extravasa o container não é clipado

---

## 📋 Tasks

### 🔴 T053 — position: absolute + fixed (2 dias)

**Objetivo:** Completar o suporte a posicionamento CSS que foi cortado por timeout no M6.

**Arquivos:** `src/render/layout.rs`, `src/render/tree.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T053.1 | Adicionar `bottom`, `right` ao ComputedStyle (já tem top, left) |
| T053.2 | `position: absolute` — elemento removido do fluxo normal do pai (não contribui pra altura) |
| T053.3 | Encontrar ancestral positionado (first parent cujo `position ≠ static`) |
| T053.4 | Usar viewport (0,0) como fallback se nenhum ancestral positionado |
| T053.5 | Posicionar absolute baseado em top/right/bottom/left em relação ao **containing block** |
| T053.6 | `position: fixed` — posicionar em relação à viewport sempre (ignora scroll) |
| T053.7 | Elementos absolute NÃO iterados no fluxo normal — lista separada pra render |
| T053.8 | z-index também funciona com absolute (já implementado no relative) |

**⚠️ Edge cases:**
- Ancestral positionado = qualquer ancestor que não seja static (relative, absolute, fixed)
- Se top + bottom setados: altura = (container_height - top - bottom)
- Se left + right setados: largura = (container_width - left - right)
- `position: fixed` usa viewport como containing block
- Absolute com `width: auto` = conteúdo encolhe ao redor dos filhos

**Critério de aceite:** Teste com 3 divs aninhadas, div interna com `position: absolute; top: 10px; left: 10px` aparece na posição correta relativa ao ancestral positionado mais próximo.

---

### 🟡 T054 — Text Rendering Accuracy (1 dia)

**Objetivo:** Substituir a heurística `estimate_text_width()` por medição real com ab_glyph, e implementar line-height explícito, text-align e font-weight.

**Arquivos:** `src/render/layout.rs`, `src/render/screenshot.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T054.1 | Criar `measure_text_width(text, font, font_size) -> f32` usando h_advance real de cada glyph do ab_glyph |
| T054.2 | Substituir `estimate_text_width()` → `measure_text_width()` no layout.rs |
| T054.3 | Implementar `line-height` explícito no ComputedStyle (default: "normal" = 1.2× font_size) |
| T054.4 | `text-align: left` (default), `center`, `right` na render_node() |
| T054.5 | `font-weight` bold (700+) renderiza com fonte bold (se disponível) |
| T054.6 | Whitespace collapsing: múltiplos espaços → 1 espaço |

**⚠️ Impacto:** `measure_text_width` com ab_glyph real vai tornar o text wrap MUITO mais preciso. A heurística atual (chars × font_size × 0.5) é ~30% imprecisa.

**Critério de aceite:** Texto com `font-size: 20px` em viewport de 400px quebra de linha em posição visualmente correta.

---

### 🟡 T055 — Overflow Handling (1 dia)

**Objetivo:** Tratar conteúdo que extravasa o container.

**Arquivos:** `src/render/layout.rs`, `src/render/screenshot.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T055.1 | `overflow: visible` (default) — conteúdo extravasa sem corte |
| T055.2 | `overflow: hidden` — clipar conteúdo fora do rect do pai no render_node() |
| T055.3 | Implementar clip via tiny_skia: salvar clip rect antes de filhos, restaurar depois |
| T055.4 | `overflow: scroll` / `auto` — trata como hidden (sem scroll real) |

**Critério de aceite:** `<div style="overflow: hidden; width: 50px; height: 50px;"><div style="width: 200px;">X</div></div>` — filho de 200px clipado para 50×50.

---

### 🟡 T056 — Performance & Font Cache (1 dia)

**Objetivo:** Medir e otimizar performance do screenshot.

**Arquivos:** `src/render/screenshot.rs` (principal), `src/render/tree.rs`

| Subtask | Descrição |
|---------|-----------|
| T056.1 | Cache de fontes: carregar TTF uma vez, reusar em todos os nós (atualmente load_font_simple é chamado por nó) |
| T056.2 | Pular renderização de nós fora da viewport (early skip no render_node) |
| T056.3 | Medir tempo gasto em cada etapa: parse CSS → layout tree → compute_layout → render_node |
| T056.4 | Otimizar gargalos identificados |

**Critério de aceite:** Screenshot de página real (ex: books.toscrape) em < 500ms. FAF notavelmente mais rápido que abrir um navegador.

---

### 🔴 T057 — Testes M7 + Regressão (1 dia)

**Objetivo:** Garantir que M7 não quebra nada e os novos recursos têm cobertura de teste.

**Arquivos:** `tests/m7_test.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T057.1 | Teste de position: absolute com ancestral positionado |
| T057.2 | Teste de position: fixed (viewport) |
| T057.3 | Teste de overflow: hidden com clip |
| T057.4 | Teste de text-align: center/right |
| T057.5 | Teste de line-height explícito |
| T057.6 | Teste de regressão: 287+ testes existentes continuam passando |

**Critério de aceite:** 295+ testes passando, 0 falhas, clippy limpo.

---

## 📐 Estrutura esperada

```
faf-browser/
├── src/render/
│   ├── layout.rs       ← T053 (absolute), T054 (text width), T055 (overflow)
│   ├── screenshot.rs   ← T054 (text-align), T055 (clip), T056 (cache)
│   ├── tree.rs
│   └── ...
├── tests/
│   └── m7_test.rs      ← T057
└── TASKS_M7.md
```

---

## 🧪 Critérios de Aceite Gerais

- [ ] position: absolute funciona com ancestral positionado e viewport fallback
- [ ] position: fixed funciona independente de scroll
- [ ] Text wrap usa ab_glyph real (não heurística)
- [ ] text-align: center/right funcionam
- [ ] line-height explícito respeitado
- [ ] overflow: hidden clipa conteúdo
- [ ] Cache de fontes (carrega uma vez, reusa)
- [ ] Screenshot de página real em < 500ms
- [ ] 295+ testes passando
- [ ] Build release com 0 warnings
- [ ] cargo clippy limpo

---

## 📊 Comparativo: M6 → M7

| Aspecto | M6 (agora) | M7 (meta) |
|---------|-----------|-----------|
| Positioning | relative + z-index | **absolute + fixed** |
| Text width | Heurística (chars × 0.5) | **ab_glyph real** |
| Text align | ❌ | **left + center + right** |
| Line-height | Fixo 1.2× | **Explícito** |
| Overflow | ❌ Ignorado | **hidden + visible** |
| Font cache | Load por nó | **Uma vez, reusado** |
| Performance | Não medido | **Otimizado, < 500ms** |
| Testes | 287 | **295+** |
