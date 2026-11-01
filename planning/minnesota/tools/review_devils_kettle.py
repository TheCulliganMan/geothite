#!/usr/bin/env python3
"""Read-only native-art preview and collision review; not runtime evidence."""
import argparse
from collections import deque
import json
from pathlib import Path

from render_reference import SourceArt, load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    parser.add_argument('site', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    site = json.loads(args.site.read_text())
    pack = load_pack(args.pack)
    w, h = site['dimensions_metatiles']
    rows = [row.split() for row in site['rows']]
    assert len(rows) == h and all(len(row) == w for row in rows)
    blocks = [int(site['legend'][token], 16) for row in rows for token in row]
    collision = pack['data']['tilesets'][site['tileset']]['collision']
    assert collision['80'] == ['WATERFALL'] * 4
    assert all(f'{block:02x}' in collision for block in blocks)

    def kind(x, y):
        block = blocks[(y // 2) * w + x // 2]
        return collision[f'{block:02x}'][(y % 2) * 2 + x % 2]

    # A solid, visible native perimeter closes both walking and Surf escape.
    perimeter = [(x, y) for y in range(h * 2) for x in range(w * 2)
                 if x in (0, w * 2 - 1) or y in (0, h * 2 - 1)]
    assert all(kind(x, y) == 'WALL' for x, y in perimeter)
    start = tuple(site['arrival'])
    reached, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for point in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            xx, yy = point
            if (point not in reached and 0 <= xx < w*2 and 0 <= yy < h*2
                    and kind(xx, yy) in {'FLOOR', 'TALL_GRASS'}):
                reached.add(point)
                queue.append(point)
    anchors = [start, tuple(site['exit_read_tile']),
               tuple(site['encounter']['actor']), tuple(site['encounter']['approach']),
               (6, 8)]  # opposite-bank view of the two native falls
    assert all(point in reached for point in anchors), anchors
    entrance = site['entrance']
    outside = pack['data']['maps'][entrance['map']]
    x, y = entrance['metatile']
    assert outside['blocks'][y * outside['attributes']['width'] + x] == int(
        entrance['required_previous_block'], 16)
    for point in [entrance['warp_tile'], entrance['approach']]:
        assert not any([obj['x'], obj['y']] == point for obj in outside['objects'])
        assert not any([warp['x'], warp['y']] == point for warp in outside['events']['warps'])
    assert site['encounter']['species'] in pack['data']['pokemon']
    args.output.mkdir(parents=True, exist_ok=False)
    SourceArt(pack).render(site['tileset'], w, h, blocks).save(args.output/'whole-terrain.png')
    report = dict(status='PASS', scope=__doc__, reachable_walk_tiles=len(reached),
                  reachable_anchors=anchors, solid_perimeter_tiles=len(perimeter),
                  preserved_existing_entrance_warps=len(outside['events']['warps']),
                  native_waterfall_collision=collision['80'],
                  pending=['Rust publication', 'native traversal and source art comparison',
                           'actual encounter outcomes and Continue', 'preservation and audio'])
    (args.output/'collision-review.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
