#!/bin/sh
# Run from any directory. Content packs are supplied separately, never fetched.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ACTION=${1:-build}
if [ "$#" -gt 0 ]; then shift; fi
usage() {
    cat <<'EOF'
Usage: scripts/cold-start-3d.sh [build|play|preview|web|check|setup-web] [client arguments...]
  build      Validate local content/artwork and build the native 3D client (default).
  play       Validate, build, and launch the normal title/new-game client; F3 selects 3D.
  preview    Validate, build, and launch the New Bark 3D developer preview.
  web        Build and serve the browser client on localhost:3003.
  check      Validate the pack and character/scenery artwork without opening a window.
  setup-web  Install the pinned Wasm target and matching wasm-bindgen CLI using rustup/cargo.
Environment: CRYSTAL_PACK, CRYSTAL_OPEN_MODEL_ROOT, CARGO_BUILD_JOBS,
             CRYSTAL_BUILD_PROFILE (dev), CARGO_TARGET_DIR (target/),
             CRYSTAL_WEB_HOST (127.0.0.1), CRYSTAL_WEB_PORT (3003).
Requires the repository's pinned Rust toolchain and native platform libraries.
EOF
}
case "$ACTION" in
    -h|--help|help) usage; exit 0 ;;
    build|play|preview|web|check|setup-web) ;;
    *) usage >&2; exit 2 ;;
esac
# Resolve user-supplied relative paths before changing the working directory.
CALLER=$PWD
PACK=${CRYSTAL_PACK:-$ROOT/content-packs/realtime-clock.browser.crystalpack}
ART=${CRYSTAL_OPEN_MODEL_ROOT:-$ROOT/assets/open-models}
TARGET=${CARGO_TARGET_DIR:-$ROOT/target}
PROFILE=${CRYSTAL_BUILD_PROFILE:-dev}
case "$PROFILE" in dev|wasm-dev|release) ;; *) echo 'CRYSTAL_BUILD_PROFILE must be dev, wasm-dev, or release.' >&2; exit 2 ;; esac
case "$TARGET" in /*) ;; *) TARGET=$CALLER/$TARGET ;; esac
BIN_PROFILE=$PROFILE
[ "$PROFILE" != dev ] || BIN_PROFILE=debug
case "$PACK" in /*) ;; *) PACK=$CALLER/$PACK ;; esac
case "$ART" in /*) ;; *) ART=$CALLER/$ART ;; esac
cd "$ROOT"
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
export CRYSTAL_OPEN_MODEL_ROOT=$ART
export CARGO_TARGET_DIR=$TARGET
command -v cargo >/dev/null 2>&1 || { echo 'Install Rust via rustup, then rerun this script.' >&2; exit 1; }
rustc --version
if [ "$ACTION" = setup-web ]; then
    command -v rustup >/dev/null 2>&1 || { echo 'setup-web requires rustup.' >&2; exit 1; }
    rustup target add wasm32-unknown-unknown
    VERSION=$(sed -n '/^name = "wasm-bindgen"$/ { n; s/^version = "\([^"]*\)"$/\1/p; }' Cargo.lock)
    [ -n "$VERSION" ] || { echo 'Cannot resolve wasm-bindgen version from Cargo.lock.' >&2; exit 1; }
    cargo install --locked wasm-bindgen-cli --version "$VERSION"
    exit 0
fi
[ -f "$PACK" ] || {
    echo "Missing separately supplied pack: $PACK" >&2
    echo 'Transfer content-packs/ from the old host or set CRYSTAL_PACK to your local pack.' >&2
    exit 1
}
case "$PACK" in *.crystalpack) ;; *) echo 'CRYSTAL_PACK must end in .crystalpack.' >&2; exit 1 ;; esac
[ -d "$ART/people/kenney" ] && [ -d "$ART/people/npc-textures" ] || {
    echo "Missing character artwork under $ART; restore the tracked assets or set CRYSTAL_OPEN_MODEL_ROOT." >&2
    exit 1
}
# GLB input must be real artwork, rather than a small Git LFS pointer file.
if LC_ALL=C head -n 1 "$ART/people/kenney/human-male.glb" 2>/dev/null | LC_ALL=C grep -q '^version https://git-lfs.github.com/spec/v1'; then
    echo 'Artwork contains Git LFS pointers. Restore LFS objects and run git lfs checkout (see docs/cold-start-3d.md).' >&2
    exit 1
fi
# Build the native tools together so Bevy features are unified once, rather than
# recompiling separate renderer graphs for each validation command.
if [ "$ACTION" = preview ]; then
    cargo build --locked --profile "$PROFILE" -p geothite -p crystal-voxel-view -p crystal-bevy \
        --features crystal-bevy/location-tester --bin geothite --bin bundle_open_models \
        --bin bundle_open_people --bin crystal-bevy --example new_bark_3d
else
    cargo build --locked --profile "$PROFILE" -p geothite -p crystal-voxel-view -p crystal-bevy \
        --features crystal-bevy/voxel-view --bin geothite --bin bundle_open_models \
        --bin bundle_open_people --bin crystal-bevy
fi
# Rust loads and verifies the actual pack. A dump is only a loading/render smoke check.
"$TARGET/$BIN_PROFILE/geothite" dump "$PACK"
mkdir -p "$ROOT/target"
VERIFY=$(mktemp -d "$ROOT/target/cold-start-3d.XXXXXX")
trap 'rm -rf "$VERIFY"' EXIT
trap 'exit 130' INT
trap 'exit 143' HUP TERM
"$TARGET/$BIN_PROFILE/bundle_open_models" "$ART" "$VERIFY/scenery.json"
"$TARGET/$BIN_PROFILE/bundle_open_people" "$ART" "$VERIFY/people.json"
rm -rf "$VERIFY"
trap - EXIT HUP INT TERM
case "$ACTION" in
    check) echo 'Pack and artwork validated; no gameplay or GPU check was performed.' ;;
    build|play)
        if [ "$ACTION" = play ]; then
            exec "$TARGET/$BIN_PROFILE/crystal-bevy" --pack "$PACK" "$@"
        fi
        echo 'Native 3D client built. Launch with scripts/cold-start-3d.sh play or preview.'
        ;;
    preview)
        exec "$TARGET/$BIN_PROFILE/examples/new_bark_3d" "$PACK" "$@"
        ;;
    web)
        # The existing browser helper uses these fixed output locations.
        export CARGO_TARGET_DIR=$ROOT/target
        CRYSTAL_PACK=$PACK CRYSTAL_WEB_OUT=target/new-bark-3d-web sh tools/new-bark-3d-web.sh
        export CRYSTAL_DATA_DIR=$ROOT/target/new-bark-3d-data
        echo "Open http://localhost:${CRYSTAL_WEB_PORT:-3003}/?multiplayer=off (normal game) or add &preview=new-bark."
        exec "$ROOT/target/debug/crystal-web-server" --dir "$ROOT/target/new-bark-3d-web" \
            --host "${CRYSTAL_WEB_HOST:-127.0.0.1}" --port "${CRYSTAL_WEB_PORT:-3003}" "$@"
        ;;
esac
