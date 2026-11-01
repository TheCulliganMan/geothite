#!/usr/bin/env python3
"""Read-only exact-parent preservation and bridge-collision audit."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2]/'tools'))
from render_reference import load_pack

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--parent', type=Path, required=True)
parser.add_argument('--child', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
assert not args.output.exists(), 'preserve prior audit'
a, b = load_pack(args.parent), load_pack(args.child)
source = json.loads((Path(__file__).parent/'plan.json').read_text())
changed = []
for name, old in a['data']['maps'].items():
    new = b['data']['maps'][name]
    expected = copy.deepcopy(old)
    for cell in source['changes']:
        if cell['map'] == name:
            x, y = cell['metatile']
            index = y*old['attributes']['width']+x
            assert expected['blocks'][index] == 6
            expected['blocks'][index] = 0x84
    assert expected == new, f'unexpected complete map change: {name}'
    if old != new:
        changed.append(name)
    old_collision = a['data']['tilesets'][old['attributes']['tileset_name']]['collision']
    new_collision = b['data']['tilesets'][new['attributes']['tileset_name']]['collision']
    assert all(old_collision[f'{x:02x}'] == new_collision[f'{y:02x}'] for x,y in zip(old['blocks'],new['blocks'])), f'collision drift: {name}'
assert set(a['data']['maps']) == set(b['data']['maps'])
for key in a['data']:
    if key not in {'maps','map_blocks','tilesets'}:
        assert a['data'][key] == b['data'][key], f'data drift: {key}'
assert a['data']['tilesets'].keys() == b['data']['tilesets'].keys()
for key, old in a['data']['tilesets'].items():
    expected = copy.deepcopy(old)
    if key == 'mn_north_shore':
        expected['collision']['84'] = old['collision']['06']
    assert b['data']['tilesets'][key] == expected, f'art definition drift: {key}'
for key, old in a['data']['map_blocks'].items():
    if key not in {a['data']['maps'][name]['attributes']['blocks_label']for name in changed}:
        assert b['data']['map_blocks'][key] == old, f'block mirror drift: {key}'
assert a['runtime_files'].keys() == b['runtime_files'].keys()
for key, old in a['runtime_files'].items():
    new = b['runtime_files'][key]
    if key == 'data/tilesets/mn_north_shore_metatiles.bin':
        assert bytes(new[:len(old)]) == bytes(old) and len(new) == len(old)+16
        expected = [old[block*16+row*4+column] for row in range(4) for block,column in [(0x44,0),(1,1),(1,2),(0x44,0)]]
        assert list(new[len(old):]) == expected
    elif key == 'data/tilesets/mn_north_shore.json':
        expected=json.loads(bytes(old));expected['84']=expected['06'];assert json.loads(bytes(new))==expected
    else:
        assert old == new, f'runtime file drift: {key}'
assert a['audio_manifest'] == b['audio_manifest']
assert b['report']['manifests'] == [*a['report']['manifests'], 'minnesota-north-shore-footbridges-v1']
old_report=copy.deepcopy(a['report']);old_report['manifests']=b['report']['manifests'];assert old_report==b['report']
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
report=dict(status='PASS',parent=str(args.parent),parent_sha256=sha(args.parent),pack=str(args.child),pack_sha256=sha(args.child),content_identity=b['identity']['content_hash'],changed_maps=changed,changed_crossing_metatiles=8,footbridges=7,all_collision_quadrants_preserved=True,all_original_pixels_and_palettes_preserved=True,all_other_data_and_audio_exact=True,unchanged_maps=len(a['data']['maps'])-len(changed))
args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
