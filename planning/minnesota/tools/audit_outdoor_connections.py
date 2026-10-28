#!/usr/bin/env python3
"""Audit actual outdoor terrain/connection geometry and render an honest atlas.

Read-only pack inspection. Water components are collision evidence, not a claim
that script gates, Waterfall prerequisites or controller traversal have passed.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

from render_archive import save_render
from render_progress_atlas import font, native_room
from render_reference import load_pack

WATER = {'WATER', 'WATERFALL'}
OPPOSITE = {'north': 'south', 'south': 'north', 'east': 'west', 'west': 'east'}


def collision(pack, module):
    a = module['attributes']
    definitions = pack['data']['tilesets'][a['tileset_name']]['collision']
    return {(x, y): definitions[f"{module['blocks'][(y // 2) * a['width'] + x // 2]:02x}"][(y % 2) * 2 + x % 2]
            for y in range(a['height'] * 2) for x in range(a['width'] * 2)}


def neighbor_origin(at, a, b, edge):
    x, y = at
    offset = edge['offset']
    return {'north': (x + offset, y - b['height']),
            'south': (x + offset, y + a['height']),
            'west': (x - b['width'], y + offset),
            'east': (x + a['width'], y + offset)}[edge['direction']]


def project(a, b, edge, position):
    x, y = position
    offset = edge['offset'] * 2
    return {'north': (x - offset, b['height'] * 2 - 1),
            'south': (x - offset, 0), 'west': (b['width'] * 2 - 1, y - offset),
            'east': (0, y - offset)}[edge['direction']]


def boundary(a, direction):
    w, h = a['width'] * 2, a['height'] * 2
    return {'north': [(x, 0) for x in range(w)],
            'south': [(x, h - 1) for x in range(w)],
            'west': [(0, y) for y in range(h)],
            'east': [(w - 1, y) for y in range(h)]}[direction]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--scale', type=int, default=8)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    pack = load_pack(args.pack)
    maps = {k: v for k, v in pack['data']['maps'].items()
            if k.startswith('Mn') and (v['attributes']['environment'] or '').lower() in {'town', 'route', 'outdoor'}}
    kinds = {k: collision(pack, v) for k, v in maps.items()}
    all_maps = pack['data']['maps']
    constants = {v['attributes']['map_constant']: n for n, v in all_maps.items()}
    native_graph = {n: set() for n in all_maps}
    warp_edges = []
    for name, module in all_maps.items():
        for edge in module['attributes']['connections']:
            if edge['target_map'] in all_maps:
                native_graph[name].add(edge['target_map'])
        for warp in module['events']['warps']:
            target = constants.get(warp['target_map'], warp['target_map'])
            if target in all_maps:
                native_graph[name].add(target)
                if name != target:
                    warp_edges.append({'from': name, 'from_core': [warp['x'], warp['y']],
                                       'to': target, 'to_warp_index': warp['target_warp_id']})
    reached, queue = {'MnWinona'}, collections.deque(['MnWinona'])
    while queue:
        for target in native_graph[queue.popleft()]:
            if target not in reached:
                reached.add(target)
                queue.append(target)
    world = json.loads((Path(__file__).resolve().parents[1] / 'world.json').read_text())
    missing_hubs = [t['id'] for t in world['towns'] if t['id'] not in maps]
    positions, groups, conflicts, overlaps = {}, [], [], []
    for root in ['MnMinneapolis'] + sorted(maps):
        if root in positions:
            continue
        positions[root] = (0, 0)
        queue, group = collections.deque([root]), []
        while queue:
            name = queue.popleft()
            group.append(name)
            for edge in maps[name]['attributes']['connections']:
                target = edge['target_map']
                if target not in maps:
                    continue
                at = neighbor_origin(positions[name], maps[name]['attributes'], maps[target]['attributes'], edge)
                if target in positions:
                    if positions[target] != at:
                        conflicts.append({'from': name, 'to': target, 'existing': positions[target], 'implied': at})
                else:
                    positions[target] = at
                    queue.append(target)
        groups.append(group)
    for gi, group in enumerate(groups):
        for i, name in enumerate(group):
            x, y = positions[name]
            a = maps[name]['attributes']
            for target in group[i + 1:]:
                u, v = positions[target]
                b = maps[target]['attributes']
                rect = [max(x, u), max(y, v), min(x + a['width'], u + b['width']), min(y + a['height'], v + b['height'])]
                if rect[0] < rect[2] and rect[1] < rect[3]:
                    overlaps.append({'group': gi, 'maps': [name, target], 'rectangle_metatiles': rect})
    components, cell_components = {}, {}
    for name in maps:
        pending = {p for p, kind in kinds[name].items() if kind in WATER}
        local = []
        while pending:
            start = min(pending, key=lambda p: (p[1], p[0]))
            pending.remove(start)
            queue, cells = collections.deque([start]), []
            while queue:
                x, y = queue.popleft()
                cells.append((x, y))
                for p in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                    if p in pending:
                        pending.remove(p)
                        queue.append(p)
            index = len(local)
            for p in cells:
                cell_components[name, p] = index
            border = {d: [list(p) for p in boundary(maps[name]['attributes'], d) if p in cells] for d in OPPOSITE}
            local.append({'index': index, 'core_tiles': len(cells), 'representative': list(start),
                          'bounds': [min(x for x, _ in cells), min(y for _, y in cells), max(x for x, _ in cells), max(y for _, y in cells)],
                          'border_water': border,
                          'waterfall_core_tiles': sum(kinds[name][p] == 'WATERFALL' for p in cells)})
        components[name] = local
    seams, exposed = [], []
    for name, module in maps.items():
        a = module['attributes']
        for direction in OPPOSITE:
            edge = next((e for e in a['connections'] if e['direction'] == direction and e['target_map'] in maps), None)
            for p in boundary(a, direction):
                if kinds[name][p] not in WATER:
                    continue
                target = edge['target_map'] if edge else None
                q = project(a, maps[target]['attributes'], edge, p) if edge else None
                inverse = next((e for e in maps[target]['attributes']['connections']
                                if e['direction'] == OPPOSITE[direction] and e['target_map'] == name), None) if target else None
                reciprocal = inverse and project(maps[target]['attributes'], a, inverse, q) == p
                if target and kinds[target].get(q) in WATER and reciprocal:
                    seams.append({'from': name, 'from_core': list(p), 'from_component': cell_components[name, p],
                                  'direction': direction, 'to': target, 'to_core': list(q), 'to_component': cell_components[target, q]})
                else:
                    exposed.append({'map': name, 'core': list(p), 'direction': direction,
                                    'neighbor': target, 'neighbor_core': list(q) if q else None,
                                    'neighbor_collision': kinds[target].get(q) if target else None,
                                    'reciprocal_connection': bool(reciprocal)})
    # Complete source-pixel originals are retained before the atlas resampling.
    originals, scenes = {}, {}
    for name, module in maps.items():
        folder = args.output / 'maps' / name
        image = native_room(module, pack, archive=folder, native_origin=True)
        originals[name] = image
        scenes[name] = image.resize((module['attributes']['width'] * args.scale,
                                    module['attributes']['height'] * args.scale), Image.Resampling.NEAREST)
    # Each geometric component retains exact offsets. Detached groups are panels.
    # Overlapping footprints are explicitly marked and separately preserved.
    panels, placement_receipt = [], []
    for gi, group in enumerate(groups):
        x0, y0 = min(positions[n][0] for n in group), min(positions[n][1] for n in group)
        x1 = max(positions[n][0] + maps[n]['attributes']['width'] for n in group)
        y1 = max(positions[n][1] + maps[n]['attributes']['height'] for n in group)
        panel = Image.new('RGB', (max(360, (x1 - x0) * args.scale + 48), (y1 - y0) * args.scale + 76), '#121f2b')
        draw = ImageDraw.Draw(panel)
        draw.text((12, 8), f'DIRECT SEAM GROUP {gi + 1} · {len(group)} MAPS', font=font(15, True), fill='#eaf0ed')
        for name in group:
            x, y = positions[name]
            px, py = 24 + (x - x0) * args.scale, 48 + (y - y0) * args.scale
            panel.paste(scenes[name], (px, py))
            draw.rectangle((px, py, px + scenes[name].width - 1, py + scenes[name].height - 1), outline='#527772')
            label = name.removeprefix('Mn')
            draw.text((px + 2, py + 2), label, font=font(10, True), fill='#ffffff', stroke_width=1, stroke_fill='#13202b')
            placement_receipt.append({'map': name, 'component': gi, 'origin_metatiles': [x, y],
                                      'dimensions_metatiles': [maps[name]['attributes']['width'], maps[name]['attributes']['height']]})
        for item in overlaps:
            if item['group'] != gi:
                continue
            r = item['rectangle_metatiles']
            box = [24 + (r[0] - x0) * args.scale, 48 + (r[1] - y0) * args.scale,
                   24 + (r[2] - x0) * args.scale - 1, 48 + (r[3] - y0) * args.scale - 1]
            draw.rectangle(box, outline='#ff5959', width=3)
            draw.text((box[0], box[1] - 13), 'OVERLAP', font=font(10, True), fill='#ff7575')
        save_render(panel, args.output / f'component-{gi + 1:02}.png')
        panels.append(panel)
    left = panels[0]
    right_width = max((panel.width for panel in panels[1:]), default=0)
    right_height = sum(panel.height + 22 for panel in panels[1:])
    atlas_width = left.width + right_width + 88 if len(panels) > 1 else left.width + 48
    atlas = Image.new('RGB', (atlas_width, max(left.height, right_height) + 180), '#0c1721')
    draw = ImageDraw.Draw(atlas)
    draw.text((24, 16), 'MINNESOTA · CURRENT NATIVE OUTDOOR MAPS', font=font(28, True), fill='#ffffff')
    draw.text((24, 56), f"{len(maps)} actual exterior maps · one scale ({args.scale}px/metatile) · pack {hashlib.sha256(args.pack.read_bytes()).hexdigest()[:12]}", font=font(16), fill='#b9d0cf')
    coverage = 'Panels separate direct seams; Itasca also connects through a gatehouse.' if len(panels) > 1 else 'All exterior maps share one direct-seam group.'
    draw.text((24, 84), 'Dark gaps have no terrain. ' + coverage + ' Red = footprint overlap.', font=font(16), fill='#e4bf84')
    draw.text((24, 110), f"{len(set(maps) & reached)}/{len(maps)} exteriors share Winona's explicit warp/seam graph; {len(missing_hubs)} planned hubs are absent. Surf requires separate controller proof.", font=font(16), fill='#e4bf84')
    atlas.paste(left, (24, 156))
    y = 156
    for panel in panels[1:]:
        atlas.paste(panel, (left.width + 48, y))
        y += panel.height + 22
    save_render(atlas, args.output / 'full-state-native-current.png')
    preview = atlas.copy()
    preview.thumbnail((2200, 3000), Image.Resampling.NEAREST)
    save_render(preview, args.output / 'full-state-native-current-preview.png')
    report = {'schema_version': 1, 'pack': str(args.pack), 'pack_sha256': hashlib.sha256(args.pack.read_bytes()).hexdigest(),
              'outdoor_map_count': len(maps), 'connection_components': groups, 'placements': placement_receipt,
              'inconsistent_connection_cycles': conflicts, 'overlapping_native_footprints': overlaps,
              'water_components': components, 'reciprocal_water_collision_lanes': seams,
              'explicit_warp_edges': warp_edges,
              'explicit_warp_and_seam_graph_outdoors_reachable_from_winona': sorted(set(maps) & reached),
              'explicit_warp_and_seam_graph_outdoors_not_reached_from_winona': sorted(set(maps) - reached),
              'planned_hubs_absent_from_pack': missing_hubs,
              'graph_limitations': ['Native gates and prerequisite scripts may block a graph edge.',
                                    'Engine special transports such as Safari entry are outside the plain warp/seam graph.',
                                    'This graph is not controller traversal evidence.',
                                    'Water components inventory ordinary Water/Waterfall only; layered bridge tokens require traversal-mode analysis.'],
              'exposed_or_nonwater_neighbor_boundary': exposed,
              'interpretation': 'Static native collision evidence only; WATERFALL ascent and script gates are not assumed passed.'}
    (args.output / 'native-connection-audit.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'outdoor_maps': len(maps), 'components': len(groups), 'overlaps': len(overlaps),
                      'water_lanes': len(seams), 'exposed_water_lanes': len(exposed), 'atlas': str(args.output / 'full-state-native-current.png')}))


if __name__ == '__main__':
    main()
