#!/usr/bin/env python3
"""Review the explicit Mississippi source proposal; never publish a game pack.

The Surf proof is conditional on the declared bridge layer, native Waterfall
prerequisites and the existing Minneapolis badge gate being open. It is not a
controller test. Every image/version is retained by render_archive.
"""
import argparse
import collections
import copy
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

from audit_city_expansions import module_hash
from audit_outdoor_connections import boundary, collision, project, OPPOSITE
from render_archive import save_render
from render_progress_atlas import font, native_room
from render_reference import layout_blocks, load_pack


def bfs_water(maps, kinds, start, target, bridge_axes=None, blocked_maps=None):
    bridge_axes = bridge_axes or {}
    blocked_maps = blocked_maps or set()
    queue, parents = collections.deque([start]), {start: None}
    while queue:
        node = queue.popleft()
        if node == target:
            break
        name, x, y = node
        a = maps[name]['attributes']
        for direction, dx, dy in [('west', -1, 0), ('east', 1, 0), ('north', 0, -1), ('south', 0, 1)]:
            t = (name, x + dx, y + dy)
            if not (0 <= t[1] < a['width'] * 2 and 0 <= t[2] < a['height'] * 2):
                edge = next((e for e in a['connections'] if e['direction'] == direction), None)
                if edge is None or edge['target_map'] not in maps:
                    continue
                other = edge['target_map']
                q = project(a, maps[other]['attributes'], edge, (x, y))
                t = (other, *q)
            if t[0] in blocked_maps:
                continue
            axes = [bridge_axes.get(node), bridge_axes.get(t)]
            if any(axis == 'north_south' and dx != 0 or axis == 'east_west' and dy != 0 for axis in axes):
                continue
            if kinds[t[0]].get((t[1], t[2])) in {'WATER', 'WATERFALL'} and t not in parents:
                parents[t] = node
                queue.append(t)
    if target not in parents:
        return None
    path, node = [], target
    while node:
        path.append(list(node))
        node = parents[node]
    return list(reversed(path))



def input_plan(maps, kinds, bridge_axes, manifest, manifest_path, waypoints=()):
    name = 'MnWinona'
    bodies = {(o['x'], o['y']) for o in maps[name]['objects']}
    choices = []
    directions = [('left', -1, 0), ('right', 1, 0), ('up', 0, -1), ('down', 0, 1)]
    for p, kind in kinds[name].items():
        if kind != 'FLOOR' or p in bodies:
            continue
        for direction, dx, dy in directions:
            q = p[0]+dx, p[1]+dy
            if kinds[name].get(q) == 'WATER':
                choices.append((abs(p[0]-25)+abs(p[1]-14), p, q, direction))
    _, start, launch, launch_direction = min(choices)
    target = ('MnStCloud', 40, 20)
    route = [(name, *launch)]
    for waypoint in [*waypoints, target]:
        leg = bfs_water(maps, kinds, tuple(route[-1]), waypoint, bridge_axes)
        assert leg is not None
        route.extend(leg[1:])
    steps, checkpoints = [], []
    i = 1
    compass = {'west': 'left', 'east': 'right', 'north': 'up', 'south': 'down'}
    while i < len(route):
        before, after = route[i-1], route[i]
        if before[0] != after[0]:
            direction = next(compass[e['direction']] for e in maps[before[0]]['attributes']['connections']
                if e['target_map'] == after[0] and project(maps[before[0]]['attributes'], maps[after[0]]['attributes'], e, tuple(before[1:])) == tuple(after[1:]))
        else:
            dx, dy = after[1]-before[1], after[2]-before[2]
            direction = next(d for d, x, y in directions if (x, y) == (dx, dy))
        step = dict(map=after[0], point=after[1:], direction=direction)
        if direction == 'up' and kinds[after[0]].get(tuple(after[1:])) == 'WATERFALL':
            j = i
            while j < len(route) and route[j][0] == after[0] and kinds[after[0]].get(tuple(route[j][1:])) == 'WATERFALL':
                assert route[j][1] == before[1], 'climb must remain in one native column'
                j += 1
            assert j < len(route) and route[j][0] == after[0]
            step.update(point=route[j][1:], action='waterfall_up')
            i = j
        if before[0] != step['map']:
            checkpoints.append(dict(kind='map_transition', input_index=len(steps), map=step['map'], point=step['point']))
        if tuple(after) in bridge_axes and tuple(before) not in bridge_axes:
            checkpoints.append(dict(kind='bridge_entry', input_index=len(steps), map=step['map'], point=step['point'], axis=bridge_axes[tuple(after)]))
        steps.append(step)
        i += 1
    sequence = [name]
    for step in steps:
        if sequence[-1] != step['map']:
            sequence.append(step['map'])
    assert 'MnStPaul' in sequence and 'MnMinneapolis' in sequence
    # A shortest legal river trip need not enter an optional backwater window.
    # Require the long bends in both reaches and retain exact coverage for review.
    for reach in ['LowerWoods', 'UpstreamWoods']:
        visited = {name for name in sequence if reach in name}
        assert len(visited) >= 5, reach + ': journey bypasses the long wooded reach'
    unvisited_windows = [r['id'] for r in manifest['new_maps'] if r['id'] not in sequence]
    return dict(schema_version=1, status='preliminary source geometry; bind actual published pack SHA before controller execution',
        pack_sha256=None, parent_pack_sha256=manifest['base_pack_sha256'],
        source_manifest_sha256=hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
        start=dict(map=name, point=list(start)), launch=dict(direction=launch_direction, map=name, point=list(launch)),
        steps=steps, target=dict(map=target[0], point=list(target[1:])),
        checkpoints=checkpoints, expected_map_sequence=sequence,
        unvisited_source_windows=unvisited_windows,
        city_waterfront_probes=[dict(map='MnStPaul', point=p) for p in [[18,52],[42,52],[72,53]]],
        fixture_requirements=['initial legal Surf/Waterfall party', 'initial Fog/Plain/Rising badges',
                              'no position/terrain/progression/RNG mutations after setup',
                              'native gate callbacks open the Minneapolis buoy gate',
                              'native Surf initiation at the Winona bank'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=Path('planning/minnesota/connections/mississippi-v01/manifest.json'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    base = Path(manifest['base_pack'])
    assert hashlib.sha256(base.read_bytes()).hexdigest() == manifest['base_pack_sha256']
    pack = load_pack(base)
    maps = pack['data']['maps']
    source_before = {}
    for revision in manifest['terrain_revisions']:
        name = revision['map']
        module = maps[name]
        source_before[name] = copy.deepcopy(module)
        assert module_hash(module) == revision['native_module_sha256'], name + ': stale module'
        seen = set()
        w = module['attributes']['width']
        for row in revision['changes']:
            x, y = row['metatile']
            assert (x, y) not in seen, name + ': duplicate edit'
            seen.add((x, y))
            index = y * w + x
            assert module['blocks'][index] == row['before'], name + ': stale cell'
            module['blocks'][index] = row['after']
    for row in manifest['new_maps']:
        name = row['id']
        assert name not in maps
        path = Path(row['layout'])
        assert hashlib.sha256(path.read_bytes()).hexdigest() == row['layout_sha256']
        layout = json.loads(path.read_text())
        blocks = layout_blocks(layout)
        sw = layout['dimensions_metatiles'][0]
        x0, y0 = row['source_origin_metatiles']
        w, h = row['dimensions_metatiles']
        blocks = [blocks[(y0 + y) * sw + x0 + x] for y in range(h) for x in range(w)]
        assert hashlib.sha256(bytes(blocks)).hexdigest() == row['source_blocks_sha256']
        attributes = dict(maps['MnR05C00R00']['attributes'], width=w, height=h,
                          tileset_name=row['tileset'], connections=row['connections'], environment='ROUTE')
        maps[name] = dict(id=name, attributes=attributes, blocks=blocks, objects=[],
                          events=dict(warps=[], coord_events=[], bg_events=[]))
    for row in manifest['existing_connection_additions']:
        module = maps[row['map']]
        assert not any(e['direction'] == row['direction'] for e in module['attributes']['connections'])
        module['attributes']['connections'].append({k: row[k] for k in ['direction', 'target_map', 'offset']})
    kinds = {n: collision(pack, m) for n, m in maps.items() if n.startswith('Mn')}
    for row in manifest['objects']:
        name, (x, y) = row['map'], row['core']
        assert kinds[name][x, y] in {'FLOOR', 'TALL_GRASS', 'GRASS'}, (row['id'], kinds[name][x, y])
        maps[name]['objects'].append(dict(x=x, y=y, sprite=row['sprite'],
            spritemovedata='SPRITEMOVEDATA_STANDING_' + row['facing'].upper()))
    anchor_checks = 0
    for name, before in source_before.items():
        old_kinds = collision(pack, before)
        for rows in [before['objects'], before['events']['warps'], before['events']['coord_events'], before['events']['bg_events']]:
            for row in rows:
                p = row['x'], row['y']
                assert old_kinds[p] == kinds[name][p], (name, p)
                anchor_checks += 1
    # No unpaired water exits or incidental Cut quadrants in any new exterior.
    water_ports, cut_tiles, internal_shores = [], [], []
    new_rows = {r['id']: r for r in manifest['new_maps']}
    for row in manifest['new_maps']:
        name = row['id']
        a = maps[name]['attributes']
        cut_tiles.extend((name, list(p)) for p, kind in kinds[name].items() if kind == 'CUT_TREE')
        for direction in OPPOSITE:
            edge = next((e for e in a['connections'] if e['direction'] == direction), None)
            for p in boundary(a, direction):
                if kinds[name][p] not in {'WATER', 'WATERFALL'}:
                    continue
                assert edge is not None, (name, direction, p, 'unconnected water')
                target = edge['target_map']
                q = project(a, maps[target]['attributes'], edge, p)
                if kinds[target].get(q) not in {'WATER', 'WATERFALL'}:
                    # A natural bank can straddle a chunk boundary. The same
                    # explicit source raster and valid neighbor terrain close
                    # that shore; it is not a missing exterior water port.
                    assert target in new_rows and new_rows[target]['layout'] == row['layout'] and q in kinds[target], (name, p, target, q)
                    internal_shores.append(dict(map=name, core=list(p), target=target, target_core=list(q), neighbor_collision=kinds[target][q]))
                    continue
                inverse = next((e for e in maps[target]['attributes']['connections']
                                if e['target_map'] == name and e['direction'] == OPPOSITE[direction]), None)
                assert inverse and project(maps[target]['attributes'], a, inverse, q) == p
                water_ports.append(dict(map=name, direction=direction, core=list(p), target=target, target_core=list(q)))
    assert not cut_tiles, cut_tiles
    # Open the existing gate only in this explicitly declared geometry fixture.
    mp = maps['MnMinneapolis']
    gate_rows = [r for r in mp['script_block_changes'] if r['source_script'] == 'MnMPUpperPoolBuoyOpen']
    assert len(gate_rows) == 3
    for r in gate_rows:
        mp['blocks'][(r['y'] // 2) * mp['attributes']['width'] + r['x'] // 2] = r['block_id']
    kinds['MnMinneapolis'] = collision(pack, mp)
    no_bridge = bfs_water(maps, kinds, ('MnWinona', 26, 12), ('MnStCloud', 40, 20))
    assert no_bridge is None, 'negative control: route should require bridge layer'
    inventory = json.loads(Path(manifest['bridge_inventory']).read_text())
    assert inventory['base_pack_sha256'] == manifest['base_pack_sha256']
    bridge_axes = {}
    for row in inventory['bridges']:
        assert module_hash(source_before.get(row['map'], maps[row['map']])) == row['native_module_sha256'] or row['map'] == 'MnMinneapolis'
        # Native MP block overrides above are the explicit gate-open fixture;
        # exact bridge preimage block values remain checked independently.
        for tile in row['metatiles']:
            x, y = tile['metatile']
            assert maps[row['map']]['blocks'][y * maps[row['map']]['attributes']['width'] + x] == tile['block_decimal']
            for dx, dy in [(0, 0), (1, 0), (0, 1), (1, 1)]:
                kinds[row['map']][2 * x + dx, 2 * y + dy] = 'WATER'
                bridge_axes[row['map'], 2*x+dx, 2*y+dy] = row['axis']
    path = bfs_water(maps, kinds, ('MnWinona', 26, 12), ('MnStCloud', 40, 20), bridge_axes)
    assert path is not None, 'proposed bridge layer + real water seams still disconnected'
    upstream_maps = {r['id'] for r in manifest['new_maps'] if 'UpstreamWoods' in r['id']}
    assert not upstream_maps or bfs_water(maps, kinds, ('MnWinona', 26, 12), ('MnStCloud', 40, 20), bridge_axes, upstream_maps) is None, 'new upstream woods can be bypassed while Surfing'
    assert bfs_water(maps, kinds, ('MnWinona', 26, 12), ('MnStCloud', 40, 20), bridge_axes, {'MnStPaul'}) is None, 'the itinerary bypasses St Paul'
    for target in [('MnStPaul',18,52),('MnStPaul',42,52),('MnStPaul',72,53)]:
        assert bfs_water(maps,kinds,('MnWinona',26,12),target,bridge_axes), 'St Paul basin disconnected'
    args.output.mkdir(parents=True, exist_ok=True)
    plan = input_plan(maps, kinds, bridge_axes, manifest, args.manifest)
    (args.output / 'source-input-plan.json').write_text(json.dumps(plan, indent=2)+'\n')
    all_bridges_plan = input_plan(maps, kinds, bridge_axes, manifest, args.manifest,
                                 waypoints=[('MnStPaul', 18, 52)])
    all_bridges_plan['required_waypoints'] = [dict(map='MnStPaul', point=[18, 52])]
    (args.output / 'source-all-bridges-input-plan.json').write_text(json.dumps(all_bridges_plan, indent=2)+'\n')
    # Every destination must have an actor-free ordinary landing reachable from water.
    landing_proofs = []
    for actor in manifest['objects']:
        name, p = actor['map'], tuple(actor['core'])
        bodies = {(o['x'], o['y']) for o in maps[name]['objects']}
        candidates = [(p[0] - 1, p[1]), (p[0] + 1, p[1]), (p[0], p[1] - 1), (p[0], p[1] + 1)]
        starts = [q for q in candidates if kinds[name].get(q) in {'FLOOR', 'TALL_GRASS', 'GRASS'} and q not in bodies]
        queue, seen, launch = collections.deque(starts), set(starts), None
        while queue and launch is None:
            q = queue.popleft()
            for t in [(q[0] - 1, q[1]), (q[0] + 1, q[1]), (q[0], q[1] - 1), (q[0], q[1] + 1)]:
                if kinds[name].get(t) == 'WATER':
                    launch = (q, t)
                    break
                if kinds[name].get(t) in {'FLOOR', 'TALL_GRASS', 'GRASS'} and t not in seen and t not in bodies:
                    seen.add(t)
                    queue.append(t)
        assert launch, (actor['id'], 'no ordinary landing')
        assert bfs_water(maps, kinds, ('MnWinona', 26, 12), (name, *launch[1]), bridge_axes), (actor['id'], 'isolated water landing')
        landing_proofs.append(dict(object=actor['id'], map=name, bank_core=list(launch[0]), water_core=list(launch[1]), land_component_core_tiles=len(seen)))
    for row in manifest['new_maps']:
        save_render(native_room(maps[row['id']], pack, native_origin=True), args.output / (row['id'] + '-whole.png'))
    # Actual native map shapes, exact authored placements; no invented interstitial terrain.
    origins = {r['id']: r['world_origin_metatiles'] for r in manifest['new_maps']}
    origins.update({'MnMinneapolis': [0, 0], 'MnStPaul': [68, -6], 'MnR04C00R00': [40, 9],
                    'MnR03C00R00': [10, 32], 'MnWinona': [113, 56], 'MnStCloud': [9, -74],
                    'MnR05C00R00': [9, -56], 'MnR05C00R01': [9, -32], 'MnR05C00R02': [9, -8]})
    x0, y0 = min(p[0] for p in origins.values()), min(p[1] for p in origins.values())
    x1 = max(p[0] + maps[n]['attributes']['width'] for n, p in origins.items())
    y1 = max(p[1] + maps[n]['attributes']['height'] for n, p in origins.items())
    scale = 12
    canvas = Image.new('RGB', ((x1-x0)*scale, (y1-y0)*scale+96), '#121f2b')
    draw = ImageDraw.Draw(canvas)
    draw.text((14, 12), 'WINONA → ST CLOUD · CONNECTED SOURCE CANDIDATE', font=font(24, True), fill='white')
    draw.text((14, 48), 'Actual native maps + explicit proposed river terrain. Bridge layer / gate / Waterfall fixture; not controller proof.', font=font(15), fill='#e8c38c')
    for name, (x, y) in origins.items():
        im = native_room(maps[name], pack, native_origin=True)
        a = maps[name]['attributes']
        canvas.paste(im.resize((a['width']*scale, a['height']*scale), Image.Resampling.NEAREST), ((x-x0)*scale, (y-y0)*scale+96))
    save_render(canvas, args.output / 'winona-st-cloud-connected-source.png')
    proof = dict(base_pack_sha256=manifest['base_pack_sha256'], native_controller_executed=False,
        conditional_requirements=['proper native dual-layer bridges installed', 'native Waterfall prerequisites', 'existing Minneapolis buoy gate opened by both badges'],
        anchor_collision_checks=anchor_checks, new_map_count=len(manifest['new_maps']),
        water_ports=water_ports, internal_closed_shores=internal_shores, incidental_cut_tiles=cut_tiles, landing_proofs=landing_proofs,
        missing_bridge_negative_rejected=True, upstream_bypass_negative_rejected=bool(upstream_maps), st_paul_bypass_negative_rejected=True, all_three_st_paul_basins_connected=True, water_only_inputs=len(path)-1, water_only_path=path)
    (args.output / 'source-geometry-proof.json').write_text(json.dumps(proof, indent=2) + '\n')
    print(json.dumps({k: proof[k] for k in ['new_map_count', 'anchor_collision_checks', 'water_only_inputs', 'missing_bridge_negative_rejected']}))


if __name__ == '__main__':
    main()
