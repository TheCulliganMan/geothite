#!/usr/bin/env bash
# Replay terrain deltas with the Rust publisher; preserve the current parent.
set -euo pipefail

if (( $# < 1 || $# > 3 )); then
  printf 'usage: %s PARENT [FRESH_R01_OUTPUT [FRESH_R02_OUTPUT]]\n' "$0" >&2
  exit 2
fi

task_repo="$(cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
task_parent="$1"
task_r01="${2:-$task_repo/content-packs/minnesota-region-polish-grass-r01-v01.crystalpack}"
task_r02="${3:-$task_repo/content-packs/minnesota-region-polish-grass-r02-v01.crystalpack}"
task_inputs="$task_repo/planning/minnesota/revisions/route-grass-v01/compact-rebase-v01"
task_publisher="$task_repo/target/route-grass/wasm-dev/revise_authored_route"

git -C "$task_repo" check-ignore --quiet --no-index "$task_r01"
git -C "$task_repo" check-ignore --quiet --no-index "$task_r02"

RUST_MIN_STACK="${RUST_MIN_STACK:-67108864}" "$task_publisher" "$task_parent" \
  "$task_inputs/MN_R01.before.layout.json" \
  "$task_inputs/MN_R01.after.layout.json" \
  "$task_inputs/MN_R01.installed-dossier.json" "$task_r01"
RUST_MIN_STACK="${RUST_MIN_STACK:-67108864}" "$task_publisher" "$task_r01" \
  "$task_inputs/MN_R02.before.layout.json" \
  "$task_inputs/MN_R02.after.layout.json" \
  "$task_inputs/MN_R02.installed-dossier.json" "$task_r02"
