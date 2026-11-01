#!/usr/bin/env python3
"""Read the current native river; plan inputs without changing or writing packs."""
import argparse
from collections import Counter, deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path[:0] = [str(ROOT / 'planning/minnesota/tools'),
               str(ROOT / 'planning/minnesota/connections/river-continuity-long-v01')]
from audit_outdoor_connections import collision, project, OPPOSITE
from physical_proof import water_step
from render_archive import save_render
from render_progress_atlas import font, native_room
from render_reference import load_pack
from review_mississippi_connections import bfs_water, input_plan

DIRECTIONS = {'left': (-1, 0, 'west'), 'right': (1, 0, 'east'),
              'up': (0, -1, 'north'), 'down': (0, 1, 'south')}


def bridges(grids):
    result = []
    for name, grid in sorted(grids.items()):
        remaining = {p for p, kind in grid.items() if kind.startswith('BRIDGE_')}
        while remaining:
            first = min(remaining)
            connected, todo = {first}, deque([first])
            remaining.remove(first)
            while todo:
                x, y = todo.popleft()
                for dx, dy, _ in DIRECTIONS.values():
                    q = x + dx, y + dy
                    if q in remaining:
                        remaining.remove(q)
                        connected.add(q)
                        todo.append(q)
            x0, y0 = min(x for x, y in connected), min(y for x, y in connected)
            x1, y1 = max(x for x, y in connected) + 1, max(y for x, y in connected) + 1
            result.append(dict(id=f'{name}-{x0}-{y0}', map=name,
                               metatile_bounds_exclusive=[x0 // 2, y0 // 2, x1 // 2, y1 // 2],
                               actual_bridge_quadrants=len(connected)))
    return result


def reachable(maps, grid, start):
    """Follow actual water permissions, bridges and reciprocal map seams."""
    seen, todo, mouths, unbound = {start}, deque([start]), set(), set()
    while todo:
        name, x, y = todo.popleft()
        a = maps[name]['attributes']
        for dx, dy, direction in DIRECTIONS.values():
            q = x + dx, y + dy
            target = name
            if not (0 <= q[0] < a['width'] * 2 and 0 <= q[1] < a['height'] * 2):
                edge = next((e for e in a['connections'] if e['direction'] == direction), None)
                if edge is None or edge['target_map'] not in maps:
                    unbound.add((name, x, y, direction))
                    continue
                target = edge['target_map']
                q = project(a, maps[target]['attributes'], edge, (x, y))
                if grid[target].get(q) is None:
                    unbound.add((name, x, y, direction))
                    continue
                if water_step(grid[name][x, y], grid[target][q], direction):
                    inverse = next((e for e in maps[target]['attributes']['connections']
                                    if e['direction'] == OPPOSITE[direction]
                                    and e['target_map'] == name), None)
                    assert inverse and project(maps[target]['attributes'], a, inverse, q) == (x, y)
                    mouths.add((name, x, y, target, *q))
            if grid[target].get(q) is None or not water_step(grid[name][x, y], grid[target][q], direction):
                continue
            node = target, *q
            if node not in seen:
                seen.add(node)
                todo.append(node)
    return seen, mouths, unbound


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack', type=Path, required=True)
    ap.add_argument('--expected-sha256', required=True)
    ap.add_argument('--atlas-audit', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    digest = hashlib.sha256(args.pack.read_bytes()).hexdigest()
    assert digest == args.expected_sha256
    assert not args.output.exists(), 'Retain every review epoch.'
    args.output.mkdir(parents=True)
    pack = load_pack(args.pack)
    actual = pack['data']['maps']
    maps = {n: m for n, m in actual.items() if n.startswith('Mn')}
    # The only planning override is the already authored badge callback. The
    # controller must independently run that callback with legal prerequisites.
    city = deepcopy(maps['MnMinneapolis'])
    maps['MnMinneapolis'] = city
    gates = [r for r in city['script_block_changes']
             if r['source_script'] == 'MnMPUpperPoolBuoyOpen']
    assert [(r['x'], r['y'], r['block_id']) for r in gates] == [(32, 2, 88), (34, 2, 53), (36, 2, 89)]
    for row in gates:
        city['blocks'][(row['y'] // 2) * city['attributes']['width'] + row['x'] // 2] = row['block_id']
    native = {n: collision(pack, m) for n, m in maps.items()}
    planned, axes = deepcopy(native), {}
    for name, grid in native.items():
        for point, permission in grid.items():
            if permission.startswith('BRIDGE_'):
                axes[(name, *point)] = 'north_south' if permission.startswith('BRIDGE_NS') else 'east_west'
                planned[name][point] = 'WATER'
    manifest_path = ROOT / 'planning/minnesota/connections/mississippi-v02/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    waypoints = [('MnMississippiWinonaMouth', 26, 0),
                 ('MnMississippiLowerWoodsC00R02', 36, 39),
                 ('MnMississippiLowerWoodsC00R01', 0, 24),
                 ('MnStPaul', 18, 52), ('MnR05C00R02', 63, 0),
                 ('MnMississippiUpstreamWoodsC00R02', 0, 36),
                 ('MnMississippiUpstreamWoodsC00R00', 0, 40)]
    plan = input_plan(maps, planned, axes, manifest, manifest_path, waypoints)
    names = {r['id'] for r in manifest['new_maps']} | {
        'MnWinona', 'MnStPaul', 'MnMinneapolis', 'MnStCloud', 'MnR03C00R00',
        'MnR04C00R00', 'MnR05C00R00', 'MnR05C00R01', 'MnR05C00R02'}
    apertures = bridges({n: native[n] for n in names})
    assert len(apertures) == 7
    counts, crossed, waterfall_actions = Counter(), set(), []
    node = plan['launch']['map'], *plan['launch']['point']
    origin = node
    for i, step in enumerate(plan['steps']):
        target = step['map'], *step['point']
        dx, dy, direction = DIRECTIONS[step['direction']]
        if step.get('action') == 'waterfall_up':
            assert direction == 'north' and target[:2] == node[:2] and target[2] < node[2] - 1
            assert native[node[0]][node[1:]] == native[target[0]][target[1:]] == 'WATER'
            assert all(native[node[0]][node[1], y] == 'WATERFALL' for y in range(target[2] + 1, node[2]))
            waterfall_actions.append(dict(input_index=i, before=list(node), after=list(target)))
        else:
            q, a = (node[1] + dx, node[2] + dy), maps[node[0]]['attributes']
            if 0 <= q[0] < a['width'] * 2 and 0 <= q[1] < a['height'] * 2:
                expected = node[0], *q
            else:
                edge = next(e for e in a['connections'] if e['direction'] == direction)
                expected = edge['target_map'], *project(a, maps[edge['target_map']]['attributes'], edge, node[1:])
            assert expected == target, (i, node, expected, target)
            assert water_step(native[node[0]][node[1:]], native[target[0]][target[1:]], direction), (i, node, target)
        assert target[1:] not in {(o['x'], o['y']) for o in maps[target[0]]['objects']}, ('Body in water itinerary', i, target)
        for reach in ['LowerWoods', 'UpstreamWoods']:
            if reach in node[0] and reach in target[0]:
                counts[reach] += 1
        for aperture in apertures:
            x0, y0, x1, y1 = aperture['metatile_bounds_exclusive']
            if target[0] == aperture['map'] and x0 * 2 <= target[1] < x1 * 2 and y0 * 2 <= target[2] < y1 * 2:
                crossed.add(aperture['id'])
        node = target
    assert node == (plan['target']['map'], *plan['target']['point'])
    assert crossed == {a['id'] for a in apertures}
    target = node
    for excluded in [{'MnStPaul'}, {n for n in maps if 'LowerWoods' in n},
                     {n for n in maps if 'UpstreamWoods' in n}]:
        assert bfs_water(maps, planned, origin, target, axes, excluded) is None
    assert bfs_water(maps, native, origin, target) is None, 'Removing bridge underpasses must break the trip.'
    seen, mouths, unbound = reachable(maps, native, origin)
    assert not unbound, ('Reachable unbound Surf perimeter', sorted(unbound))
    plan.update(status='CURRENT_NATIVE_PACK_BOUND_CONTROLLER_PENDING', pack_sha256=digest,
                bridge_apertures=apertures, source_only_wooded_reach_inputs=dict(counts))
    (args.output / 'all-bridges-input-plan.json').write_text(json.dumps(plan, indent=2) + '\n')
    proof = dict(pack=str(args.pack), pack_sha256=digest, maps=len(actual),
                 itinerary_inputs=len(plan['steps']), bridge_apertures=apertures,
                 required_wooded_reach_inputs=dict(counts), waterfall_actions=waterfall_actions,
                 river_water_component_tiles=len(seen), reciprocal_water_seam_lanes=len(mouths),
                 reachable_unbound_surf_edges=sorted(unbound),
                 missing_st_paul_lower_upper_or_bridges_negatives_rejected=True,
                 gate_setup='Private planning copy only. Three exact existing badge-callback changes; native input must independently open the gate.',
                 native_controller_executed=False)
    (args.output / 'physical-proof.json').write_text(json.dumps(proof, indent=2) + '\n')
    report = json.loads(args.atlas_audit.read_text())
    assert report['pack_sha256'] == digest
    rows = [r for r in report['placements'] if r['map'] in names]
    assert len(rows) == len(names) == 26 and len({r['component'] for r in rows}) == 1
    x0 = min(r['origin_metatiles'][0] for r in rows)
    y0 = min(r['origin_metatiles'][1] for r in rows)
    x1 = max(r['origin_metatiles'][0] + r['dimensions_metatiles'][0] for r in rows)
    y1 = max(r['origin_metatiles'][1] + r['dimensions_metatiles'][1] for r in rows)
    scale = 12
    image = Image.new('RGB', ((x1 - x0) * scale, (y1 - y0) * scale + 100), '#121f2b')
    draw = ImageDraw.Draw(image)
    draw.text((14, 12), 'WINONA TO ST CLOUD - CURRENT NATIVE RIVER', font=font(24, True), fill='white')
    draw.text((14, 49), f'Pack {digest[:12]} - {len(rows)} actual exteriors; {len(plan["steps"])} planned inputs; 7 bridge apertures.', font=font(15), fill='#e8c38c')
    draw.text((14, 73), 'Initial gates shown. Geometry proof only; native controller replay pending. Dark gaps are not invented terrain.', font=font(15), fill='#b9d0cf')
    scenes = args.output / 'maps'
    scenes.mkdir()
    for row in rows:
        name = row['map']
        scene = native_room(actual[name], pack, native_origin=True)
        save_render(scene, scenes / f'{name}-whole.png')
        x, y = row['origin_metatiles']
        w, h = row['dimensions_metatiles']
        image.paste(scene.resize((w * scale, h * scale), Image.Resampling.NEAREST),
                    ((x - x0) * scale, (y - y0) * scale + 100))
    save_render(image, args.output / 'whole-current-corridor.png')
    assert digest == hashlib.sha256(args.pack.read_bytes()).hexdigest()
    print(json.dumps(proof, indent=2))


if __name__ == '__main__':
    main()
