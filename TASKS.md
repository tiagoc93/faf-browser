# ✅ Tasks — FAF BROWSER (Fast As Fuck)

**Total:** 61 tasks | **Concluídas:** 49 | **MVP Completo!** 🎉  
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

## 📋 M5 — Interação com Páginas (Page Interaction) (6 tasks · ✅ Concluído)

**Objetivo do M5:** Transformar o FAF de um "leitor de páginas" para um "interator". Adicionar capacidade de clicar, preencher formulários, navegar por SPAs, e capturar screenshots.

**Dependências entre tasks:** T039 (click) → T040 (forms usa click) → T041 (screenshot depende de renderização). T042 é independente. T043/T044 são paralelizáveis.

---

### T039 — Click via dispatchEvent (🔴 complexo) ✅
**Arquivos afetados:**
- `src/js/dom_bridge.rs` — ADICIONAR método `.click()` no objeto Element retornado por querySelector/querySelectorAll
- `src/api/commands.rs` — ADICIONAR subcomando `Command::Click { selector: String }` no enum Command + handler no match + struct ClickArgs com clap derive
- `src/js/mod.rs` — ADICIONAR função `inject_window_methods(ctx)` (se não existir, criar) para expor helpers no objeto global `window`
- `tests/m5_test.rs` — ADICIONAR 4+ testes de click

**O que faz:** Simular click do usuário em elementos da página via `dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))` no runtime QuickJS. Essencial para interagir com SPAs (paginação, modais, botões "load more", tabs).

**Flags (novo subcomando `click`):**
```bash
faf click ".pagination .next" --url https://site.com/blog?page=1
faf click "#modal .fechar" --url https://site.com --json --show-status

# Click também funciona via REPL/stdin (já existe, só adicionar método no Element):
echo 'document.querySelector("button").click()' | faf --url <url> --stdin
```

**Struct clap para o ClickArgs (commands.rs, logo após WaitArgs):**
```rust
#[derive(clap::Args, Debug)]
pub struct ClickArgs {
    /// Seletor CSS do elemento a clicar
    pub selector: String,
    /// Timeout em segundos para aguardar o elemento (default: 5)
    #[arg(long = "timeout", default_value = "5")]
    pub timeout: u64,
}
```

**Enum Command (commands.rs, junto com os outros):**
```rust
pub enum Command {
    Links,
    Images,
    Metadata,
    Query { selector: String },
    Follow(FollowArgs),
    Wait(WaitArgs),
    Click(ClickArgs),        // <-- NOVO
    Repl(ReplArgs),
}
```

**Implementação passo a passo:**

**PASSO 1 — Adicionar .click() no objeto Element da DOM bridge (dom_bridge.rs)**

Localize `fn inject_dom(ctx: &Ctx<'_>, doc: &HtmlDocument)` em dom_bridge.rs (linha ~26). Dentro dessa função, o objeto `document` é populado com métodos. Onde querySelector/querySelectorAll são definidos, ADICIONAR:

```rust
// Dentro de inject_dom(), depois de definir querySelector/querySelectorAll
// Fazer o objeto Element prototype ter método .click()

// 1. Criar função JS que simula click
let click_fn = Function::new(ctx.clone(), |element_val: Value| -> Result<()> {
    // element_val é o objeto Element retornado pelo JS
    // Criar: element.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    // Usar ctx.eval() ou ctx.globals() para acessar o runtime
    
    // CUIDADO: O objeto Element é um rquickjs Object. Para acessar dispatchEvent,
    // precisamos pegar o objeto real do DOM (scraper::ElementRef) que está wrapado.
    // Se o Element já é um Object JS, podemos chamar:
    //   ctx.eval("new MouseEvent('click', {bubbles: true, cancelable: true})")
    //   element.call("dispatchEvent", [mouse_event])
    
    Ok(())
})?;

// 2. Atribuir ao prototype de Element
// Se querySelector retorna um objeto JS, adicionar a prop "click" nele
// Melhor: retornar um objeto JS com todas as props (tag, id, classes, text, attrs, innerText, innerHTML, click)
```

⚠️ **Detalhe crítico:** O rquickjs retorna objetos JS puros (criados via `Object::new()` + `obj.set("prop", val)`). Para adicionar `.click()`, você precisa adicionar a função como propriedade do objeto retornado por querySelector. No código atual, `query_result_to_json()` retorna um `serde_json::Value` que é convertido pra JS. Em vez disso, crie manualmente um `Object` JS e adicione `.click()` como `Function`.

Alternativa mais limpa: criar um prototype de Element no contexto JS global:
```javascript
// Avaliado via ctx.eval()
if (typeof Element === 'undefined') {
    globalThis.Element = class Element {
        click() {
            this.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
        }
    };
}
```
E modificar os objetos retornados por querySelector para herdar desse prototype (usando `Object.setPrototypeOf` ou `__proto__`).

**PASSO 2 — Adicionar handler do subcomando Click (commands.rs)**

No match do `cli.command` (onde os outros comandos são tratados ~linha 620), ADICIONAR:

```rust
Some(Command::Click(args)) => {
    // 1. Fetch + parse HTML (igual ao flow do Wait)
    let resp = client.get(&url).await?;
    let html = resp.body;
    let doc = HtmlDocument::parse(&html);
    
    // 2. Query seletor
    let elements = doc.query(&args.selector)?;
    if elements.is_empty() {
        anyhow::bail!("Elemento '{}' não encontrado", args.selector);
    }
    
    // 3. Disparar click no primeiro elemento via JS
    let mut rt = crate::js::JsRuntime::with_client(client.clone())?;
    rt.set_dom(&doc)?;
    rt.init_timers()?;
    rt.init_fetch()?;
    
    // Executar scripts da página (se não --no-scripts)
    if !cli.no_scripts {
        let base_url = url::Url::parse(&url)?;
        rt.execute_page_scripts(&doc, &base_url).await?;
    }
    
    // Disparar click
    let js_code = format!(
        "document.querySelector({}).click()",
        serde_json::to_string(&args.selector)?
    );
    let result = rt.eval_with_timeout(&js_code, cli.js_timeout)?;
    
    // 4. Re-fetch da página para capturar estado pós-clique
    let resp2 = client.get(&url).await?;
    let html2 = resp2.body;
    let doc2 = HtmlDocument::parse(&html2);
    
    // 5. Output (reusar output::format_page_result)
    let result = crate::api::output::FollowPageResult {
        url: url.clone(),
        title: doc2.title(),
        first_heading: doc2.query("h1").ok().and_then(|r| r.into_iter().next()).map(|r| r.text),
        text_snippet: crate::api::commands::truncate(&doc2.visible_text(), 200),
        extracted: None,
    };
    
    // Formatar output
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("🖱️ Click em '{}': disparado", args.selector);
        if let Some(title) = &result.title {
            println!("📌 Título pós-click: {}", title);
        }
        println!("📝 {}", result.text_snippet);
    }
}
```

**PASSO 3 — Garantir que `truncate()` e `FollowPageResult` são acessíveis**

Em commands.rs, a função `truncate()` é definida como `fn truncate(s: &str, max: usize) -> String` (linha ~784). O struct `FollowPageResult` está em `src/api/output.rs` e precisa ser importado. Verificar se já está no `use` do commands.rs:
```rust
use crate::api::output::FollowPageResult;
```
Se não estiver, ADICIONAR.

**⚠️ Cuidados especiais:**
1. **MouseEvent não existe no QuickJS por padrão.** O runtime QuickJS do rquickjs é um JS puro sem DOM. Você precisa POLYFILL o `MouseEvent` antes de usar. Criar uma string JS com a implementação e executar com `ctx.eval()`. Exemplo de polyfill mínimo:
   ```javascript
   if (typeof MouseEvent === 'undefined') {
       globalThis.MouseEvent = class MouseEvent extends Event {
           constructor(type, opts = {}) {
               super(type, opts);
               this.bubbles = opts.bubbles || false;
               this.cancelable = opts.cancelable || false;
           }
       };
   }
   ```
2. **`dispatchEvent` também não existe** nos objetos retornados por querySelector (são objetos JS puros, não Element reais). Você precisa adicionar `dispatchEvent` ao objeto, ou fazer o click funcionar sem dispatchEvent real (apenas chamar handlers JS registrados).
3. **Abordagem alternativa:** em vez de tentar recriar o DOM event system completo, o click pode ser simulado encontrando o `<a>` no HTML e extraindo o href (para links), ou executando `onclick` diretamente se o elemento tiver atributo onclick.
4. **Re-fetch:** após o click, a página pode ter mudado (conteúdo carregado via AJAX, navegação SPA). O re-fetch captura o estado atual. Isso pode não refletir mudanças feitas por JS pós-load (SPA). Para SPAs, o melhor é usar o REPL e executar comandos manualmente.

**Testes a adicionar em tests/m5_test.rs:**

Seguir o padrão EXATO dos testes existentes em tests/m4_test.rs (TcpListener + thread::spawn + Cli::parse_from + run + assert):

```rust
// Teste 1: Botão que muda texto ao clicar
fn start_click_button_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        let body = r#"<html><body>
            <button id="btn" onclick="this.textContent='clicado'">Clique</button>
        </body></html>"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        let _ = stream.write_all(response.as_bytes());
    });
    port
}

#[tokio::test]
async fn test_click_changes_text() {
    let port = start_click_button_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--js", "document.querySelector('#btn').textContent",
        "--stdin",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok());
    // O texto do botão mudou?
}
```

**Critério de aceite:**
```bash
# Via subcomando click
faf click ".btn" --url http://127.0.0.1:PORT/
# → "🖱️ Click em '.btn': disparado"

# Via REPL/stdin
echo 'document.querySelector("button").click()' | faf --url http://127.0.0.1:PORT/ --stdin
# → click disparado

# Click em elemento inexistente
faf click ".nao-existe" --url http://127.0.0.1:PORT/
# → Error: Elemento '.nao-existe' não encontrado

cargo test tests::m5_test  # 4+ testes passando
cargo clippy  # 0 warnings
```

---

### T040 — Formulários: fill, select, submit (🟡 médio) ✅
**Arquivos afetados:**
- `src/js/dom_bridge.rs` — ADICIONAR no inject_dom(): helper `fill_form(selector, data)`, garantir que `.value`, `.checked`, `.submit()` funcionem nos objetos Element
- `src/api/commands.rs` — ADICIONAR handler no REPL/stdin (já deve funcionar se bridge DOM estiver correta). NÃO precisa de novo subcomando — forms são manipulados via JS/REPL.
- `tests/m5_test.rs` — ADICIONAR 3+ testes de form

**O que faz:** Permitir preencher campos de formulário, selecionar opções, marcar checkboxes e submeter formulários via JS bridge. Tudo via REPL/stdin.

**Como usar (nenhuma flag nova — tudo via JS):**
```bash
# Preencher input
echo 'document.querySelector("#email").value = "user@test.com"' | faf --stdin --url <url>
echo 'document.querySelector("#email").value' | faf --stdin --url <url>
# → "user@test.com"

# Selecionar option
echo 'document.querySelector("select[name=pais]").value = "BR"' | faf --stdin --url <url>

# Checkbox
echo 'document.querySelector("#aceito").checked = true' | faf --stdin --url <url>

# Submeter form
echo 'document.querySelector("form").submit()' | faf --stdin --url <url>
```

**⚠️ Requer T039 (click) concluído** — `.submit()` em form dispara os mesmos mecanismos de evento.

**Implementação passo a passo:**

**PASSO 1 — Garantir que objetos Element tenham `.value` (dom_bridge.rs)**

No `inject_dom()`, os objetos retornados por querySelector/querySelectorAll são construídos via `query_result_to_json()`. Essa função retorna campos como `text`, `innerText`, `innerHTML`, `attributes`. O `.value` NÃO é um atributo HTML — é uma propriedade do elemento JS.

Para suportar `.value = "texto"`, você PRECISA que o objeto Element JS tenha um setter/getter para `value`. Como os objetos são JS puros (não Element reais), você tem duas opções:

**Opção A (recomendada):** Em vez de retornar objetos JS planos, retornar objetos com getter/setter via `Object.defineProperty()`:

```javascript
// Avaliar no contexto JS:
var elementProto = {
    get value() { return this.attributes?.value || this.text || ''; },
    set value(v) { this.text = v; this.attributes = this.attributes || {}; this.attributes.value = v; },
    get checked() { return this.attributes?.checked === 'true'; },
    set checked(v) { 
        this.attributes = this.attributes || {}; 
        this.attributes.checked = v ? 'true' : 'false'; 
    },
    submit() { /* dispara evento de submit */ },
    click() { /* implementado no T039 */ }
};
```

E aplicar `Object.setPrototypeOf(element, elementProto)` em cada elemento retornado por querySelector.

**Opção B (alternativa):** Adicionar `value` e `checked` como propriedades no `query_result_to_json()` quando o elemento for `<input>`, `<select>` ou `<textarea>`:
```rust
// No match do tag em query_result_to_json(), adicionar:
let value = if ["input", "textarea", "select"].contains(&result.tag.as_str()) {
    result.attributes.get("value").cloned()
} else {
    None
};

json!({
    // ... existing fields ...
    "value": value,
    "checked": result.attributes.get("checked").map(|v| v == "true" || v == "checked"),
})
```

**PASSO 2 — Suporte a submit (dom_bridge.rs)**

O `.submit()` de um `<form>` não dispara um evento `submit` no elemento — ele ENVIA o formulário diretamente. No FAF, não temos navegação real. Então `.submit()` deve:

1. Coletar todos os inputs com name + value dentro do form
2. Construir querystring: `name1=value1&name2=value2`
3. Extrair `method` (GET/POST) e `action` do form
4. Se method=GET: retornar a URL com querystring para o usuário
5. Se method=POST: retornar os dados para o usuário
6. **⚠️ NÃO fazer o request** — apenas informar o que seria enviado

Implementação em JS polyfill:
```javascript
if (typeof HTMLFormElement === 'undefined') {
    globalThis.HTMLFormElement = class HTMLFormElement {
        submit() {
            var form = this;
            var method = (form.attributes.method || 'get').toUpperCase();
            var action = form.attributes.action || window.location.href;
            var inputs = form.querySelectorAll('input, select, textarea') || [];
            var data = {};
            inputs.forEach(function(el) {
                if (el.name) data[el.name] = el.value;
            });
            // Retornar como resultado da avaliação JS
            return JSON.stringify({ method: method, action: action, data: data });
        }
    };
}
```

**PASSO 3 — Suporte a FormData e URLSearchParams (js/mod.rs ou engine.rs)**

Se o usuário tentar usar `new FormData(form)` ou `new URLSearchParams()`, esses construtores não existem no QuickJS puro. ADICIONAR polyfills:

```javascript
// Polyfill URLSearchParams
if (typeof URLSearchParams === 'undefined') {
    globalThis.URLSearchParams = class URLSearchParams {
        constructor(init) {
            this.params = {};
            if (typeof init === 'string') {
                init.split('&').forEach(pair => {
                    var [k, v] = pair.split('=').map(decodeURIComponent);
                    this.params[k] = v;
                });
            }
        }
        get(name) { return this.params[name]; }
        set(name, value) { this.params[name] = value; }
        toString() { 
            return Object.entries(this.params)
                .map(([k, v]) => encodeURIComponent(k) + '=' + encodeURIComponent(v))
                .join('&'); 
        }
    };
}
```

Estes polyfills podem ser injetados UMA VEZ no runtime, no método `JsRuntime::new()` ou em `init_fetch()` (já que fetch também usa URLSearchParams).

**Testes a adicionar em tests/m5_test.rs:**

Seguir o padrão TcpListener + thread::spawn:

```rust
// Servidor com formulário HTML
fn start_form_server() -> u16 { ... }
// Servidor retorna HTML com:
// <form action="/login" method="POST">
//   <input name="email" type="text">
//   <input name="senha" type="password">
//   <select name="pais"><option value="BR">Brasil</option></select>
//   <input name="aceito" type="checkbox">
//   <button type="submit">Entrar</button>
// </form>

#[tokio::test]
async fn test_fill_input() {
    // Preenche input e verifica valor
}

#[tokio::test]
async fn test_select_option() {
    // Seleciona option e verifica
}

#[tokio::test]
async fn test_checkbox() {
    // Marca checkbox e verifica
}
```

**Critério de aceite:**
```bash
echo 'document.querySelector("#email").value = "teste@test.com"' | faf --stdin --url <url>
echo 'document.querySelector("#email").value' | faf --stdin --url <url>
# → "teste@test.com"

echo 'document.querySelector("#aceito").checked = true' | faf --stdin --url <url>
echo 'document.querySelector("#aceito").checked' | faf --stdin --url <url>
# → true

cargo test tests::m5_test  # 3+ testes passando
cargo clippy  # 0 warnings
```

---

### T041 — Screenshot via tiny-skia (🔴 complexo) ✅
**Arquivos afetados:**
- `Cargo.toml` — ADICIONAR dependências: `tiny-skia = "0.11"`, `font-kit = "0.14"`, `image = "0.25"`
- `src/render/mod.rs` — CRIAR novo módulo com `pub mod screenshot;`
- `src/render/screenshot.rs` — CRIAR pipeline de renderização completo
- `src/api/commands.rs` — ADICIONAR subcomando `Command::Screenshot(ScreenshotArgs)` + handler
- `src/lib.rs` — ADICIONAR `pub mod render;`
- `tests/m5_test.rs` — ADICIONAR 3+ testes de screenshot

**O que faz:** Renderizar o HTML de uma página em imagem PNG usando o CSS engine existente + tiny-skia. Gera uma representação visual aproximada da página (não pixel-perfect).

**Flags (novo subcomando `screenshot`):**
```bash
faf screenshot https://books.toscrape.com/ --width 1280 --output pagina.png
faf screenshot https://site.com --width 1920 --height 1080 --json
```

**Struct clap (commands.rs, após ClickArgs):**
```rust
#[derive(clap::Args, Debug)]
pub struct ScreenshotArgs {
    /// URL para capturar
    pub url: String,
    /// Largura do viewport em pixels (default: 1280)
    #[arg(long = "width", default_value = "1280")]
    pub width: u32,
    /// Altura do viewport em pixels (default: 0 = scroll inteiro)
    #[arg(long = "height", default_value = "0")]
    pub height: u32,
    /// Caminho do arquivo PNG de saída
    #[arg(long = "output", default_value = "screenshot.png")]
    pub output: String,
}
```

**Enum Command — ADICIONAR:**
```rust
pub enum Command {
    // ... existentes ...
    Wait(WaitArgs),
    Click(ClickArgs),
    Screenshot(ScreenshotArgs),  // <-- NOVO
    Repl(ReplArgs),
}
```

**Implementação passo a passo:**

**PASSO 1 — Criar src/render/screenshot.rs (NOVO ARQUIVO)**

Estrutura do arquivo:
```rust
use crate::css::layout::BoxModel;
use crate::css::style::ComputedStyle;
use crate::dom::HtmlDocument;
use tiny_skia::{Canvas, Paint, PathBuilder, Pixmap, Transform};
use std::path::Path;

/// Configuração da renderização
pub struct ScreenshotConfig {
    pub width: u32,
    pub height: u32,
}

/// Renderiza um HtmlDocument em um PNG, salvando no caminho especificado.
pub fn render_to_image(
    doc: &HtmlDocument,
    config: &ScreenshotConfig,
    output_path: &str,
) -> anyhow::Result<()> {
    // 1. Parse CSS da página
    let css_text = doc.extract_css().unwrap_or_default();
    let stylesheet = crate::css::parser::parse_css(&css_text)?;
    let styles = crate::css::style::compute_styles(doc, &stylesheet);
    
    // 2. Criar canvas
    let width = config.width;
    let height = if config.height > 0 {
        config.height
    } else {
        // Altura total do documento (soma das alturas dos elementos)
        compute_document_height(doc, &styles)
    };
    
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("Falha ao criar pixmap {}x{}", width, height))?;
    
    // 3. Fundo branco
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);
    pixmap.fill_rect(
        tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );
    
    // 4. Renderizar elementos visíveis do body
    if let Some(body) = doc.query("body").ok().and_then(|r| r.into_iter().next()) {
        // Percorrer elementos filho recursivamente
        render_element(&body, doc, &styles, &mut pixmap, 0.0, 0.0, width as f32)?;
    }
    
    // 5. Salvar PNG
    pixmap.save_png(output_path)
        .map_err(|e| anyhow::anyhow!("Falha ao salvar PNG: {}", e))?;
    
    Ok(())
}

/// Renderiza um elemento e seus filhos recursivamente.
fn render_element(
    element: &crate::dom::QueryResult,
    doc: &HtmlDocument,
    styles: &std::collections::HashMap<String, ComputedStyle>,
    pixmap: &mut Pixmap,
    parent_x: f32,
    parent_y: f32,
    parent_width: f32,
) -> anyhow::Result<()> {
    // 1. Obter computed style
    let style = styles.get(&element.id.clone().unwrap_or_default())
        .unwrap_or(&ComputedStyle::default());
    
    // 2. Pular display: none
    if style.display == "none" {
        return Ok(());
    }
    
    // 3. Calcular posição e tamanho
    let x = parent_x + style.margin_left;
    let y = parent_y + style.margin_top;
    let w = style.width.unwrap_or(parent_width - style.margin_left - style.margin_right);
    let h = style.height.unwrap_or(16.0); // altura mínima
    
    // 4. Desenhar background-color
    if let Some(bg) = &style.background_color {
        let color = crate::css::color::parse_color(bg);
        if let Some(c) = color {
            let mut paint = Paint::default();
            paint.set_color_rgba8(c.r, c.g, c.b, (c.a * 255.0) as u8);
            let rect = tiny_skia::Rect::from_xywh(x, y, w, h).unwrap_or_default();
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }
    
    // 5. Desenhar texto
    if !element.text.is_empty() {
        // Usar font-kit para carregar fonte
        // Desenhar texto com tiny-skia (ou usar biblioteca de texto)
        // Por agora: apenas desenhar um placeholder visual
        let mut paint = Paint::default();
        let fg = style.color.as_deref().unwrap_or("#000000");
        if let Some(c) = crate::css::color::parse_color(fg) {
            paint.set_color_rgba8(c.r, c.g, c.b, 255);
        }
        // Desenhar o texto como retângulo simples + cor
        // NOTA: renderização de texto REAL requer font-kit + layout
        // Para MVP, desenhar um retângulo com a cor do texto
    }
    
    // 6. Renderizar filhos recursivamente
    // Se o QueryResult tiver filhos, iterar sobre eles
    // (O HtmlDocument::query() atual não expõe filhos diretamente.
    //  Para renderização recursiva, precisamos de uma API de DOM tree.
    //  ALTERNATIVA: usar scraper::ElementRef diretamente se exposto.)
    
    Ok(())
}

/// Calcula a altura total do documento somando alturas dos filhos do body.
fn compute_document_height(
    doc: &HtmlDocument,
    styles: &std::collections::HashMap<String, ComputedStyle>,
) -> u32 {
    // Placeholder: altura fixa para MVP
    800
}
```

**PASSO 2 — Conectar no lib.rs**

Em `src/lib.rs`, ADICIONAR:
```rust
pub mod render;
```

**PASSO 3 — Handler do subcomando (commands.rs)**

No match do `cli.command`:
```rust
Some(Command::Screenshot(args)) => {
    // Fetch da URL
    let resp = client.get(&args.url).await?;
    let html = resp.body;
    let doc = HtmlDocument::parse(&html);
    
    // Renderizar
    let config = render::screenshot::ScreenshotConfig {
        width: args.width,
        height: args.height,
    };
    
    render::screenshot::render_to_image(&doc, &config, &args.output)?;
    
    println!("📸 Screenshot salvo em: {}", args.output);
}
```

**⚠️ Cuidados especiais:**

1. **Escopo limitado para MVP:** Este screenshot é uma renderização aproximada. Não tente fazer pixel-perfect. Priorize:
   - Layout básico (elementos empilhados verticalmente)
   - Cores de fundo e texto
   - Texto em posições aproximadas (usando font bitmap)
   - `display: none` respeitado

2. **Não tentar renderizar:**
   - Imagens (`<img>`, `<picture>`, `<svg>`)
   - CSS complexo (flexbox, grid, float, position: absolute/fixed)
   - Iframes, vídeos, canvas
   - Web fonts (usar fonte monospace padrão do sistema)
   - Overflow, scroll, clip

3. **Dependências Cargo.toml:**
   ```toml
   tiny-skia = "0.11"
   font-kit = "0.14"  # Para carregar fontes do sistema
   image = "0.25"     # Fallback para salvar PNG se tiny-skia não der conta
   ```
   Verificar compatibilidade com a edition 2024. Se alguma dep não compilar, usar alternativa.

4. **Nested runtime:** O screenshot NÃO usa tokio — é puramente síncrono (parse CSS + tiny-skia paint). Isso evita o nested runtime panic.

5. **API do HtmlDocument:** O módulo `dom` atualmente expõe `query(selector)` que retorna `Vec<QueryResult>` achatado (não uma árvore). Para renderização hierárquica, você PRECISA de acesso à árvore DOM real. Opções:
   - Opção A: Expor `scraper::Html` internamente via método `doc.inner_html()` e re-parsar
   - Opção B: Usar `scraper::ElementRef` diretamente (mais complexo)
   - **Opção C (recomendada para MVP):** Renderizar APENAS os elementos do `body` que são retornados por `query("*")`, ordenados por posição no HTML (que scraper respeita). Isso dá uma renderização linear (como um documento de texto) sem hierarquia real.

**Testes (tests/m5_test.rs):**

```rust
/// Servidor com HTML simples para screenshot
fn start_screenshot_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let body = r#"<html><head><style>h1 { color: red; }</style></head>
            <body><h1>Titulo</h1><p>Paragrafo</p></body></html>"#;
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
        let _ = stream.write_all(response.as_bytes());
    });
    port
}

#[tokio::test]
async fn test_screenshot_generated() {
    let port = start_screenshot_server();
    let output = format!("/tmp/faf_test_screenshot_{}.png", std::process::id());
    let cli = Cli::parse_from([
        "faf",
        "screenshot",
        &format!("http://127.0.0.1:{}/", port),
        "--width", "800",
        "--output", &output,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok());
    assert!(std::path::Path::new(&output).exists(), "PNG deve existir");
    let metadata = std::fs::metadata(&output).unwrap();
    assert!(metadata.len() > 500, "PNG deve ter > 500 bytes");
    let _ = std::fs::remove_file(&output);
}
```

**Critério de aceite:**
```bash
faf screenshot https://books.toscrape.com/ --width 800 --output /tmp/faf-test.png
# → "📸 Screenshot salvo em: /tmp/faf-test.png"
ls -la /tmp/faf-test.png  # Deve existir com ≥ 1KB

cargo test tests::m5_test  # 3+ testes passando
cargo clippy  # 0 warnings
```

---

### T042 — Watch Mode: monitorar mudanças (🟡 médio) ✅
**Arquivos afetados:**
- `src/api/commands.rs` — ADICIONAR subcomando `Command::Watch(WatchArgs)` + struct WatchArgs + handler com loop tokio
- `src/utils/config.rs` — ADICIONAR campo `watch_interval` se necessário (ou usar flag inline no WatchArgs)
- `tests/m5_test.rs` — ADICIONAR 3+ testes de watch

**O que faz:** Monitorar periodicamente uma URL (ou elemento específico) e notificar quando o conteúdo mudar. Essencial para:
- Preços de produtos que flutuam
- Status de pedidos (em trânsito → entregue)
- Disponibilidade de estoque (esgotado → disponível)
- Novas notícias/resultados de busca

**Flags (novo subcomando `watch`):**
```bash
faf watch ".price" --url https://loja.com/produto --interval 30 --max-checks 5
# → [14:30:01] £51.77
# → [14:30:31] £49.99 ⚠️ MUDOU! (antes: £51.77)
# → [14:31:01] £49.99
# → [14:31:31] £49.99

faf watch "h1" --url https://site.com --interval 60 --json
```

**Struct clap (commands.rs, após ScreenshotArgs):**
```rust
#[derive(clap::Args, Debug)]
pub struct WatchArgs {
    /// Seletor CSS do elemento a monitorar (opcional: monitora a página inteira)
    pub selector: Option<String>,
    /// URL para monitorar
    #[arg(long = "url")]
    pub url: String,
    /// Intervalo em segundos entre verificações (default: 30)
    #[arg(long = "interval", default_value = "30")]
    pub interval: u64,
    /// Número máximo de verificações (0 = infinito, default: 0)
    #[arg(long = "max-checks", default_value = "0")]
    pub max_checks: u64,
}
```

**Enum Command — ADICIONAR:**
```rust
pub enum Command {
    // ... existentes ...
    Screenshot(ScreenshotArgs),
    Watch(WatchArgs),  // <-- NOVO
    Repl(ReplArgs),
}
```

**Implementação passo a passo:**

**PASSO 1 — Handler do watch (commands.rs)**

```rust
Some(Command::Watch(args)) => {
    let mut previous_value: Option<String> = None;
    let mut checks: u64 = 0;
    
    loop {
        // Verificar limite
        if args.max_checks > 0 && checks >= args.max_checks {
            break;
        }
        
        // Fetch + parse
        let resp = client.get(&args.url).await?;
        let html = resp.body;
        let doc = HtmlDocument::parse(&html);
        
        // Extrair valor
        let current_value = if let Some(ref selector) = args.selector {
            // Extrair texto do primeiro elemento que match
            doc.query(selector)
                .ok()
                .and_then(|r| r.into_iter().next())
                .map(|r| r.text)
                .unwrap_or_else(|| "(elemento não encontrado)".to_string())
        } else {
            // Monitorar página inteira (texto visível + título)
            let title = doc.title().unwrap_or_default();
            let text = doc.visible_text();
            let snippet = if text.len() > 100 {
                format!("{}...", &text[..100])
            } else {
                text.clone()
            };
            format!("{} | {}", title, snippet)
        };
        
        // Timestamp
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        // ⚠️ Se chrono não estiver nas deps, usar std::time::SystemTime
        
        // Comparar com valor anterior
        if let Some(ref prev) = previous_value {
            if *prev != current_value {
                // MUDOU!
                if cli.json {
                    println!(r#"{{"time":"{}","selector":"{}","previous":"{}","current":"{}","changed":true}}"#,
                        now,
                        args.selector.as_deref().unwrap_or("*"),
                        prev.replace('"', "\\\""),
                        current_value.replace('"', "\\\""));
                } else {
                    println!("[{}] ⚠️ MUDOU! (antes: {}) → {}", now, prev, current_value);
                }
            } else {
                if cli.json {
                    println!(r#"{{"time":"{}","value":"{}","changed":false}}"#,
                        now,
                        current_value.replace('"', "\\\""));
                } else {
                    println!("[{}] {}", now, current_value);
                }
            }
        } else {
            // Primeira execução
            if cli.json {
                println!(r#"{{"time":"{}","value":"{}","changed":false,"first":true}}"#,
                    now,
                    current_value.replace('"', "\\\""));
            } else {
                println!("[{}] {}", now, current_value);
            }
        }
        
        previous_value = Some(current_value);
        checks += 1;
        
        // ⚠️ Importante: se --max-checks == 0, loop infinito.
        // O usuário deve Ctrl+C para parar.
        // Para evitar loop infinito em testes, sempre incrementamos checks
        // e respeitamos max_checks.
        
        // Aguardar intervalo (exceto na última iteração)
        if args.max_checks == 0 || checks < args.max_checks {
            tokio::time::sleep(std::time::Duration::from_secs(args.interval)).await;
        }
    }
}
```

**PASSO 2 — Registrar subcomando no enum + parser**

O clap derive já cuida do parse. Apenas adicionar ao enum e ao match.

**⚠️ Cuidados especiais:**

1. **Loop infinito:** watch com `--max-checks 0` fica em loop até Ctrl+C. IMPORTANTE: o runtime tokio precisa de `ctrl_c` handler ou o usuário só sai matando o processo. Para segurança, limitar a 1000 checks se max_checks=0 e emitir warning.

2. **Rate limiting:** watch bate no servidor a cada `--interval` segundos. Para intervalos < 5s, emitir warning sobre possível bloqueio.

3. **Cache:** watch DEVE usar `--cache` se disponível para não sobrecarregar o servidor. Mas cuidado: cache pode mascarar mudanças reais. Por default, watch NÃO usa cache (faz fetch real a cada vez). Se `--cache` for passado explicitamente, usar cache com TTL curto.

4. **JSON output:** o `--json` produz linhas JSON separadas (JSONL), uma por verificação. Cada linha tem: time, selector, value, changed, (opcional) previous.

5. **Dependência chrono:** Se quiser timestamp formatado, precisa de `chrono = "0.4"` no Cargo.toml. Alternativa sem chrono: usar `std::time::SystemTime::now()` e formatar manualmente.

**Testes (tests/m5_test.rs):**

```rust
/// Servidor que alterna conteúdo entre requests
fn start_watch_changing_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let counter = Arc::new(AtomicU64::new(0));
    let c = counter.clone();
    thread::spawn(move || {
        for _ in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            let count = c.fetch_add(1, Ordering::SeqCst);
            let body = format!("<html><body><h1>Valor: {}</h1></body></html>", count);
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

#[tokio::test]
async fn test_watch_detect_change() {
    let port = start_watch_changing_server();
    let cli = Cli::parse_from([
        "faf",
        "watch",
        "h1",
        "--url", &format!("http://127.0.0.1:{}/", port),
        "--interval", "1",
        "--max-checks", "3",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok());
}
```

**Critério de aceite:**
```bash
faf watch "h1" --url http://127.0.0.1:PORT/ --interval 1 --max-checks 2
# → [HH:MM:SS] Valor: 0
# → [HH:MM:SS] ⚠️ MUDOU! (antes: Valor: 0) → Valor: 1

cargo test tests::m5_test  # 3+ testes passando
cargo clippy  # 0 warnings
```

---

### T043 — Scroll e Navegação via JS (🟢 pequeno) ✅
**Arquivos afetados:**
- `src/js/dom_bridge.rs` — ADICIONAR no inject_dom(): helpers `window.scrollTo`, `window.scrollBy`, `element.scrollIntoView`
- `src/js/mod.rs` — Se não existir função de injeção de window methods, ADICIONAR
- `tests/m5_test.rs` — ADICIONAR 2+ testes de scroll

**O que faz:** Adicionar métodos de scroll e navegação na bridge DOM para interagir com páginas longas e conteúdo lazy-loaded. Essencial para:
- Scroll infinito (combinado com T039 click em "load more")
- Páginas de documentação longa
- Resultados de busca que carregam ao scrollar

**Métodos a expor (via polyfill JS no runtime):**
```javascript
// scrollTo — scroll absoluto
window.scrollTo(0, 1000);
window.scrollTo({ top: 1000, behavior: 'smooth' });

// scrollBy — scroll relativo
window.scrollBy(0, 500);

// scrollIntoView — scroll até elemento
document.querySelector(".footer").scrollIntoView();
document.querySelector(".footer").scrollIntoView({ behavior: 'smooth', block: 'start' });
```

**⚠️ IMPORTANTE:** O FAF não tem uma "janela" real com viewport. O scroll é SIMULADO — mantemos uma posição Y global no runtime JS. A utilidade prática é:
1. Mudar o foco de extração: após scroll, re-executar `querySelector`/`extract` para capturar elementos que estavam fora da "viewport"
2. Disparar event listeners de scroll que a página possa ter (scroll infinito, lazy loading)
3. A posição Y simulada fica em `window.pageYOffset`

**Implementação (dom_bridge.rs, no inject_dom()):**

```javascript
// Polyfill scroll simulado
// Avaliar no contexto JS via ctx.eval()
(function() {
    if (typeof window === 'undefined') { globalThis.window = {}; }
    
    var scrollY = 0;
    
    window.scrollY = 0;
    window.pageYOffset = 0;
    
    window.scrollTo = function(xOrOpts, y) {
        if (typeof xOrOpts === 'object') {
            y = xOrOpts.top || 0;
        } else if (y === undefined) {
            y = xOrOpts || 0;
        }
        scrollY = Math.max(0, y);
        window.scrollY = scrollY;
        window.pageYOffset = scrollY;
        // Disparar evento de scroll (se houver listeners)
        var event = new Event('scroll');
        window.dispatchEvent(event);
    };
    
    window.scrollBy = function(xOrOpts, y) {
        if (typeof xOrOpts === 'object') {
            y = xOrOpts.top || 0;
        } else if (y === undefined) {
            y = xOrOpts || 0;
        }
        window.scrollTo(0, scrollY + y);
    };
})();
```

```rust
// Em Rust, dentro de inject_dom(), adicionar:
let scroll_polyfill = r#"
(function() {
    if (typeof window === 'undefined') { globalThis.window = {}; }
    var scrollY = 0;
    window.scrollY = 0;
    window.pageYOffset = 0;
    window.scrollTo = function(xOrOpts, y) {
        if (typeof xOrOpts === 'object') {
            y = xOrOpts.top || 0;
        } else if (y === undefined) {
            y = xOrOpts || 0;
        }
        scrollY = Math.max(0, y);
        window.scrollY = scrollY;
        window.pageYOffset = scrollY;
        var event = new Event('scroll');
        window.dispatchEvent(event);
    };
    window.scrollBy = function(xOrOpts, y) {
        if (typeof xOrOpts === 'object') {
            y = xOrOpts.top || 0;
        } else if (y === undefined) {
            y = xOrOpts || 0;
        }
        window.scrollTo(0, scrollY + y);
    };
})();
"#;
ctx.eval::<(), _>(scroll_polyfill)?;
```

E para scrollIntoView nos elementos:
```javascript
// No elementProto (criado no T039/T040):
elementProto.scrollIntoView = function(opts) {
    // Simular scroll até a posição do elemento
    // Como não temos posição real, apenas marcar que foi scrollado
    window.lastScrolledElement = this;
};
```

**Testes (tests/m5_test.rs):**
```rust
#[tokio::test]
async fn test_scroll_to() {
    let port = start_basic_server(); // Servidor que retorna HTML simples
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--js", "window.scrollTo(0, 500); window.pageYOffset",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok());
    // O resultado deve conter "500" (posição Y após scroll)
}

#[tokio::test]
async fn test_scroll_by() {
    let port = start_basic_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--js", "window.scrollBy(0, 200); window.scrollY",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok());
}
```

**Critério de aceite:**
```bash
echo 'window.scrollTo(0, document.body.scrollHeight); window.pageYOffset' \
  | faf --stdin --url <url>
# → 500 (ou altura do documento)

cargo test tests::m5_test  # 2+ testes passando
cargo clippy  # 0 warnings
```

---

### T044 — Testes M5: integração completa (🟡 médio) ✅
**Arquivo principal:** `tests/m5_test.rs` (CRIAR se não existir, ou ADICIONAR aos existentes)

**⚠️ ATENÇÃO:** Verificar se `tests/m5_test.rs` já existe (pode ter sido criado pelo M4). Se existir, ADICIONAR ao arquivo existente. Se não, CRIAR.

**Se for criar (modelo — copiar de tests/m4_test.rs):**
```rust
// tests/m5_test.rs
use faf_browser::api::commands::{Cli, run};
use clap::Parser;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

// ... funções auxiliares de servidor ...

// Servidor base para testes
fn start_basic_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let body = "<html><body><h1>Teste</h1></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        let _ = stream.write_all(response.as_bytes());
    });
    port
}
```

**Testes a implementar (total: 9):**

| # | Teste | Task | O que verifica |
|---|---|---|---|
| 1 | `test_click_changes_text` | T039 | Botão com onclick → FAF click → texto mudou |
| 2 | `test_click_nonexistent` | T039 | Click em elemento que não existe → erro |
| 3 | `test_fill_input` | T040 | Preencher input de texto → .value reflete |
| 4 | `test_select_option` | T040 | Selecionar option → .value mudou |
| 5 | `test_checkbox` | T040 | Marcar checkbox → .checked = true |
| 6 | `test_screenshot_generated` | T041 | Screenshot → PNG existe com ≥ 500 bytes |
| 7 | `test_watch_detect_change` | T042 | Watch detecta mudança entre requests |
| 8 | `test_watch_max_checks` | T042 | Watch com --max-checks N → executa N× |
| 9 | `test_scroll_to` | T043 | window.scrollTo → pageYOffset mudou |

**Padrão de cada teste (template):**
```rust
#[tokio::test]
async fn test_NOME_DO_TESTE() {
    // 1. Iniciar servidor local
    let port = start_SERVERTYPE_server();
    
    // 2. Configurar CLI
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        // flags específicas do teste
    ]);
    
    // 3. Executar
    let result = run(cli).await;
    
    // 4. Assert
    assert!(result.is_ok(), "...");
}
```

**⚠️ Cuidados nos testes:**
1. **Timeout:** Testes com watch precisam de timeout pequeno (--interval 1 --max-checks 2). O runtime tokio do teste lida com isso.
2. **Porta:** Cada servidor usa `127.0.0.1:0` (porta aleatória) para não conflitar.
3. **Thread:** Servidores rodam em `thread::spawn`. O listener precisa aceitar N conexões (o número de requests que o teste fará). Usar `for _ in 0..N` no loop do servidor.
4. **Import do `run`:** `use faf_browser::api::commands::run;` — esta função é `pub async fn run(cli: Cli) -> anyhow::Result<()>`.
5. **Import do `Cli`:** `use faf_browser::api::commands::Cli;` — o struct com `Parser` derive.

**Critério de aceite:**
```bash
cargo test tests::m5_test  # 9 testes passando
cargo clippy  # 0 warnings
```

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
| **M5 — Interação com Páginas** | **6** | **✅ Concluído** |
| **Total** | **61** | **49 concluídas · 0 planejadas** |

## ✅ Critério de conclusão do MVP

Todas as 61 tasks concluídas com:
- `cargo test` passando ✅ 266 testes
- `cargo clippy` sem warnings ✅
- `cargo fmt` aplicado
- README com exemplos funcionais
- Binário compilando com `cargo build --release`
- Teste manual em 3 sites reais
