use clap::Parser;

/// FAF BROWSER — Fast As Fuck. Navegador headless 100% Rust.
#[derive(Parser, Debug)]
#[command(name = "faf", version, about)]
pub struct Cli {
    /// URL para buscar
    pub url: Option<String>,

    /// Arquivo de script JavaScript
    #[arg(short = 's', long = "js")]
    pub js_script: Option<String>,

    /// Seletor CSS para query
    #[arg(short = 'q', long = "query")]
    pub query: Option<String>,

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
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Comando: links, images, metadata, query
    #[command(subcommand)]
    pub command: Option<Command>,
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
}

/// Executa o comando CLI
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    // Se não tem URL, mostra ajuda
    let url = match &cli.url {
        Some(u) => u.clone(),
        None => {
            println!("FAF BROWSER v{}", env!("CARGO_PKG_VERSION"));
            println!("Uso: faf <url> [opções]");
            println!("     faf --query 'h1' --url https://site.com");
            println!("     faf links --url https://site.com");
            return Ok(());
        }
    };

    // Configurar e fazer request
    let config = crate::utils::config::Config {
        user_agent: cli.user_agent.clone().unwrap_or_else(|| crate::utils::config::Config::default().user_agent),
        timeout_secs: cli.timeout,
        proxy: cli.proxy.clone(),
    };

    let client = crate::http::client::HttpClient::new(config)?;
    let html = client.get(&url).await?;

    // Parsear HTML
    let doc = crate::dom::HtmlDocument::parse(&html);

    // Executar comando específico ou extração completa
    match &cli.command {
        Some(Command::Links) => {
            let links = doc.links();
            if cli.json {
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
            if cli.json {
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
            if cli.json {
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
            if cli.json {
                let output = crate::api::output::query_to_output(selector, results);
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("🔍 Query '{}': {} resultado(s)", selector, results.len());
                for (i, r) in results.iter().enumerate() {
                    println!("  [{}.] <{}> texto: {}", i + 1, r.tag, truncate(&r.text, 80));
                }
            }
        }
        None => {
            // Extração completa da página
            let output = crate::api::output::extract_page(&url, &doc);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("📄 Página: {}", url);
                println!("📌 Título: {}", output.title.as_deref().unwrap_or("(sem título)"));
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

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}
