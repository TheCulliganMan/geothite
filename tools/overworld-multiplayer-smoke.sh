#!/bin/sh
# Build and test hosted overworld gameplay with isolated browser saves.
set -eu
cd "$(dirname "$0")/.."
command -v wasm-bindgen >/dev/null
test -s content-packs/core-modular.browser.crystalpack
MP_TARGET="${CARGO_TARGET_DIR:-target}"
MP_OUTPUT="${MP_TEST_OUTPUT:-target/overworld-multiplayer-smoke}"
MP_PORT="${MP_TEST_PORT:-18743}"
export MP_TEST_SECRET="${MP_TEST_SECRET:-hosted-overworld-test-secret-local-only-2026}"
mkdir -p "$MP_OUTPUT/web/tui" "$MP_OUTPUT/fixtures"
cargo build --locked -p crystal-web-server --bin crystal-web-server
cargo build --locked -p crystal-runtime --example hosted_overworld_fixture --features test-fixtures
cargo build --locked --profile web-release -p crystal-bevy --bin crystal-bevy --features fullscreen-scaling,voxel-view --target wasm32-unknown-unknown
cargo rustc --locked --profile web-release -p crystal-audio --lib --crate-type cdylib --features browser-synth --target wasm32-unknown-unknown
wasm-bindgen "$MP_TARGET/wasm32-unknown-unknown/web-release/crystal-bevy.wasm" --target web --no-typescript --out-name crystal-bevy --out-dir "$MP_OUTPUT/web"
wasm-bindgen "$MP_TARGET/wasm32-unknown-unknown/web-release/crystal_audio.wasm" --target web --no-typescript --out-name crystal-audio --out-dir "$MP_OUTPUT/web"
cp web-client/*.js web-client/*.css web-client/index.html "$MP_OUTPUT/web/"
cp content-packs/core-modular.browser.crystalpack "$MP_OUTPUT/web/"
# Non-Minnesota clients load the shared character artwork during startup,
# including in 2D. Use the same Rust bundlers as the production Docker build.
cargo run --locked -p crystal-voxel-view --bin bundle_open_models -- assets/open-models "$MP_OUTPUT/web/open-models.json"
cargo run --locked -p crystal-voxel-view --bin bundle_open_people -- assets/open-models "$MP_OUTPUT/web/open-people.json"
npm ci --prefix tools/tui-ascii --ignore-scripts --no-audit --no-fund
cp web-client/tui/ascii-frame.js "$MP_OUTPUT/web/tui/"
cp tools/tui-ascii/node_modules/ascii.rest/dist/mount.js "$MP_OUTPUT/web/tui/ascii-mount.js"
cp tools/tui-ascii/node_modules/ascii.rest/LICENSE "$MP_OUTPUT/web/tui/ASCII-LICENSE.txt"
sh tools/version-tui-bundle.sh "$MP_OUTPUT/web/tui"
sh tools/version-browser-bundle.sh "$MP_OUTPUT/web"
CRYSTAL_DATA_DIR="$MP_OUTPUT/data" CRYSTAL_AUTH_SECRET="$MP_TEST_SECRET" "$MP_TARGET/debug/crystal-web-server" --dir "$MP_OUTPUT/web" --pack-dir "$MP_OUTPUT/packs" --host 127.0.0.1 --port "$MP_PORT" >"$MP_OUTPUT/server.log" 2>&1 &
MP_SERVER_PID=$!
trap 'kill "$MP_SERVER_PID" 2>/dev/null || true; wait "$MP_SERVER_PID" 2>/dev/null || true' EXIT HUP INT TERM
MP_TRIES=0
until curl -fsS "http://127.0.0.1:$MP_PORT/healthz" >/dev/null 2>&1; do
    if ! kill -0 "$MP_SERVER_PID" 2>/dev/null; then cat "$MP_OUTPUT/server.log" >&2; exit 1; fi
    MP_TRIES=$((MP_TRIES + 1))
    if test "$MP_TRIES" -ge 180; then cat "$MP_OUTPUT/server.log" >&2; exit 1; fi
    sleep 1
done
MP_COMPOSED_PACK="$(cd "$MP_OUTPUT/data/modpacks" && pwd)/all-251.browser.crystalpack"
CRYSTAL_RENDER_TEST_PACK="$MP_COMPOSED_PACK" cargo test --locked -p crystal-runtime --test hosted_link_battle -- --ignored
"$MP_TARGET/debug/examples/hosted_overworld_fixture" "$MP_COMPOSED_PACK" "$MP_OUTPUT/fixtures"
node tools/overworld-multiplayer-smoke.mjs "http://127.0.0.1:$MP_PORT/" "$MP_OUTPUT/fixtures"
