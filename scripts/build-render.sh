#!/usr/bin/env bash
# Build the Rust-drawn view for the web.
#
#   scripts/build-render.sh
#
# lins-render is compiled to WebAssembly and put in lins-web/static/render/, where the page
# loads it. Run it before `npm run build` whenever lins-render changed. It needs rustup (for
# the wasm32 target; a Rust from Homebrew has none), npm's packages installed in lins-web
# (the fonts come from there) and uv (to unpack the fonts).
#
# The placing rules have tests, which run on this machine and not in a browser:
#   PATH="$(dirname "$(rustup which --toolchain stable cargo)"):$PATH" cargo test -p lins-render --lib
#
# Everything it fetches stays inside the repository and is ignored by git:
#   lins-render/assets/fonts/   the page's own Manrope files, unpacked from WOFF to TTF
#   .tools/                     the wasm-bindgen command, at the version the crate pins
#   lins-web/static/render/     the built view
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOLCHAIN="${LINS_WASM_TOOLCHAIN:-stable}"
# Must equal the `wasm-bindgen = "=…"` line in lins-render/Cargo.toml.
BINDGEN_VERSION="0.2.129"
TOOLS="$ROOT/.tools"
OUT="$ROOT/lins-web/static/render"
FONTS="$ROOT/lins-render/assets/fonts"
FONT_SOURCE="$ROOT/lins-web/node_modules/@fontsource/manrope/files"

say() { printf '\n== %s\n' "$*"; }

command -v rustup >/dev/null || { echo "rustup is not on the PATH; the wasm32 target needs it." >&2; exit 1; }
command -v uv >/dev/null || { echo "uv is not on the PATH; it unpacks the fonts." >&2; exit 1; }
[ -d "$FONT_SOURCE" ] || { echo "No fonts at $FONT_SOURCE. Run: (cd lins-web && npm install)" >&2; exit 1; }

say "fonts: the page's Manrope, unpacked for the Rust view"
mkdir -p "$FONTS"
uv run --quiet --with fonttools python - "$FONT_SOURCE" "$FONTS" <<'PY'
import sys
from pathlib import Path
from fontTools.ttLib import TTFont

source, out = Path(sys.argv[1]), Path(sys.argv[2])
for weight, name in ((500, "Manrope-Medium"), (600, "Manrope-SemiBold")):
    target = out / f"{name}.ttf"
    packed = source / f"manrope-latin-{weight}-normal.woff"
    if target.exists() and target.stat().st_mtime >= packed.stat().st_mtime:
        print(f"  {target.name}: up to date")
        continue
    font = TTFont(packed)
    font.flavor = None  # plain TrueType, which is what egui reads
    font.save(target)
    print(f"  {target.name}: {target.stat().st_size // 1024} KB")
PY
for f in Manrope-Medium.ttf Manrope-SemiBold.ttf; do
  [ -s "$FONTS/$f" ] || { echo "The font $f was not written." >&2; exit 1; }
done

say "toolchain: $TOOLCHAIN with the wasm32 target"
rustup toolchain list | grep -q "^$TOOLCHAIN" || rustup toolchain install "$TOOLCHAIN" --profile minimal
rustup target list --toolchain "$TOOLCHAIN" --installed | grep -q '^wasm32-unknown-unknown$' \
  || rustup target add wasm32-unknown-unknown --toolchain "$TOOLCHAIN"
# The toolchain's own folder goes first on the PATH: a Rust from Homebrew found earlier would
# otherwise be the compiler, and egui asks for a newer one than Homebrew ships.
export PATH="$(dirname "$(rustup which --toolchain "$TOOLCHAIN" cargo)"):$PATH"
rustc --version

say "wasm-bindgen $BINDGEN_VERSION"
BINDGEN="$TOOLS/bin/wasm-bindgen"
if ! "$BINDGEN" --version 2>/dev/null | grep -q " $BINDGEN_VERSION\$"; then
  cargo install wasm-bindgen-cli --version "$BINDGEN_VERSION" --locked --root "$TOOLS"
fi
"$BINDGEN" --version

say "lins-render, for the web"
(cd "$ROOT" && cargo build -p lins-render --lib --target wasm32-unknown-unknown --profile web)
WASM="$ROOT/target/wasm32-unknown-unknown/web/lins_render.wasm"
[ -s "$WASM" ] || { echo "No build at $WASM." >&2; exit 1; }

mkdir -p "$OUT"
"$BINDGEN" --target web --no-typescript --out-dir "$OUT" --out-name lins_render "$WASM"
ls -l "$OUT" | sed 's/^/  /'
say "done. Now build the page: (cd lins-web && npm run build)"
