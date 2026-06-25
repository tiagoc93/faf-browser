use clap::Parser;
use rand::Rng;
use std::io::{self, BufRead, Write};
use url::Url;

/// FAF BROWSER — Fast As Fuck. Navegador headless 100% Rust.
#[derive(Parser, Debug)]
#[command(name = "faf", version, about)]
pub struct Cli {
    /// URL para buscar
    pub url: Option<String>,

    /// URL para buscar (alternativa ao argumento posicional)
    #[arg(long = "url", global = true)]
    pub url_flag: Option<String>,

    /// Arquivo de script JavaScript
    #[arg(short = 's', long = "js")]
    pub js_script: Option<String>,

    /// Caminho para arquivo JavaScript a ser executado
    #[arg(long = "js-file")]
    pub js_file: Option<String>,

    /// Lê expressões JS da entrada padrão (útil para pipes)
    #[arg(long = "stdin")]
    pub stdin: bool,

    /// Desabilita execução automática de scripts da página
    #[arg(long = "no-scripts")]
    pub no_scripts: bool,

    /// Timeout em segundos para execução de JavaScript
    #[arg(long = "js-timeout", default_value = "5", global = true)]
    pub js_timeout: u64,

    /// CSS inline ou caminho para arquivo CSS
    #[arg(long = "css", visible_alias = "style", global = true)]
    pub css: Option<String>,

    /// Desabilita extração automática de CSS da página
    #[arg(long = "no-page-css", global = true)]
    pub no_page_css: bool,

    /// Proxy (http:// ou socks5://)
    #[arg(short = 'x', long = "proxy")]
    pub proxy: Option<String>,

    /// Timeout em segundos
    #[arg(short = 't', long = "timeout", default_value = "30")]
    pub timeout: u64,

    /// User-Agent customizado
    #[arg(short = 'u', long = "user-agent")]
    pub user_agent: Option<String>,

    /// Modo verbose
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Saída em JSON
    #[arg(short = 'j', long = "json", global = true)]
    pub json: bool,

    /// Extrair campos específicos (tag,id,classes,text,html,href,src,alt,color,bg,font-size,font-family,display)
    #[arg(long = "get", global = true, value_delimiter = ',')]
    pub get: Option<Vec<String>>,

    /// Formato de saída: text, json, jsonl, csv
    #[arg(long = "format", global = true, default_value = "text")]
    pub format: String,

    /// Filtros de query no formato campo~=valor, campo==valor, etc.
    #[arg(long = "filter", global = true)]
    pub filter: Option<Vec<String>>,

    /// Número máximo de tentativas de retry em requisições falhas
    #[arg(long = "retries", global = true, default_value = "0")]
    pub retries: u64,

    /// Delay inicial entre retries em milissegundos
    #[arg(long = "retry-delay", global = true, default_value = "1000")]
    pub retry_delay: u64,

    /// Exibir status HTTP da resposta
    #[arg(long = "show-status", global = true)]
    pub show_status: bool,

    /// Exibir headers HTTP da resposta
    #[arg(long = "show-headers", global = true)]
    pub show_headers: bool,

    /// Carregar cookies de arquivo no formato Netscape
    #[arg(long = "cookies", global = true)]
    pub cookies: Option<String>,

    /// Salvar cookies em arquivo no formato Netscape após a requisição
    #[arg(long = "cookies-jar", global = true)]
    pub cookies_jar: Option<String>,

    /// Diretório de cache para respostas HTTP
    #[arg(long = "cache", global = true)]
    pub cache: Option<String>,

    /// TTL do cache em segundos
    #[arg(long = "cache-ttl", global = true, default_value = "300")]
    pub cache_ttl: u64,

    /// Ignorar cache mesmo se o diretório existir
    #[arg(long = "no-cache", global = true)]
    pub no_cache: bool,

    /// Comando: links, images, metadata, query
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Args, Debug)]
pub struct FollowArgs {
    /// Seletor CSS para encontrar links
    pub selector: String,

    /// Seletor CSS para extrair dados de cada página seguida
    #[arg(long = "extract")]
    pub extract: Option<String>,

    /// Máximo de páginas a visitar
    #[arg(long = "max", default_value = "10")]
    pub max: u64,

    /// Número de requisições concorrentes
    #[arg(long = "concurrency", default_value = "3")]
    pub concurrency: usize,

    /// Restringir ao mesmo domínio
    #[arg(long = "same-domain", default_value = "true", action = clap::ArgAction::Set)]
    pub same_domain: bool,

    /// Delay fixo entre batches de requisições em milissegundos
    #[arg(long = "delay", default_value = "0")]
    pub delay: u64,

    /// Delay aleatório entre batches em milissegundos (min max)
    #[arg(long = "random-delay", num_args = 2, value_names = ["MIN", "MAX"])]
    pub random_delay: Option<Vec<u64>>,
}

#[derive(clap::Args, Debug)]
pub struct ReplArgs {
    // No additional flags needed currently
}

#[derive(clap::Args, Debug)]
pub struct WaitArgs {
    /// Seletor CSS para aguardar
    pub selector: String,

    /// Timeout em segundos (default: 10)
    #[arg(long = "timeout", default_value = "10")]
    pub timeout: u64,

    /// Intervalo de polling em milissegundos (default: 200)
    #[arg(long = "interval", default_value = "200")]
    pub interval: u64,
}

#[derive(clap::Args, Debug)]
pub struct ClickArgs {
    /// Seletor CSS do elemento a clicar
    pub selector: String,

    /// Timeout em segundos para aguardar o elemento (default: 5)
    #[arg(long = "timeout", default_value = "5")]
    pub timeout: u64,
}

#[derive(clap::Args, Debug)]
pub struct WatchArgs {
    /// Seletor CSS do elemento a monitorar (opcional: monitora a página inteira)
    pub selector: Option<String>,

    /// Intervalo em segundos entre verificações (default: 30)
    #[arg(long = "interval", default_value = "30")]
    pub interval: u64,

    /// Número máximo de verificações (0 = infinito, default: 0)
    #[arg(long = "max-checks", default_value = "0")]
    pub max_checks: u64,
}

#[derive(clap::Args, Debug)]
pub struct DumpArgs {
    /// Caminho do arquivo HTML de saída (stdout se omitido)
    #[arg(long = "output")]
    pub output: Option<String>,

    /// Converter imagens para base64 inline
    #[arg(long = "inline-images")]
    pub inline_images: bool,

    /// Não inlinear CSS externo (manter <link>)
    #[arg(long = "no-inline-css")]
    pub no_inline_css: bool,

    /// Remover scripts da página
    #[arg(long = "no-scripts")]
    pub no_scripts: bool,

    /// Formato de saída: html, markdown, text, json
    #[arg(long = "format", default_value = "html")]
    pub format: String,

    /// Extrair apenas o conteúdo principal (remove nav, footer, sidebar)
    #[arg(long = "readability")]
    pub readability: bool,

    /// Extrair dados estruturados (JSON-LD, Open Graph, meta tags)
    #[arg(long = "structured-data")]
    pub structured_data: bool,

    /// Incluir YAML frontmatter com metadadosOpenGraph na saída markdown (default: on)
    #[arg(long = "frontmatter", action = clap::ArgAction::Set, default_value = "true")]
    pub frontmatter: bool,

    /// Dividir markdown em chunks de N tokens (0 = desativado)
    #[arg(long = "chunk-size", default_value = "0")]
    pub chunk_size: usize,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Extrair todos os links da página
    Links,
    /// Extrair todas as imagens da página
    Images,
    /// Extrair metadados (Open Graph, title, description)
    Metadata,
    /// Executar query CSS
    Query {
        /// Seletor CSS
        selector: String,
    },
    /// Seguir links encontrados por um seletor, visitando cada página
    Follow(FollowArgs),
    /// Aguardar elemento CSS aparecer no DOM
    Wait(WaitArgs),
    /// Simular click em elemento CSS
    Click(ClickArgs),
    /// Monitorar mudanças em uma URL ou elemento
    Watch(WatchArgs),
    /// Gerar HTML autocontido da página
    Dump(DumpArgs),
    /// Modo interativo REPL para executar múltiplos comandos JS
    Repl(ReplArgs),
}

/// Executa o comando CLI
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    // Se não tem URL, mostra ajuda
    let url = match cli.url_flag.as_ref().or(cli.url.as_ref()) {
        Some(u) => u.clone(),
        None => {
            println!("FAF BROWSER v{}", env!("CARGO_PKG_VERSION"));
            println!("Uso: faf <url> [opções]");
            println!("     faf query 'h1' --url https://site.com");
            println!("     faf links --url https://site.com");
            return Ok(());
        }
    };

    // Configurar e fazer request
    let config = crate::utils::config::Config {
        user_agent: cli
            .user_agent
            .clone()
            .unwrap_or_else(|| crate::utils::config::Config::default().user_agent),
        timeout_secs: cli.timeout,
        proxy: cli.proxy.clone(),
        retries: cli.retries,
        retry_delay_ms: cli.retry_delay,
        cookies_path: cli.cookies.clone(),
        cookies_jar_path: cli.cookies_jar.clone(),
        cache_dir: cli.cache.clone(),
        cache_ttl_secs: cli.cache_ttl,
        no_cache: cli.no_cache,
    };

    let client = crate::http::client::HttpClient::new(config)?;
    let resp = client.get(&url).await?;
    let html = resp.body.clone();

    // Exibir status e headers se solicitado
    if cli.show_status {
        println!("📋 Status: {} {}", resp.status, resp.status_text);
    }
    if cli.show_headers {
        for (key, value) in &resp.headers {
            println!("  {}: {}", key, value);
        }
    }

    // Parsear HTML
    let doc = crate::dom::HtmlDocument::parse(&html);

    // Determinar formato efetivo: --format vence sobre --json
    let format = if cli.format != "text" {
        cli.format.clone()
    } else if cli.json {
        "json".to_string()
    } else {
        "text".to_string()
    };

    // Se --js ou --js-file foi passado, executar JS
    let user_js = cli.js_script.as_ref().or(cli.js_file.as_ref());
    if let Some(js_input) = user_js {
        let js_code = if cli.js_file.is_some() {
            // --js-file sempre lê de arquivo
            std::fs::read_to_string(js_input)
                .map_err(|e| anyhow::anyhow!("Falha ao ler arquivo JS '{}': {}", js_input, e))?
        } else {
            // --js é sempre código inline
            js_input.clone()
        };

        let mut rt = crate::js::JsRuntime::with_client(client.clone())?;
        rt.set_dom(&doc)?;
        rt.init_timers()?;
        rt.init_fetch()?;

        // Executar scripts da página (se não --no-scripts)
        if !cli.no_scripts {
            let base_url = url::Url::parse(&url)?;
            rt.execute_page_scripts(&doc, &base_url, &client).await?;
        }

        // Executar JS do usuário
        let result = rt.eval_with_timeout(&js_code, cli.js_timeout)?;

        // Output
        match format.as_str() {
            "json" => {
                let json_result = rt
                    .eval_json_with_timeout(&js_code, cli.js_timeout)
                    .unwrap_or(serde_json::json!({"result": result}));
                println!("{}", serde_json::to_string_pretty(&json_result)?);
            }
            _ => println!("{}", result),
        }
        return Ok(());
    }

    // Se --stdin foi passado, ler expressões JS da entrada padrão
    if cli.stdin {
        let mut rt = crate::js::JsRuntime::with_client(client.clone())?;
        rt.set_dom(&doc)?;
        rt.init_timers()?;
        rt.init_fetch()?;

        if !cli.no_scripts {
            let base_url = url::Url::parse(&url)?;
            rt.execute_page_scripts(&doc, &base_url, &client).await?;
        }

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match format.as_str() {
                "json" => match rt.eval_json_with_timeout(trimmed, cli.js_timeout) {
                    Ok(result) => println!("{}", serde_json::to_string_pretty(&result)?),
                    Err(e) => eprintln!("Erro: {}", e),
                },
                _ => match rt.eval_with_timeout(trimmed, cli.js_timeout) {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Erro: {}", e),
                },
            }
        }
        return Ok(());
    }

    // Executar comando específico ou extração completa
    match &cli.command {
        Some(Command::Links) => {
            let links = doc.links();
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&links)?);
            } else {
                println!("🔗 Links encontrados: {}", links.len());
                for (text, href) in &links {
                    let t = if text.is_empty() { "(sem texto)" } else { text };
                    println!("  • {} → {}", t, href);
                }
            }
        }
        Some(Command::Images) => {
            let images = doc.images();
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&images)?);
            } else {
                println!("🖼️ Imagens encontradas: {}", images.len());
                for (alt, src) in &images {
                    let a = if alt.is_empty() { "(sem alt)" } else { alt };
                    println!("  • {} → {}", a, src);
                }
            }
        }
        Some(Command::Metadata) => {
            let meta = doc.metadata();
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&meta)?);
            } else {
                println!("📋 Metadados:");
                for (key, value) in &meta {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Some(Command::Query { selector }) => {
            let results = doc.query(selector)?;
            let results = if let Some(filters) = &cli.filter {
                let parsed: Vec<crate::api::filter::QueryFilter> = filters
                    .iter()
                    .map(|f| crate::api::filter::QueryFilter::parse(f))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| anyhow::anyhow!("Filtro inválido: {}", e))?;
                crate::api::filter::apply_filters(results, &parsed)
            } else {
                results
            };
            let css_input = if cli.no_page_css || cli.css.is_some() {
                load_css_input(&cli.css)?
            } else {
                let page_css = crate::css::parser::extract_page_stylesheets(
                    doc.scraper_html(),
                    &Url::parse(&url)?,
                    &client,
                )
                .await;
                if page_css.is_empty() {
                    None
                } else {
                    Some(page_css.join("\n"))
                }
            };

            if let Some(css_text) = css_input {
                let stylesheet = crate::css::parser::parse_css(&css_text)?;
                let styles = crate::css::style::compute_styles(&doc, &stylesheet);

                match format.as_str() {
                    "json" => {
                        if cli.get.is_some() {
                            let filtered = crate::api::output::styled_results_to_filtered_json(
                                results, &styles, &cli.get,
                            );
                            println!("{}", serde_json::to_string_pretty(&filtered)?);
                        } else {
                            let output = crate::api::output::styled_query_to_output(
                                selector, results, &styles,
                            );
                            println!("{}", serde_json::to_string_pretty(&output)?);
                        }
                    }
                    "jsonl" => {
                        let items = crate::api::output::to_styled_items(results, &styles);
                        println!("{}", crate::api::output::to_jsonl(&items, &cli.get));
                    }
                    "csv" => {
                        let fields = cli.get.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("--format csv requer --get com lista de campos")
                        })?;
                        let items = crate::api::output::to_styled_items(results, &styles);
                        println!("{}", crate::api::output::to_csv(&items, fields));
                    }
                    _ => {
                        println!("🔍 Query '{}': {} resultado(s)", selector, results.len());
                        for (i, r) in results.iter().enumerate() {
                            println!(
                                "  [{}.] <{}> texto: {}",
                                i + 1,
                                r.tag,
                                truncate(&r.text, 80)
                            );
                            let style = styles
                                .iter()
                                .find(|(em, _)| {
                                    em.tag == r.tag
                                        && em.id == r.id
                                        && em.classes == r.classes
                                        && em.text == r.text
                                })
                                .map(|(_, s)| s);
                            if let Some(s) = style {
                                println!(
                                    "      🎨 color: {} | bg: {} | font-size: {} | font-family: {} | display: {}",
                                    s.color,
                                    s.background_color,
                                    s.font_size,
                                    s.font_family,
                                    s.display
                                );
                            }
                        }
                    }
                }
            } else {
                match format.as_str() {
                    "json" => {
                        let output = crate::api::output::query_to_output(selector, results);
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    }
                    _ => {
                        println!("🔍 Query '{}': {} resultado(s)", selector, results.len());
                        for (i, r) in results.iter().enumerate() {
                            println!(
                                "  [{}.] <{}> texto: {}",
                                i + 1,
                                r.tag,
                                truncate(&r.text, 80)
                            );
                        }
                    }
                }
            }
        }
        Some(Command::Follow(args)) => {
            // 1. Query links on base page
            let link_results = doc.query(&args.selector)?;
            let link_results = if let Some(filters) = &cli.filter {
                let parsed: Vec<crate::api::filter::QueryFilter> = filters
                    .iter()
                    .map(|f| crate::api::filter::QueryFilter::parse(f))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| anyhow::anyhow!("Filtro inválido: {}", e))?;
                crate::api::filter::apply_filters(link_results, &parsed)
            } else {
                link_results
            };

            // 2. Extract hrefs, resolve URLs, filter same-domain
            let base_url = Url::parse(&url)?;
            let mut hrefs: Vec<String> = Vec::new();
            for r in &link_results {
                if let Some(href) = r
                    .attributes
                    .iter()
                    .find(|(k, _)| k == "href")
                    .map(|(_, v)| v)
                {
                    match base_url.join(href) {
                        Ok(abs_url) => {
                            if args.same_domain && abs_url.host_str() != base_url.host_str() {
                                continue;
                            }
                            hrefs.push(abs_url.to_string());
                        }
                        Err(_) => continue,
                    }
                }
            }

            // Deduplicar preservando ordem
            let mut seen = std::collections::HashSet::new();
            hrefs.retain(|h| seen.insert(h.clone()));

            // 3. Limitar por --max
            let max = (args.max as usize).min(hrefs.len());
            hrefs.truncate(max);

            // 4. CSS base (se houver --css ou --no-page-css)
            let css_input_base = if cli.no_page_css || cli.css.is_some() {
                load_css_input(&cli.css)?
            } else {
                None
            };

            // 5. Visitar páginas em batches concorrentes com delay entre batches
            let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(args.concurrency));
            let mut page_results = Vec::with_capacity(hrefs.len());

            for batch in hrefs.chunks(args.concurrency) {
                let mut handles = Vec::with_capacity(batch.len());

                for (idx, href) in batch.iter().cloned().enumerate() {
                    let sem = semaphore.clone();
                    let c = client.clone();
                    let extract = args.extract.clone();
                    let css_base = css_input_base.clone();
                    let cli_css = cli.css.clone();
                    let get_fields = cli.get.clone();
                    let no_page_css = cli.no_page_css;
                    let batch_offset = page_results.len();

                    let handle = tokio::spawn(async move {
                        let _permit = sem.acquire().await.ok();
                        let result = follow_page(
                            c,
                            &href,
                            extract.as_deref(),
                            css_base,
                            &cli_css,
                            no_page_css,
                            &get_fields,
                        )
                        .await;
                        (batch_offset + idx, href, result)
                    });
                    handles.push(handle);
                }

                // Coletar resultados do batch
                for handle in handles {
                    let (idx, href, result) = match handle.await {
                        Ok((idx, href, Ok(res))) => (idx, href, res),
                        Ok((_, href, Err(e))) => {
                            log::warn!("Falha ao visitar {}: {}", href, e);
                            continue;
                        }
                        Err(e) => {
                            log::warn!("Task panicked: {}", e);
                            continue;
                        }
                    };
                    page_results.push((idx, href, result));
                }

                // Aplicar delay entre batches (exceto após o último)
                let delay_ms = if let Some(ref rd) = args.random_delay {
                    if rd.len() == 2 {
                        let min = rd[0];
                        let max = rd[1];
                        rand::thread_rng().gen_range(min..=max)
                    } else {
                        args.delay
                    }
                } else {
                    args.delay
                };

                if delay_ms > 0 && batch.len() == args.concurrency {
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                }
            }

            page_results.sort_by_key(|(idx, _, _)| *idx);

            // 7. Output
            match format.as_str() {
                "json" => {
                    let json_results: Vec<serde_json::Value> = page_results
                        .into_iter()
                        .map(|(_, _, r)| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&json_results)?);
                }
                "jsonl" => {
                    for (_, _, r) in page_results {
                        println!("{}", serde_json::to_string(&r).unwrap_or_default());
                    }
                }
                "csv" => {
                    println!("url,title,text_snippet");
                    for (_, _, r) in page_results {
                        let title = escape_csv_field(&r.title.unwrap_or_default());
                        let snippet = escape_csv_field(&r.text_snippet);
                        println!("{},{},{}", r.url, title, snippet);
                    }
                }
                _ => {
                    println!("🔍 Follow: {} página(s) visitada(s)", page_results.len());
                    for (i, (_, _, r)) in page_results.iter().enumerate() {
                        let title = r.title.as_deref().unwrap_or("(sem título)");
                        println!(
                            "  [{}] {} → {} | {}",
                            i + 1,
                            r.url,
                            title,
                            truncate(&r.text_snippet, 80)
                        );
                    }
                }
            }
        }
        Some(Command::Wait(args)) => {
            let mut rt = crate::js::JsRuntime::with_client(client.clone())?;
            rt.set_dom(&doc)?;
            rt.init_timers()?;
            rt.init_fetch()?;

            if !cli.no_scripts {
                let base_url = url::Url::parse(&url)?;
                rt.execute_page_scripts(&doc, &base_url, &client).await?;
            }

            let max_iterations = args.timeout * 1000 / args.interval;
            for i in 0..=max_iterations {
                match rt.eval_json(&format!("document.querySelector('{}')", args.selector)) {
                    Ok(result) => {
                        if !result.is_null() {
                            if format == "json" {
                                println!("{}", serde_json::to_string_pretty(&result)?);
                            } else {
                                let tag = result.get("tag").and_then(|v| v.as_str()).unwrap_or("?");
                                let id = result.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let text =
                                    result.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                println!(
                                    "✅ Elemento encontrado: <{}> id=\"{}\" texto: {}",
                                    tag, id, text
                                );
                            }
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        // Seletor inválido ou erro JS: falhar imediatamente (retry não ajuda)
                        anyhow::bail!("Erro ao avaliar seletor '{}': {}", args.selector, e);
                    }
                }

                if i < max_iterations {
                    tokio::time::sleep(std::time::Duration::from_millis(args.interval)).await;
                }
            }

            anyhow::bail!(
                "Elemento '{}' não encontrado após {}s",
                args.selector,
                args.timeout
            );
        }
        Some(Command::Click(args)) => {
            let elements = doc.query(&args.selector)?;
            if elements.is_empty() {
                anyhow::bail!("Elemento '{}' não encontrado", args.selector);
            }

            let mut rt = crate::js::JsRuntime::with_client(client.clone())?;
            rt.set_dom(&doc)?;
            rt.init_timers()?;
            rt.init_fetch()?;

            if !cli.no_scripts {
                let base_url = url::Url::parse(&url)?;
                rt.execute_page_scripts(&doc, &base_url, &client).await?;
            }

            let js_code = format!(
                "document.querySelector({}).click()",
                serde_json::to_string(&args.selector)?
            );
            let _click_result = rt.eval_with_timeout(&js_code, cli.js_timeout)?;

            let resp2 = client.get(&url).await?;
            let html2 = resp2.body;
            let doc2 = crate::dom::HtmlDocument::parse(&html2);

            let page_result = crate::api::output::FollowPageResult {
                url: url.clone(),
                title: doc2.title(),
                first_heading: doc2
                    .query("h1")
                    .ok()
                    .and_then(|r| r.into_iter().next())
                    .map(|r| r.text),
                text_snippet: truncate(&doc2.visible_text(), 200),
                extracted: None,
            };

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&page_result)?);
            } else {
                println!("🖱️ Click em '{}': disparado", args.selector);
                if let Some(title) = &page_result.title {
                    println!("📌 Título pós-click: {}", title);
                }
                println!("📝 {}", page_result.text_snippet);
            }
        }
        Some(Command::Watch(args)) => {
            let mut previous_value: Option<String> = None;
            let mut checks: u64 = 0;

            loop {
                if args.max_checks > 0 && checks >= args.max_checks {
                    break;
                }

                let resp = client.get(&url).await?;

                if cli.show_status {
                    println!("📋 Status: {} {}", resp.status, resp.status_text);
                }
                if cli.show_headers {
                    for (key, value) in &resp.headers {
                        println!("  {}: {}", key, value);
                    }
                }

                let html = resp.body;
                let doc = crate::dom::HtmlDocument::parse(&html);

                let current_value = if let Some(ref selector) = args.selector {
                    doc.query(selector)
                        .ok()
                        .and_then(|r| r.into_iter().next())
                        .map(|r| r.text)
                        .unwrap_or_else(|| "(elemento não encontrado)".to_string())
                } else {
                    let title = doc.title().unwrap_or_default();
                    let text = doc.visible_text();
                    let snippet = if text.len() > 100 {
                        format!("{}...", &text[..100])
                    } else {
                        text.clone()
                    };
                    format!("{} | {}", title, snippet)
                };

                let now = chrono::Local::now().format("%H:%M:%S").to_string();

                if let Some(ref prev) = previous_value {
                    if *prev != current_value {
                        if format == "json" {
                            let json = serde_json::json!({
                                "time": now,
                                "selector": args.selector.as_deref().unwrap_or("*"),
                                "previous": prev,
                                "current": current_value,
                                "changed": true,
                            });
                            println!("{}", serde_json::to_string(&json)?);
                        } else {
                            println!("[{}] ⚠️ MUDOU! (antes: {}) → {}", now, prev, current_value);
                        }
                    } else {
                        if format == "json" {
                            let json = serde_json::json!({
                                "time": now,
                                "value": current_value,
                                "changed": false,
                            });
                            println!("{}", serde_json::to_string(&json)?);
                        } else {
                            println!("[{}] {}", now, current_value);
                        }
                    }
                } else {
                    if format == "json" {
                        let json = serde_json::json!({
                            "time": now,
                            "value": current_value,
                            "changed": false,
                            "first": true,
                        });
                        println!("{}", serde_json::to_string(&json)?);
                    } else {
                        println!("[{}] {}", now, current_value);
                    }
                }

                previous_value = Some(current_value);
                checks += 1;

                if args.max_checks == 0 || checks < args.max_checks {
                    tokio::time::sleep(std::time::Duration::from_secs(args.interval)).await;
                }
            }
        }
        Some(Command::Dump(args)) => {
            let config = crate::dump::DumpConfig {
                inline_images: args.inline_images,
                inline_css: !args.no_inline_css,
                remove_scripts: args.no_scripts,
                base_url: url.clone(),
                format: args.format.clone(),
                readability: args.readability,
                structured_data: args.structured_data,
                frontmatter: args.frontmatter,
                chunk_size: args.chunk_size,
            };

            let result_html = crate::dump::dump_to_string(&html, &config)?;

            if let Some(ref output_path) = args.output {
                if config.chunk_size > 0 && (config.format == "markdown" || config.format == "md") {
                    crate::dump::write_chunked_output(&result_html, output_path)?;
                    if format == "json" {
                        println!(
                            "{}",
                            serde_json::json!({"dump": output_path, "url": url})
                        );
                    } else {
                        println!("💾 Chunks salvos em: {}", output_path);
                    }
                } else {
                    crate::dump::write_to_file(&result_html, output_path)?;
                    if format == "json" {
                        println!(
                            "{}",
                            serde_json::json!({"dump": output_path, "url": url})
                        );
                    } else {
                        println!("💾 HTML salvo em: {}", output_path);
                    }
                }
            } else {
                use std::io::Write;
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(result_html.as_bytes())?;
                handle.flush()?;
            }
        }
        Some(Command::Repl(_args)) => {
            let mut rt = crate::js::JsRuntime::with_client(client.clone())?;
            rt.set_dom(&doc)?;
            rt.init_timers()?;
            rt.init_fetch()?;

            if !cli.no_scripts {
                let base_url = url::Url::parse(&url)?;
                rt.execute_page_scripts(&doc, &base_url, &client).await?;
            }

            println!("FAF REPL — Digite .help para ajuda, .exit para sair.");
            let mut json_mode = format == "json";

            loop {
                print!("> ");
                io::stdout().flush()?;
                let mut line = String::new();
                if io::stdin().read_line(&mut line)? == 0 {
                    break;
                }
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match trimmed {
                    ".exit" => break,
                    ".help" => {
                        println!("Comandos REPL:");
                        println!("  .exit  — Sair do REPL");
                        println!("  .help  — Mostrar esta ajuda");
                        println!("  .json  — Alternar modo JSON");
                        println!("  .clear — Limpar tela (placeholder)");
                    }
                    ".json" => {
                        json_mode = !json_mode;
                        println!(
                            "Modo JSON: {}",
                            if json_mode { "ligado" } else { "desligado" }
                        );
                    }
                    ".clear" => {
                        println!("clear");
                    }
                    _ => {
                        if json_mode {
                            match rt.eval_json_with_timeout(trimmed, cli.js_timeout) {
                                Ok(result) => {
                                    println!("{}", serde_json::to_string_pretty(&result)?)
                                }
                                Err(e) => eprintln!("Erro: {}", e),
                            }
                        } else {
                            match rt.eval_with_timeout(trimmed, cli.js_timeout) {
                                Ok(result) => println!("{}", result),
                                Err(e) => eprintln!("Erro: {}", e),
                            }
                        }
                    }
                }
            }
        }
        None => {
            // Extração completa da página
            let output = crate::api::output::extract_page(&url, &doc);
            if format == "json" {
                let mut json_output = serde_json::to_value(&output)?;
                if let Some(map) = json_output.as_object_mut() {
                    if cli.show_status {
                        map.insert("status".to_string(), serde_json::json!(resp.status));
                    }
                    if cli.show_headers {
                        let headers: serde_json::Map<String, serde_json::Value> = resp
                            .headers
                            .iter()
                            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                            .collect();
                        map.insert("headers".to_string(), serde_json::Value::Object(headers));
                    }
                }
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                println!("📄 Página: {}", url);
                println!(
                    "📌 Título: {}",
                    output.title.as_deref().unwrap_or("(sem título)")
                );
                println!("🔗 Links: {}", output.links.len());
                println!("🖼️ Imagens: {}", output.images.len());
                println!("📋 Metadados: {}", output.metadata.len());
                println!("\n📝 Texto:");
                println!("{}", truncate(&output.text, 500));
            }
        }
    }

    Ok(())
}

/// Carrega CSS a partir de string inline ou arquivo.
/// Se o texto contém '{', trata como CSS inline; caso contrário, tenta ler como arquivo.
fn load_css_input(css_opt: &Option<String>) -> anyhow::Result<Option<String>> {
    let css_str = match css_opt {
        Some(s) => s,
        None => return Ok(None),
    };

    if css_str.contains('{') {
        Ok(Some(css_str.clone()))
    } else {
        match std::fs::read_to_string(css_str) {
            Ok(content) => Ok(Some(content)),
            Err(e) => Err(anyhow::anyhow!(
                "Falha ao ler arquivo CSS '{}': {}",
                css_str,
                e
            )),
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}

/// Visita uma página seguida e extrai dados.
async fn follow_page(
    client: crate::http::client::HttpClient,
    url: &str,
    extract: Option<&str>,
    css_input_base: Option<String>,
    cli_css: &Option<String>,
    no_page_css: bool,
    get_fields: &Option<Vec<String>>,
) -> anyhow::Result<crate::api::output::FollowPageResult> {
    let resp = client.get(url).await?;
    let html = resp.body;
    let doc = crate::dom::HtmlDocument::parse(&html);

    let title = doc.title();
    let first_heading = doc
        .query("h1")
        .ok()
        .and_then(|r| r.into_iter().next())
        .map(|r| r.text);
    let text = doc.visible_text();
    let text_snippet = truncate(&text, 200);

    let extracted = if let Some(selector) = extract {
        let results = doc.query(selector)?;

        let css_input = if no_page_css || cli_css.is_some() {
            css_input_base
        } else {
            // Não extraímos CSS automaticamente de cada página seguida
            // para evitar problemas de Send com scraper::Html em tasks spawnadas.
            None
        };

        if let Some(css_text) = css_input {
            let stylesheet = crate::css::parser::parse_css(&css_text)?;
            let styles = crate::css::style::compute_styles(&doc, &stylesheet);
            let items = crate::api::output::to_styled_items(results, &styles);
            Some(
                items
                    .into_iter()
                    .map(|item| crate::api::output::filter_fields(get_fields, &item))
                    .collect(),
            )
        } else {
            let output = crate::api::output::query_to_output(selector, results);
            Some(
                output
                    .results
                    .into_iter()
                    .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
                    .collect(),
            )
        }
    } else {
        None
    };

    Ok(crate::api::output::FollowPageResult {
        url: url.to_string(),
        title,
        first_heading,
        text_snippet,
        extracted,
    })
}

fn escape_csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
