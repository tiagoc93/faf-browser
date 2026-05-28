use clap::Parser;
use faf_browser::api::commands::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    faf_browser::api::commands::run(cli).await
}
