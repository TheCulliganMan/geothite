#!/usr/bin/env python3
"""Read-only complete native residual proof for the current regional mine."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import load_pack
from review_cave_quality import inspect

MINE = 'MnR09Cave1F'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    assert not args.output.exists(), 'Keep all review epochs.'
    parent = ROOT / 'content-packs/minnesota-r10-rock-gallery-v01.crystalpack'
    contract_path = Path(__file__).parent / 'native.contract.json'
    contract = json.loads(contract_path.read_text())
    assert sha(parent) == contract['parent_sha256']
    before, actual = load_pack(parent), load_pack(args.pack)
    source_path = ROOT / f'planning/minnesota/caves/johto-quality-v01/{MINE}.json'
    source = json.loads(source_path.read_text())
    assert sha(source_path) == contract['revised_source_sha256']
    old, new = before['data']['maps'][MINE], actual['data']['maps'][MINE]
    blocks = [int(v, 16) for row in source['rows'] for v in row.split()]
    assert new['blocks'] == blocks and new['attributes']['border_block'] == 0x2e
    expected = deepcopy(before)
    expected['identity'] = actual['identity']
    expected['data']['maps'][MINE]['blocks'] = blocks
    expected['data']['maps'][MINE]['attributes']['border_block'] = 0x2e
    expected['data']['map_attributes'][MINE]['border_block'] = 0x2e
    expected['data']['map_blocks'][old['attributes']['blocks_label']] = base64.b64encode(bytes(blocks)).decode()
    assert actual == expected, 'Changed current story, team, text, events, art or unrelated data.'
    party = actual['data']['trainers']['trainers']['MN_LODE_FOREMAN']['party']
    assert [(p['species'], p['level']) for p in party] == [('BANDRAKE', 35), ('HELIKNOCK', 35), ('MESABLAST', 36)]
    assert len(actual['data']['maps']) == 691
    assert sum(a != b for a, b in zip(old['blocks'], blocks)) == 83
    args.output.mkdir(parents=True)
    start = time.perf_counter()
    inspect(actual, source_path.parent, args.output, True, MINE)
    elapsed = time.perf_counter() - start
    receipt = dict(pack=str(args.pack), pack_sha256=sha(args.pack),
                   identity=actual['identity']['content_hash'], parent_sha256=sha(parent),
                   contract_sha256=sha(contract_path), source_sha256=sha(source_path),
                   changed_metatiles=83, native_border_block='2e', unchanged_other_maps=690,
                   foreman_team=[dict(species=p['species'], level=p['level']) for p in party],
                   all_native_story_text_party_moves_flags_programs_indexes_exact=True,
                   all_other_maps_species_art_audio_home_recovery_reports_exact=True,
                   complete_normalized_native_equality=True,
                   warm_single_room_seconds=round(elapsed, 3),
                   scope='Published terrain proof. Native controller and GPU evidence recorded separately.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'audit.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
