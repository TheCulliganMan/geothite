#!/usr/bin/env bash
# Build and verify local content from the repository's exact source pin.
set -euo pipefail
export GIT_TERMINAL_PROMPT=0
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
for tool in git curl tar make cc cargo; do
  command -v "$tool" >/dev/null || { printf 'Required build tool missing: %s\n' "$tool" >&2; exit 1; }
done
mkdir -p -- "$workspace"
workspace=$(cd -- "$workspace" && pwd -P)
case "$workspace/" in "$repo/"*) echo 'Export workspace must be outside the checkout.' >&2; exit 2 ;; esac
pin="$repo/pret-source.lock.json"
pin_value() { sed -n "s/.*\"$1\": \"\([^\"]*\)\".*/\1/p" "$pin"; }
commit=$(pin_value commit)
tree=$(pin_value tree)
rgbds_version=$(pin_value rgbds)
rgbds_source_commit=$(pin_value rgbds_source_commit)
rgbds_hash=$(pin_value rgbds_linux_x86_64_sha256)
rom_hash=$(pin_value reference_rom_sha1)
[[ "$commit" =~ ^[0-9a-f]{40}$ && "$tree" =~ ^[0-9a-f]{40}$ && "$rgbds_source_commit" =~ ^[0-9a-f]{40}$ && "$rgbds_hash" =~ ^[0-9a-f]{64}$ && "$rom_hash" =~ ^[0-9a-f]{40}$ && "$rgbds_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Invalid source/toolchain pin.' >&2; exit 1; }
file_hash() {
  if command -v "sha${1}sum" >/dev/null; then
    "sha${1}sum" "$2" | cut -d ' ' -f 1
  else
    shasum -a "$1" "$2" | cut -d ' ' -f 1
  fi
}
mkdir -p "$workspace/vendor" "$workspace/toolchain"
vendor="$workspace/vendor/pokecrystal"
vendor_created=false
if [[ ! -d "$vendor/.git" ]]; then
  [[ ! -e "$vendor" ]] || { echo 'Existing vendor directory is not a Git checkout.' >&2; exit 1; }
  git clone --no-checkout "$(pin_value repository)" "$vendor"
  vendor_created=true
fi
if [[ "$vendor_created" = false ]]; then
  [[ -z "$(git -C "$vendor" status --porcelain --untracked-files=no)" ]] || { echo 'External pret checkout has tracked edits; preserve them before building.' >&2; exit 1; }
fi
git -C "$vendor" cat-file -e "$commit^{commit}" 2>/dev/null || git -C "$vendor" fetch origin "$commit"
git -C "$vendor" checkout --detach "$commit"
[[ "$(git -C "$vendor" rev-parse 'HEAD^{tree}')" = "$tree" ]] || { echo 'Pret tree differs from pin.' >&2; exit 1; }
export RGBDS="$workspace/toolchain/rgbds-$rgbds_version"
if [[ ! -x "$RGBDS/rgbasm" ]]; then
  if [[ "$(uname -s)" = Darwin ]]; then
    # Apple's system Bison is too old for RGBDS. Use the installed Homebrew
    # tools without modifying the user's shell configuration.
    if command -v brew >/dev/null; then
      export PATH="$(brew --prefix)/opt/bison/bin:$PATH"
    fi
    for tool in bison pkg-config; do
      command -v "$tool" >/dev/null || { printf 'Required RGBDS source build tool missing: %s\n' "$tool" >&2; exit 1; }
    done
    source_dir="$workspace/toolchain/rgbds-source-$rgbds_version"
    source_created=false
    if [[ ! -d "$source_dir/.git" ]]; then
      git clone --no-checkout https://github.com/gbdev/rgbds.git "$source_dir"
      source_created=true
    fi
    if [[ "$source_created" = false ]]; then
      [[ -z "$(git -C "$source_dir" status --porcelain --untracked-files=no)" ]] || { echo 'External RGBDS checkout has tracked edits.' >&2; exit 1; }
    fi
    git -C "$source_dir" checkout --detach "$rgbds_source_commit"
    make -C "$source_dir" -j "$jobs"
    mkdir -p "$RGBDS"
    cp "$source_dir/"{rgbasm,rgblink,rgbfix,rgbgfx} "$RGBDS/"
  else
  [[ "$(uname -s)" = Linux && "$(uname -m)" = x86_64 ]] || { echo 'Pinned assembler archive requires Linux x86_64.' >&2; exit 1; }
  archive="$workspace/toolchain/rgbds-$rgbds_version.tar.xz"
  curl --fail --location --retry 3 "https://github.com/gbdev/rgbds/releases/download/v$rgbds_version/rgbds-linux-x86_64.tar.xz" --output "$archive"
  [[ "$(file_hash 256 "$archive")" = "$rgbds_hash" ]] || { echo 'Assembler archive differs from pin.' >&2; exit 1; }
  mkdir -p "$RGBDS"
  tar -xJf "$archive" -C "$RGBDS"
  fi
fi
[[ "$("$RGBDS/rgbasm" --version)" = "rgbasm v$rgbds_version" ]] || { echo 'Assembler differs from pin.' >&2; exit 1; }
printf 'Building pinned pret source %s\n' "$commit"
make -C "$vendor" -j "$jobs" RGBDS="$RGBDS/" pokecrystal.gbc
[[ "$(file_hash 1 "$vendor/pokecrystal.gbc")" = "$rom_hash" ]] || { echo 'Assembled reference ROM differs from pin.' >&2; exit 1; }
cd "$repo"
git lfs install --local
git lfs pull
git lfs fsck
mkdir -p "$workspace/rust/crates/crystal-bevy/src/battle_anim_program"
cp "$repo/crates/crystal-bevy/src/battle_anim_program/"{bank.bin,provenance.json} "$workspace/rust/crates/crystal-bevy/src/battle_anim_program/"
export CARGO_TARGET_DIR="${CRYSTAL_PACK_BUILD_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/geothite/pack-builder}"
export CARGO_BUILD_JOBS="$jobs"
export CRYSTAL_EXPORT_WORKSPACE="$workspace"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_core -- "$workspace" --prepare-source-workspace
diagnostic="$workspace/content-packs/intro-diagnostic.unvalidated.crystalpack"
trace="$workspace/intro-trace.json"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_core -- "$workspace" --export-intro-trace
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_source_checkpoint -- "$workspace" "$diagnostic" --title-unvalidated "$workspace/apps/web/assets/data/runtime_title_screen.unvalidated.json"
cargo run --profile "$profile" --locked -p crystal-bevy --features intro-certification --bin certify_intro -- "$diagnostic" "$trace" "$workspace"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_core -- "$workspace" --pack-prepared-workspace
cargo run --profile "$profile" --locked -p crystal-assets --bin verify_crystalpack -- "$workspace/content-packs/core-modular.browser.crystalpack" --audio --desktop
# Verify both candidate packs before replacing the local agent inputs.
mkdir -p "$repo/content-packs"
browser_candidate="$repo/content-packs/core-modular.browser.pending.crystalpack"
native_candidate="$repo/content-packs/core-modular.pending.crystalpack"
tui_candidate="$repo/content-packs/text-tui.pending.crystalpack"
battle_candidate="$repo/content-packs/default-battles.pending.crystalpack"
trap 'rm -f -- "$browser_candidate" "$native_candidate" "$tui_candidate" "$battle_candidate"' EXIT
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_all_pokemon -- "$workspace/content-packs/core-modular.crystalpack" "$native_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_default_battles -- "$native_candidate" "$battle_candidate"
mv "$battle_candidate" "$native_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin verify_crystalpack -- "$native_candidate" --desktop
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_all_pokemon -- "$workspace/content-packs/core-modular.browser.crystalpack" "$browser_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_default_battles -- "$browser_candidate" "$battle_candidate"
mv "$battle_candidate" "$browser_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin pack_text_tui -- . "$browser_candidate" "$tui_candidate"
cargo run --profile "$profile" --locked -p crystal-assets --bin verify_crystalpack -- "$tui_candidate" --audio
cargo run --profile "$profile" --locked -p geothite -- dump "$tui_candidate"
export CRYSTAL_RENDER_TEST_PACK="$tui_candidate"
CRYSTAL_MODPACK_TEST_BASE="$workspace/content-packs/core-modular.browser.crystalpack" cargo test --profile "$profile" --locked -p crystal-assets --test competitive_battles
CRYSTAL_WILD_BASE_PACK="$workspace/content-packs/core-modular.browser.crystalpack" cargo test --profile "$profile" --locked -p crystal-assets --test wild_challenge
CRYSTAL_DEFAULT_BATTLE_BASE="$workspace/content-packs/core-modular.browser.crystalpack" cargo test --profile "$profile" --locked -p crystal-assets --test default_battles
CRYSTAL_MODPACK_TEST_BASE="$workspace/content-packs/core-modular.browser.crystalpack" cargo test --profile "$profile" --locked -p crystal-assets --test all_pokemon
cargo test --profile "$profile" --locked -p crystal-bevy renderer_neutral_controller_uses_link_cable_through_production_pack
cargo test --profile "$profile" --locked -p crystal-bevy renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn
cargo test --profile "$profile" --locked -p crystal-bevy renderer_neutral_controller_plays_mom_and_new_bark_gate
cargo test --profile "$profile" --locked -p crystal-bevy renderer_neutral_controller_starts_research_center_with_scene_callback
# Exercise the earned champion scene before publishing a native pack. A pack
# can pass early-game controls while missing its credits presentation program.
CRYSTAL_RENDER_TEST_PACK="$native_candidate" cargo test --profile "$profile" --locked -p crystal-bevy kanto_lance_controller_completes_earned_ceremony_and_continues_home -- --ignored
cargo test --profile "$profile" --locked -p geothite mcp_drives_real_movement_and_production_start_menu
cargo test --profile "$profile" --locked -p geothite --test stdio_clean
cargo build --profile "$profile" --locked -p geothite
mv "$browser_candidate" "$repo/content-packs/core-modular.browser.crystalpack"
mv "$native_candidate" "$repo/content-packs/core-modular.crystalpack"
mv "$tui_candidate" "$repo/content-packs/text-tui.crystalpack"
printf 'Verified pack: %s\nPlay: %s play %s\n' "$repo/content-packs/text-tui.crystalpack" "$CARGO_TARGET_DIR/$profile/geothite" "$repo/content-packs/text-tui.crystalpack"
