mod api;

use anyhow::{Context, Result};
use axum::Router;
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
    },
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
        Cli::Serve { port, config: config_path } => {
            run_server(port, config_path).await?;
        }
    }

    Ok(())
}

async fn run_server(port_override: Option<u16>, config_path: PathBuf) -> Result<()> {
    // Load config
    let config = if config_path.exists() {
        load_config(&config_path)
            .with_context(|| format!("Failed to load config from {}", config_path.display()))?
    } else {
        warn!(
            "Config file '{}' not found, using defaults",
            config_path.display()
        );
        LinsConfig::default()
    };

    let listen_port = port_override.unwrap_or(config.server.port);
    let listen_host = config.server.host.clone();

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
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", listen_host, listen_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {}", addr))?;

    info!("Lins server listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
