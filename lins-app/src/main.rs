//! Lins as a desktop app: one program that holds the server and shows its page in a window.
//!
//! The page and the API are the same ones the `lins-server` binary serves. Here they are
//! served to this program's own window only, on a free port of this machine, and they stop
//! when the app quits.
//!
//! Opened from the dock the app is given nothing, so it remembers what a command last started
//! it with (`remembered.rs`).

// No console window beside the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod remembered;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use lins_core::LinsConfig;
use remembered::{account, keep_password, Keychain, LastStart, Passwords};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::{error, info, warn};

/// How long the database may take to answer before the window says it did not.
const PATIENCE: Duration = Duration::from_secs(10);

/// Lins — a FalkorDB graph explorer, as a desktop app.
#[derive(Parser, Debug)]
#[command(name = "lins-app", version, about)]
struct Args {
    /// Path to a lins.toml. Without it: $LINS_CONFIG, then ~/.config/lins/lins.toml, then what
    /// the app was last started with, then the defaults with the connection taken from the
    /// environment.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// The graph to open on. Without it: the config's default graph.
    #[arg(short, long)]
    graph: Option<String>,
}

/// Where the settings come from, in the order they are looked for.
#[derive(Debug, Clone, PartialEq)]
enum Source {
    /// A file named with `--config`.
    Given(PathBuf),
    /// A file named by `LINS_CONFIG`.
    FromEnv(PathBuf),
    /// The person's own file, `~/.config/lins/lins.toml`.
    UserFile(PathBuf),
    /// What the app was last started with, since nothing above was there.
    Remembered(LastStart),
    /// Nothing at all: the defaults. Holds the person's own file that was looked for.
    Defaults(Option<PathBuf>),
}

/// Which source to read. `user_file` is the person's own file's path; it counts only if it exists.
fn source(
    given: Option<&Path>,
    lins_config: Option<PathBuf>,
    user_file: Option<PathBuf>,
    remembered: Option<LastStart>,
) -> Source {
    if let Some(path) = given {
        return Source::Given(path.to_path_buf());
    }
    if let Some(path) = lins_config {
        return Source::FromEnv(path);
    }
    if let Some(path) = user_file.as_ref().filter(|path| path.exists()) {
        return Source::UserFile(path.clone());
    }
    if let Some(start) = remembered {
        return Source::Remembered(start);
    }
    Source::Defaults(user_file)
}

/// The person's own settings file: `$XDG_CONFIG_HOME/lins/lins.toml`, else `~/.config/lins/lins.toml`.
fn user_file() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|dir| dir.join("lins").join("lins.toml"))
}

/// When a start was kept, as a date a person reads, in this machine's time.
fn when(at: i64) -> String {
    chrono::DateTime::from_timestamp(at, 0)
        .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "at an unknown time".to_string())
}

/// The settings to start with, and a sentence saying where they came from. `env` is how the
/// environment is read, so that the tests can give one.
fn settings(source: &Source, env: &dyn Fn(&str) -> Option<String>) -> Result<(LinsConfig, String)> {
    let from_file = |path: &Path| -> Result<LinsConfig> {
        let mut config = if path.exists() {
            lins_core::config::load_config(path)
                .with_context(|| format!("Failed to load config from {}", path.display()))?
        } else {
            LinsConfig::default()
        };
        config.connection.apply_env(env);
        Ok(config)
    };
    let defaults = || {
        let mut config = LinsConfig::default();
        config.connection.apply_env(env);
        config
    };
    Ok(match source {
        Source::Given(path) => (from_file(path)?, format!("Settings: {} (given with --config).", path.display())),
        Source::FromEnv(path) => (from_file(path)?, format!("Settings: {} (from LINS_CONFIG).", path.display())),
        Source::UserFile(path) => (from_file(path)?, format!("Settings: {}.", path.display())),
        Source::Remembered(start) => {
            let (mut config, file) = match &start.config {
                Some(path) if path.exists() => (from_file(path)?, path.display().to_string()),
                Some(path) => (defaults(), format!("{} (no longer there, so the defaults)", path.display())),
                None => (defaults(), "the defaults".to_string()),
            };
            // The database it reached, unless the environment names another: what is given
            // now wins over what was kept.
            config.connection.host = start.host.clone();
            config.connection.port = start.port;
            config.connection.apply_env(env);
            let note = format!(
                "Settings: as Lins was last started, {}: {}, with the database at {}:{}.",
                when(start.at),
                file,
                start.host,
                start.port
            );
            (config, note)
        }
        Source::Defaults(Some(looked_for)) => (
            defaults(),
            format!(
                "Settings: no file at {}, and Lins has not been started with any settings before, so the \
                 defaults were used. Give a file with --config, or put one there.",
                looked_for.display()
            ),
        ),
        Source::Defaults(None) => (defaults(), "Settings: the defaults, since no settings file was found.".to_string()),
    })
}

/// Fill in a password the settings do not hold from the keychain. Says whether it did.
fn password_from_keychain(config: &mut LinsConfig, passwords: &dyn Passwords) -> bool {
    if config.connection.password.is_some() {
        return false;
    }
    let found = passwords.get(&account(&config.connection.host, config.connection.port));
    let filled = found.is_some();
    config.connection.password = found;
    filled
}

/// What to keep after a start that reached the database. Nothing when the start was itself the
/// remembered one, so that opening from the dock never rewrites what a command gave.
fn to_remember(source: &Source, config: &LinsConfig, graph: Option<&str>, now: i64) -> Option<LastStart> {
    let file = match source {
        Source::Given(path) | Source::FromEnv(path) | Source::UserFile(path) => {
            Some(std::fs::canonicalize(path).unwrap_or_else(|_| path.clone()))
        }
        Source::Defaults(_) => None,
        Source::Remembered(_) => return None,
    };
    Some(LastStart {
        config: file,
        host: config.connection.host.clone(),
        port: config.connection.port,
        graph: graph.map(str::to_string),
        at: now,
    })
}

/// Connect, take a free port on this machine, and start answering. The port is returned.
async fn start(config: LinsConfig) -> Result<u16> {
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

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            // The app's own folder of settings, where the last start is kept.
            let kept_in = app.path().app_config_dir().ok();
            let source = source(
                args.config.as_deref(),
                std::env::var_os("LINS_CONFIG").map(PathBuf::from),
                user_file(),
                kept_in.as_deref().and_then(LastStart::read),
            );
            let env = |name: &str| std::env::var(name).ok();
            let graph = match &source {
                Source::Remembered(start) if args.graph.is_none() => start.graph.clone(),
                _ => args.graph.clone(),
            };

            let mut settings_note = String::new();
            let started = (|| -> Result<(u16, LinsConfig)> {
                if !lins_server::carries_the_page() {
                    return Err(anyhow!("This build does not carry the page. Build lins-web first, then the app again."));
                }
                let (mut config, note) = settings(&source, &env)?;
                settings_note = note;
                if password_from_keychain(&mut config, &Keychain) {
                    info!("The database's password was taken from the keychain");
                    settings_note.push_str(" Password: from the keychain.");
                }
                let port = tauri::async_runtime::block_on(start(config.clone()))?;
                Ok((port, config))
            })();

            let window = match started {
                Ok((port, config)) => {
                    remember(&source, &config, args.graph.as_deref(), kept_in.as_deref());
                    let address = lins_server::page_address("127.0.0.1", port, graph.as_deref());
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

/// Keep a start that reached the database, and its password in the keychain. A failure here is
/// logged and never stops the app, which is already open on the database.
fn remember(source: &Source, config: &LinsConfig, graph: Option<&str>, kept_in: Option<&Path>) {
    let Some(start) = to_remember(source, config, graph, chrono::Utc::now().timestamp()) else {
        return;
    };
    match kept_in {
        Some(dir) => match start.write(dir) {
            Ok(()) => info!("Kept this start for the next time Lins is opened with nothing given"),
            Err(e) => warn!("This start could not be kept: {e:#}"),
        },
        None => warn!("This start could not be kept: the app has no folder of its own"),
    }
    if let Some(password) = &config.connection.password {
        match keep_password(&Keychain, &account(&start.host, start.port), password) {
            Ok(true) => info!("The database's password is kept in the keychain"),
            Ok(false) => {}
            Err(e) => warn!("{e:#}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use remembered::tests::Memory;

    fn kept() -> LastStart {
        LastStart {
            config: None,
            host: "db.example".into(),
            port: 6390,
            graph: Some("notes".into()),
            at: 1_760_000_000,
        }
    }

    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn a_config_given_on_the_command_line_wins() {
        let given = PathBuf::from("/tmp/x/lins.toml");
        let found = source(Some(&given), Some("/tmp/y.toml".into()), None, Some(kept()));
        assert_eq!(found, Source::Given(given));
    }

    #[test]
    fn the_last_start_counts_only_when_nothing_else_is_there() {
        let missing = PathBuf::from("/nowhere/lins/lins.toml");
        assert_eq!(source(None, None, Some(missing.clone()), Some(kept())), Source::Remembered(kept()));
        assert_eq!(source(None, None, Some(missing.clone()), None), Source::Defaults(Some(missing)));
        assert_eq!(
            source(None, Some("/tmp/y.toml".into()), None, Some(kept())),
            Source::FromEnv("/tmp/y.toml".into())
        );
    }

    #[test]
    fn the_persons_own_file_wins_over_the_last_start() {
        let dir = std::env::temp_dir().join(format!("lins-app-user-file-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lins.toml");
        std::fs::write(&file, "").unwrap();
        assert_eq!(source(None, None, Some(file.clone()), Some(kept())), Source::UserFile(file));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_last_start_names_the_database_and_the_environment_can_overrule_it() {
        let (config, note) = settings(&Source::Remembered(kept()), &no_env).unwrap();
        assert_eq!((config.connection.host.as_str(), config.connection.port), ("db.example", 6390));
        assert!(note.contains("db.example:6390"), "{note}");

        let env = |name: &str| (name == "LINS_FALKORDB_HOST").then(|| "other.example".to_string());
        let (config, _) = settings(&Source::Remembered(kept()), &env).unwrap();
        assert_eq!(config.connection.host, "other.example");
        assert_eq!(config.connection.port, 6390);
    }

    #[test]
    fn a_remembered_settings_file_that_has_gone_falls_back_to_the_defaults_and_says_so() {
        let mut start = kept();
        start.config = Some(PathBuf::from("/nowhere/gone/lins.toml"));
        let (config, note) = settings(&Source::Remembered(start), &no_env).unwrap();
        assert_eq!(config.connection.host, "db.example");
        assert!(note.contains("no longer there"), "{note}");
    }

    #[test]
    fn the_keychain_fills_a_missing_password_and_never_replaces_a_given_one() {
        let keychain = Memory::default();
        keychain.set("db.example:6390", "kept").unwrap();

        let (mut config, _) = settings(&Source::Remembered(kept()), &no_env).unwrap();
        assert!(password_from_keychain(&mut config, &keychain));
        assert_eq!(config.connection.password.as_deref(), Some("kept"));

        config.connection.password = Some("given".into());
        assert!(!password_from_keychain(&mut config, &keychain));
        assert_eq!(config.connection.password.as_deref(), Some("given"));
    }

    #[test]
    fn a_start_from_the_last_start_is_not_kept_again() {
        let config = LinsConfig::default();
        assert_eq!(to_remember(&Source::Remembered(kept()), &config, None, 1), None);
        let given = to_remember(&Source::Given("/tmp/a.toml".into()), &config, Some("g"), 7).unwrap();
        assert_eq!((given.host.as_str(), given.port, given.graph.as_deref(), given.at), ("127.0.0.1", 6380, Some("g"), 7));
        assert_eq!(given.config, Some(PathBuf::from("/tmp/a.toml")));
    }
}
