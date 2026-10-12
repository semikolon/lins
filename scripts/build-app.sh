#!/usr/bin/env bash
# Build Lins as a desktop app.
#
#   scripts/build-app.sh             # build it, and say where it is
#   scripts/build-app.sh --install   # and put it among your applications (macOS: ~/Applications)
#
# The app is the server and its page in one window (the crate lins-app, Tauri 2). The page
# is built first, since the server carries it inside the binary. Needs npm, cargo and the
# Tauri command:  cargo install tauri-cli --version "^2"
#
# On macOS this builds the .app alone. A disk image, for handing the app to someone else:
#   (cd lins-app && cargo tauri build --bundles dmg)
#
# LINS_APP_DIR names another folder to install into.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL=0
for arg in "$@"; do
  case "$arg" in
    --install) INSTALL=1 ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

say() { printf '\n== %s\n' "$*"; }

command -v npm >/dev/null || { echo "npm is not on the PATH; the page is built with it." >&2; exit 1; }
cargo tauri --version >/dev/null 2>&1 || {
  echo 'The Tauri command is missing. Install it:  cargo install tauri-cli --version "^2"' >&2; exit 1; }

say "the page"
cd "$ROOT/lins-web"
[ -d node_modules ] || npm install
npm run build
# The page is put inside the binary when the server is compiled, and the compiler does not
# see that the page changed. Touching the file that takes it in makes it look again.
touch "$ROOT/lins-server/src/lib.rs"

say "the app"
cd "$ROOT/lins-app"
if [ "$(uname)" = "Darwin" ]; then
  cargo tauri build --bundles app
  BUILT="$ROOT/target/release/bundle/macos/Lins.app"
  [ -d "$BUILT" ] || { echo "The build finished and there is no app at $BUILT" >&2; exit 1; }
  echo "Built: $BUILT"
  if [ "$INSTALL" = 1 ]; then
    DEST="${LINS_APP_DIR:-$HOME/Applications}/Lins.app"
    mkdir -p "$DEST"
    # Make the installed app equal to the built one, file for file.
    rsync -a --delete "$BUILT/" "$DEST/"
    echo "Installed: $DEST"
  fi
else
  cargo tauri build
  echo "Built: see $ROOT/target/release/bundle/"
  [ "$INSTALL" = 0 ] || echo "--install is for macOS; install the package the build made." >&2
fi
