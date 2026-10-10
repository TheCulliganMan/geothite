#!/bin/sh
# Build the actual production game with New Bark's modeled renderer locally.
# Requires the repository's Rust toolchain, wasm32 target and wasm-bindgen 0.2.128.
# Content remains external/ignored. No downloads or deployment are performed here.
set -eu
cd "$(dirname "$0")/.."
PACK="${CRYSTAL_PACK:-content-packs/core-modular.browser.crystalpack}"
OUT="${CRYSTAL_WEB_OUT:-target/new-bark-3d-web}"
[ -f "$PACK" ] || { echo "Missing compatible content pack: $PACK" >&2; exit 1; }
command -v wasm-bindgen >/dev/null 2>&1 || { echo 'Install official wasm-bindgen-cli 0.2.128.' >&2; exit 1; }
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
cargo build --locked --profile wasm-dev -p crystal-bevy --bin crystal-bevy \
    --features fullscreen-scaling,voxel-view,location-tester --target wasm32-unknown-unknown
cargo build --locked --profile wasm-dev -p crystal-audio --lib \
    --features browser-synth --target wasm32-unknown-unknown
cargo build --locked -p crystal-web-server --bin crystal-web-server
mkdir -p "$OUT"
wasm-bindgen --target web --no-typescript --out-dir "$OUT" --out-name crystal-bevy \
    target/wasm32-unknown-unknown/wasm-dev/crystal-bevy.wasm
wasm-bindgen --target web --no-typescript --out-dir "$OUT" --out-name crystal-audio \
    target/wasm32-unknown-unknown/wasm-dev/crystal_audio.wasm
cp web-client/*.js web-client/*.css web-client/index.html "$OUT/"
cp "$PACK" "$OUT/core-modular.browser.crystalpack"
# Optional converted CC0 scenery stays external and ignored, just like the pack.
if [ -n "${CRYSTAL_OPEN_MODEL_ROOT:-}" ]; then
    cargo run --locked -p crystal-voxel-view --bin bundle_open_models -- \
        "$CRYSTAL_OPEN_MODEL_ROOT" "$OUT/open-models.json"
else
    # Do not accidentally reuse a previous preview's optional scenery bundle.
    rm -f "$OUT/open-models.json" "$OUT/open-models.json.gz"
fi
printf '\nBuilt %s. Run:\nCRYSTAL_DATA_DIR=target/new-bark-3d-data target/debug/crystal-web-server --dir %s --port 3003\nOpen http://localhost:3003/?multiplayer=off&preview=new-bark\n' "$OUT" "$OUT"
