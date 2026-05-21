mod cli;
mod docker;
mod config;
mod commands;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(if cli.verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .format_timestamp(None)   // skip timestamps for CLI tool feel
        .init();

    if let Err(e) = cli.run().await {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}
