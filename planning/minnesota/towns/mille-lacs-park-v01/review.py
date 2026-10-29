#!/usr/bin/env python3
"""Fast read-only park source review and exact current-parent preservation audit."""
import argparse
from collections import deque
import hashlib
import heapq
import json
from pathlib import Path
import sys
import time

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_north_shore import oriented_sprite

MAP = 'MnMilleLacs'
ROUTE = 'MnR18C01R00'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def differences(old, new, path=''):
    if old == new:
        return
    if isinstance(old, dict) and isinstance(new, dict):
        assert old.keys() <= new.keys(), ('removed original data', path)
        for key in old:
            yield from differences(old[key], new[key], path + '/' + key)
        for key in new.keys() - old.keys():
            yield path + '/' + key, None, new[key]
    else:
        yield path, old, new


def preserve(parent, pack):
    changes = []
    for path, old, new in differences(parent, pack):
        parts = path.split('/')
        owned = False
        if path == '/identity/content_hash':
            owned = True
        elif path == '/identity/maps':
            owned = new == old + 1
        elif path == '/identity/runtime_modpack_id':
            owned = new == old + '+minnesota-mille-lacs-park-v1-0+minnesota-mille-lacs-park-v1'
        elif path in ['/data/maps/' + ROUTE + '/attributes/connections',
                       '/data/map_attributes/' + ROUTE + '/connections']:
            owned = old + [dict(direction='east', target_map=MAP, offset=-1)] == new
        elif path in ['/data/maps/' + ROUTE + '/attributes/connection_flags',
                       '/data/map_attributes/' + ROUTE + '/connection_flags']:
            owned = old == 'WEST' and new == 'WEST | EAST'
        elif old is None and parts[:2] == ['', 'data'] and len(parts) == 4:
            table, key = parts[2:]
            if table in ['maps', 'map_attributes', 'npcs', 'wild_encounters']:
                owned = key == MAP
            elif table in ['runtime_map_metadata', 'map_dimensions']:
                owned = key == 'MN_MILLE_LACS'
            elif table == 'map_blocks':
                owned = key == MAP + '_Blocks'
            elif table == 'map_scripts':
                owned = key.startswith(('MN_ML_PARK_', 'MnMilleLacs_Map'))
            elif table == 'runtime_spawn_points':
                owned = new.get('mapName', new.get('map_name')) == MAP
        elif old is None and path.startswith('/data/story_event_script_constants/global/'):
            owned = parts[-1] in ['EVENT_MN_MILLE_LACS_ROOST', 'MAPGROUP_MINNESOTA_MILLE_LACS_PARK']
        elif path == '/data/pokegear_landmarks/map_to_landmark/' + MAP:
            owned = old is None
        elif path == '/data/pokegear_landmarks/landmarks':
            owned = new[:len(old)] == old and all(v['constant'] == 'LANDMARK_MN_MILLE_LACS_PARK'
                                                 for v in new[len(old):])
        elif path == '/report/maps':
            owned = new == old + 1
        elif path == '/report/manifests':
            owned = new == old + ['minnesota-mille-lacs-park-v1-0', 'minnesota-mille-lacs-park-v1']
        elif path in ['/report/reachable_maps', '/report/solvable_maps']:
            owned = new == old + [MAP]
        assert owned, ('unowned pack change', path)
        changes.append(path)
    return changes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source', type=Path, default=Path(__file__).parent)
    parser.add_argument('--published', action='store_true')
    args = parser.parse_args()
    town = json.loads((args.source / 'town.json').read_text())
    layout = json.loads((args.source / 'layout.json').read_text())
    contract = json.loads((args.source / 'native.contract.json').read_text())
    assert digest(args.source / 'town.json') == contract['town_sha256']
    assert digest(args.source / 'layout.json') == contract['layout_sha256']
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    changes = []
    if args.published:
        parent_path = ROOT / contract['parent_pack']
        assert digest(parent_path) == contract['parent_pack_sha256']
        changes = preserve(load_pack(parent_path), pack)
    began = time.perf_counter()
    blocks = layout_blocks(layout)
    if args.published:
        native = pack['data']['maps'][MAP]
        assert native['blocks'] == blocks
        assert native['attributes']['border_block'] == 5
    collision = pack['data']['tilesets']['mn_headwaters']['collision']
    kind = lambda p: collision[f'{blocks[p[1] // 2 * 24 + p[0] // 2]:02x}'][p[1] % 2 * 2 + p[0] % 2]
    actors = town['residents'] + town['items']
    occupied = {tuple(a['tile']) for a in actors}
    floor = {(x, y) for x in range(48) for y in range(40)
             if kind((x, y)) in ['FLOOR', 'TALL_GRASS'] and (x, y) not in occupied}
    water = {(x, y) for x in range(48) for y in range(40) if kind((x, y)) == 'WATER'}

    def flood(start, allowed):
        seen, queue = {start}, deque([start])
        while queue:
            x, y = queue.popleft()
            for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if at in allowed and at not in seen:
                    seen.add(at)
                    queue.append(at)
        return seen

    walk, surf = flood((0, 12), floor), flood((0, 12), floor | water)
    assert all(tuple(a['approach']) in walk and kind(tuple(a['tile'])) == 'FLOOR'
               for a in town['residents'])
    assert floor <= surf
    assert all(34 <= x <= 37 and 28 <= y <= 31 for x, y in floor - walk)
    assert (37, 29) in surf - walk
    assert {p for p in surf if p[0] in [0, 47] or p[1] in [0, 39]} == {(0, 12), (0, 13)}
    assert not any('CUT_TREE' in collision[f'{b:02x}'] for b in blocks)
    no_grass = {p for p in floor if kind(p) != 'TALL_GRASS'}
    assert (16, 16) not in flood((0, 12), no_grass)
    assert (16, 16) not in flood((0, 12), no_grass | water)
    for x in [16, 17]:
        assert kind((x, 18)) == 'HOP_DOWN' and kind((x, 19)) == 'WALL'
        assert (x, 17) in walk and (x, 20) in walk
    assert all(kind(p) == 'WALL' for p in [(15, 18), (15, 19), (18, 18), (18, 19)])
    queue, costs = [(0, (0, 12))], {(0, 12): 0}
    while queue:
        cost, (x, y) = heapq.heappop(queue)
        if cost != costs[(x, y)]:
            continue
        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if at in floor:
                next_cost = cost + int(kind(at) == 'TALL_GRASS')
                if next_cost < costs.get(at, 10**9):
                    costs[at] = next_cost
                    heapq.heappush(queue, (next_cost, at))
    art = SourceArt(pack)
    scene = art.render('mn_headwaters', 24, 20, blocks, 'day').convert('RGBA')
    for actor in actors:
        face = actor.get('facing', 'south')
        scene.alpha_composite(oriented_sprite(pack, actor.get('sprite', 'SPRITE_POKE_BALL'), face),
                              (actor['tile'][0] * 16, actor['tile'][1] * 16))
    save_render(scene, args.output / 'whole-park.png')
    padded = Image.new('RGB', (scene.width + 192, scene.height + 192))
    border = art.blocks('mn_headwaters')[5]
    for y in range(0, padded.height, 32):
        for x in range(0, padded.width, 32):
            padded.paste(border, (x, y))
    padded.paste(scene, (96, 96))
    views = []
    for view in town['viewpoints']:
        x, y = view['tile']
        assert (x, y) in surf
        crop = padded.crop((96 + x * 16 - 72, 96 + y * 16 - 64,
                            96 + x * 16 + 88, 96 + y * 16 + 80))
        save_render(crop, args.output / (view['id'] + '-160.png'))
        views.append((view['id'], crop))
    save_render(room_sheet(views, columns=3), args.output / 'player-scale-source-views.png')
    receipt = dict(status='PASS', pack=str(args.pack), pack_sha256=digest(args.pack),
                   source_sha256=contract['town_sha256'], layout_sha256=contract['layout_sha256'],
                   published_rows_match=args.published, normalized_native_change_paths=changes,
                   original_maps_preserved_except_declared_r18_connection=args.published,
                   ordinary_reachable_tiles=len(walk), water_tiles=len(water),
                   minimum_grass_steps_to_lookout=costs[(16, 16)], grassless_surf_bypass=False,
                   two_native_drop_lanes=True, blocking_dock_and_water_edges=True,
                   surf_only_roost=True, incidental_cut_trees=0,
                   warm_source_loop_seconds=round(time.perf_counter() - began, 3),
                   scope='Read-only source/static/native preservation evidence; production controller and GPU results are separate.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'review.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
