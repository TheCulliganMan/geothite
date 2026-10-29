#!/usr/bin/env python3
"""Read-only shelter preservation, spring, compact-mouth and native-art review."""
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
from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_cave_quality import inspect
from review_north_shore import oriented_sprite

ROOM = 'MnR12Cave1F'
SOURCE = ROOT / 'planning/minnesota/caves/johto-quality-v01/MnR12Cave1F.json'
PARENT_HASHES = {
    '133226dbc5b8c164e5fbf17a475093acd6a26836394e5fe73d2f9bf763749097',
    '67fb7ee40c022f0de78b561bbb144767d1483fe97a7dba9a2e6ec111d8c3360e',
}
PREVIOUS_SHA = 'bf39bc3d09073561fafef7053d66b09ce675b5227d84d5437a492916ec9357df'
SOURCE_SHA = '52a29500418da67532498a74aa72742017fe09c21fc756bce03532a23494218f'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blocks(source):
    return [int(word, 16) for row in source['rows'] for word in row.split()]


def kind(pack, module, at):
    x, y = at
    a = module['attributes']
    block = module['blocks'][y // 2 * a['width'] + x // 2]
    return pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]


def cells(pack, module, permission):
    a = module['attributes']
    return {(x, y) for y in range(a['height'] * 2) for x in range(a['width'] * 2)
            if kind(pack, module, (x, y)) == permission}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent', type=Path, default=ROOT / 'content-packs/minnesota-r05-rock-cut-v01.crystalpack')
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Keep every review epoch.'
    parent = args.parent
    previous = SOURCE.with_name('MnR12Cave1F.previous.json')
    parent_sha = sha(parent)
    assert parent_sha in PARENT_HASHES and sha(previous) == PREVIOUS_SHA and sha(SOURCE) == SOURCE_SHA
    before, actual = load_pack(parent), load_pack(args.pack)
    old, new = [p['data']['maps'][ROOM] for p in [before, actual]]
    assert old['blocks'] == blocks(json.loads(previous.read_text()))
    assert new['blocks'] == blocks(json.loads(SOURCE.read_text()))
    assert [new['attributes'][k] for k in ['width', 'height']] == [16, 12]
    assert new['attributes']['time_of_day'] == 'nite'
    assert new['attributes']['border_block'] == 0x2e
    expected = deepcopy(before)
    expected['identity'] = actual['identity']
    expected['data']['maps'][ROOM]['blocks'] = new['blocks']
    expected['data']['maps'][ROOM]['attributes']['border_block'] = 0x2e
    expected['data']['map_attributes'][ROOM]['border_block'] = 0x2e
    expected['data']['map_blocks'][old['attributes']['blocks_label']] = base64.b64encode(bytes(new['blocks'])).decode()
    assert expected == actual, 'Native programs, events or unrelated content changed.'

    water = cells(actual, new, 'WATER')
    assert water == cells(before, old, 'WATER') and len(water) == 32
    assert all(x not in (0, 31) and y not in (0, 23) for x, y in water)
    assert old['blocks'][11 * 16 + 8] == new['blocks'][11 * 16 + 8] == 0x07
    for y in range(24):
        for x in range(32):
            if x in (0, 31) or y in (0, 23):
                # Native07 retains its two-tile mouth and one active ladder.
                expected_permission = {(16, 23): 'FLOOR', (17, 23): 'LADDER'}.get((x, y), 'WALL')
                assert kind(actual, new, (x, y)) == expected_permission, (x, y)
                if (x, y) in {(16, 23), (17, 23)}:
                    assert kind(before, old, (x, y)) == expected_permission
    assert cells(actual, new, 'LADDER') == {(17, 23)}
    assert kind(actual, new, (14, 12)) == kind(actual, new, (15, 12)) == 'WALL'
    assert kind(actual, new, (14, 13)) == kind(actual, new, (15, 13)) == 'FLOOR'
    assert cells(actual, new, 'HOP_DOWN') == {(14, 14), (15, 14)}

    warp = new['events']['warps'][0]
    assert (warp['x'], warp['y'], warp['target_map_constant'], warp['target_warp_id']) == (17, 23, 'MN_R12_C00R00', 1)
    outside = actual['data']['maps']['MnR12C00R00']
    active = outside['events']['warps'][0]
    assert (active['index'], active['x'], active['y'], active['target_map_constant'], active['target_warp_id']) == (1, 40, 15, 'MN_R12_CAVE_1F', 1)
    assert kind(actual, outside, (40, 15)) == 'CAVE'
    assert kind(actual, outside, (40, 16)) == 'FLOOR'
    args.output.mkdir(parents=True)
    start = time.perf_counter()
    inspect(actual, SOURCE.parent, args.output, True, ROOM)
    elapsed = time.perf_counter() - start
    art = SourceArt(before)
    prior = art.render('cave', 16, 12, old['blocks'], 'nite').convert('RGBA')
    sprite_pack = npc_palette_pack(before, 'nite')
    for ob in old['objects']:
        face = {'UP': 'north', 'LEFT': 'west', 'RIGHT': 'east'}.get(ob['spritemovedata'].rsplit('_', 1)[-1], 'south')
        prior.alpha_composite(oriented_sprite(sprite_pack, ob['sprite'], face), (ob['x'] * 16, ob['y'] * 16))
    prior = prior.convert('RGB')
    save_render(prior, args.output / 'before-whole.png')
    revised = Image.open(args.output / f'{ROOM}-whole.png').convert('RGB')
    assert prior.size == revised.size == (512, 384)
    sheet = Image.new('RGB', (1072, 456), '#17232c')
    draw = ImageDraw.Draw(sheet)
    draw.text((16, 12), 'R12 GUNFLINT SHELTER - SAME NATIVE SCALE', fill='white')
    draw.text((16, 36), 'Parent: broad crossing / open black ceiling', fill='#c9d5df')
    draw.text((544, 36), 'Revised: fitted roof / extended shoulder / enclosed stone', fill='#c9d5df')
    sheet.paste(prior, (16, 56))
    sheet.paste(revised, (544, 56))
    save_render(sheet, args.output / 'whole-before-after.png')
    receipt = dict(status='PASS', pack=str(args.pack), pack_sha256=sha(args.pack),
                   identity=actual['identity']['content_hash'], parent_sha256=parent_sha,
                   source_sha256=SOURCE_SHA, previous_source_sha256=PREVIOUS_SHA,
                   changed_metatiles=sum(a != b for a, b in zip(old['blocks'], new['blocks'])),
                   unchanged_other_maps=len(actual['data']['maps']) - 1,
                   complete_normalized_native_equality=True,
                   retained_water_core_cells=32, all_water_positions_exact=True,
                   solid_perimeter_except_retained_native07_aperture=True,
                   retained_mouth_edge_floor=[16, 23], enclosed_water_no_open_edge=True,
                   attached_shoulder_metatile=[7, 6], both_drop_approaches_clear=True,
                   actual_exterior='MnR12C00R00', active_mouth=[40, 15], safe_ground=[40, 16],
                   native_exit_targets_active_index=1, source_loop_after_pack_load_seconds=round(elapsed, 3),
                   scope='Static preservation/source evidence. Actual traversal, items, persistence and GPU results are separate.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'audit.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
