#!/usr/bin/env bash
# Add separate NPC bicycle banks without rebinding trainer classes or player art.
set -euo pipefail
if [[ $# -ne 2 ]]; then
  echo "usage: bash tools/build-city-cyclist-sprites.sh BASE.crystalpack NEW.crystalpack" >&2
  exit 2
fi
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
parent="$1"
destination="$2"
[[ -f "$parent" && ! -e "$destination" ]] || {
  echo "Supply an existing parent and a fresh output" >&2; exit 1;
}
git check-ignore --quiet --no-index "$destination" || {
  echo "Generated packs must stay ignored" >&2; exit 1;
}
bin="${CRYSTAL_SPRITE_BIN_DIR:-${CARGO_TARGET_DIR:-target}/debug}"
encoder="${CRYSTAL_SPRITE_ENCODER:-$bin/examples/npc_sprite_candidate}"
[[ -x "$encoder" && -x "$bin/pack_trainer_art" ]] || {
  echo "Build the Rust npc_sprite_candidate example and pack_trainer_art first" >&2; exit 1;
}
work="$repo/target/city-cyclists-v01/publish-$(basename "$destination" .crystalpack)"
[[ ! -e "$work" ]] || { echo "Preserve existing build evidence: $work" >&2; exit 1; }
mkdir -p "$work"
source="planning/minnesota/npcs/city-cyclists-v01"
"$encoder" --native "$source/01-commuter/native-pixels-v2.json" "$work/commuter"
"$bin/pack_trainer_art" "$parent" "$source/01-commuter/binding-v1.json" \
  "$work/commuter" "$work/commuter.crystalpack"
"$encoder" --native "$source/02-city-rider/native-pixels-v1.json" "$work/city-rider"
"$bin/pack_trainer_art" "$work/commuter.crystalpack" \
  "$source/02-city-rider/binding-v1.json" "$work/city-rider" "$destination"
echo "Built separate NPC bicycle art for all 18 city riders: $destination"
