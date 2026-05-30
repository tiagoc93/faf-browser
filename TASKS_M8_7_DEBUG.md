# TASKS_M8_7_DEBUG.md — Debug: Imagens não renderizam no screenshot

## Bug confirmado — CAUSA RAIZ IDENTIFICADA

**Problema:** `count_img_nodes` retorna 20 img nodes no visual tree, mas `render_node` NUNCA visita nenhum nó img. ZERO `RENDER_CALL tag="img"` nos logs.

**Sintoma:**
- 20 img nodes existem no tree (`TREE_BEFORE_LAYOUT: img_count=20`, `TREE_AFTER_LAYOUT: img_count=20`)
- MAS `NODE_DEBUG` e `RENDER_CALL` NUNCA mostram `tag="img"`
- O traversal de `render_node` não está alcançando os nós img

**Conclusão:** O bug está em como o visual tree é construido ou em como o traversal acontece. Os img nodes estão no tree mas não são visitados.

## Tasks de debug

### T-DIAG-IMG-001: Investigar onde img nodes estão no visual tree

O tree tem 20 img nodes mas não são visitados. Isso significa:
1. Ou os img nodes estão em branches que não são atravessadas
2. Ou os img nodes têm w=0 ou h=0 e são pulados por algum filter
3. Ou a função count_img_nodes está contando errado

**Tarefa:**
1. Adicionar log em `build_layout_tree` para mostrar onde img nodes são criados:
   ```rust
   // Quando criar um VisualNode com tag "img"
   log::info!("BUILD_IMG: creating img node with rect={:?}", node.rect);
   ```
2. Adicionar log em `compute_layout` quando processar img nodes:
   ```rust
   // Quando processar um nó img no layout
   log::info!("LAYOUT_IMG: tag={:?} rect={:?}", node.tag, node.rect);
   ```
3. Buildar e rodar screenshot
4. Grep: `grep "BUILD_IMG\|LAYOUT_IMG" /tmp/diag.log`
5. Verificar se os logs aparecem e onde os img nodes estão sendo criados

---

### T-DIAG-IMG-002: Verificar se img nodes estão como children de nós com w=0

Se um nó pai tem w=0, seus filhos podem não ser renderizados corretamente.

**Tarefa:**
1. Buscar nos logs `NODE_DEBUG` por tags que têm w=0 E children:
   ```
   grep "NODE_DEBUG.*w=0" /tmp/diag.log | head -20
   ```
2. Verificar se algum nó com w=0 tem children que incluem img nodes
3. Se encontrado, o bug pode estar em como o layout trata nós com w=0

---

### T-DIAG-IMG-003: Verificar estrutura do tree na área de produtos

Os img nodes deberían estar na área de produtos (y=1200-4000, dentro de article/product nodes).

**Tarefa:**
1. Grep todos os NODE_DEBUG com y entre 1200 e 4000
2. Verificar a hierarquia: qual tag contém os img nodes?
3. Exemplo de busca:
   ```bash
   grep "NODE_DEBUG" /tmp/diag.log | awk '{if ($8 ~ /y=[0-9.]+/ && $8 ~ /y=[1-9][2-9][0-9][0-9]/) print}' | head -50
   ```
4. Identificar onde os img nodes estão na hierarquia

---

### T-DIAG-IMG-004: Verificar se img nodes são criados com rect válido

**Tarefa:**
1. Adicionar log no início de `build_layout_tree`:
   ```rust
   // Log todos os nós criados com tag e rect
   log::info!("BUILD_NODE: tag={:?} rect={:?}", node.tag, node.rect);
   ```
2. Buildar e rodar
3. Grep: `grep "BUILD_NODE.*img" /tmp/diag.log`
4. Verificar se img nodes são criados com rect não-nulo

---

### T-DIAG-IMG-005: Verificar se compute_layout está removendo ou alterando img nodes

O log mostra `TREE_BEFORE_LAYOUT: img_count=20` e `TREE_AFTER_LAYOUT: img_count=20` — então compute_layout não está removendo os img nodes.

**Tarefa:**
1. Adicionar log DENTRO de compute_layout quando processar cada nó:
   ```rust
   log::info!("COMPUTE_LAYOUT: tag={:?} rect={:?}", node.tag, node.rect);
   ```
2. Buildar e rodar
3. Grep: `grep "COMPUTE_LAYOUT.*img" /tmp/diag.log`
4. Se não aparecer, significa compute_layout não está processando img nodes

---

## Comando para testar

```bash
cd /home/hermes/faf-browser
cargo build --release 2>&1 | grep -E "error|warning" | head -10
./target/release/faf-browser screenshot https://books.toscrape.com/ --output /tmp/debug.png 2>&1 | tee /tmp/diag.log
# Analisar
grep "BUILD_IMG\|LAYOUT_IMG\|COMPUTE_LAYOUT.*img\|NODE_DEBUG.*img" /tmp/diag.log
```

## Critério de sucesso

1. Encontrar onde os 20 img nodes estão no tree (BUILD_IMG logs)
2. Entender por que render_node não os visita
3. Corrigir para que render_node visite e renderize os img nodes
4. Screenshot final mostra capas de livros coloridas

## Arquivos principais

- `/home/hermes/faf-browser/src/render/screenshot.rs` — render_node(), build_layout_tree(), compute_layout()
- `/home/hermes/faf-browser/src/render/tree.rs` — VisualNode struct