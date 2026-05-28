use clap::Parser;
use faf_browser::api::commands::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Configura logging: --verbose ativa info level
    if cli.verbose || std::env::var("RUST_LOG").is_ok() {
        env_logger::Builder::from_env(
            env_logger::Env::default()
                .default_filter_or(if cli.verbose { "info" } else { "warn" }),
        )
        .init();
    } else {
        env_logger::init();
    }

    faf_browser::api::commands::run(cli).await
}
