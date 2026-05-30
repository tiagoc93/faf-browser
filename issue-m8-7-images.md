# Issue: M8.7 - Imagens não aparecem visualmente no screenshot

**Data:** 2025-05-30
**Status:** ABERTO (bloqueado por timing de download)

## Bug Descrito

O FAF Browser faz download de imagens, baixa 20 entradas no cache, mas as capas de livro NÃO aparecem visualmente no screenshot final do books.toscrape.com.

## Sintomas

- Cache de imagens preenchido: 20 entradas em `/tmp/image_cache/`
- Dimensões buscadas: 40 imagens via HTTP HEAD
- `img_count=20` no TREE_BEFORE_LAYOUT e TREE_AFTER_LAYOUT
- Pixels coloridos no screenshot: apenas 697 (muito baixo para 20 capas)
- `DRAW_IMAGE` não aparece nos logs com `RUST_LOG=trace`

## Causa Raiz Identificada

**Problema 1:** `count_img_nodes` (screenshot.rs:280) ainda usa `node.tag == "img"` em vez de `node.tag.contains("img")` — mesmo bug que foi corrigido em `render_node`.

**Problema 2 (suspeita):** As imagens podem não estar sendo baixadas/sincronizadas antes do render. O download é HTTP e pode ser assíncrono, enquanto o render é síncrono.

## Correções Já Aplicadas

- `node.tag == "img"` → `node.tag.contains("img")` em `render_node` (screenshot.rs)

## Correções Pendentes

1. [ ] Corrigir `count_img_nodes`: `== "img"` → `contains("img")` (screenshot.rs:280)
2. [ ] Verificar se imagens estão no cache ANTES de renderizar (adicionar barreira sincronizada)
3. [ ] Adicionar logging em `draw_image` para confirmar se é chamado
4. [ ] Validar que imagens aparecem no screenshot após correção
5. [ ] Testar em mais páginas (não só books.toscrape)

## Como Reproduzir

```bash
cd /home/hermes/faf-browser
cargo run --release -- screenshot
# Abrir screenshot.png e verificar: não há capas de livro visíveis
```

## Logs Relevantes

```
TREE_BEFORE_LAYOUT: img_count=20
TREE_AFTER_LAYOUT: img_count=20
Cache de imagens preenchido: 20 entradas
Dimensões de imagens buscadas: 40 imagens
# DRAW_IMAGE NÃO aparece mesmo com RUST_LOG=trace
```

## Prioridade

Alta — M8.7 é milestone de renderização de imagens reais.