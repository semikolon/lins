use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
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

        /// The graph `--open` opens on. Without it: the config's default graph.
        #[arg(short, long)]
        graph: Option<String>,
    },
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
        Cli::Serve { port, config: config_path, host, open, graph } => {
            run_server(port, config_path, host, open, graph).await?;
        }
    }

    Ok(())
}

async fn run_server(
    port_override: Option<u16>,
    config_path: PathBuf,
    host_override: Option<String>,
    open: bool,
    graph: Option<String>,
) -> Result<()> {
    let config = lins_server::config_at(&config_path)?;
    let listen_port = port_override.unwrap_or(config.server.port);
    let listen_host = host_override.unwrap_or_else(|| config.server.host.clone());

    let bound = lins_server::bind(config, &listen_host, listen_port).await?;

    info!("Lins server listening on http://{}", bound.addr);
    if open {
        let shown = if listen_host == "0.0.0.0" { "localhost" } else { listen_host.as_str() };
        open_in_browser(&lins_server::page_address(shown, bound.addr.port(), graph.as_deref()));
    }
    bound.run().await
}

/// Open a page in the default browser. Failing to is not an error: the address is in the log.
fn open_in_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    if let Err(e) = std::process::Command::new(opener).arg(url).spawn() {
        warn!("Could not open {url} with {opener}: {e}");
    }
}
