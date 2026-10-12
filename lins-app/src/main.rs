//! Lins as a desktop app: one program that holds the server and shows its page in a window.
//!
//! The page and the API are the same ones the `lins-server` binary serves. Here they are
//! served to this program's own window only, on a free port of this machine, and they stop
//! when the app quits.

// No console window beside the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tracing::{error, info};

/// How long the database may take to answer before the window says it did not.
const PATIENCE: Duration = Duration::from_secs(10);

/// Lins — a FalkorDB graph explorer, as a desktop app.
#[derive(Parser, Debug)]
#[command(name = "lins-app", version, about)]
struct Args {
    /// Path to a lins.toml. Without it: $LINS_CONFIG, then ~/.config/lins/lins.toml,
    /// then the defaults with the connection taken from the environment.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// The graph to open on. Without it: the config's default graph.
    #[arg(short, long)]
    graph: Option<String>,
}

/// The settings file to read, in the order it is looked for, and a sentence saying which was used.
fn settings(args: &Args) -> (Option<PathBuf>, String) {
    if let Some(path) = &args.config {
        return (Some(path.clone()), format!("Settings: {} (given with --config).", path.display()));
    }
    if let Some(path) = std::env::var_os("LINS_CONFIG").map(PathBuf::from) {
        return (Some(path.clone()), format!("Settings: {} (from LINS_CONFIG).", path.display()));
    }
    let home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    if let Some(path) = home.map(|dir| dir.join("lins").join("lins.toml")) {
        if path.exists() {
            return (Some(path.clone()), format!("Settings: {}.", path.display()));
        }
        return (
            None,
            format!(
                "Settings: no file at {}, so the defaults were used. Give one with --config, or put one there.",
                path.display()
            ),
        );
    }
    (None, "Settings: the defaults, since no settings file was found.".to_string())
}

/// Connect, take a free port on this machine, and start answering. The port is returned.
async fn start(config: Option<PathBuf>) -> Result<u16> {
    let config = match config {
        Some(path) => lins_server::config_at(&path)?,
        None => lins_server::config_from_env(),
    };
    let at = format!("{}:{}", config.connection.host, config.connection.port);
    let bound = tokio::time::timeout(PATIENCE, lins_server::bind(config, "127.0.0.1", 0))
        .await
        .map_err(|_| anyhow!("The database at {at} did not answer within {} seconds.", PATIENCE.as_secs()))?
        .with_context(|| format!("Could not open the database at {at}"))?;
    let port = bound.addr.port();
    info!("Lins is answering its own window on http://127.0.0.1:{port}");
    tauri::async_runtime::spawn(async move {
        if let Err(e) = bound.run().await {
            error!("The server stopped: {e:#}");
        }
    });
    Ok(port)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lins_app=info,lins_server=info,lins_core=info".into()),
        )
        .init();

    let args = Args::parse();
    let (config, settings_note) = settings(&args);

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            let started = if lins_server::carries_the_page() {
                tauri::async_runtime::block_on(start(config.clone()))
            } else {
                Err(anyhow!("This build does not carry the page. Build lins-web first, then the app again."))
            };
            let window = match started {
                Ok(port) => {
                    let address = lins_server::page_address("127.0.0.1", port, args.graph.as_deref());
                    WebviewWindowBuilder::new(app, "main", WebviewUrl::External(address.parse()?))
                        // The window shows this program's own page and nothing else.
                        .on_navigation(move |url| url.host_str() == Some("127.0.0.1") && url.port() == Some(port))
                }
                Err(e) => {
                    error!("Lins could not start: {e:#}");
                    let failure = serde_json::json!({ "why": format!("{e:#}"), "settings": settings_note });
                    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                        .initialization_script(format!("window.__LINS_START_FAILURE__ = {failure};"))
                }
            };
            window
                .title("Lins")
                .inner_size(1440.0, 900.0)
                .min_inner_size(720.0, 480.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Lins could not open its window");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_config_given_on_the_command_line_wins() {
        let args = Args { config: Some(PathBuf::from("/tmp/x/lins.toml")), graph: None };
        let (path, note) = settings(&args);
        assert_eq!(path, Some(PathBuf::from("/tmp/x/lins.toml")));
        assert!(note.contains("--config"));
    }
}
