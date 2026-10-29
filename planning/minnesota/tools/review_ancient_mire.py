#!/usr/bin/env python3
"""Review the authored R08 mire against a supplied native pack without exporting art."""
import argparse
from collections import deque
import json
from pathlib import Path

from render_archive import save_render
from render_progress_atlas import FACING, scene
from render_reference import SourceArt, layout_blocks, load_pack
from review_northern_lakes import check_layout_contract, check_piers, check_cut_and_choices

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/revisions/ancient-swamp-v01'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    design = json.loads((SOURCE / 'MN_R08.json').read_text())
    layout = json.loads((SOURCE / 'MN_R08.layout.json').read_text())
    pack = load_pack(args.pack)
    blocks = layout_blocks(layout)
    width, height = layout['dimensions_metatiles']
    tileset = layout['compiled_tileset']
    collision = pack['data']['tilesets'][tileset]['collision']
    actors = []
    for row in range(design['chunk_rows']):
        for column in range(design['chunk_columns']):
            name = f"{design['map_prefix']}C{column:02}R{row:02}"
            module = pack['data']['maps'][name]
            a = module['attributes']
            assert a['tileset_name'] == tileset, name + ': native tileset differs'
            expected = [blocks[y * width + x]
                        for y in range(row * 24, row * 24 + a['height'])
                        for x in range(column * 32, column * 32 + a['width'])]
            assert module['blocks'] == expected, name + ': installed terrain differs'
            for actor in module['objects']:
                direction = actor['spritemovedata'].removeprefix('SPRITEMOVEDATA_STANDING_')
                actors.append(dict(tile=[actor['x'] + column * 64,
                                         actor['y'] + row * 48],
                                   sprite=actor['sprite'],
                                   facing=FACING.get(direction, 'south')))

    def kind(point):
        x, y = point
        return collision[f'{blocks[y // 2 * width + x // 2]:02x}'][y % 2 * 2 + x % 2]

    occupied = {tuple(actor['tile']) for actor in actors}
    walk = {(x, y) for y in range(height * 2) for x in range(width * 2)
            if kind((x, y)) in {'FLOOR', 'TALL_GRASS'} and (x, y) not in occupied}
    start = tuple(v * 2 for v in design['endpoints'][0]['gate_metatile'])
    seen, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if at in walk and at not in seen:
                seen.add(at)
                queue.append(at)
    mire = design['ancient_mire']
    targets = [tuple(a['interaction_core']) for a in mire['clue_stations']]
    targets += [tuple(mire['interaction_core']),
                tuple(v * 2 for v in design['endpoints'][1]['gate_metatile'])]
    assert all(at in seen for at in targets), 'Mire inspection or regional port is stranded'
    for clue in mire['clue_stations']:
        target, approach = clue['target_core'], clue['interaction_core']
        assert sum(abs(x - y) for x, y in zip(target, approach)) == 1
        assert kind(target) == 'WALL', 'Contextual Headbutt must not own clue input'

    report = dict(native_terrain_parity=True, foot_reachable_core_tiles=len(seen),
                  mire_approaches_reachable=True, map_count=4,
                  **check_layout_contract(layout), **check_piers(design, layout, pack),
                  **check_cut_and_choices(design, layout, pack))
    image = scene(SourceArt(pack), pack, tileset, width, height, blocks, actors,
                  archive=args.output / 'originals', native_origin=True)
    save_render(image, args.output / 'whole-route.png')
    x0, y0, x1, y1 = mire['bounds_metatiles']
    save_render(image.crop((x0 * 32, y0 * 32, x1 * 32, y1 * 32)),
                args.output / 'ancient-mire.png')
    for name, (x, y) in [('entry', mire['entry_core']),
                         ('sanctuary', mire['interaction_core']),
                         *[(a['id'].lower(), a['interaction_core'])
                           for a in mire['clue_stations']]]:
        # Native camera extent is ten by nine 16px gameplay tiles.
        cx, cy = x * 16 + 8, y * 16 + 8
        save_render(image.crop((cx - 80, cy - 72, cx + 80, cy + 72)),
                    args.output / f'camera-{name}.png')
    (args.output / 'review.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
