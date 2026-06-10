mod dto;
mod handlers;
use crate::cli::DEFAULT_STACK;
use crate::config::{ParsedConfig, parse_config};
use crate::docker::pull_config;
use anyhow::{Context, Result};

use axum::{
    Router,
    routing::{get, post},
};
use clap::Args;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use crate::commands::ui::handlers::{api_generate, api_services, index};

#[derive(Clone)]
pub struct AppState {
    config: Arc<RwLock<ParsedConfig>>,
    config_file: String,
    docs: Arc<crate::docs::PlatysIndex>,
}


#[derive(Args, Debug)]
pub struct UiArgs {
    ///port to bind to (0 = random available port)
    #[arg(short = 'p', long = "port", default_value_t = 0)]
    pub port: u16,
    ///Don't open the browser automatically but print the url by default
    #[arg(long = "no-browser")]
    pub no_browser: bool,

    /// Stack image to pull services from
    #[arg(short = 's', long = "stack", default_value = DEFAULT_STACK)]
    pub stack: String,

    /// Version of the stack
    #[arg(short = 'w', long = "stack-version", default_value = "latest")]
    pub stack_version: String,

    /// Config file to write when the user clicks Generate
    #[arg(short = 'c', long = "config-file", default_value = "config.yml")]
    pub config_file: String,

    /// Local folder containing services.yml and index.yml (dev override;
    /// falls back to no docs if omitted)
    #[arg(long = "docs-path")]
    pub docs_path: Option<String>,
}

pub async fn run(args: UiArgs) -> Result<()> {
    //pull seed from image
    println!(
        "Pulling seed config from {} : {}",
        args.stack, args.stack_version
    );
    let raw = pull_config(&args.stack, &args.stack_version)
        .await
        .context("Failed to pull seed config from Docker immage")?;
    let cfg = parse_config(&raw).context("Failed to parse config file")?;

    println!("loaded services : {}", cfg.services.len());

    let docs = match &args.docs_path {
        Some(path) => crate::docs::read_local(path).context("Failed to read docs from --docs-path")?,
        None => crate::docs::PlatysIndex::empty()
    };

    //Build Shared state of the app
    let state = AppState {
        config: Arc::new(RwLock::new(cfg)),
        config_file: args.config_file.clone(),
        docs: Arc::new(docs)
    };

    //Build routes
    let app = Router::new()
        .route("/", get(index))
        .route("/api/services", get(api_services))
        .route("/api/generate", post(api_generate))
        .with_state(state);

    // Create server and bind
    let address = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("Could not bind to {}", address))?;

    let actual_address = listener
        .local_addr()
        .context("Failed to read local address")?;
    let url = format!("http://{actual_address}");

    log::info!("Platys listening on url : {}", url);

    if args.no_browser {
        println!("Open your browser on url {}", url);
    } else if let Err(e) = webbrowser::open(&url) {
        log::warn!("Couldn't open browser: {}", e);
        println!("Open your browser on url {}", url);
    }

    println!("Press Ctrl-C to sto server");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}


