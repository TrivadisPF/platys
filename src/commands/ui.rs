use anyhow::{Context, Result};
use axum::{response::Html, routing::get, Router};
use clap::Args;
use std::net::SocketAddr;
use tokio::net::TcpListener;


#[derive(Args, Debug)]
pub struct UiArgs {
    //port to bind to (0 = random available port)
    #[arg(short = 'p', long = "port", default_value_t = 0)]
    pub port: u16,
    //Don't open the browser automatically but print the url by default
    #[arg(long = "no-browser",help = "Do not open automatically the browser when starting the UI")]
    pub no_browser: bool,
}

pub async fn run(args: UiArgs) -> Result<()> {
    let app = Router::new().route("/", get(index));
    let address = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(address).await
        .with_context(|| format!("Could not bind to {}", address))?;

    let actual_address = listener.local_addr().context("Failed to read local address")?;
    let url = format!("http://{actual_address}");

    log::info!("Platys listening on url : {}", url);

    if args.no_browser {
        println!("Open your browser on url {}", url);
    }else if let Err(e) = webbrowser::open(&url) {
        log::warn!("Couldn't open browser: {}", e);
        println!("Open your browser on url {}", url);
    }

    println!("Press Ctrl-C to sto server");

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
  <html>
  <head>
      <title>platys UI</title>
      <style>
          body { font-family: system-ui, sans-serif; padding: 2em; }
          h1 { color: #c8102e; }
      </style>
  </head>
  <body>
      <h1>platys UI</h1>
      <p>The server is running. The real UI will live here in later phases.</p>
  </body>
  </html>"#,
    )
}


