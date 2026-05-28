# 🎯 FAF Browser — M7: Playwright Parity + Polish

**Status:** Planejado
**Previsão:** 5-8 dias
**Meta:** Screenshot com qualidade visual equivalente ao Playwright, mas com mais performance

---

## 🎯 Objetivo

O M6 entregou um motor de layout funcional com árvore visual, fluxo block/inline, text wrap, backgrounds, bordas, imagens e relative positioning. O M7 fecha as lacunas restantes para que o screenshot do FAF Browser seja visualmente indistinguível do Playwright — e mais rápido.

**Áreas de melhoria identificadas no M6:**
1. `position: absolute` — implementação incompleta (timeout no Kimi)
2. `position: fixed` — não implementado
3. Overflow — conteúdo que extravasa o container não é tratado
4. `display: flex` — sem implementação (layout engine só block/inline)
5. Text rendering — `estimate_text_width()` é heurística (chars × font_size × 0.5), não real
6. Line-height — fixo em 1.2×, sem suporte a line-height explícito
7. Semântica visual — `text-align`, `vertical-align`, `font-weight` sem efeito real
8. **Ferramenta de comparação Playwright** — sem ela, não sabemos o que está diferente

---

## 📋 Tasks

---

### 🔴 T053 — Playwright Comparison Framework (2 dias)

**Objetivo:** Criar ferramenta CLI + script que compara screenshot do FAF com Playwright e gera diff visual + relatório.

**Arquivos:** `tools/compare-playwright/` (novo diretório), `Cargo.toml` (opcional)

| Subtask | Descrição |
|---------|-----------|
| T053.1 | Script Python/Node para gerar screenshot de uma URL com Playwright (headless, 1280x800) |
| T053.2 | Script que roda FAF screenshot da mesma URL com mesmas dimensões |
| T053.3 | Comparação pixel-a-pixel: `faf-expected.png` vs `faf-actual.png` → `faf-diff.png` |
| T053.4 | Métricas: diff %, MSE, SSIM entre as duas imagens |
| T053.5 | Relatório HTML com side-by-side: expected, actual, diff highlight |
| T053.6 | Batelada: comparar N URLs conhecidas (wikipedia, books.toscrape, example.com, etc) |

**Flags:**
- `--url` — URL única para comparar
- `--batch` — roda todas as URLs da lista
- `--output-dir` — onde salvar os screenshots + diffs
- `--threshold` — % de diff tolerável (default: 1%)

**Critério de aceite:** `tools/compare-playwright/compare.py --url https://example.com` produz 3 PNGs (expected, actual, diff) + relatório com métricas.

---

### 🔴 T054 — position: absolute + fixed (2 dias)

**Objetivo:** Completar o suporte a posicionamento CSS que foi cortado por timeout no M6.

**Arquivos:** `src/render/layout.rs`, `src/render/tree.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T054.1 | Adicionar `bottom`, `right` ao ComputedStyle (já tem top, left) |
| T054.2 | `position: absolute` — remover do fluxo normal (não contribui pra altura do pai) |
| T054.3 | Encontrar ancestral positionado (first parent with position ≠ static) |
| T054.4 | Usar viewport (0,0) como fallback se nenhum ancestral positionado |
| T054.5 | Posicionar absolute baseado em top/right/bottom/left em relação ao **containing block** |
| T054.6 | `position: fixed` — posicionar em relação à viewport sempre (ignorar scroll) |
| T054.7 | Elementos absolute NÃO são iterados no fluxo normal (lista separada para render) |
| T054.8 | z-index também funciona em absolute (implementação já existe no relative) |

**⚠️ Edge cases:**
- Ancestral positionado pode ser qualquer ancestor, não só pai direto
- Se top + bottom estão setados, altura = (container_height - top - bottom)
- Se left + right estão setados, largura = (container_width - left - right)
- `position: fixed` mesmo ancestral que absolute mas com viewport como containing block
- Absolute com `width: auto` = conteúdo encolhe ao redor dos filhos

**Critério de aceite:** Teste com 3 divs aninhadas, div interna com `position: absolute; top: 10px; left: 10px` aparece na posição correta relativa ao ancestral positionado, não ao pai imediato.

---

### 🟡 T055 — Text Rendering Accuracy (1 dia)

**Objetivo:** Substituir a heurística `estimate_text_width()` por medição real com ab_glyph, e implementar suporte a line-height explícito, text-align e font-weight.

**Arquivos:** `src/render/layout.rs`, `src/render/screenshot.rs`

| Subtask | Descrição |
|---------|-----------|
| T055.1 | Criar `measure_text_width(text, font, font_size) -> f32` usando ab_glyph real (h_advance de cada glyph) |
| T055.2 | Substituir `estimate_text_width()` por `measure_text_width()` no layout.rs |
| T055.3 | Implementar `line-height` explícito no ComputedStyle (default: "normal" = 1.2× font_size) |
| T055.4 | `text-align: left` (default), `center`, `right` |
| T055.5 | `font-weight` mais leve: pelo menos bold (700+) renderiza mais grosso |
| T055.6 | Whitespace collapsing: múltiplos espaços → 1 espaço (já parcialmente feito) |

**⚠️ Observação:** `measure_text_width` com ab_glyph real vai tornar o text wrap MUITO mais preciso. A heurística atual (chars × font_size × 0.5) é ~30% imprecisa para fontes variadas.

**Critério de aceite:** Texto com `font-size: 20px` em viewport de 400px quebra no mesmo lugar que no Playwright.

---

### 🟡 T056 — Overflow Handling (1 dia)

**Objetivo:** Tratar conteúdo que extravasa o container.

**Arquivos:** `src/render/layout.rs`, `src/render/screenshot.rs`, `src/css/style.rs`

| Subtask | Descrição |
|---------|-----------|
| T056.1 | `overflow: hidden` — clipar conteúdo fora do rect do pai no render_node() |
| T056.2 | `overflow: visible` (default) — conteúdo extravasa sem clip |
| T056.3 | `overflow: scroll` / `auto` — sem scroll real, apenas trata como hidden + indicador |
| T056.4 | Implementar clip via tiny_skia: salvar clip rect antes de desenhar filhos, restaurar depois |

**Critério de aceite:** `<div style="overflow: hidden; width: 50px; height: 50px;"><div style="width: 200px;">X</div></div>` — o filho de 200px é clipado para 50×50.

---

### 🟡 T057 — Comparison & Gap Fixing (2 dias)

**Objetivo:** Rodar o Playwright Comparison Framework (T053) e corrigir os gaps encontrados.

**Arquivos:** Variados conforme os gaps descobertos

| Subtask | Descrição |
|---------|-----------|
| T057.1 | Rodar batch de comparação com 10+ URLs |
| T057.2 | Analisar diffs e categorizar gaps: layout, cor, borda, texto, imagem |
| T057.3 | Corrigir os 3 maiores gaps (prioridade por impacto visual) |
| T057.4 | Re-rodar comparação, verificar melhoria |
| T057.5 | Documentar gaps restantes como "known limitations" |

**⚠️ Importante:** Esta task é ITERATIVA. Cada gap corrigido pode revelar novos gaps. Foco em corrigir o que mais impacta a percepção visual (posicionamento > cor > textura).

**Critério de aceite:** Pelo menos 5 URLs com diff < 5% (vs Playwright).

---

### 🟡 T058 — Performance Benchmark (1 dia)

**Objetivo:** Medir e otimizar performance do screenshot FAF vs Playwright.

**Arquivos:** `tools/benchmark/` (novo), variados

| Subtask | Descrição |
|---------|-----------|
| T058.1 | Script de benchmark: tempo de screenshot FAF vs Playwright para mesma URL |
| T058.2 | Medir: tempo total, tempo de layout, tempo de render (separados) |
| T058.3 | Identificar gargalos (provável: load_font_simple para cada nó) |
| T058.4 | Cache de fontes (carregar uma vez, reusar) |
| T058.5 | Otimizar render_node para pular nós invisíveis (fora da viewport) |
| T058.6 | Resultado: FAF deve ser 3-5× mais rápido que Playwright |

**Critério de aceite:** `tools/benchmark/bench.sh` gera relatório com tempos comparativos. FAF pelo menos 2× mais rápido que Playwright na mesma URL.

---

### 🔴 T059 — Testes M7 + Regressão (1 dia)

**Objetivo:** Garantir que M7 não quebra nada e os novos recursos têm cobertura.

**Arquivos:** `tests/m7_test.rs` (novo)

| Subtask | Descrição |
|---------|-----------|
| T059.1 | Teste de position: absolute com ancestral positionado |
| T059.2 | Teste de position: fixed (viewport) |
| T059.3 | Teste de overflow: hidden com clip |
| T059.4 | Teste de text-align: center/right |
| T059.5 | Teste de line-height explícito |
| T059.6 | Teste de regressão: 287+ testes existentes continuam passando |
| T059.7 | Teste de comparação visual (se T053 estiver pronto) |

**Critério de aceite:** 295+ testes passando, 0 falhas, clippy limpo.

---

## 📐 Estrutura final esperada

```
faf-browser/
├── tools/
│   ├── compare-playwright/
│   │   ├── compare.py          ← Script de comparação Playwright × FAF
│   │   ├── urls.txt            ← Lista de URLs para batch
│   │   └── requirements.txt    ← playwright, Pillow, numpy
│   └── benchmark/
│       └── bench.sh            ← Script de benchmark
├── src/
│   └── render/
│       ├── mod.rs
│       ├── screenshot.rs
│       ├── tree.rs
│       ├── layout.rs           ← T054 (absolute), T055 (text width), T056 (overflow)
│       ├── background.rs
│       ├── border.rs
│       └── image.rs
├── tests/
│   └── m7_test.rs              ← T059
├── TASKS_M7.md                 ← Este arquivo
└── README.md                   ← Roadmap atualizado
```

---

## 🧪 Critérios de Aceite Gerais M7

- [ ] position: absolute funciona com ancestral positionado e viewport fallback
- [ ] position: fixed funciona independente de scroll
- [ ] Playwright Comparison Framework produz diffs com métricas
- [ ] Pelo menos 5 URLs com diff < 5% vs Playwright
- [ ] Text wrap usa ab_glyph real (não heurística)
- [ ] text-align: center/right funcionam
- [ ] line-height explícito respeitado
- [ ] overflow: hidden clipa conteúdo
- [ ] FAF 2×+ mais rápido que Playwright
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
| Comparação Playwright | ❌ | **Framework de diff + relatório** |
| Performance | Não medido | **Benchmarkado, 2×+ rápido** |
| Testes | 287 | **295+** |
