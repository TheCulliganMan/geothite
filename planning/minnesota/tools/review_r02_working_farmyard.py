#!/usr/bin/env python3
"""Preview the whole early farm road and its proposed working garden."""
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
from review_selection_lineage import require_selected_ancestor

ROOT = Path(__file__).resolve().parents[3]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def graph(pack, module, hops=True):
    attrs = module['attributes']
    width, height = attrs['width'], attrs['height']
    collisions = pack['data']['tilesets'][attrs['tileset_name']]['collision']
    terrain = {(x, y): collisions[f"{module['blocks'][y//2*width+x//2]:02x}"][y%2*2+x%2]
               for y in range(height*2) for x in range(width*2)}
    bodies = {(o['x'], o['y']) for o in module['objects']}
    land = {'FLOOR', 'TALL_GRASS'} | ({'HOP_LEFT'} if hops else set())
    edges = {}
    for at, kind in terrain.items():
        if kind not in land or at in bodies:
            continue
        edges[at] = []
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            target = at[0]+dx, at[1]+dy
            if hops and kind == 'HOP_LEFT' and (dx, dy) == (-1, 0):
                target = target[0]-1, target[1]
            if target not in bodies and terrain.get(target) in land:
                edges[at].append(target)
    return terrain, edges


def minimum(edges, terrain, start, target, dry=False):
    costs, pending = {start: 0}, deque([start])
    while pending:
        at = pending.popleft()
        for q in edges.get(at, []):
            grass = int(terrain[q] == 'TALL_GRASS')
            if dry and grass:
                continue
            cost = costs[at]+grass
            if cost < costs.get(q, 10**9):
                costs[q] = cost
                (pending.append if grass else pending.appendleft)(q)
    return costs.get(target), set(costs)


def compose(pack, source):
    original = pack['data']['maps'][source['map']]
    assert sha(json.dumps(original, sort_keys=True, separators=(',', ':')).encode()) == source['native_module_sha256']
    attrs = original['attributes']
    assert [attrs['width'], attrs['height']] == source['dimensions_metatiles'] == [30, 22]
    assert attrs['tileset_name'] == source['tileset'] == 'mn_prairie_kanto'
    assert source['source_pixels_modified'] is False
    def parse(rows):
        assert len(rows) == 22 and all(len(r.split()) == 30 for r in rows)
        return [int(b, 16) for row in rows for b in row.split()]
    assert original['blocks'] == parse(source['before_rows'])
    revised = deepcopy(original)
    revised['blocks'] = parse(source['rows'])
    changes = [i for i, (a, b) in enumerate(zip(original['blocks'], revised['blocks'])) if a != b]
    assert len(changes) == source['changed_metatiles'] == 17
    assert all(3 <= i%30 <= 9 and 4 <= i//30 <= 10 for i in changes)
    for placement in source['placements']:
        x, y = placement['metatile']
        assert original['blocks'][y*30+x] == int(placement['before'], 16)
        assert revised['blocks'][y*30+x] == int(placement['after'], 16)
    actor = deepcopy(original['objects'][4])
    spec = source['new_actor']
    actor.update(sprite=spec['sprite'], object_identifier=spec['id'], script=spec['id'],
                 x=spec['tile'][0], y=spec['tile'][1], spritemovedata='SPRITEMOVEDATA_STANDING_DOWN')
    revised['objects'].append(actor)
    before, old_edges = graph(pack, original)
    terrain, edges = graph(pack, revised)
    _, ordinary = graph(pack, revised, hops=False)
    west, east = tuple(source['preserved_roads']['west']), tuple(source['preserved_roads']['east'])
    prices = []
    for a, b in [(west, east), (east, west)]:
        old, _ = minimum(old_edges, before, a, b)
        new, reached = minimum(edges, terrain, a, b)
        assert old == new and old is not None and old >= 6, ('main route grass changed', old, new)
        prices.append(dict(start=a, target=b, original=old, proposed=new))
    old_reached = minimum(old_edges, before, east, west)[1]
    assert {at for at in old_reached if terrain[at] in {'FLOOR', 'TALL_GRASS', 'HOP_LEFT'} and at != tuple(spec['tile'])} <= reached
    for obj in original['objects']:
        x, y = obj['x'], obj['y']
        assert terrain[x, y] == before[x, y] == 'FLOOR'
        assert any(at in reached for at in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    for post in original['events']['bg_events']:
        x, y = post['x'], post['y']
        assert terrain[x, y] == before[x, y]
        assert any(at in reached for at in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    assert terrain[tuple(spec['tile'])] == terrain[tuple(spec['approach'])] == 'FLOOR'
    assert minimum(ordinary, terrain, (11,12), tuple(spec['approach']), dry=True)[0] == 0
    for warp in original['events']['warps']:
        assert terrain[warp['x'], warp['y']] == before[warp['x'], warp['y']]
        assert (warp['x'], warp['y']) in reached or terrain[warp['x'], warp['y']] == 'DOOR'
    for at, old in before.items():
        if old in {'WATER', 'WATERFALL', 'HOP_LEFT', 'CUT_TREE'}:
            assert terrain[at] == old
    assert not any(kind in {'CUT_TREE', 'ICE', 'ICE_2B'} for kind in terrain.values())
    return original, revised, dict(main_grass_prices=prices, original_floor_and_actor_approaches_preserved=True,
                                   dry_farmhouse_to_grower=True, all_water_and_original_hop_quadrants_exact=True)


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=ROOT/'planning/minnesota/routes/r02-working-farmyard-v01/proposal.json')
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    assert not args.output.exists(), 'preserve all review passes'
    source = json.loads(args.source.read_text())
    world = json.loads((ROOT/'planning/minnesota/world.json').read_text())
    require_selected_ancestor(world, 'north_shore_passing_width_revision',
                              'e951fddafee3a0c09a2356d4dd683890ac498e23daeb266d08c5f12d79ec1167')
    parent = world['r16_seed_bank_revision']
    assert parent['pack'] == source['parent_pack'] and parent['pack_sha256'] == source['parent_pack_sha256']
    if world['active_local_review_revision'] != 'r16_seed_bank_revision':
        selected = world[world['active_local_review_revision']]
        recipe = json.loads((ROOT/selected['build_recipe']).read_text())
        assert any(stage['kind'] == 'r02_working_farmyard'
                   and any(binding['sha256'] == sha(args.source.read_bytes()) for binding in stage['sources'])
                   for stage in recipe['stages']), 'selected continuation does not include this farmyard source'
    pack_path = ROOT/source['parent_pack']
    assert sha(pack_path.read_bytes()) == source['parent_pack_sha256']
    pack = load_pack(pack_path)
    started = time.perf_counter()
    old, new, report = compose(pack, source)
    args.output.mkdir(parents=True)
    wholes = []
    for label, module in [('before', old), ('proposal', new)]:
        image = native_room(module, pack, archive=args.output/label, native_origin=True)
        save_render(image, args.output/(label+'-whole-original.png'))
        views = []
        for view in source['review_views']:
            x, y = view['core']
            crop = image.crop((x*16-80,y*16-72,x*16+80,y*16+72))
            save_render(crop,args.output/label/(view['id']+'-source160.png'))
            views.append((view['id'],crop))
        save_render(room_sheet(views,columns=3),args.output/(label+'-decisions.png'))
        wholes.append(image)
    combined = Image.new('RGB',(960,1408))
    combined.paste(wholes[0],(0,0));combined.paste(wholes[1],(0,704))
    save_render(combined,args.output/'before-after-whole-original.png')
    report.update(status='PASS_SOURCE_ONLY', source=str(args.source), source_sha256=sha(args.source.read_bytes()),
                  seconds_after_load=round(time.perf_counter()-started,3),
                  scope='Complete proposed terrain and new actor anchor only. Reward program, native publication/input/capacity/Continue and visual acceptance remain required.')
    (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
