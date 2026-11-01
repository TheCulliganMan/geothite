#!/usr/bin/env bash
# Encode the selected native grids with Rust, then publish the regional cast.
set -euo pipefail
if [[ $# -ne 2 ]]; then
  echo "usage: bash tools/build-regional-trainer-sprites.sh BASE.crystalpack NEW.crystalpack" >&2
  exit 2
fi
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
parent="$1"
destination="$2"
[[ -f "$parent" ]] || { echo "Missing parent: $parent" >&2; exit 1; }
[[ ! -e "$destination" ]] || { echo "Preserve existing output: $destination" >&2; exit 1; }
git check-ignore --quiet --no-index "$destination" || {
  echo "Generated packs must be ignored: $destination" >&2; exit 1;
}
bin="${CRYSTAL_SPRITE_BIN_DIR:-${CARGO_TARGET_DIR:-target}/debug}"
encoder="$bin/examples/npc_sprite_candidate"
[[ -x "$encoder" ]] || encoder="$bin/npc_sprite_candidate"
[[ -x "$encoder" && -x "$bin/pack_trainer_art_batch" ]] || {
  echo "Build the Rust npc_sprite_candidate example and pack_trainer_art_batch first" >&2; exit 1;
}
work="$repo/target/regional-trainers-v01/publish-$(basename "$destination" .crystalpack)"
[[ ! -e "$work" ]] || { echo "Preserve existing build evidence: $work" >&2; exit 1; }
mkdir -p "$work"
python3 - "$work" <<'PY'
import json
import sys
from pathlib import Path

work = Path(sys.argv[1])
source = Path('planning/minnesota/npcs/regional-trainers-v01')
assets = json.loads((source / 'asset-plan-v1.json').read_text())['assets']
assert len(assets) == 58
plan = {'schema_version': 1, 'placements': [], 'bindings': [],
        'distribution': str(source / 'distribution-v2.json')}
rows = []
for index, asset in enumerate(assets):
    native = work / f'{index + 1:02d}' / 'native'
    rows.append(f"{asset['source']}\t{native}\n")
    if asset['mode'] == 'bind':
        plan['bindings'].append({'binding': asset['integration'], 'native_art': str(native)})
    else:
        assert asset['mode'] == 'new'
        plan['placements'].append({'placement': asset['integration'], 'native_art': str(native)})
assert len(plan['placements']) == 10 and len(plan['bindings']) == 48
(work / 'sources.tsv').write_text(''.join(rows))
(work / 'publication.json').write_text(json.dumps(plan, indent=2) + '\n')
PY
while IFS=$'\t' read -r source native; do
  "$encoder" --native "$source" "$native"
done < "$work/sources.tsv"
"$bin/pack_trainer_art_batch" "$parent" "$work/publication.json" "$destination"
echo "Built $destination from 58 native sets. Gameplay review is separate."
