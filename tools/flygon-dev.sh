#!/bin/sh
# Native Rust source-preview server. Does not build or alter the game.
set -eu
cd "$(dirname "$0")/.."
FLYGON_TOOL="${FLYGON_TOOL:-target/web-release/flygon-package}"
[ -x "$FLYGON_TOOL" ] || { echo 'Build the small native tool once: cargo build --locked -p crystal-flygon --bin flygon-package --profile web-release' >&2; exit 1; }
exec "$FLYGON_TOOL" --dev "$PWD" "${FLYGON_GAME_ROOT:-target/3d-web}" "${FLYGON_DATA_ROOT:-/tmp/flygon-data/prepared}" "${FLYGON_PORT:-33006}"
