#!/bin/sh
# Build the actual production game with New Bark's modeled renderer locally.
# Requires the repository's Rust toolchain, wasm32 target and wasm-bindgen 0.2.128.
# Content remains external/ignored. No downloads or deployment are performed here.
set -eu
cd "$(dirname "$0")/.."
PACK="${CRYSTAL_PACK:-content-packs/core-modular.browser.crystalpack}"
OUT="${CRYSTAL_WEB_OUT:-target/new-bark-3d-web}"
TARGET="${CARGO_TARGET_DIR:-target}"
PROFILE="${CRYSTAL_WEB_PROFILE:-wasm-dev}"
# Workers activate only when 3D is used. Keep explicit controls for profiling
# and preserve the earlier per-worker QA environment names for callers.
CPU_WORKERS="${CRYSTAL_CPU_WORKERS:-1}"
TERRAIN_WORKER="${CRYSTAL_TERRAIN_WORKER:-${CRYSTAL_TERRAIN_WORKER_QA:-$CPU_WORKERS}}"
ACTOR_WORKER="${CRYSTAL_ACTOR_WORKER:-${CRYSTAL_ACTOR_WORKER_QA:-$CPU_WORKERS}}"
WEBGPU="${CRYSTAL_WEBGPU:-1}"
case "$PROFILE" in
    wasm-dev|web-release) ;;
    *) echo 'CRYSTAL_WEB_PROFILE must be wasm-dev or web-release.' >&2; exit 1 ;;
esac
[ -f "$PACK" ] || { echo "Missing compatible content pack: $PACK" >&2; exit 1; }
BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
command -v "$BINDGEN" >/dev/null 2>&1 || { echo 'Install official wasm-bindgen-cli 0.2.128.' >&2; exit 1; }
[ "$("$BINDGEN" --version)" = 'wasm-bindgen 0.2.128' ] || {
    echo 'This Cargo.lock requires wasm-bindgen-cli 0.2.128; set WASM_BINDGEN to its executable.' >&2
    exit 1
}
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
FEATURES=fullscreen-scaling,voxel-view,location-tester
set --
if [ "$TERRAIN_WORKER" = 1 ]; then
    FEATURES=$FEATURES,terrain-worker-qa
    set -- "$@" -p crystal-terrain-worker
fi
if [ "$ACTOR_WORKER" = 1 ]; then
    FEATURES=$FEATURES,actor-worker-qa
    set -- "$@" -p crystal-actor-worker
fi
if [ "$#" -gt 0 ]; then
    if [ "${CRYSTAL_SCENERY_FULL_DETAIL:-0}" = 1 ]; then
        set -- "$@" --features crystal-voxel-view/scenery-full-detail
    fi
    cargo build --locked --profile "$PROFILE" "$@" --target wasm32-unknown-unknown
fi
if [ "${CRYSTAL_ALLOCATIONS:-0}" = 1 ]; then
    FEATURES=$FEATURES,allocation-tester
fi
if [ "${CRYSTAL_SCENERY_FULL_DETAIL:-0}" = 1 ]; then
    FEATURES=$FEATURES,crystal-voxel-view/scenery-full-detail
fi
cargo build --locked --profile "$PROFILE" -p crystal-bevy --bin crystal-bevy \
    --features "$FEATURES" --target wasm32-unknown-unknown
cargo rustc --locked --profile "$PROFILE" -p crystal-audio --lib --crate-type cdylib \
    --features browser-synth --target wasm32-unknown-unknown
cargo build --locked -p crystal-web-server --bin crystal-web-server
mkdir -p "$OUT"
if [ "$TERRAIN_WORKER" = 1 ]; then
    "$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" --out-name crystal-terrain-worker \
        "$TARGET/wasm32-unknown-unknown/$PROFILE/crystal_terrain_worker.wasm"
fi
if [ "$ACTOR_WORKER" = 1 ]; then
    "$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" --out-name crystal-actor-worker \
        "$TARGET/wasm32-unknown-unknown/$PROFILE/crystal_actor_worker.wasm"
fi
"$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" --out-name crystal-bevy \
    "$TARGET/wasm32-unknown-unknown/$PROFILE/crystal-bevy.wasm"
# Keep separate modern and WebGL artifacts: their uniform layouts differ.
if [ "$WEBGPU" = 1 ]; then
    cargo build --locked --profile "$PROFILE" -p crystal-bevy --bin crystal-bevy \
        --features "$FEATURES,browser-webgpu" --target wasm32-unknown-unknown
    "$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" --out-name crystal-bevy-webgpu \
        "$TARGET/wasm32-unknown-unknown/$PROFILE/crystal-bevy.wasm"
    printf '%s\n' '{"webgpu":{"module":"./crystal-bevy-webgpu.js","binary":"./crystal-bevy-webgpu_bg.wasm"}}' > "$OUT/renderer-bundles.json"
else
    rm -f "$OUT/renderer-bundles.json"
fi
"$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" --out-name crystal-audio \
    "$TARGET/wasm32-unknown-unknown/$PROFILE/crystal_audio.wasm"
cp web-client/*.js web-client/*.css web-client/index.html "$OUT/"
# The renderer chooser loads the painted TUI integration on demand.
if [ -d web-client/tui ]; then
    mkdir -p "$OUT/tui"
    cp -R web-client/tui/. "$OUT/tui/"
    npm ci --prefix tools/tui-ascii --ignore-scripts --no-audit
    cp tools/tui-ascii/node_modules/ascii.rest/dist/mount.js "$OUT/tui/ascii-mount.js"
    cp tools/tui-ascii/node_modules/ascii.rest/LICENSE "$OUT/tui/ASCII-LICENSE.txt"
fi
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
