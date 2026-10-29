#!/usr/bin/env python3
"""Read-only native preservation, paired-mouth audit and whole Rock Cut review."""
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

ROOM = 'MnR05RockCut'
SOURCES = ROOT / 'planning/minnesota/caves/johto-quality-v01'
OLD_SHA = 'b2ec8e138d0174f535264fbd3dc4e6892171a74a89912a147c405e00effb9ff1'
NEW_SHA = '69956475c9e0afc210db345c8a08eb43c8be8f6c87e5f267924a42d4d19d1af2'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(source):
    return [int(word, 16) for row in source['rows'] for word in row.split()]


def collision(pack, module, x, y):
    attrs = module['attributes']
    block = module['blocks'][y // 2 * attrs['width'] + x // 2]
    return pack['data']['tilesets'][attrs['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Retain every prior review epoch.'
    old_path, new_path = [SOURCES / f'{ROOM}{suffix}.json' for suffix in ['.previous', '']]
    assert sha(old_path) == OLD_SHA and sha(new_path) == NEW_SHA
    old_source, new_source = [json.loads(p.read_text()) for p in [old_path, new_path]]
    before, actual = load_pack(args.parent), load_pack(args.pack)
    old, new = [p['data']['maps'][ROOM] for p in [before, actual]]
    assert old['blocks'] == rows(old_source) and new['blocks'] == rows(new_source)
    assert old['attributes']['border_block'] == 1 and new['attributes']['border_block'] == 0x2e
    assert new['attributes']['time_of_day'] == 'nite'
    assert [new['attributes'][k] for k in ['width', 'height']] == [8, 10]
    assert actual['data']['items']['REPEL']['repel_steps'] == 100
    assert actual['data']['tilesets']['cave']['collision']['2e'] == ['WALL'] * 4

    expected = deepcopy(before)
    expected['identity'] = actual['identity']
    expected['data']['maps'][ROOM]['blocks'] = new['blocks']
    expected['data']['maps'][ROOM]['attributes']['border_block'] = 0x2e
    expected['data']['map_attributes'][ROOM]['border_block'] = 0x2e
    label = old['attributes']['blocks_label']
    expected['data']['map_blocks'][label] = base64.b64encode(bytes(new['blocks'])).decode()
    assert expected == actual, 'Changed events, healer, rewards, indices or unrelated content.'

    maps = actual['data']['maps']
    constants = {m['attributes']['map_constant']: name for name, m in maps.items()}
    pairs = []
    for inside in new['events']['warps']:
        outside_name = constants[inside['target_map_constant']]
        outside = maps[outside_name]
        passive = next(w for w in outside['events']['warps'] if w['index'] == inside['target_warp_id'])
        active = [w for w in outside['events']['warps']
                  if w['target_map_constant'] == new['attributes']['map_constant']
                  and w['target_warp_id'] == inside['index']
                  and collision(actual, outside, w['x'], w['y']) == 'CAVE']
        assert len(active) == 1
        active = active[0]
        assert [passive['x'], passive['y']] == [active['x'], active['y'] + 1]
        assert collision(actual, outside, passive['x'], passive['y']) == 'FLOOR'
        assert collision(actual, new, inside['x'], inside['y']) == 'LADDER'
        assert collision(actual, new, inside['x'], inside['y'] - 1) == 'FLOOR'
        pairs.append(dict(inside=[inside['x'], inside['y']],
                          clear=[inside['x'], inside['y'] - 1],
                          outside=outside_name, active=[active['x'], active['y']],
                          passive=[passive['x'], passive['y']],
                          target_passive_index=inside['target_warp_id']))
    assert len(pairs) == 2
    assert {p['outside'] for p in pairs} == {'MnR05C00R00', 'MnR05C00R01'}
    width, height = [new['attributes'][k] * 2 for k in ['width', 'height']]
    assert all(collision(actual, new, x, y) == 'WALL'
               for y in range(height) for x in range(width)
               if x in (0, width - 1) or y in (0, height - 1))

    args.output.mkdir(parents=True)
    started = time.perf_counter()
    inspect(actual, SOURCES, args.output, True, ROOM)
    warm = time.perf_counter() - started
    art = SourceArt(before)
    prior = art.render('cave', 8, 10, old['blocks'], 'nite').convert('RGBA')
    sprite_pack = npc_palette_pack(before, 'nite')
    for ob in old['objects']:
        face = {'UP': 'north', 'LEFT': 'west', 'RIGHT': 'east'}.get(ob['spritemovedata'].rsplit('_', 1)[-1], 'south')
        prior.alpha_composite(oriented_sprite(sprite_pack, ob['sprite'], face), (ob['x'] * 16, ob['y'] * 16))
    save_render(prior, args.output / 'before-whole.png')
    revised = Image.open(args.output / f'{ROOM}-whole.png').convert('RGB')
    sheet = Image.new('RGB', (560, 384), '#17232c')
    draw = ImageDraw.Draw(sheet)
    draw.text((16, 12), 'R05 ROCK CUT - SAME NATIVE SCALE', fill='white')
    draw.text((16, 36), 'Parent: black void / straight neck', fill='#c9d5df')
    draw.text((288, 36), 'Revised: fitted stone / bent neck', fill='#c9d5df')
    sheet.paste(prior.convert('RGB'), (16, 56))
    sheet.paste(revised, (288, 56))
    save_render(sheet, args.output / 'whole-before-after.png')
    receipt = dict(status='PASS', parent=str(args.parent), parent_sha256=sha(args.parent),
                   pack=str(args.pack), pack_sha256=sha(args.pack), identity=actual['identity']['content_hash'],
                   previous_source_sha256=OLD_SHA, source_sha256=NEW_SHA,
                   changed_metatiles=sum(a != b for a, b in zip(old['blocks'], new['blocks'])),
                   unchanged_other_maps=len(maps) - 1, native_border_block='2e',
                   complete_normalized_native_equality=True,
                   all_native_events_healer_reward_programs_indexes_exact=True,
                   all_unowned_maps_species_art_audio_home_recovery_reports_exact=True,
                   paired_mouths=pairs, complete_outer_perimeter_solid=True,
                   source_review_after_pack_load_seconds=round(warm, 3),
                   scope='Static native preservation and paired-mouth evidence. Controller, healing, item effects and GPU checks are separate.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'audit.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
