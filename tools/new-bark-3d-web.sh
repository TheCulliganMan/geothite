#!/bin/sh
# Build the actual production game with New Bark's modeled renderer locally.
# Requires the repository's Rust toolchain, wasm32 target and wasm-bindgen 0.2.128.
# Content remains external/ignored. No downloads or deployment are performed here.
set -eu
cd "$(dirname "$0")/.."
PACK="${CRYSTAL_PACK:-content-packs/core-modular.browser.crystalpack}"
OUT="${CRYSTAL_WEB_OUT:-target/new-bark-3d-web}"
TARGET="${CARGO_TARGET_DIR:-target}"
[ -f "$PACK" ] || { echo "Missing compatible content pack: $PACK" >&2; exit 1; }
BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
command -v "$BINDGEN" >/dev/null 2>&1 || { echo 'Install official wasm-bindgen-cli 0.2.128.' >&2; exit 1; }
[ "$("$BINDGEN" --version)" = 'wasm-bindgen 0.2.128' ] || {
    echo 'This Cargo.lock requires wasm-bindgen-cli 0.2.128; set WASM_BINDGEN to its executable.' >&2
    exit 1
}
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
FEATURES=fullscreen-scaling,voxel-view,location-tester
if [ "${CRYSTAL_SCENERY_FULL_DETAIL:-0}" = 1 ]; then
    FEATURES=$FEATURES,crystal-voxel-view/scenery-full-detail
fi
cargo build --locked --profile wasm-dev -p crystal-bevy --bin crystal-bevy \
    --features "$FEATURES" --target wasm32-unknown-unknown
cargo rustc --locked --profile wasm-dev -p crystal-audio --lib --crate-type cdylib \
    --features browser-synth --target wasm32-unknown-unknown
cargo build --locked -p crystal-web-server --bin crystal-web-server
mkdir -p "$OUT"
"$BINDGEN" --target web --no-typescript --out-dir "$OUT" --out-name crystal-bevy \
    "$TARGET/wasm32-unknown-unknown/wasm-dev/crystal-bevy.wasm"
"$BINDGEN" --target web --no-typescript --out-dir "$OUT" --out-name crystal-audio \
    "$TARGET/wasm32-unknown-unknown/wasm-dev/crystal_audio.wasm"
cp web-client/*.js web-client/*.css web-client/index.html "$OUT/"
# The server derives its clock/customization pack from the core-named input;
# the browser-named copy also makes a static disposable preview self-contained.
cp "$PACK" "$OUT/core-modular.browser.crystalpack"
cp "$PACK" "$OUT/realtime-clock.browser.crystalpack"
# Character artwork is required. Bundling rejects missing/invalid rigs.
CRYSTAL_OPEN_MODEL_ROOT="${CRYSTAL_OPEN_MODEL_ROOT:-assets/open-models}"
cargo run --locked -p crystal-voxel-view --bin bundle_open_models -- \
    "$CRYSTAL_OPEN_MODEL_ROOT" "$OUT/open-models.json"
cargo run --locked -p crystal-voxel-view --bin bundle_open_people -- \
    "$CRYSTAL_OPEN_MODEL_ROOT" "$OUT/open-people.json"
printf '\nBuilt %s. Run:\nCRYSTAL_DATA_DIR=target/new-bark-3d-data "%s/debug/crystal-web-server" --dir "%s" --port 3003\nOpen http://localhost:3003/?multiplayer=off&preview=new-bark\n' "$OUT" "$TARGET" "$OUT"
