mod api;

use anyhow::{Context, Result};
use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use clap::Parser;
use lins_core::{FalkorConnection, LinsConfig, VocabularyIndex};
use lins_core::config::load_config;
use lins_core::schema::{discover_schema, GraphSchema};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

/// Lins — a lightweight FalkorDB graph explorer.
#[derive(Parser, Debug)]
#[command(name = "lins", version, about)]
enum Cli {
    /// Start the Lins web server.
    Serve {
        /// Port to listen on (overrides config).
        #[arg(short, long)]
        port: Option<u16>,

        /// Path to lins.toml config file.
        #[arg(short, long, default_value = "lins.toml")]
        config: PathBuf,

        /// Address to listen on (overrides config). 127.0.0.1 keeps it to this machine.
        #[arg(long)]
        host: Option<String>,

        /// Open the page in the default browser once the server is up.
        #[arg(long)]
        open: bool,
    },
}

/// The built web app, carried inside the binary. Empty when `lins-web/build`
/// did not exist at compile time; the server then says how to build it.
#[derive(rust_embed::RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../lins-web/build"]
#[allow_missing = true]
struct WebAssets;

/// Serve the web app: the file asked for, or the page itself for any other path.
async fn web_app(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let wanted = if path.is_empty() { "index.html" } else { path };
    let (name, file) = match WebAssets::get(wanted) {
        Some(file) => (wanted, Some(file)),
        None => ("index.html", WebAssets::get("index.html")),
    };
    match file {
        Some(file) => {
            let mime = mime_guess::from_path(name).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref().to_string())], file.data.into_owned()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            "The Lins web app is not built into this binary.\n\
             Build it and the server again:\n\
             \x20 (cd lins-web && npm install && npm run build) && cargo build --release -p lins-server\n\
             Or run the page in development: (cd lins-web && npm run dev)\n",
        )
            .into_response(),
    }
}

/// Shared application state accessible by all API handlers.
#[derive(Clone)]
pub struct AppState {
    pub conn: FalkorConnection,
    pub config: Arc<RwLock<LinsConfig>>,
    pub schemas: Arc<RwLock<std::collections::HashMap<String, GraphSchema>>>,
    pub vocab_indexes: Arc<RwLock<std::collections::HashMap<String, VocabularyIndex>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lins_server=info,lins_core=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli {
        Cli::Serve { port, config: config_path, host, open } => {
            run_server(port, config_path, host, open).await?;
        }
    }

    Ok(())
}

async fn run_server(
    port_override: Option<u16>,
    config_path: PathBuf,
    host_override: Option<String>,
    open: bool,
) -> Result<()> {
    // Load config
    let config = if config_path.exists() {
        load_config(&config_path)
            .with_context(|| format!("Failed to load config from {}", config_path.display()))?
    } else {
        warn!(
            "Config file '{}' not found, using defaults",
            config_path.display()
        );
        let mut config = LinsConfig::default();
        config.connection.apply_env(|name| std::env::var(name).ok());
        config
    };

    let listen_port = port_override.unwrap_or(config.server.port);
    let listen_host = host_override.unwrap_or_else(|| config.server.host.clone());

    // Connect to FalkorDB
    info!(
        "Connecting to FalkorDB at {}:{}",
        config.connection.host, config.connection.port
    );
    let conn = FalkorConnection::connect(&config.connection)
        .await
        .context("Failed to connect to FalkorDB")?;
    info!("Connected to FalkorDB");

    // Discover schema for default graph if configured
    let mut schemas = std::collections::HashMap::new();
    let mut vocab_indexes = std::collections::HashMap::new();

    if let Some(ref default_graph) = config.connection.default_graph {
        info!("Discovering schema for default graph '{}'", default_graph);
        match discover_schema(&conn, default_graph).await {
            Ok(schema) => {
                info!(
                    "Schema discovered: {} labels, {} relationship types, {} nodes, {} edges",
                    schema.labels.len(),
                    schema.relationship_types.len(),
                    schema.node_count,
                    schema.edge_count,
                );
                let vocab = VocabularyIndex::build(&schema);
                vocab_indexes.insert(default_graph.clone(), vocab);
                schemas.insert(default_graph.clone(), schema);
            }
            Err(e) => {
                warn!("Failed to discover schema for '{}': {}", default_graph, e);
            }
        }
    }

    let state = AppState {
        conn,
        config: Arc::new(RwLock::new(config)),
        schemas: Arc::new(RwLock::new(schemas)),
        vocab_indexes: Arc::new(RwLock::new(vocab_indexes)),
    };

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api::router())
        .fallback(web_app)
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", listen_host, listen_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {}", addr))?;

    info!("Lins server listening on http://{}", addr);
    if open {
        let shown = if listen_host == "0.0.0.0" { "localhost" } else { listen_host.as_str() };
        open_in_browser(&format!("http://{shown}:{listen_port}"));
    }
    axum::serve(listener, app).await?;

    Ok(())
}

/// Open a page in the default browser. Failing to is not an error: the address is in the log.
fn open_in_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    if let Err(e) = std::process::Command::new(opener).arg(url).spawn() {
        warn!("Could not open {url} with {opener}: {e}");
    }
}
