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
use crate::commands::ui::handlers::{api_generate, api_preview, api_services, asset, index};

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

    /// (dev builds only) Read services.yml/index.yml from a local folder
    #[cfg(debug_assertions)]
    #[arg(long = "docs-path")]
    pub docs_path: Option<String>,

    /// (dev builds only) Load seed config from disk instead of pulling the image
    #[cfg(debug_assertions)]
    #[arg(long = "seed-file")]
    pub seed_file: Option<String>,
}

impl UiArgs {
    fn docs_path(&self) -> Option<&str> {
        #[cfg(debug_assertions)]
        {
            self.docs_path.as_deref()
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }

    fn seed_file(&self) -> Option<&str> {
        #[cfg(debug_assertions)]
        {
            self.seed_file.as_deref()
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}

pub async fn run(args: UiArgs) -> Result<()> {
    // Seed config: from disk in dev (--seed-file), else pulled from the image.
    let raw = match args.seed_file() {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read seed file {path}"))?,
        None => {
            println!(
                "Pulling seed config from {} : {}",
                args.stack, args.stack_version
            );
            pull_config(&args.stack, &args.stack_version)
                .await
                .context("Failed to pull seed config from Docker image")?
        }
    };
    let cfg = parse_config(&raw).context("Failed to parse config file")?;

    println!("loaded services : {}", cfg.services.len());

    let docs = match args.docs_path() {
        Some(path) => crate::docs::read_local(path).context("Failed to read docs from --docs-path")?,
        None => {
            let built = async {
                let (services_raw, index_raw)=
                crate::docker::pull_docs(&args.stack, &args.stack_version).await?;
                crate::docs::PlatysIndex::build(&services_raw, &index_raw)
            }.await;

            built.unwrap_or_else(|e| {
                log::warn!("Could not load docs from image, continuing without: {e:#}");
                crate::docs::PlatysIndex::empty()
            })
        }
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
        .route("/assets/{file}", get(asset))
        .route("/api/services", get(api_services))
        .route("/api/generate", post(api_generate))
        .route("/api/preview", post(api_preview))
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


