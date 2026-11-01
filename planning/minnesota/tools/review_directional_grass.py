#!/usr/bin/env python3
"""Prove a route's grass-out / dry-ledge-return contract without writing packs."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path

from render_reference import layout_blocks, load_pack


def assess(design, layout, blocks, collision):
    w, h = layout['dimensions_metatiles']
    occupied = {tuple(a['tile']) for key in ['trainers', 'residents', 'items']
                for a in design.get(key, [])}

    def kind(at):
        x, y = at
        raw = collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
        return {'BRIDGE_NS_DECK': 'FLOOR', 'BRIDGE_EW_DECK': 'FLOOR',
                'BRIDGE_NS_RAIL': 'WALL', 'BRIDGE_EW_RAIL': 'WALL'}.get(raw, raw)

    def in_bounds(at):
        return 0 <= at[0] < w*2 and 0 <= at[1] < h*2

    def minimum(start, end, band, region=None):
        def allowed(at):
            return in_bounds(at) and (region is None or
                (region[0] <= at[0] < region[2] and region[1] <= at[1] < region[3]))
        assert allowed(start) and allowed(end), 'Directional scope excludes its endpoints'
        costs = {start: 0}
        todo = deque([start])
        while todo:
            x, y = at = todo.popleft()
            for dx, dy in [(0, 1), (0, -1), (1, 0), (-1, 0)]:
                target = (x+dx, y+dy)
                if not allowed(target):
                    continue
                hop = {'HOP_DOWN': (0, 1), 'HOP_LEFT': (-1, 0),
                       'HOP_RIGHT': (1, 0)}.get(kind(target))
                if hop == (dx, dy):
                    target = (target[0]+dx*2, target[1]+dy*2)
                if (not allowed(target) or target in occupied
                        or kind(target) not in {'FLOOR', 'TALL_GRASS'}):
                    continue
                weight = int(target in band)
                cost = costs[at] + weight
                if cost < costs.get(target, 10**9):
                    costs[target] = cost
                    (todo.append if weight else todo.appendleft)(target)
        assert end in costs, 'Directional route is disconnected'
        return costs[end]

    south, north = [tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    reports = []
    for loop in design['directional_grass_loops']:
        band = {(x, y) for x0, y0, x1, y1 in loop['grass_regions_core']
                for y in range(y0, y1) for x in range(x0, x1)}
        assert band and all(kind(at) == 'TALL_GRASS' for at in band), 'False grass neck'
        start = tuple(loop.get('outbound_start_core', south))
        end = tuple(loop.get('outbound_end_core', north))
        region = loop.get('walk_region_core')
        uphill = minimum(start, end, band, region)
        downhill = minimum(end, start, band, region)
        outward_minimum = loop.get('outbound_minimum_grass_steps',
                                    loop.get('northbound_minimum_grass_steps'))
        return_minimum = loop.get('return_minimum_grass_steps',
                                  loop.get('southbound_minimum_grass_steps'))
        assert outward_minimum is not None and return_minimum == 0, 'Missing directional grass contract'
        assert uphill >= outward_minimum, f'Free uphill grass bypass: {uphill}'
        assert downhill == return_minimum, 'No dry ledge return'
        assert all(kind(tuple(at)) == 'FLOOR' for at in loop['dry_trail_core']), 'Dry trail interrupted'
        lane_costs = []
        if 'all_ledge_lanes_minimum_grass_steps' in loop:
            ledge = next(v for v in design['one_way_ledges'] if v['id'] == loop['ledge_id'])
            x0, y0, x1, y1 = ledge['bounds_core']
            direction = ledge['direction']
            assert direction in {'south', 'east', 'west'}, 'Unsupported native ledge direction'
            vertical = direction == 'south'
            for lane in (range(x0, x1) if vertical else range(y0, y1)):
                lower, upper = (((lane, ledge['landing'][1]),
                                 (lane, ledge['upper_approach'][1])) if vertical else
                                ((ledge['landing'][0], lane),
                                 (ledge['upper_approach'][0], lane)))
                exposure = minimum(lower, upper, band, region)
                assert exposure >= loop['all_ledge_lanes_minimum_grass_steps'], 'Dry ledge lane bypass'
                assert minimum(upper, lower, band, region) == 0, 'Ledge lane loses dry hop'
                lane_costs.append(dict(axis='x' if vertical else 'y', lane=lane,
                                       uphill_grass_steps=exposure, downhill_grass_steps=0))
        reports.append(dict(id=loop['id'], northbound_grass_steps=uphill,
                            southbound_grass_steps=downhill,
                            outbound_start_core=start, outbound_end_core=end,
                            walk_region_core=region, scope=loop.get('scope', 'Whole outdoor route'), lane_costs=lane_costs))
    return reports


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--layout', type=Path, required=True)
    parser.add_argument('--dossier', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--published', action='store_true')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    layout, design = [json.loads(p.read_text()) for p in [args.layout, args.dossier]]
    assert layout['id'] == design['id'], 'Layout/dossier route mismatch'
    blocks = layout_blocks(layout)
    w, h = layout['dimensions_metatiles']
    collision = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    if args.published:
        for row in range(design['chunk_rows']):
            for col in range(design['chunk_columns']):
                m = pack['data']['maps'][f'{design["map_prefix"]}C{col:02}R{row:02}']
                a = m['attributes']
                expected = [blocks[(row*24+y)*w+col*32+x]
                            for y in range(a['height']) for x in range(a['width'])]
                assert expected == m['blocks'], 'Published route differs from checked source'
    report = dict(route=design['id'], pack_identity=pack['identity']['content_hash'],
                  published_terrain_matched=args.published,
                  directional_loops=assess(design, layout, blocks, collision),
                  inputs=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest())
                          for p in [args.layout, args.dossier]])
    if args.self_test:
        fixtures = {
            'MN_R01': [
                ('dry-uphill-bypass', [(x, y, 2) for y in [10, 11]
                                       for x in range(19, 23) if (x, y) != (20, 10)]),
                ('reverse-wall-removed', [(13, 8, 1), (14, 8, 1)]),
                ('dry-return-replaced-by-grass', [(13, 9, 3)])],
            'MN_R02': [
                ('reverse-wall-removed', [(23, 7, 1), (23, 8, 1)]),
                ('dry-return-replaced-by-grass', [(22, 7, 0x0b)])],
            'MN_R03': [
                ('dry-uphill-bypass', [(8, 9, 1), (8, 10, 1)]),
                ('reverse-wall-removed', [(5, 10, 1)]),
                ('dry-return-replaced-by-grass', [(5, 8, 3)])],
            'MN_R05': [
                ('dry-uphill-bypass', [(x, y, 1) for y in range(34, 38)
                                       for x in range(6, 9)]),
                ('reverse-wall-removed', [(18, 20, 1), (19, 20, 1)]),
                ('dry-return-replaced-by-grass', [(18, 19, 3)])],
            'MN_R06': [
                ('dry-uphill-bypass', [(21, 5, 1)]),
                ('reverse-wall-removed', [(22, 5, 1)]),
                ('dry-return-replaced-by-grass', [(22, 4, 3)])],
            'MN_R07': [
                ('dry-uphill-bypass', [(x, y, 1) for y in range(21, 25)
                                       for x in range(22, 28)] + [(21, 24, 1)]),
                ('reverse-wall-removed', [(8, 27, 1), (9, 27, 1)]),
                ('dry-return-replaced-by-grass', [(8, 26, 3)])],
            'MN_R08': [
                ('dry-uphill-bypass', [(x, y, 1) for y in range(14, 16) for x in range(16, 20) if (x, y) != (16, 14)]),
                ('reverse-wall-removed', [(14, 13, 1)]),
                ('dry-return-replaced-by-grass', [(14, 12, 3)])],
            'MN_R09': [
                ('dry-uphill-bypass', [(40, 15, 0x71), (40, 16, 0x71)]),
                ('cliff-foot-side-bypass', [(39, 15, 0x68), (39, 16, 0x68)]),
                ('reverse-wall-removed', [(40, 20, 1)]),
                ('dry-return-replaced-by-grass', [(40, 19, 3)])],
            'MN_R10': [
                ('dry-uphill-bypass', [(5, 7, 1), (6, 7, 0x71), (5, 8, 1), (6, 8, 1)]),
                ('cliff-foot-side-bypass', [(4, 9, 0x68), (6, 10, 0x72), (6, 9, 0x71), (7, 9, 0x6f), (7, 8, 1)]),
                ('reverse-wall-removed', [(12, 10, 1), (13, 10, 1)]),
                ('dry-return-replaced-by-grass', [(12, 9, 3)])],
            'MN_R11': [
                ('dry-uphill-bypass', [(16, 44, 1), (17, 44, 1)]),
                ('cliff-foot-side-bypass', [(17, 45, 0x72)]),
                ('reverse-wall-removed', [(16, 45, 1)]),
                ('dry-return-replaced-by-grass', [(16, 46, 3)])],
        }
        assert design['id'] in fixtures, 'No mutation fixtures for this route'
        rejected = []
        for name, edits in fixtures[design['id']]:
            altered = list(blocks)
            for x, y, block in edits:
                altered[y*w+x] = block
            try:
                assess(design, layout, altered, collision)
            except AssertionError:
                rejected.append(name)
            else:
                raise AssertionError('Accepted broken directional contract: ' + name)
        report['negative_cases_rejected'] = rejected
    args.output.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(report, indent=2) + '\n'
    path = args.output.with_name(args.output.stem + '-' + hashlib.sha256(payload.encode()).hexdigest()[:16] + '.json')
    if path.exists():
        assert path.read_text() == payload, 'Immutable report collision'
    else:
        path.write_text(payload)
    print(json.dumps(dict(ok=True, report=str(path), loops=report['directional_loops']), indent=2))


if __name__ == '__main__':
    main()
