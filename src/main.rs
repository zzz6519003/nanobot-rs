use anyhow::Result;
use clap::Parser;

use nanobot_rs::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    nanobot_rs::observability::init();
    let cli = Cli::parse();
    nanobot_rs::cli::run(cli).await
}
