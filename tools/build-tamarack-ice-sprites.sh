#!/usr/bin/env bash
# Encode authored native grids and apply the twelve sets through Rust publishers.
set -euo pipefail
if [[ $# -ne 2 ]]; then
  echo "usage: bash tools/build-tamarack-ice-sprites.sh BASE.crystalpack NEW.crystalpack" >&2
  exit 2
fi
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
parent="$1"
destination="$2"
[[ -f "$parent" ]] || { echo "Missing parent: $parent" >&2; exit 1; }
[[ ! -e "$destination" ]] || { echo "Preserve existing output: $destination" >&2; exit 1; }
bin="${CRYSTAL_SPRITE_BIN_DIR:-${CARGO_TARGET_DIR:-target}/debug}"
for tool in npc_sprite_candidate pack_trainer_art pack_original_npc; do
  [[ -x "$bin/$tool" ]] || { echo "Build Rust tool first: $bin/$tool" >&2; exit 1; }
done
work="$repo/target/tamarack-ice/publish-$(basename "$destination" .crystalpack)"
[[ ! -e "$work" ]] || { echo "Preserve existing build evidence: $work" >&2; exit 1; }
mkdir -p "$work"
count=0
while IFS=$'\t' read -r mode folder pixels integration field_base; do
  source="planning/minnesota/npcs/tamarack-ice/$folder"
  native="$work/$folder/native"
  next="$work/$folder/result.crystalpack"
  "$bin/npc_sprite_candidate" --native "$source/$pixels" "$native"
  case "$mode" in
    bind) publisher=pack_trainer_art ;;
    new) publisher=pack_original_npc ;;
    *) echo "Unknown publication mode: $mode" >&2; exit 1 ;;
  esac
  "$bin/$publisher" "$parent" "$source/$integration" "$native" "$next"
  parent="$next"
  count=$((count + 1))
done < planning/minnesota/npcs/tamarack-ice/roster.tsv
[[ "$count" -eq 12 ]] || { echo "Expected twelve native sets" >&2; exit 1; }
cp "$parent" "$destination"
echo "Built $destination from twelve native sprite sets. Gameplay review is separate."
