#!/bin/sh
# Build the current dialogue candidate with the production Rust 2D browser client.
# Outputs, content packs and server state remain ignored. No deployment occurs.
set -eu
cd "$(dirname "$0")/.."
PACK=${CRYSTAL_PACK:-content-packs/minnesota-dialogue-overhaul-v03.crystalpack}
OUT=${CRYSTAL_WEB_OUT:-target/dialogue-overhaul-2d-web}
TARGET=${CARGO_TARGET_DIR:-target/northstar-story}
BINDGEN=${WASM_BINDGEN:-wasm-bindgen}
export CARGO_TARGET_DIR=$TARGET
[ -f "$PACK" ] || { echo "Missing content pack: $PACK" >&2; exit 1; }
[ "$("$BINDGEN" --version)" = 'wasm-bindgen 0.2.128' ] || {
    echo 'Set WASM_BINDGEN to the official wasm-bindgen-cli 0.2.128 executable.' >&2
    exit 1
}
git check-ignore --quiet --no-index "$OUT/core-modular.browser.crystalpack" || {
    echo 'Browser output must remain ignored.' >&2; exit 1;
}
cargo build --locked --profile wasm-dev --target wasm32-unknown-unknown \
    -p crystal-bevy --bin crystal-bevy --no-default-features \
    --features bevy-shell,fullscreen-scaling
cargo rustc --locked --profile wasm-dev --target wasm32-unknown-unknown \
    -p crystal-audio --lib --crate-type cdylib --features browser-synth
cargo build --locked -p crystal-web-server --bin crystal-web-server
mkdir -p "$OUT"
"$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" \
    --out-name crystal-bevy "$TARGET/wasm32-unknown-unknown/wasm-dev/crystal-bevy.wasm"
"$BINDGEN" --target web --no-typescript --remove-name-section --out-dir "$OUT" \
    --out-name crystal-audio "$TARGET/wasm32-unknown-unknown/wasm-dev/crystal_audio.wasm"
cp web-client/*.js web-client/*.css web-client/index.html "$OUT/"
cp "$PACK" "$OUT/core-modular.browser.crystalpack"
cp "$PACK" "$OUT/realtime-clock.browser.crystalpack"
printf 'Built native Rust 2D browser review in %s\n' "$OUT"
