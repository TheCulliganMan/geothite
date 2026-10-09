#!/bin/sh
# Build only the shared Rust terminal client; generated output stays ignored.
set -eu
cd "$(dirname "$0")/.."
command -v wasm-bindgen >/dev/null 2>&1 || { echo "Install wasm-bindgen-cli matching Cargo.lock before building the TUI" >&2; exit 1; }
TUI_WEB_ROOT="${TUI_WEB_ROOT:-target/tui-web}"
mkdir -p "$TUI_WEB_ROOT/tui"
npm ci --prefix tools/tui-ascii --ignore-scripts --no-audit --no-fund
cp tools/tui-ascii/node_modules/ascii.rest/dist/mount.js "$TUI_WEB_ROOT/tui/ascii-mount.js"
cp tools/tui-ascii/node_modules/ascii.rest/LICENSE "$TUI_WEB_ROOT/tui/ASCII-LICENSE.txt"
cargo build --locked -p geothite --lib --target wasm32-unknown-unknown --profile web-release
wasm-bindgen target/wasm32-unknown-unknown/web-release/geothite.wasm --target web --no-typescript --out-dir "$TUI_WEB_ROOT/tui"
cp web-client/tui/index.html web-client/tui/browser.js web-client/tui/bridge.js web-client/tui/browser.css web-client/tui/ascii-frame.js "$TUI_WEB_ROOT/tui/"
sh tools/version-tui-bundle.sh "$TUI_WEB_ROOT/tui"
TUI_LOCAL_PACK="${TUI_LOCAL_PACK:-content-packs/realtime-clock.browser.crystalpack}"
if test -s "$TUI_LOCAL_PACK"; then
    cp "$TUI_LOCAL_PACK" "$TUI_WEB_ROOT/realtime-clock.browser.crystalpack"
fi
