#!/usr/bin/env python3
"""Read-only whole-pack preservation and native spring-shelter review."""
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


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads(Path(__file__).with_name('native.contract.json').read_text())
    parent = ROOT / contract['parent_pack']
    previous = ROOT / contract['previous_source']
    source = ROOT / contract['revised_source']
    for path, field in [(parent, 'parent_sha256'), (previous, 'previous_source_sha256'),
                        (source, 'revised_source_sha256')]:
        assert sha(path) == contract[field], ('contract input drift', path)
    assert not args.output.exists(), 'Preserve every review epoch.'
    before, actual = load_pack(parent), load_pack(args.pack)
    room = contract['room']
    old, new = [pack['data']['maps'][room] for pack in [before, actual]]
    flatten = lambda path: [int(word, 16) for row in json.loads(path.read_text())['rows']
                           for word in row.split()]
    assert old['blocks'] == flatten(previous)
    assert new['blocks'] == flatten(source)
    assert [new['attributes'][key] for key in ['width', 'height']] == [10, 8]
    assert new['attributes']['time_of_day'] == 'nite'
    assert new['attributes']['border_block'] == 0x2e
    expected = deepcopy(before)
    expected['identity'] = actual['identity']
    expected['data']['maps'][room]['blocks'] = new['blocks']
    expected['data']['maps'][room]['attributes']['border_block'] = 0x2e
    expected['data']['map_attributes'][room]['border_block'] = 0x2e
    expected['data']['map_blocks'][old['attributes']['blocks_label']] = base64.b64encode(bytes(new['blocks'])).decode()
    assert expected == actual, 'Unowned native content changed.'
    collision = actual['data']['tilesets']['cave']['collision']

    def kind(module, x, y):
        block = module['blocks'][y // 2 * module['attributes']['width'] + x // 2]
        return collision[f'{block:02x}'][y % 2 * 2 + x % 2]

    water = {(x, y) for y in range(16) for x in range(20) if kind(new, x, y) == 'WATER'}
    assert len(water) == 16
    assert all(0 < x < 19 and 0 < y < 15 for x, y in water)
    for y in range(16):
        for x in range(20):
            if x in (0, 19) or y in (0, 15):
                permission = {(6, 15): 'FLOOR', (7, 15): 'LADDER'}.get((x, y), 'WALL')
                assert kind(new, x, y) == permission, ('perimeter', x, y)
                if (x, y) in {(6, 15), (7, 15)}:
                    assert kind(old, x, y) == permission
    inside = new['events']['warps'][0]
    outside = actual['data']['maps'][contract['exterior']]
    assert (inside['x'], inside['y'], inside['target_warp_id']) == (7, 15, 2)
    passive = next(warp for warp in outside['events']['warps'] if warp['index'] == 2)
    active = next(warp for warp in outside['events']['warps']
                  if warp['target_map'] == room and (warp['x'], warp['y']) != (passive['x'], passive['y']))
    assert (active['x'], active['y'], active['target_warp_id']) == (14, 5, 1)
    assert (passive['x'], passive['y']) == (14, 6)
    outside_collision = actual['data']['tilesets'][outside['attributes']['tileset_name']]['collision']
    for at, permission in [((14, 5), 'CAVE'), ((14, 6), 'FLOOR')]:
        x, y = at
        block = outside['blocks'][y // 2 * outside['attributes']['width'] + x // 2]
        assert outside_collision[f'{block:02x}'][y % 2 * 2 + x % 2] == permission
    args.output.mkdir(parents=True)
    began = time.perf_counter()
    inspect(actual, source.parent, args.output, True, room)
    elapsed = time.perf_counter() - began
    image = SourceArt(before).render('cave', 10, 8, old['blocks'], 'nite').convert('RGBA')
    sprites = npc_palette_pack(before, 'nite')
    for actor in old['objects']:
        image.alpha_composite(oriented_sprite(sprites, actor['sprite'], 'south'),
                              (actor['x'] * 16, actor['y'] * 16))
    save_render(image, args.output / 'before-whole.png')
    revised = Image.open(args.output / f'{room}-whole.png').convert('RGB')
    sheet = Image.new('RGB', (688, 328), '#17232c')
    draw = ImageDraw.Draw(sheet)
    draw.text((16, 12), 'R13 CEDAR SHELTER - EXACT NATIVE SCALE', fill='white')
    draw.text((16, 36), 'Parent: black void / bare entry', fill='#c9d5df')
    draw.text((352, 36), 'Revised: enclosed stone / spring bank', fill='#c9d5df')
    sheet.paste(image.convert('RGB'), (16, 56))
    sheet.paste(revised, (352, 56))
    save_render(sheet, args.output / 'whole-before-after.png')
    receipt = dict(status='PASS', pack=str(args.pack), pack_sha256=sha(args.pack),
                   identity=actual['identity']['content_hash'], parent_sha256=sha(parent),
                   previous_source_sha256=sha(previous), source_sha256=sha(source),
                   changed_metatiles=sum(a != b for a, b in zip(old['blocks'], new['blocks'])),
                   unchanged_other_maps=len(actual['data']['maps']) - 1,
                   complete_normalized_native_equality=True, spring_water_core_cells=16,
                   enclosed_water_no_open_edge=True, retained_native_mouth_aperture=True,
                   actual_exterior=contract['exterior'], active_mouth=[14, 5], passive_return=[14, 6],
                   source_loop_after_pack_load_seconds=round(elapsed, 3),
                   scope='Static preservation and native-source evidence. Production gameplay and GPU captures are separate.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'audit.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
