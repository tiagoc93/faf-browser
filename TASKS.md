# ✅ Tasks — FAF BROWSER (Fast As Fuck)

**Total:** 52 tasks | **Concluídas:** 36 | **Estimativa restante:** ~4 meses  
**Modelo de delegação:** Kimi K2.6 via `delegate_task()`  
**Stack:** Rust + Cargo (Edition 2024), rquickjs, tokio, reqwest

---

## ✅ M1 — Core Engine (12 tasks · Concluído)

### HTTP Client & Setup
- [x] **T001** — Inicializar projeto Cargo + dependências (reqwest, tokio, scraper, rquickjs, clap, serde)
- [x] **T002** — HTTP Client: fetch página via reqwest com headers customizáveis (User-Agent, Accept, etc)
- [x] **T003** — Suporte a proxy: SOCKS5 + HTTP via reqwest + timeout configurável
- [x] **T004** — CLI básica com clap: `faf <url>` + flags (--proxy, --timeout, --user-agent)

### HTML Parser & DOM
- [x] **T005** — Parser HTML com scraper/html5ever: converter bytes em DOM tree
- [x] **T006** — DOM tree: HtmlDocument struct com scraper::Html interno
- [x] **T007** — Navegação na árvore: links(), images(), metadata(), visible_text()
- [x] **T008** — Search por seletor CSS: query() com scraper::Selector

### Integração & Output
- [x] **T009** — Pipeline HTTP → Parse → DOM: juntar fetch + parser no run()
- [x] **T010** — Output JSON: serializar DOM tree (tag, id, classes, text, links, imagens, metadados)
- [x] **T011** — Tratamento de erros (anyhow) + logs (env_logger)
- [x] **T012** — Testes de integração M1: fetch página real, parse, verificar DOM

---

## ✅ M2 — CSS Engine (8 tasks · Concluído)

- [x] **T013** — Parser CSS com cssparser: tokenizar folhas de estilo, suporta @rules, comentários
- [x] **T014** — Selector matching com function compute_specificity(): `h1`=1, `.class`=10, `#id`=100
- [x] **T015** — Computed styles + cascata: inline > ID > class > tag (ordem stable por especificidade)
- [x] **T016** — Box model: width, height, margin, padding com shorthand (1/2/3/4 valores)
- [x] **T017** — Cores: 19 named colors, hex (#RGB/#RGBA/#RRGGBB/#RRGGBBAA), rgb/rgba, blending
- [x] **T018** — Fontes: font-family + fallback, size (px/em/rem/%), weight parsing
- [x] **T019** — CLI: `faf query "h1" --url <url>` — retorna texto + computed style
- [x] **T020** — Testes M2: página HTML+CSS inline, verificar selector + layout

---

## ✅ M2.5 — Polimento CLI + Extração Avançada (8 tasks · Concluído)

- [x] **P01** — `--css` e `--json` aceitos em qualquer posição (flags globais com clap `global = true`)
- [x] **P02** — Remover `--query` flag morto do struct Cli (só subcomando `query` funciona)
- [x] **P03** — Defaults CSS reais no ComputedStyle: `display:block`, `color:inherit`, `font-size:16px`
- [x] **P04** — Parse automático do CSS da página: extrai `<style>` e `<link>` do HTML, baixa externos
- [x] **Q01** — `--filter "campo~=valor"`: text match, regex, attribute match, múltiplos filtros AND
- [x] **Q02** — `--get "campo1, campo2"`: extração seletiva de campos (tag, id, text, href, color, etc)
- [x] **Q03** — `faf follow <seletor>`: crawler multithread com tokio semaphore, --extract, --max, --concurrency
- [x] **Q04** — `--format csv|jsonl|json|text`: múltiplos formatos de saída pipe-friendly

---

## ✅ M3 — JavaScript Engine (10 tasks · Concluído)

- [x] **T021** — Embed QuickJS via rquickjs 0.12: Runtime + Context + eval(), eval_json()
- [x] **T022** — Bridge DOM ↔ JS: document.title, getElementById, querySelector, querySelectorAll com wrappers JSON
- [x] **T023** — setTimeout / setInterval com event loop: usa thread::spawn + tokio::Runtime local
- [x] **T024** — Fetch API via JS → reqwest: `fetch(url)` síncrono com thread separada, suporta POST/PUT/DELETE
- [x] **T025** — Timeout de execução JS: eval_with_timeout() com canal mpsc, kill após N segundos
- [x] **T026** — Console.log/warn/error do JS → log::info!/warn!/error! do Rust
- [x] **T027** — Tratamento de erros JS: format_js_error() extrai TypeError, stack trace, SyntaxError
- [x] **T028** — Suporte a scripts `<script>` inline + externos: execute_page_scripts() baixa e executa
- [x] **T029** — CLI: `faf --js "document.title"`, `--js-file script.js`, `--no-scripts`, `--js-timeout <s>`
- [x] **T030** — Testes M3: 9 testes (DOM bridge, setTimeout, fetch, error handling, CLI --js, console.log)

---

## 📋 M4 — Sessão, Interação & Pipeline (8 tasks)

### T031 — Cookie Persistence (🟡 médio) ✅
**Arquivos:** `src/http/cookies.rs` (novo), `src/utils/config.rs`, `src/http/client.rs`, `src/api/commands.rs`

**O que faz:** Adicionar suporte a cookies de sessão persistente entre chamadas do FAF.

**Flags:**
- `--cookies <path>` — carrega cookies de arquivo no formato Netscape (igual curl)
- `--cookies-jar <path>` — salva cookies atualizados no arquivo após request

**Implementação:**
O `reqwest::Client` já tem `cookie_store(true)` habilitado, mas é volátil (memória). Precisamos:
1. Criar `src/http/cookies.rs` com funções:
   - `load_netscape_cookies(path: &str) -> Vec<Cookie>` — parser do formato Netscape
   - `save_netscape_cookies(store: &Arc<CookieStore>, path: &str)` — serializa cookies atuais
   - Formato Netscape: `domain\tTRUE\tpath\tFALSE\t<expires>\tname\tvalue`
2. Em `Config` (utils/config.rs), adicionar campos `cookies_path` e `cookies_jar_path`
3. Em `HttpClient::new()`, se `cookies_path` existe, carregar cookies no cookie store
4. Após request, se `cookies_jar_path` definido, salvar cookies
5. O `follow` já clona o HttpClient, então cookies são compartilhados entre páginas

**Critério:**
- `faf https://site.com --cookies session.txt` carrega cookies do arquivo
- `faf https://site.com --cookies-jar session.txt` salva cookies após request
- Segunda chamada com --cookies reenvia os cookies salvos
- Formato compatível com `curl -b session.txt -c session.txt`
- `cargo test`, `cargo clippy`

---

### T032 — WaitForSelector (🟡 médio) ✅
**Arquivos:** `src/api/commands.rs` (subcomando `Wait`), `src/js/engine.rs` (helper)

**O que faz:** Aguarda até que um elemento CSS apareça no DOM (útil para páginas com carregamento dinâmico via JS).

**Uso:**
```bash
faf wait ".product-card" --url https://site.com --timeout 10
faf wait "#app-loaded" --url https://site.com --json
```

**Implementação:**
1. Adicionar `Command::Wait { selector: String }` no enum
2. Flags: `--timeout <s>` (default 10), `--interval <ms>` (default 200)
3. Fluxo:
   a. Fetch + parse HTML
   b. Criar JsRuntime, set_dom, init_timers, init_fetch
   c. Executar scripts da página (se não --no-scripts)
   d. Loop: a cada `interval` ms, executar `document.querySelector(selector)` via JS
   e. Se retornou não-null → elemento encontrado, retornar resultado
   f. Se atingiu `timeout` → erro "Elemento '#{.selector}' não encontrado após {timeout}s"
4. Re-usar `JsRuntime::eval_json()` para query no DOM vivo

**Critério:**
- `faf wait ".loaded" --url https://site.com --timeout 5` espera e retorna o elemento
- Timeout expirado → mensagem de erro clara
- `--json` retorna JSON do elemento
- `--interval 100` faz polling mais rápido

---

### T033 — Modo Interativo / REPL (🟡 médio) ✅
**Arquivos:** `src/api/commands.rs` (subcomando `Repl`, flag `--stdin`), `src/api/repl.rs` (novo), `Cargo.toml` (rustyline)

**O que faz:** Permite executar múltiplos comandos JS numa mesma sessão, mantendo o estado do runtime entre comandos.

**Duas abordagens:**

**A) REPL interativo (`faf repl --url <url>`):**
```bash
faf repl --url https://books.toscrape.com/
> document.title
"All products | Books to Scrape - Sandbox"
> document.querySelectorAll('h3').length
20
> .exit
```
- Usar `rustyline` crate para input com histórico
- Cada linha é avaliada com `rt.eval_with_timeout(line, js_timeout)`
- Comandos: `.exit`, `.help`, `.json` (toggle), `.clear`
- Ctrl+C interrompe expressão atual, Ctrl+D sai

**B) Modo stdin (`--stdin`):**
```bash
echo "document.title" | faf --url https://books.toscrape.com/ --stdin
# → "All products | Books to Scrape - Sandbox"
```
- Ler código JS de stdin linha por linha
- Executar cada linha no mesmo runtime
- Printar resultado de cada linha
- Útil para pipes e scripts shell

**Critério:**
- `faf repl --url https://site.com` abre prompt, executa JS, .exit funciona
- `echo "1+1" | faf --url https://site.com --stdin` → "2"
- Histórico de comandos (se rustyline)
- Runtime preserva estado entre comandos (variáveis globais)

---

### T034 — Rate Limiting no Follow (🟢 pequeno) ✅
**Arquivos:** `src/api/commands.rs` (FollowArgs)

**O que faz:** Adiciona delay entre requests no `follow` para não sobrecarregar sites.

**Flags em FollowArgs:**
- `--delay <ms>` — delay fixo entre cada request (default: 0)
- `--random-delay <min_ms> <max_ms>` — delay aleatório entre min e max

**Implementação:**
- Modificar o loop do `follow` para chamar `tokio::time::sleep()` entre requests
- Se `--delay` é 0 e `--random-delay` não foi passado: comportamento atual (sem delay)
- `--random-delay` sobrescreve `--delay`
- Delay é aplicado **entre** conclusão de uma página e início da próxima
- Para concorrência > 1, delay é por lote (não por request individual)

**Critério:**
- `faf follow "a" --url https://site.com --delay 1000` espera 1s entre páginas
- `faf follow "a" --url https://site.com --random-delay 500 2000` espera entre 500ms e 2s

---

### T035 — Retry com Exponential Backoff (🟢 pequeno) ✅
**Arquivos:** `src/http/client.rs`, `src/utils/config.rs`, `src/api/commands.rs`

**O que faz:** Tenta novamente requests que falharam, com delay exponencial.

**Flags globais:**
- `--retries <N>` — número máximo de tentativas (default: 0 = sem retry)
- `--retry-delay <ms>` — delay inicial entre retries (default: 1000)

**Implementação:**
- Em `HttpClient::get()` (ou novo método `get_with_retry()`):
  1. Tentar request
  2. Se falhar (erro de rede OU status 5xx):
     - Se `retries == 0`: retornar erro imediatamente (comportamento atual)
     - Se `retries > 0`: esperar `delay * 2^tentativa` ms
     - Tentar novamente
     - Incrementar tentativa
  3. Se todas tentativas falharem: retornar último erro
- Tratamento especial para HTTP 429 (Too Many Requests):
  - Ler header `Retry-After`
  - Se presente, usar esse valor como delay
  - Se ausente, usar exponential backoff padrão
- Logar cada tentativa: `log::warn!("Request falhou (tentativa {}/{}), retentando em {}ms", tentativa, retries, delay)`

**Critério:**
- `faf https://site.com --retries 3` tenta 3× antes de falhar
- Delay dobra a cada tentativa: 1s, 2s, 4s
- 429 lê Retry-After header
- Se suceder no retry, retorna resultado normalmente (sem erro)

---

### T036 — Output com Headers HTTP (🟢 pequeno) ✅
**Arquivos:** `src/http/client.rs`, `src/api/output.rs`, `src/api/commands.rs`

**O que faz:** Mostra headers HTTP e status code da resposta no output.

**Flags:**
- `--show-headers` — exibe response headers no output
- `--show-status` — exibe status code (ex: "200 OK")

**Implementação:**
1. Modificar `HttpClient::get()` para retornar struct `FetchResponse { status: u16, status_text: String, headers: HashMap<String,String>, body: String }` em vez de `String`
2. Atualizar todos os callers de `client.get()` para lidar com o novo tipo
3. No output:
   - Se `--show-status`: printar `📋 Status: {status} {status_text}`
   - Se `--show-headers`: printar headers formatados (1 por linha)
4. Em JSON: incluir `status` e `headers` no objeto de resposta
5. Compatível com todos os subcomandos (query, follow, links, images, metadata)

**Critério:**
- `faf https://site.com --show-status` → "📋 Status: 200 OK"
- `faf https://site.com --show-headers` → headers do response
- `--json --show-headers` inclui headers no JSON

---

### T037 — Cache de Responses (🟡 médio) ✅
**Arquivos:** `src/http/cache.rs` (novo), `src/http/mod.rs`, `src/http/client.rs`, `src/utils/config.rs`, `src/api/commands.rs`

**O que faz:** Cacheia respostas HTTP em disco para evitar re-baixar a mesma URL durante desenvolvimento.

**Flags:**
- `--cache <dir>` — diretório de cache (ex: `--cache .faf-cache`)
- `--cache-ttl <s>` — tempo de vida do cache em segundos (default: 300 = 5 min)
- `--no-cache` — ignora cache mesmo se diretório existir

**Implementação:**
1. Criar `src/http/cache.rs`:
   - `CacheEntry { url, status, headers, body, cached_at }` — struct serializável
   - `cache_key(url: &str) -> String` — SHA256 da URL
   - `cache_path(cache_dir: &str, key: &str) -> PathBuf` — `<dir>/<key>.json`
   - `get_cached(cache_dir: &str, url: &str, ttl: Duration) -> Option<CacheEntry>`
   - `set_cached(cache_dir: &str, url: &str, entry: &CacheEntry) -> Result<()>`
2. Em `HttpClient::get()`:
   - Se `--cache` definido e método GET: verificar cache primeiro
   - Se cache hit (existe e não expirou): retornar resposta cached (sem request)
   - Se cache miss: fazer request, salvar no cache, retornar
3. Usar `serde_json` para serializar CacheEntry

**Critério:**
- `faf https://site.com --cache .faf-cache` salva resposta
- Segunda chamada com mesmo --cache é instantânea (lê do disco)
- `--cache-ttl 10` expira após 10 segundos
- `--no-cache` ignora cache sempre

---

### T038 — Testes M4 (🟡 médio) ✅
**Arquivos:** `tests/m4_test.rs` (novo)

**O que fazer:** Criar testes de integração para todas as tasks M4 usando servidor HTTP local.

**Testes:**
1. **cookie_persistence** — Servidor seta cookie → FAF salva em jar → segunda request envia cookie
2. **cookie_netscape_format** — Salvar e carregar no formato Netscape, verificar que é compatível com curl
3. **wait_for_selector** — Servidor HTML com script que insere div após 100ms → FAF wait encontra
4. **wait_selector_timeout** — Servidor sem o elemento → FAF wait com timeout 1s → erro
5. **repl_stdin** — Echo pipe: `echo "1+1" | faf --stdin --url http://localhost:port` → "2"
6. **retry_exponential** — Servidor que falha 2x (503), depois sucede na 3ª → FAF --retries 3 sucede
7. **retry_429** — Servidor retorna 429 com Retry-After → FAF respeita e retenta
8. **cache_hit** — Request → cache → segunda request lê do cache (verificar pelo tempo)
9. **cache_ttl** — Cache com TTL curto → esperar → request faz novo fetch
10. **rate_limit_delay** — Follow com --delay 500ms → verificar timestamp entre requests
11. **show_headers** — Verificar que --show-headers exibe headers no output
12. **show_status** — Verificar que --show-status exibe status code

---

## 📋 M4.5 — Refinamentos Pós-M4 (3 tasks · 2 concluídas · 0 pendentes)

### F001 — Investigado: follow --extract NÃO vaza DOM entre páginas (🔴 crítica → ✅ Resolvido)
**Arquivos:** `tests/m4_test.rs` (teste adicionado)

**Diagnóstico:** O código do FAF está correto. Teste local com servidor de 3 páginas distintas confirmou que cada `follow_page()` cria DOM fresco e extrai APENAS elementos da página atual.

**Causa real:** As páginas individuais de books.toscrape.com TÊM sidebars com `h3` e `.price_color` de livros relacionados. O que parecia "vazamento de DOM entre páginas" era na verdade o conteúdo das próprias páginas.

**Evidência:** Teste local `test_follow_extract_no_leak` passou com cada página retornando exatamente 1 h1 distinto — sem leak.

**Recomendação:** Para filtrar resultados indesejados em sites com sidebars, usar `--filter` (agora com `!~=` graças ao F003):
```bash
faf follow ".product_pod h3 a" --extract "h3, .price_color" \
  --filter "text!~=A Light" \
  --url https://books.toscrape.com/
```

---

### F002 — Fix: cookies-jar ignorando Set-Cookie em redirects (🟡 médio)
**Arquivos:** `src/http/client.rs`, `src/http/cookies.rs`

**Problema:** O `--cookies-jar` não salva cookies quando o site redireciona (ex: httpbin.org/cookies/set faz 302 → /cookies). O reqwest segue redirects por padrão, e os headers `Set-Cookie` das respostas intermediárias (3xx) são perdidos — só vemos os headers da resposta final (200).

**Evidência:** `faf https://httpbin.org/cookies/set?foo=bar --cookies-jar jar.txt` → jar.txt vazio (só cabeçalho Netscape)

**Solução 1 (recomendada):** Habilitar `cookie_store(true)` no reqwest e usar o CookieStore dele em vez do parser manual. O reqwest gerencia cookies entre redirects corretamente.

**Solução 2 (alternativa):** Desabilitar `follow_redirects(false)` no reqwest quando `--cookies-jar` estiver ativo, e tratar redirects manualmente (seguindo Location header + capturando Set-Cookie de cada 3xx).

**Implementação (Solução 1):**
1. Em `HttpClient::new()`, habilitar `cookie_store(true)` no builder
2. Usar `reqwest::cookie::Jar` em vez do `Vec<NetscapeCookie>` manual para o store em memória
3. Converter entre Jar e formato Netscape para persistência
4. Atualizar `build_cookie_header()` para usar o CookieStore do reqwest
5. Remover parsing manual de Set-Cookie em `client.rs` (delegar ao reqwest)

**Critério:**
- `faf https://httpbin.org/cookies/set?k=v --cookies-jar jar.txt` salva cookies mesmo com redirect
- `faf https://httpbin.org/cookies --cookies jar.txt` reenvia cookies
- Compatibilidade com formato Netscape mantida (curl -b/-c)
- `cargo test`, `cargo clippy`

---

### F003 — Enhancement: --filter com operador !~= (negative match) (🟢 pequeno)
**Arquivos:** `src/api/filter.rs`

**Problema:** O `--filter` suporta `~=` (contém), `==` (exato), `!=` (negação exata) e `^=`/`$=` (prefixo/sufixo), mas não tem operador de **negação de substring**. Não é possível dizer "me dê links que NÃO contenham 'categoria'".

**Evidência:** `faf query "a" --filter "href~=catalogue"` retorna TUDO que contém "catalogue" (incluindo links de categoria), sem como excluir subconjuntos.

**Implementação:**
1. Adicionar operador `!~=` no parser de filtros: "href!~=category" significa "href NÃO contém 'category'"
2. Adicionar operador `!^=` (não começa com)
3. Adicionar operador `!$=` (não termina com)
4. Atualizar a documentação do filtro no README
5. Atualizar testes em `filter.rs`

**Critério:**
- `--filter "href!~=category"` exclui links que contêm "category"
- `--filter "href!^=http"` exclui links que começam com "http"
- Compatível com múltiplos filtros AND
- `cargo test`, `cargo clippy`

---

## 📋 M5 — Interação com Páginas (Page Interaction) (Planejado · 6 tasks)

**Objetivo do M5:** Transformar o FAF de um "leitor de páginas" para um "interator". Adicionar capacidade de clicar, preencher formulários, navegar por SPAs, e capturar screenshots.

**Dependências entre tasks:** T039 (click) → T040 (forms usa click) → T041 (screenshot depende de renderização). T042 é independente. T043/T044 são paralelizáveis.

---

### T039 — Click via dispatchEvent (🔴 complexo)
**Arquivos:** `src/js/dom_bridge.rs`, `src/js/mod.rs`, `src/api/commands.rs`

**O que faz:** Simular click do usuário em elementos da página via `dispatchEvent(new MouseEvent('click'))` no runtime JS. Essencial para interagir com SPAs, paginação, modais, botões "load more".

**Flags:** Nenhuma nova flag global. Click é acionado via JS no REPL/--stdin ou via novo subcomando `click`:
```bash
faf click ".btn-comprar" --url https://loja.com/produto
faf repl --url https://site.com
> document.querySelector(".pagination a").click()
```

**Implementação:**
1. No dom_bridge.rs, estender o objeto Element retornado por querySelector/querySelectorAll com método `.click()`:
   - `element.click()` → cria `new MouseEvent('click', { bubbles: true, cancelable: true })` e chama `dispatchEvent`
   - Usar `rquickjs` Function para criar e disparar o evento
   - Garantir que o evento propaga corretamente (bubbles = true)
2. Opcional: novo subcomando `click` na CLI (Command::Click):
   - `faf click ".btn" --url <url>` → fetch, parse, executa scripts, dispara click
   - Re-fetch da página após click (capturar estado pós-clique)
   - Suporta `--json`, `--show-status`, `--cache`
3. Tratar casos especiais:
   - `<a href="...">` links: click deve navegar (ou retornar URL alvo)
   - `<button type="submit">`: click em form dispara submit
   - Elementos `<option>` em `<select>`: click para selecionar
4. **⚠️ Cuidado:** click pode disparar navegação real (link), popup, ou requisição AJAX. O FAF não executa navegação real após click — apenas retorna o evento disparado + estado do DOM. Para navegação real, o usuário usa `--follow` ou faz novo fetch.
5. Adicionar ao `inject_dom()` para ficar disponível em REPL e --js scripts

**Testes:**
- Servidor HTML com botão que muda texto ao clicar → FAF click → texto mudou?
- Servidor com link que adiciona classe ao ser clicado → FAF click → classe presente?
- Click em elemento inexistente → erro amigável
- Teste via REPL/--stdin

**Critério:**
- `echo 'document.querySelector("button").click()' | faf --url <url> --stdin` → botão clicado
- `faf click ".btn" --url <url>` → evento disparado
- `cargo test`, `cargo clippy`

---

### T040 — Formulários: fill, select, submit (🟡 médio)
**Arquivos:** `src/js/dom_bridge.rs`, `src/api/commands.rs` (subcomando `fill` ou flag `--fill`)

**O que faz:** Preencher campos de formulário, selecionar opções e submeter, tudo via JS bridge. Permite simular login, busca, cadastro.

**Flags/Comando:**
```bash
# Modo 1: via JS no REPL
faf repl --url https://site.com/login
> document.querySelector("#email").value = "user@test.com"
> document.querySelector("#senha").value = "123456"
> document.querySelector("form").submit()

# Modo 2: subcomando fill (opcional, futura expansão)
# faf fill "input[name=email]" --value "user@test.com" --url ...
```

**Implementação:**
1. Garantir que `.value = "texto"` funcione em `<input>`, `<textarea>`, `<select>` via bridge DOM ↔ JS (já deve funcionar com rquickjs)
2. Opcional: helper `fill_form(selector, data)` no dom_bridge:
   - Aceita objeto `{ campo: valor }` e preenche automaticamente
   - Suporta input[type=text/email/password], textarea, select (value), checkbox (checked), radio
3. `.submit()` em elemento `<form>` → dispara submit (mas **não** segue a ação — retorna method + action para o usuário decidir)
4. Suporte a `FormData` e `URLSearchParams` no runtime JS (se não existir, criar polyfill)

**Testes:**
- Servidor com formulário → FAF preenche campos → `.value` reflete o valor setado
- Select option → .value = "opt2" → option selecionada
- Checkbox → .checked = true → checked

**Critério:**
- `echo 'document.querySelector("#email").value = "test@test.com"' | faf --stdin --url <url>` → campo preenchido
- `echo 'document.querySelector("form").submit()' | faf --stdin --url <url>` → submit disparado
- `cargo test`, `cargo clippy`

---

### T041 — Screenshot via tiny-skia (🔴 complexo)
**Arquivos:** `src/render/` (novo módulo), `src/render/mod.rs`, `src/render/screenshot.rs`, `src/api/commands.rs` (subcomando `screenshot`), `Cargo.toml` (tiny-skia, font-kit, image)

**O que faz:** Renderizar o HTML da página em uma imagem PNG usando o engine CSS já existente + tiny-skia para rasterização. Útil para debugging visual, thumbnails, e documentação.

**Flags/Comando:**
```bash
faf screenshot https://books.toscrape.com/ --output pagina.png
faf screenshot https://site.com --width 1920 --height 1080 --output preview.png
```

**Implementação:**
1. Criar módulo `src/render/`:
   - `screenshot.rs` — orquestra renderização: parse HTML → parse CSS → layout → paint
2. Re-utilizar componentes existentes do CSS engine:
   - `css::layout::BoxModel` — dimensões dos elementos
   - `css::style::ComputedStyle` — cores, fontes, display
   - `css::color::Color` — conversão de cor
3. Pipeline de renderização:
   - Parse HTML (já existe) → DOM tree
   - Parse CSS inline + da página (já existe)
   - Computar estilos (já existe) → elementos com posição/tamanho
   - **Paint:** desenhar cada elemento como retângulo com cor de fundo + texto
   - Usar `tiny-skia` para desenhar pixels em um canvas
   - Usar `font-kit` ou `rusttype` para renderizar texto nas posições corretas
4. Output: salvar como PNG via `image` ou `tiny-skia`'s `save_png`
5. Flags:
   - `--width` — largura do viewport (default: 1280)
   - `--height` — altura do viewport (default: 0 = scroll inteiro)
   - `--output` — caminho do PNG (default: `screenshot-<timestamp>.png`)
6. **⚠️ Cuidados:**
   - Isso é uma renderização **aproximada**, não pixel-perfect como Chrome
   - Prioridade: layout estável e cores corretas, não fidelidade absoluta
   - Texto sem WebGL ou aceleração — usar font bitmap simples
   - Elementos com `display: none` não são renderizados
7. Não tentar renderizar:
   - Imagens embutidas (`<img>` tags — custo muito alto por enquanto)
   - Iframes, vídeos, canvas
   - CSS avançado: flexbox, grid, position, float, overflow

**Testes:**
- Servidor com HTML simples (h1 colorido, parágrafo) → screenshot existe e não está vazio
- Servidor com display:none → elemento não aparece na screenshot
- Servidor sem CSS → renderização com defaults
- Testar dimensões: 800x600 vs 1920x1080

**Critério:**
- `faf screenshot https://books.toscrape.com/ --width 800 --output test.png` → test.png gerado com ≥ 1KB
- `cargo test`, `cargo clippy`

---

### T042 — Watch Mode: monitorar mudanças (🟡 médio)
**Arquivos:** `src/api/commands.rs` (subcomando `watch`), `src/js/engine.rs` (timeout/loop)

**O que faz:** Monitorar periodicamente uma URL (ou elemento) e notificar quando o conteúdo mudar. Útil para:
- Preços de produtos que mudam
- Status de pedidos
- Disponibilidade de estoque
- Notícias novas

**Flags/Comando:**
```bash
faf watch ".price" --url https://loja.com/produto --interval 30
# → [14:30:01] £51.77
# → [14:30:31] £49.99 ⚠️ MUDOU!

faf watch "h1" --url https://site.com --interval 60 --json
```

**Implementação:**
1. Subcomando `Command::Watch`:
   - `selector` — elemento a monitorar (opcional: monitorar página inteira)
   - `--interval <s>` — intervalo entre verificações (default: 30)
   - `--max-checks <N>` — parar após N verificações (default: 0 = infinito)
2. Loop:
   - Fetch URL + cache (usar `--cache` para não sobrecarregar)
   - Se selector: extrair texto/atributo do elemento
   - Comparar com valor anterior
   - Se mudou: printar ⚠️ com timestamp + valor antigo → novo
3. Output:
   - Timestamp + valor atual
   - Se mudou: destaque visual + diff
   - `--json` para pipe em scripts
4. **⚠️ Cuidados:**
   - Requer loop infinito — usar `tokio::time::interval`
   - Respeitar rate limiting (não floodar sites)
   - Cache obrigatório pra evitar múltiplos fetches desnecessários
   - Timeout: `--timeout` global se aplica

**Testes:**
- Servidor que muda conteúdo após N requests → watch detecta mudança
- Watch com --max-checks 2 → para após 2 iterações
- Watch sem mudança → apenas loga timestamp + valor

**Critério:**
- `faf watch "h1" --url <url> --interval 1 --max-checks 2` → executa 2 vezes e termina
- `cargo test`, `cargo clippy`

---

### T043 — Scroll e Navegação via JS (🟢 pequeno)
**Arquivos:** `src/js/dom_bridge.rs`

**O que faz:** Adicionar métodos de scroll e navegação na bridge DOM para interagir com páginas longas e conteúdo lazy-loaded.

**Métodos:**
```javascript
window.scrollTo(0, 1000)
window.scrollBy(0, 500)
document.querySelector(".produtos").scrollIntoView()
```

**Implementação:**
1. No dom_bridge.rs, expor funções helper no objeto `window`:
   - `scrollTo(x, y)` — scroll da janela
   - `scrollBy(x, y)` — scroll relativo
   - `scrollIntoView(selector)` — scroll até elemento
2. Todas implementadas via `window.scrollTo()` e `element.scrollIntoView()` no runtime JS
3. Após scroll, re-extrair conteúdo visível da página (se `--extract` foi usado)
4. **⚠️ Cuidado:** scroll não carrega novo conteúdo — depende de event listeners na página (scroll infinito). Para isso, combinar com T039 (click em "load more") ou T042 (watch mode).

**Testes:**
- Servidor com div grande → scrollTo → scrollBy → scrollIntoView executam sem erro
- scrollIntoView em elemento invisível → erro tratado

**Critério:**
- `echo 'window.scrollTo(0, document.body.scrollHeight)' | faf --stdin --url <url>` → scroll executado
- `cargo test`, `cargo clippy`

---

### T044 — Testes M5: integração em site real (🟡 médio)
**Arquivos:** `tests/m5_test.rs`

**O que fazer:** Testes de integração reais para todas as tasks M5, usando servidor local + site externo para verificar comportamento.

**Testes:**
1. **click_dispatched** — Servidor HTML com botão que muda texto ao clicar → FAF click no botão → texto mudou
2. **click_on_link** — Servidor com link → FAF click → link clicado (verificar por classe adicionada via JS)
3. **fill_form** — Servidor com formulário (input text, select, checkbox) → FAF preenche e verifica valores
4. **screenshot_generated** — Servidor com HTML simples → FAF screenshot → PNG existe e tem conteúdo
5. **screenshot_dimensions** — Screenshot com --width 800 → imagem com largura correta
6. **watch_detect_change** — Servidor que alterna conteúdo → FAF watch detecta mudança
7. **watch_max_checks** — Watch com --max-checks 3 → executa 3x e termina
8. **scroll_into_view** — Servidor com conteúdo longo → FAF scrollIntoView → posição mudou
9. **js_navigation** — Click em link que muda URL hash → window.location.hash mudou

**Critério:**
- 9+ testes passando
- `cargo test`, `cargo clippy`

---

## ✅ Resumo

| Milestone | Tasks | Status |
|---|---|---|
| M1 — Core Engine | 12 | ✅ Concluído |
| M2 — CSS Engine | 8 | ✅ Concluído |
| M2.5 — Polimento CLI | 8 | ✅ Concluído |
| M3 — JavaScript Engine | 10 | ✅ Concluído |
| **M4 — Sessão, Interação & Pipeline** | **8** | **✅ Concluído** |
| **M4.5 — Refinamentos Pós-M4** | **3** | **✅ Concluído** |
| M5 — Interação com Páginas | 6 | 📋 Planejado |
| **Total** | **55** | **39 concluídas · 9 planejadas** |

## ✅ Critério de conclusão do MVP

Todas as 52 tasks concluídas com:
- `cargo test` passando
- `cargo clippy` sem warnings
- `cargo fmt` aplicado
- README com exemplos funcionais
- Binário compilando com `cargo build --release`
- Teste manual em 3 sites reais
