#!/usr/bin/env python3
"""Inspect explicit prairie seed-bank rows using native collision and source art."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet
from review_r19_prairie_approach import MASKS
from review_selection_lineage import require_selected_ancestor

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/routes/r16-seed-bank-v01/proposal.json'
LAND = {'FLOOR', 'TALL_GRASS', 'HOP_DOWN', *MASKS}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def graph(pack, modules, hops=True):
    terrain, bodies = {}, set()
    for module in modules:
        a = module['attributes']
        ox = int(module['id'][-5:-3]) * 64
        collision = pack['data']['tilesets'][a['tileset_name']]['collision']
        for y in range(a['height'] * 2):
            for x in range(a['width'] * 2):
                block = module['blocks'][y // 2 * a['width'] + x // 2]
                terrain[x + ox, y] = collision[f'{block:02x}'][y % 2 * 2 + x % 2]
        bodies |= {(o['x'] + ox, o['y']) for o in module['objects']}
    edges = {}
    for at, kind in terrain.items():
        if kind not in LAND or at in bodies:
            continue
        neighbors = []
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            target = at[0] + dx, at[1] + dy
            if hops and kind == 'HOP_DOWN' and (dx, dy) == (0, 1):
                target = target[0], target[1] + 1
            destination = terrain.get(target, 'WALL')
            if target in bodies or destination not in LAND:
                continue
            if (dx, dy) in MASKS.get(destination, set()) or (-dx, -dy) in MASKS.get(kind, set()):
                continue
            neighbors.append(target)
        edges[at] = neighbors
    return terrain, edges


def minimum(edges, terrain, start, end, grass=True, forbidden=frozenset()):
    costs, pending = {start: 0}, deque([start])
    while pending:
        at = pending.popleft()
        for target in edges.get(at, []):
            if target in forbidden:
                continue
            step = int(terrain[target] == 'TALL_GRASS') if grass else 1
            cost = costs[at] + step
            if cost < costs.get(target, 10**9):
                costs[target] = cost
                (pending.append if step else pending.appendleft)(target)
    return costs.get(end), costs


def compose(pack, source):
    old, new = [], []
    for record in source['maps']:
        module = pack['data']['maps'][record['id']]
        assert sha(json.dumps(module, sort_keys=True, separators=(',', ':')).encode()) == record['native_module_sha256']
        a = module['attributes']
        width, height = record['dimensions_metatiles']
        assert [a['width'], a['height']] == [width, height] and a['tileset_name'] == record['tileset']

        def parse(rows):
            assert len(rows) == height and all(len(row.split()) == width for row in rows)
            return [int(word, 16) for row in rows for word in row.split()]

        assert module['blocks'] == parse(record['before_rows'])
        revised = deepcopy(module)
        revised['blocks'] = parse(record['rows'])
        changes = [i for i, (a, b) in enumerate(zip(module['blocks'], revised['blocks'])) if a != b]
        assert len(changes) == record['changed_metatiles']
        if record['id'] == 'MnR16C00R00':
            assert all(18 <= i % 32 <= 25 and 11 <= i // 32 <= 16 for i in changes)
        else:
            assert not changes
        old.append(module)
        new.append(revised)
    before, _ = graph(pack, old)
    terrain, edges = graph(pack, new)
    _, ordinary = graph(pack, new, hops=False)
    west, east = (0, 24), (95, 24)
    _, reached = minimum(edges, terrain, west, east)
    assert east in reached and west in minimum(edges, terrain, east, west)[1]
    for at in terrain:
        assert terrain[at] not in {'CUT_TREE', 'ICE', 'ICE_2B'}
        if before[at] in {'WATER', 'WATERFALL', 'BUOY', *MASKS}:
            assert terrain[at] == before[at], ('retained shore/bridge', at)
    for module in new:
        ox = int(module['id'][-5:-3]) * 64
        for obj in module['objects']:
            x, y = obj['x'] + ox, obj['y']
            assert terrain[x, y] == before[x, y] == 'FLOOR'
            assert any(q in reached for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
        for obj in module['events']['bg_events']:
            x, y = obj['x'] + ox, obj['y']
            assert terrain[x, y] == before[x, y]
            assert any(q in reached for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    report = []
    for lane in source['new_ledge']['lanes']:
        upper, lip, landing, wall = [tuple(lane[k]) for k in ['upper', 'lip', 'landing', 'reverse_wall']]
        assert terrain[upper] == terrain[landing] == 'FLOOR'
        assert terrain[lip] == 'HOP_DOWN' and terrain[wall] == 'WALL'
        downhill, _ = minimum(edges, terrain, upper, landing)
        uphill, _ = minimum(ordinary, terrain, landing, upper)
        distance, _ = minimum(ordinary, terrain, landing, upper, grass=False)
        assert downhill == source['new_ledge']['expected_dry_hop_grass']
        assert uphill is not None and uphill >= source['new_ledge']['expected_minimum_ordinary_return_grass'], ('dry uphill bypass', lane, uphill)
        assert distance is not None and distance >= 9
        report.append(dict(upper=upper, lip=lip, landing=landing, uphill_grass=uphill,
                           downhill_grass=downhill, ordinary_return_steps=distance))
    for at in source['new_ledge']['side_barriers']:
        assert terrain[tuple(at)] == 'HEADBUTT_TREE'
    for lane in source['retained_hayfield_lanes']:
        for key in ['upper', 'lip', 'landing', 'reverse_wall']:
            assert terrain[tuple(lane[key])] == before[tuple(lane[key])]
    crossing = source['preserved_cottonwood_crossing']
    x0, y0, x1, y1 = crossing['band']
    band = {(x,y) for x in range(x0,x1) for y in range(y0,y1)}
    assert all(terrain[at] == 'TALL_GRASS' for at in band)
    assert east not in minimum(edges, terrain, west, east, forbidden=band)[1]
    return old, new, dict(new_ledge_lanes=report,
                         minimum_main_grass_west_to_east=minimum(edges, terrain, west, east)[0],
                         minimum_main_grass_east_to_west=minimum(edges, terrain, east, west)[0])


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=SOURCE)
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    assert not args.output.exists(), 'retain prior outputs'
    source = json.loads(args.source.read_text())
    require_selected_ancestor(json.loads((ROOT / 'planning/minnesota/world.json').read_text()),
                              source['selected_ancestor_revision'], source['selected_ancestor_pack_sha256'])
    packpath = ROOT / source['parent_pack']
    assert sha(packpath.read_bytes()) == source['parent_pack_sha256']
    pack = load_pack(packpath)
    started = time.perf_counter()
    old, new, report = compose(pack, source)
    args.output.mkdir(parents=True)
    pair = []
    for label, modules in [('before', old), ('after', new)]:
        image = Image.new('RGB', (48 * 32, 24 * 32), (18, 25, 28))
        for module in modules:
            ox = int(module['id'][-5:-3]) * 1024
            image.paste(native_room(module, pack, archive=args.output / label / module['id'], native_origin=True), (ox, 0))
        save_render(image, args.output / f'{label}-whole-original.png')
        cameras = []
        for view in source['review_views']:
            x, y = view['core']
            crop = image.crop((x * 16 - 80, y * 16 - 72, x * 16 + 80, y * 16 + 72))
            save_render(crop, args.output / label / (view['id'] + '-source160.png'))
            cameras.append((view['id'], crop))
        save_render(room_sheet(cameras, columns=3), args.output / (label + '-decisions.png'))
        pair.append(image)
    combined = Image.new('RGB', (48 * 32, 24 * 32 * 2), (18, 25, 28))
    combined.paste(pair[0], (0, 0)); combined.paste(pair[1], (0, 24 * 32))
    save_render(combined, args.output / 'before-after-whole-original.png')
    report.update(status='PASS_SOURCE_ONLY', source=str(args.source), source_sha256=sha(args.source.read_bytes()),
                  seconds_after_load=round(time.perf_counter()-started, 3),
                  scope='Native collision graph with directional wall masks, authored bodies and optional downward hops; production input and visual acceptance remain separate.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
