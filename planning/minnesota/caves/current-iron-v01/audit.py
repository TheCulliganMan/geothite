#!/usr/bin/env python3
"""Certify the complete R10 overlay and render actual written native terrain."""
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
from audit_outdoor_connections import collision
from render_reference import load_pack
from review_cave_quality import inspect

ROOM = 'MnR10Cave1F'
PARENT_SHA = '3862c34b044a456721b5f7ee5464b42dd123f07e3bd12d16e7e503bec972ffa2'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Preserve every cave review epoch.'
    parent_path = ROOT / 'content-packs/minnesota-waterfront-frontages-v01.crystalpack'
    assert sha(parent_path) == PARENT_SHA
    source_path = ROOT / f'planning/minnesota/caves/johto-quality-v01/{ROOM}.json'
    previous_path = source_path.with_name(ROOM + '.previous.json')
    source = json.loads(source_path.read_text())
    previous = json.loads(previous_path.read_text())
    before, actual = load_pack(parent_path), load_pack(args.pack)
    old = before['data']['maps'][ROOM]
    new = actual['data']['maps'][ROOM]
    assert old['blocks'] == [int(v, 16) for row in previous['rows'] for v in row.split()]
    blocks = [int(v, 16) for row in source['rows'] for v in row.split()]
    assert new['blocks'] == blocks
    assert new['attributes']['border_block'] == source['border_block'] == 0x2e
    expected = deepcopy(before)
    expected['identity'] = actual['identity']
    expected['data']['maps'][ROOM]['blocks'] = blocks
    expected['data']['maps'][ROOM]['attributes']['border_block'] = 0x2e
    expected['data']['map_attributes'][ROOM]['border_block'] = 0x2e
    label = old['attributes']['blocks_label']
    expected['data']['map_blocks'][label] = base64.b64encode(bytes(blocks)).decode()
    assert actual == expected, 'R10 changed data outside its exact terrain/border/mirrors/identity.'
    old_grid, grid = collision(before, old), collision(actual, new)
    water = {p for p, kind in old_grid.items() if kind == 'WATER'}
    new_water = {p for p, kind in grid.items() if kind == 'WATER'}
    added = {(x, y) for x in range(24, 26) for y in range(12, 16)} | {
        (x, y) for x in range(26, 30) for y in range(10, 14)}
    assert water <= new_water and new_water - water == added
    water_blocks = {(x // 2, y // 2) for x, y in water}
    width = old['attributes']['width']
    changed_wet_blocks = {(x, y) for x, y in water_blocks
                          if old['blocks'][y * width + x] != blocks[y * width + x]}
    assert changed_wet_blocks == {(13, 7), (14, 7)}
    assert all(grid[x, y] == 'WALL' for (x, y) in grid
               if x in (0, 35) or y in (0, 27)), 'Closed native rock perimeter required.'
    assert {(x, y) for (x, y), kind in grid.items() if kind == 'LADDER'} == {(3, 13)}
    assert len(actual['data']['maps']) == len(before['data']['maps']) == 691
    changed = sum(a != b for a, b in zip(old['blocks'], blocks))
    assert changed == 139
    args.output.mkdir(parents=True)
    started = time.perf_counter()
    inspect(actual, source_path.parent, args.output, True, ROOM)
    duration = time.perf_counter() - started
    receipt = dict(pack=str(args.pack), pack_sha256=sha(args.pack),
                   identity=actual['identity']['content_hash'], parent_sha256=PARENT_SHA,
                   source_sha256=sha(source_path), previous_source_sha256=sha(previous_path),
                   maps=691, unchanged_other_maps=690, changed_metatiles=changed,
                   original_border=old['attributes']['border_block'], native_rock_border=0x2e,
                   retained_water_core_tiles=len(water), added_water_core_tiles=len(added),
                   total_water_core_tiles=len(new_water), original_wet_metatiles=len(water_blocks),
                   revised_original_wet_metatiles=sorted(map(list, changed_wet_blocks)),
                   all_other_original_water_blocks_exact=True,
                   closed_native_rock_perimeter=True,
                   all_programs_events_actors_rooms_exteriors_species_art_audio_reports_exact=True,
                   complete_normalized_native_equality=True,
                   single_room_warm_render_seconds=round(duration, 3),
                   scope='Actual native publication and full residual equality. Controller and GPU evidence recorded separately.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'audit.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
