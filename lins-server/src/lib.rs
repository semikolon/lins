//! The Lins server as a library: the API and the page behind one router.
//!
//! The `lins-server` binary and the desktop app both run this, so there is one
//! API and one page whichever way Lins is opened.

pub mod api;

use anyhow::{Context, Result};
use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use lins_core::config::load_config;
use lins_core::schema::{discover_schema, GraphSchema};
use lins_core::{FalkorConnection, LinsConfig, VocabularyIndex};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

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

/// Is the page carried inside this binary? False when it was compiled before the page was built.
pub fn carries_the_page() -> bool {
    WebAssets::get("index.html").is_some()
}

/// Shared application state accessible by all API handlers.
#[derive(Clone)]
pub struct AppState {
    pub conn: FalkorConnection,
    pub config: Arc<RwLock<LinsConfig>>,
    pub schemas: Arc<RwLock<std::collections::HashMap<String, GraphSchema>>>,
    pub vocab_indexes: Arc<RwLock<std::collections::HashMap<String, VocabularyIndex>>>,
}

/// The config in the file at `path`. Where there is no such file: the defaults,
/// with the connection taken from the environment.
pub fn config_at(path: &Path) -> Result<LinsConfig> {
    if path.exists() {
        load_config(path).with_context(|| format!("Failed to load config from {}", path.display()))
    } else {
        warn!("Config file '{}' not found, using defaults", path.display());
        Ok(config_from_env())
    }
}

/// The defaults, with the connection taken from the environment.
pub fn config_from_env() -> LinsConfig {
    let mut config = LinsConfig::default();
    config.connection.apply_env(|name| std::env::var(name).ok());
    config
}

/// The address of the page, on the graph asked for if one was. The page reads the graph
/// from the address (`?graph=`). The name is written so that it cannot add to the address.
pub fn page_address(host: &str, port: u16, graph: Option<&str>) -> String {
    let mut address = format!("http://{host}:{port}/");
    if let Some(graph) = graph {
        let encoded: String = graph
            .bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                other => format!("%{other:02X}"),
            })
            .collect();
        address.push_str("?graph=");
        address.push_str(&encoded);
    }
    address
}

/// A server that holds its address and is not yet answering.
pub struct Bound {
    pub addr: SocketAddr,
    listener: tokio::net::TcpListener,
    app: Router,
}

impl Bound {
    /// Answer requests until the program ends.
    pub async fn run(self) -> Result<()> {
        axum::serve(self.listener, self.app).await?;
        Ok(())
    }
}

/// Connect to FalkorDB and take an address to answer on. Port 0 takes any free port.
pub async fn bind(config: LinsConfig, host: &str, port: u16) -> Result<Bound> {
    info!("Connecting to FalkorDB at {}:{}", config.connection.host, config.connection.port);
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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api::router())
        .fallback(web_app)
        .layer(cors)
        .with_state(state);

    let wanted = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&wanted)
        .await
        .with_context(|| format!("Failed to bind to {wanted}"))?;
    let addr = listener.local_addr().context("The listener has no address")?;
    Ok(Bound { addr, listener, app })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_is_at_the_root_of_the_address() {
        assert_eq!(page_address("127.0.0.1", 49152, None), "http://127.0.0.1:49152/");
    }

    #[test]
    fn a_graph_asked_for_is_named_in_the_address_and_cannot_add_to_it() {
        assert_eq!(page_address("localhost", 8080, Some("my_graph-2")), "http://localhost:8080/?graph=my_graph-2");
        assert_eq!(page_address("localhost", 8080, Some("a b&x=1")), "http://localhost:8080/?graph=a%20b%26x%3D1");
    }
}
