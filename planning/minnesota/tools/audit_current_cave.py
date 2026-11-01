#!/usr/bin/env python3
"""Read-only preservation and whole-room review of one current cave child.

An explicit contract binds the native parent, both authored sources, water
count and every real entrance. This tool never writes a content pack.
"""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_cave_quality import inspect
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['contract', 'pack', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads(args.contract.read_text())
    paths = {name: ROOT / contract[name] for name in ['parent_pack', 'previous_source', 'revised_source']}
    for name, field in [('parent_pack', 'parent_sha256'), ('previous_source', 'previous_source_sha256'),
                        ('revised_source', 'revised_source_sha256')]:
        assert digest(paths[name]) == contract[field], ('contract drift', name)
    assert not args.output.exists(), 'Preserve every review epoch.'
    before, actual = load_pack(paths['parent_pack']), load_pack(args.pack)
    room = contract['room']
    old, new = [pack['data']['maps'][room] for pack in [before, actual]]
    flatten = lambda path: [int(word, 16) for row in json.loads(path.read_text())['rows'] for word in row.split()]
    assert old['blocks'] == flatten(paths['previous_source'])
    assert new['blocks'] == flatten(paths['revised_source'])
    attributes = new['attributes']
    width, height = contract['dimensions_metatiles']
    assert [attributes[key] for key in ['width', 'height']] == [width, height]
    assert attributes['tileset_name'] == 'cave' and attributes['time_of_day'] == 'nite'
    border = int(contract['border_block'], 16)
    assert attributes['border_block'] == border
    expected = deepcopy(before)
    expected['identity'] = actual['identity']
    expected['data']['maps'][room]['blocks'] = new['blocks']
    expected['data']['maps'][room]['attributes']['border_block'] = border
    expected['data']['map_attributes'][room]['border_block'] = border
    expected['data']['map_blocks'][old['attributes']['blocks_label']] = base64.b64encode(bytes(new['blocks'])).decode()
    assert expected == actual, 'Unowned native content changed.'

    def kind(module, x, y):
        a = module['attributes']
        block = module['blocks'][y // 2 * a['width'] + x // 2]
        return actual['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]

    assert actual['data']['tilesets']['cave']['collision'][f'{border:02x}'] == ['WALL'] * 4
    water = {(x, y) for y in range(height * 2) for x in range(width * 2) if kind(new, x, y) == 'WATER'}
    assert len(water) == contract['water_core_cells']
    assert all(0 < x < width * 2 - 1 and 0 < y < height * 2 - 1 for x, y in water)
    old_water = {(x, y) for y in range(height * 2) for x in range(width * 2) if kind(old, x, y) == 'WATER'}
    assert old_water <= water, 'Retain every existing spring cell.'
    for y in range(height * 2):
        for x in range(width * 2):
            if x in (0, width * 2 - 1) or y in (0, height * 2 - 1):
                permission = kind(new, x, y)
                if permission != 'WALL':
                    index = y // 2 * width + x // 2
                    assert old['blocks'][index] == new['blocks'][index] == 0x07
                    assert permission == kind(old, x, y) and permission in {'FLOOR', 'LADDER'}
    constants = {module['attributes']['map_constant']: name for name, module in actual['data']['maps'].items()}
    for mouth in contract['mouths']:
        inside = next(warp for warp in new['events']['warps'] if warp['index'] == mouth['inside_warp_index'])
        assert [inside['x'], inside['y']] == mouth['inside_core']
        assert constants.get(inside['target_map_constant'], inside['target_map_constant']) == mouth['exterior']
        assert inside['target_warp_id'] == mouth['passive_warp_index']
        outside = actual['data']['maps'][mouth['exterior']]
        active = next(warp for warp in outside['events']['warps'] if warp['index'] == mouth['active_warp_index'])
        passive = next(warp for warp in outside['events']['warps'] if warp['index'] == mouth['passive_warp_index'])
        assert [active['x'], active['y']] == mouth['active_core']
        assert [passive['x'], passive['y']] == mouth['passive_core']
        assert constants.get(active['target_map_constant'], active['target_map_constant']) == room
        assert active['target_warp_id'] == mouth['inside_warp_index']
        assert kind(outside, *mouth['active_core']) == 'CAVE'
        assert kind(outside, *mouth['passive_core']) == 'FLOOR'
    assert len(new['events']['warps']) == len(contract['mouths'])
    args.output.mkdir(parents=True)
    began = time.perf_counter()
    inspect(actual, paths['revised_source'].parent, args.output, True, room)
    elapsed = time.perf_counter() - began
    image = SourceArt(before).render('cave', width, height, old['blocks'], 'nite').convert('RGBA')
    sprites = npc_palette_pack(before, 'nite')
    for actor in old['objects']:
        face = {'UP': 'north', 'RIGHT': 'east', 'LEFT': 'west'}.get(actor['spritemovedata'].rsplit('_', 1)[-1], 'south')
        image.alpha_composite(oriented_sprite(sprites, actor['sprite'], face), (actor['x'] * 16, actor['y'] * 16))
    save_render(image, args.output / 'before-whole.png')
    revised = Image.open(args.output / f'{room}-whole.png').convert('RGB')
    sheet = Image.new('RGB', (width * 64 + 48, height * 32 + 72), '#17232c')
    draw = ImageDraw.Draw(sheet)
    draw.text((16, 12), room + ' - SAME NATIVE SCALE', fill='white')
    draw.text((16, 36), 'Exact parent', fill='#c9d5df')
    draw.text((width * 32 + 32, 36), 'Revised current child', fill='#c9d5df')
    sheet.paste(image.convert('RGB'), (16, 56))
    sheet.paste(revised, (width * 32 + 32, 56))
    save_render(sheet, args.output / 'whole-before-after.png')
    receipt = dict(status='PASS', pack=str(args.pack), pack_sha256=digest(args.pack),
                   identity=actual['identity']['content_hash'], parent_sha256=digest(paths['parent_pack']),
                   source_sha256=digest(paths['revised_source']), contract_sha256=digest(args.contract),
                   changed_metatiles=sum(a != b for a, b in zip(old['blocks'], new['blocks'])),
                   unchanged_other_maps=len(actual['data']['maps']) - 1, complete_normalized_native_equality=True,
                   water_core_cells=len(water), all_original_water_positions_retained=True,
                   enclosed_water_no_open_edge=True, real_passive_mouths=contract['mouths'],
                   source_loop_after_pack_load_seconds=round(elapsed, 3),
                   scope='Static native preservation/source evidence; actual controller and GPU results are separate.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'audit_current_cave.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
