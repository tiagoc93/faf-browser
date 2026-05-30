# TASKS_M8_7_FIX.md — CORRIGIR: render_node não visita img nodes

## CAUSA RAIZ CONFIRMADA

Os img nodes são criados no visual tree (20 BUILD_IMG logs), MAS compute_layout NUNCA processa nenhum deles (0 COMPUTE_LAYOUT: tag="img").

O bug está em como os img nodes são adicionados aos seus nós pais no tree building.

## Diagnóstico completo

1. **tree.rs** cria img nodes com `rect=Rect { left: 0.0, top: 0.0, right: 0.0, bottom: 0.0 }` (确认: 20 BUILD_IMG logs)
2. **layout.rs** `compute_layout` é chamado recursivamente, MAS nenhum `tag="img"` aparece nos logs
3. **render_node** visita 649 nós (NODE_DEBUG), nenhum é img (0 tag="img")
4. `TREE_BEFORE_LAYOUT: img_count=20` e `TREE_AFTER_LAYOUT: img_count=20` — tree não muda entre build e layout

**Conclusão:** Os img nodes estão no tree mas não são alcançados pelo traversal de layout/render. Possível causa: os img nodes são criados como children de nós que depois têm w=0 no layout (sendo pulados), ou algo no tree building os coloca em branches que não são atravessadas.

## Tarefas de correção

### T-FIX-001: Verificar onde os img nodes estão no tree

Os img nodes são criados mas não processados pelo layout. Precisa entender:
1. Qual nó é o pai de cada img node no tree?
2. O pai tem w=0 ou h=0 que causaria skip no traversal?

**Tarefa:**
1. Em tree.rs, quando criar um img node, logar também o parent tag:
   ```rust
   // Em build_node_recursive, quando processar img como child
   log::info!("BUILD_IMG_CHILD: parent_tag={:?} img_attrs={:?}", tag, element.value().attrs().collect::<Vec<_>>());
   ```
2. Buildar e rodar
3. Grep: `grep "BUILD_IMG" /tmp/diag.log` e verificar se algum img é child de um nó específico

---

### T-FIX-002: Verificar rect dos pais dos img nodes

Se um nó pai tem w=0, o traversal pode estar falhando.

**Tarefa:**
1. Adicionar log em tree.rs quando adicionar children:
   ```rust
   // Quando adicionar child ao node
   log::info!("ADD_CHILD: parent_tag={:?} parent_rect={:?} child_tag={:?} child_rect={:?}", 
              visual.tag, visual.rect, child.tag, child.rect);
   ```
2. Buildar e rodar
3. Grep os logs para img nodes e verificar o rect do parent

---

### T-FIX-003: Verificar se img nodes estão no tree depois do layout

O log mostra `TREE_AFTER_LAYOUT: img_count=20`, então os img nodes existem depois do layout. O problema é que compute_layout não os visita — mas isso é porque eles são folhas (children=0).

**Tarefa:**
1. Verificar se o problema é no traversal de layout (não alcançando os nós que têm img como children)
2. Adicionar log no início de layout_block para cada nó que é chamado
3. Verificar se algum nó com w=0 ou h=0 está sendo pulado e contém img nodes

---

### T-FIX-004: Corrigir o problema

Depois de identificar a causa, implementar a correção:

**Se o problema for que img nodes estão com rect=0:**
- Em tree.rs, usar dimensões intrínsecas (image_dims) quando criar img nodes
- Passar image_dims para build_node_recursive e usar para setar rect inicial dos img nodes

**Se o problema for que pais de img nodes têm w=0 e são pulados:**
- Em layout.rs, não pular nós com w=0 se eles tiverem children

**Se o problema for no tree building:**
- Verificar se img nodes estão sendo adicionados como children corretamente
- Verificar se o parent do img node está sendo processado corretamente

---

## Critério de sucesso

1. `COMPUTE_LAYOUT: tag="img"` aparece nos logs (layout processando img nodes)
2. `RENDER_CALL: tag="img"` aparece nos logs (render_node visitando img nodes)
3. `RENDER_IMG: found img node` aparece nos logs (draw_image sendo chamado)
4. Screenshot mostra capas de livros coloridas

## Comando para testar

```bash
cd /home/hermes/faf-browser
cargo build --release 2>&1 | grep -E "error|warning"
./target/release/faf-browser screenshot https://books.toscrape.com/ --output /tmp/fix.png 2>&1 | tee /tmp/fix.log
grep -E "BUILD_IMG|COMPUTE_LAYOUT.*img|RENDER_CALL.*img|RENDER_IMG|DRAW_IMAGE" /tmp/fix.log
```

## Arquivos principais

- `/home/hermes/faf-browser/src/render/tree.rs` — build_layout_tree, build_node_recursive, is_skip_tag
- `/home/hermes/faf-browser/src/render/layout.rs` — compute_layout, layout_block
- `/home/hermes/faf-browser/src/render/screenshot.rs` — render_node