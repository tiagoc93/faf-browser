# ✅ Tasks — FAF BROWSER (Fast As Fuck)

**Total:** 44 tasks | **Estimativa:** ~3 meses  
**Modelo de delegação:** Kimi K2.6  
**Stack:** Rust + Cargo (Edition 2024)

---

## M1 — Core Engine (12 tasks · ~1 mês)

### HTTP Client & Setup
- [ ] **T001** — Inicializar projeto Cargo + dependências (reqwest, tokio, html5ever, etc) + CI básico (clippy + fmt)
- [ ] **T002** — HTTP Client: fetch página via reqwest com headers customizáveis (User-Agent, Accept, etc)
- [ ] **T003** — Suporte a proxy: SOCKS5 + HTTP via reqwest + timeout configurável
- [ ] **T004** — CLI básica com clap: `faf <url>` + flags (--proxy, --timeout, --user-agent)

### HTML Parser & DOM
- [ ] **T005** — Parser HTML com html5ever: converter bytes em DOM tree
- [ ] **T006** — DOM tree com arena allocation (evitar allocator contention)
- [ ] **T007** — Navegação na árvore: parent, children, siblings, attributes, text content
- [ ] **T008** — Search por tag name e attribute na DOM tree

### Integração & Output
- [ ] **T009** — Pipeline HTTP → Parse → DOM: juntar T002 + T005
- [ ] **T010** — Output JSON: serializar DOM tree (tree enxuta: tag, id, classes, text, children)
- [ ] **T011** — Tratamento de erros (anyhow/thiserror) + logs (env_logger)
- [ ] **T012** — Testes de integração M1: fetch página real, parse, verificar DOM

---

## M2 — CSS Básico (8 tasks · ~2 semanas)

- [x] **T013** — Parser CSS com cssparser: tokenizar folhas de estilo
- [x] **T014** — Selector matching com selectors crate: `h1`, `.class`, `#id`, `div span`
- [x] **T015** — Computed styles: cascata básica (inline > ID > class > tag)
- [x] **T016** — Box model: width, height, margin, padding via tiny-skia
- [x] **T017** — Cores e backgrounds simples
- [x] **T018** — Fontes: font-family + fallback para monospace/sans-serif
- [x] **T019** — CLI: `faf query "h1" --url <url>` — retorna texto + style computed
- [x] **T020** — Testes M2: página HTML+CSS, verificar selector + layout

---

## M3 — JavaScript Engine (10 tasks · ~2 semanas)

- [ ] **T021** — Embed QuickJS via quickjs-rs: runtime básico
- [ ] **T022** — Bridge DOM ↔ JS: expor document.getElementById, querySelector
- [ ] **T023** — setTimeout / setInterval com event loop tokio
- [ ] **T024** — Fetch API via JS → reqwest (ponte JS → Rust HTTP)
- [ ] **T025** — Timeout de execução JS (evitar loop infinito)
- [ ] **T026** — Console.log do JS redirecionado pro logger do Rust
- [ ] **T027** — Tratamento de erros JS (try/catch, stack trace)
- [ ] **T028** — Suporte a scripts inline + externos (<script src=>)
- [ ] **T029** — CLI: `faf --js "document.title" --url <url>`
- [ ] **T030** — Testes M3: página com JS simples, verificar execução + DOM mutado

---

## M4 — API Pública (8 tasks · ~2 semanas)

- [ ] **T031** — Modo comando único: `faf https://site.com` — fetch + parse + output JSON
- [ ] **T032** — Modo script: `faf script.js` — executar script no contexto de uma página
- [ ] **T033** — Saída JSON completa: DOM tree, links, imagens, metadados, título
- [ ] **T034** — Cookie persistence: `--cookies cookies.txt`
- [ ] **T035** — WaitForSelector: esperar elemento aparecer (com timeout)
- [ ] **T036** — Modo interativo: stdin → eval → stdout (pra pipe com outras ferramentas)
- [ ] **T037** — Documentação: README com exemplos de uso
- [ ] **T038** — Testes M4: CLI commands, JSON output válido, cookies

---

## M5 — Extração & Queries (6 tasks · ~1 semana)

- [ ] **T039** — Extrair links: `faf links --url <url>` — todos os hrefs
- [ ] **T040** — Extrair imagens: `faf images --url <url>` — src + alt text
- [ ] **T041** — Extrair metadados: Open Graph, meta tags, title, description
- [ ] **T042** — Query por seletor: `faf query "div.card h2" --url <url>`
- [ ] **T043** — Click via JS: simular click num elemento (seletor → dispatchEvent)
- [ ] **T044** — Testes M5: extração de dados reais de sites

---

## ✅ Critério de conclusão do MVP

Todas as 44 tasks concluídas com:
- `cargo test` passando
- `cargo clippy` sem warnings
- `cargo fmt` aplicado
- README com exemplos funcionais
- Binário compilando com `cargo build --release`
- Teste manual em 3 sites reais
