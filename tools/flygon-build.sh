#!/bin/sh
# Rebuild only the small neural crate, never Pokémon. UI/config need no build.
set -eu
cd "$(dirname "$0")/.."
command -v wasm-bindgen >/dev/null 2>&1 || { echo "Install wasm-bindgen-cli 0.2.128 before building Flygon" >&2; exit 1; }
FLYGON_BUNDLE_DIR="${FLYGON_GAME_ROOT:-target/3d-web}/flygon"
cargo build --locked -p crystal-flygon --lib --target wasm32-unknown-unknown --profile web-release
wasm-bindgen target/wasm32-unknown-unknown/web-release/crystal_flygon.wasm --target web --no-typescript --out-dir "$FLYGON_BUNDLE_DIR"
