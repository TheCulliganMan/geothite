#!/usr/bin/env bash
# Build and verify local content from the repository's exact source pin.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
workspace="${XDG_DATA_HOME:-$HOME/.local/share}/geothite/export-workspace"
jobs=2
profile="${CRYSTAL_PACK_BUILD_PROFILE:-release}"
case "$profile" in release|wasm-dev) ;; *) echo 'CRYSTAL_PACK_BUILD_PROFILE must be release or wasm-dev.' >&2; exit 2 ;; esac
while (($#)); do
  case "$1" in
    --workspace) [[ $# -ge 2 ]] || { echo '--workspace requires a path.' >&2; exit 2; }; workspace=$2; shift 2 ;;
    --jobs) [[ $# -ge 2 ]] || { echo '--jobs requires a count.' >&2; exit 2; }; jobs=$2; shift 2 ;;
    *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
  esac
done
[[ "$workspace" = /* && "$jobs" =~ ^[1-9][0-9]*$ ]] || { echo 'Workspace must be absolute; jobs must be positive.' >&2; exit 2; }
case "$workspace/" in "$repo/"*) echo 'Export workspace must be outside the checkout.' >&2; exit 2 ;; esac
export PATH="$HOME/.cargo/bin:$PATH"
for tool in git curl tar make cc sha1sum sha256sum cargo realpath; do
  command -v "$tool" >/dev/null || { printf 'Required build tool missing: %s\n' "$tool" >&2; exit 1; }
done
workspace=$(realpath -m -- "$workspace")
case "$workspace/" in "$repo/"*) echo 'Export workspace must be outside the checkout.' >&2; exit 2 ;; esac
pin="$repo/pret-source.lock.json"
pin_value() { sed -n "s/.*\"$1\": \"\([^\"]*\)\".*/\1/p" "$pin"; }
commit=$(pin_value commit)
tree=$(pin_value tree)
rgbds_version=$(pin_value rgbds)
rgbds_hash=$(pin_value rgbds_linux_x86_64_sha256)
rom_hash=$(pin_value reference_rom_sha1)
[[ "$commit" =~ ^[0-9a-f]{40}$ && "$tree" =~ ^[0-9a-f]{40}$ && "$rgbds_hash" =~ ^[0-9a-f]{64}$ && "$rom_hash" =~ ^[0-9a-f]{40}$ && "$rgbds_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Invalid source/toolchain pin.' >&2; exit 1; }
mkdir -p "$workspace/vendor" "$workspace/toolchain"
vendor="$workspace/vendor/pokecrystal"
if [[ ! -d "$vendor/.git" ]]; then
  [[ ! -e "$vendor" ]] || { echo 'Existing vendor directory is not a Git checkout.' >&2; exit 1; }
  git clone --no-checkout "$(pin_value repository)" "$vendor"
fi
[[ -z "$(git -C "$vendor" status --porcelain --untracked-files=no)" ]] || { echo 'External pret checkout has tracked edits; preserve them before building.' >&2; exit 1; }
git -C "$vendor" cat-file -e "$commit^{commit}" 2>/dev/null || git -C "$vendor" fetch origin "$commit"
git -C "$vendor" checkout --detach "$commit"
[[ "$(git -C "$vendor" rev-parse 'HEAD^{tree}')" = "$tree" ]] || { echo 'Pret tree differs from pin.' >&2; exit 1; }
export RGBDS="$workspace/toolchain/rgbds-$rgbds_version"
if [[ ! -x "$RGBDS/rgbasm" ]]; then
  [[ "$(uname -m)" = x86_64 ]] || { echo 'Pinned assembler archive requires Linux x86_64.' >&2; exit 1; }
  archive="$workspace/toolchain/rgbds-$rgbds_version.tar.xz"
  curl --fail --location --retry 3 "https://github.com/gbdev/rgbds/releases/download/v$rgbds_version/rgbds-linux-x86_64.tar.xz" --output "$archive"
  printf '%s  %s\n' "$rgbds_hash" "$archive" | sha256sum --check --status
  mkdir -p "$RGBDS"
  tar -xJf "$archive" -C "$RGBDS"
fi
[[ "$("$RGBDS/rgbasm" --version)" = "rgbasm v$rgbds_version" ]] || { echo 'Assembler differs from pin.' >&2; exit 1; }
printf 'Building pinned pret source %s\n' "$commit"
make -C "$vendor" -j "$jobs" RGBDS="$RGBDS/" pokecrystal.gbc
printf '%s  %s\n' "$rom_hash" "$vendor/pokecrystal.gbc" | sha1sum --check --status
cd "$repo"
git lfs install --local
git lfs pull
git lfs fsck
mkdir -p "$workspace/rust/crates/crystal-bevy/src/battle_anim_program"
cp "$repo/crates/crystal-bevy/src/battle_anim_program/"{bank.bin,provenance.json} "$workspace/rust/crates/crystal-bevy/src/battle_anim_program/"
export CARGO_TARGET_DIR="${CRYSTAL_PACK_BUILD_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/geothite/pack-builder}"
export CARGO_BUILD_JOBS="$jobs"
export CRYSTAL_EXPORT_WORKSPACE="$workspace"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_core -- "$workspace"
cargo run --profile "$profile" --locked -p crystal-assets --bin verify_crystalpack -- "$workspace/content-packs/core-modular.browser.crystalpack" --audio
# Verify both candidate packs before replacing the local agent inputs.
mkdir -p "$repo/content-packs"
browser_candidate="$repo/content-packs/core-modular.browser.pending.crystalpack"
native_candidate="$repo/content-packs/core-modular.pending.crystalpack"
tui_candidate="$repo/content-packs/text-tui.pending.crystalpack"
trap 'rm -f -- "$browser_candidate" "$native_candidate" "$tui_candidate"' EXIT
cp "$workspace/content-packs/core-modular.crystalpack" "$native_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin verify_crystalpack -- "$native_candidate"
cp "$workspace/content-packs/core-modular.browser.crystalpack" "$browser_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_text_tui -- . "$browser_candidate" "$tui_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin verify_crystalpack -- "$tui_candidate" --audio
cargo run --profile "$profile" --locked -p geothite -- dump "$tui_candidate"
export CRYSTAL_RENDER_TEST_PACK="$tui_candidate"
cargo test --profile "$profile" --locked -p crystal-bevy renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn
cargo test --profile "$profile" --locked -p crystal-bevy renderer_neutral_controller_plays_mom_and_new_bark_gate
cargo test --profile "$profile" --locked -p geothite mcp_drives_real_movement_and_production_start_menu
cargo test --profile "$profile" --locked -p geothite --test stdio_clean
cargo build --profile "$profile" --locked -p geothite
mv "$browser_candidate" "$repo/content-packs/core-modular.browser.crystalpack"
mv "$native_candidate" "$repo/content-packs/core-modular.crystalpack"
mv "$tui_candidate" "$repo/content-packs/text-tui.crystalpack"
printf 'Verified pack: %s\nPlay: %s play %s\n' "$repo/content-packs/text-tui.crystalpack" "$CARGO_TARGET_DIR/$profile/geothite" "$repo/content-packs/text-tui.crystalpack"
