//! What the app was last started with, so that opening it from the dock opens the same thing.
//!
//! A command that starts the app can hand it a settings file and a database, often through the
//! environment. The dock hands it nothing. So when a start reaches the database, the app keeps
//! what it was started with: the settings file's path, the host, the port and the graph in a
//! small file of its own (`last-start.json`, in the app's own folder of settings), and the
//! password in the system's keychain, never in a file. Opened later with nothing given, it uses
//! those.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The file, in the app's own folder of settings, that holds the last start.
pub const FILE_NAME: &str = "last-start.json";

/// The name the keychain files Lins's passwords under: the app's identifier.
pub const KEYCHAIN_SERVICE: &str = "se.fredrikbranstrom.lins";

/// A start that reached the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastStart {
    /// The settings file it was started with, as a full path; none when it ran on the defaults.
    pub config: Option<PathBuf>,
    pub host: String,
    pub port: u16,
    /// The graph it was asked to open on, if one was named.
    pub graph: Option<String>,
    /// When it was kept, in seconds since 1970.
    pub at: i64,
}

impl LastStart {
    /// The last start kept in `dir`, if there is one that can be read.
    pub fn read(dir: &Path) -> Option<LastStart> {
        let text = std::fs::read_to_string(dir.join(FILE_NAME)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Keep this start in `dir`. Written beside and then moved into place, so that a start cut
    /// short never leaves half a file.
    pub fn write(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir).with_context(|| format!("Could not make {}", dir.display()))?;
        let path = dir.join(FILE_NAME);
        let fresh = dir.join(format!("{FILE_NAME}.new"));
        std::fs::write(&fresh, serde_json::to_string_pretty(self)? + "\n")?;
        std::fs::rename(&fresh, &path)?;
        Ok(())
    }
}

/// The keychain's name for the password of the database at `host:port`.
pub fn account(host: &str, port: u16) -> String {
    format!("{host}:{port}")
}

/// Where passwords are kept: the system's keychain, or a stand-in in the tests.
pub trait Passwords {
    /// The password kept for this account, if any. A keychain that cannot be read counts as none.
    fn get(&self, account: &str) -> Option<String>;
    fn set(&self, account: &str, password: &str) -> Result<()>;
}

/// The system's keychain: the login keychain on macOS, the credential manager on Windows, the
/// secret service on Linux.
pub struct Keychain;

impl Passwords for Keychain {
    fn get(&self, account: &str) -> Option<String> {
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, account).ok()?;
        match entry.get_password() {
            Ok(password) => Some(password),
            Err(keyring::Error::NoEntry) => None,
            Err(e) => {
                tracing::warn!("The keychain could not be read for {account}: {e}");
                None
            }
        }
    }

    fn set(&self, account: &str, password: &str) -> Result<()> {
        keyring::Entry::new(KEYCHAIN_SERVICE, account)?
            .set_password(password)
            .with_context(|| format!("Could not keep the password for {account} in the keychain"))
    }
}

/// Put a password that just worked into the keychain, unless the keychain already holds it.
/// Writing only on a difference keeps the keychain from being touched at every start.
pub fn keep_password(passwords: &dyn Passwords, account: &str, password: &str) -> Result<bool> {
    if passwords.get(account).as_deref() == Some(password) {
        return Ok(false);
    }
    passwords.set(account, password)?;
    Ok(true)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// A keychain in memory, which counts its writes.
    #[derive(Default)]
    pub struct Memory {
        pub kept: RefCell<HashMap<String, String>>,
        pub writes: RefCell<usize>,
    }

    impl Passwords for Memory {
        fn get(&self, account: &str) -> Option<String> {
            self.kept.borrow().get(account).cloned()
        }
        fn set(&self, account: &str, password: &str) -> Result<()> {
            *self.writes.borrow_mut() += 1;
            self.kept.borrow_mut().insert(account.to_string(), password.to_string());
            Ok(())
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lins-app-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_last_start_is_read_back_as_it_was_written() {
        let dir = scratch("roundtrip");
        let start = LastStart {
            config: Some(PathBuf::from("/somewhere/lins.toml")),
            host: "db.example".into(),
            port: 6380,
            graph: Some("ö-graph".into()),
            at: 1_760_000_000,
        };
        start.write(&dir).unwrap();
        assert_eq!(LastStart::read(&dir), Some(start));
        assert!(!dir.join(format!("{FILE_NAME}.new")).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_file_and_a_broken_file_both_read_as_none() {
        let dir = scratch("broken");
        assert_eq!(LastStart::read(&dir), None);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), "{ not json").unwrap();
        assert_eq!(LastStart::read(&dir), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_password_already_kept_is_not_written_again() {
        let keychain = Memory::default();
        assert!(keep_password(&keychain, "db.example:6380", "one").unwrap());
        assert!(!keep_password(&keychain, "db.example:6380", "one").unwrap());
        assert_eq!(*keychain.writes.borrow(), 1);
        assert!(keep_password(&keychain, "db.example:6380", "two").unwrap());
        assert_eq!(keychain.get("db.example:6380").as_deref(), Some("two"));
    }
}
