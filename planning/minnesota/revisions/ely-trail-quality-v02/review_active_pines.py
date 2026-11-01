#!/usr/bin/env python3
"""Read-only whole-town pine and walking audit of the selected local pack."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import SourceArt, load_pack
from review_north_shore import oriented_sprite


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--epoch', required=True)
    args = parser.parse_args()
    world = json.loads((ROOT / 'planning/minnesota/world.json').read_text())
    revision = world['active_local_review_revision']
    pack_path = ROOT / world[revision]['pack']
    pack = load_pack(pack_path)
    maps = pack['data']['maps']
    town = maps['MnEly']
    attributes = town['attributes']
    width, height = attributes['width'], attributes['height']
    assert (width, height) == (20, 18)
    assert attributes['tileset_name'] == 'mn_ely_snow'
    collision = pack['data']['tilesets']['mn_ely_snow']['collision']
    assert collision['83'] == ['WALL'] * 4
    assert attributes['border_block'] == 0x83
    assert town['blocks'].count(0x83) == 171
    assert not set(town['blocks']) & {0x0f, 0x32, 0x33}
    assert not any('CUT_TREE' in collision[f'{block:02x}']
                   for block in town['blocks'])
    occupied = {(actor['x'], actor['y']) for actor in town['objects']}

    def walkable(at):
        x, y = at
        if not (0 <= x < width * 2 and 0 <= y < height * 2):
            return False
        block = town['blocks'][y // 2 * width + x // 2]
        return at not in occupied and collision[f'{block:02x}'][y % 2 * 2 + x % 2] in {'FLOOR', 'DOOR', 'TALL_GRASS'}

    reached = {(24, 0)}
    assert walkable((24, 0))
    queue = deque(reached)
    while queue:
        x, y = queue.popleft()
        for target in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if target not in reached and walkable(target):
                reached.add(target)
                queue.append(target)
    ports = [(24, 0), (25, 0), (39, 14), (39, 15)]
    assert all(port in reached for port in ports)
    doors = town['events']['warps']
    assert len(doors) == 8
    assert all((door['x'], door['y']) in reached for door in doors)
    connections = attributes['connections']
    assert {(entry['direction'], entry['target_map']) for entry in connections} == {
        ('north', 'MnR13C00R02'), ('east', 'MnElyShorePass')}
    for entry in connections:
        reverse = 'south' if entry['direction'] == 'north' else 'west'
        assert any(other['direction'] == reverse and other['target_map'] == 'MnEly'
                   and other['offset'] == -entry['offset']
                   for other in maps[entry['target_map']]['attributes']['connections'])
    output = ROOT / 'planning/minnesota/renders' / args.epoch
    output.mkdir(parents=True, exist_ok=False)
    scene = SourceArt(pack).render('mn_ely_snow', width, height, town['blocks'], 'day').convert('RGBA')
    for actor in town['objects']:
        facing = {'UP': 'north', 'RIGHT': 'east', 'LEFT': 'west'}.get(
            actor['spritemovedata'].rsplit('_', 1)[-1], 'south')
        scene.alpha_composite(oriented_sprite(pack, actor['sprite'], facing),
                              (actor['x'] * 16, actor['y'] * 16))
    save_render(scene, output / 'MnEly-whole.png')
    receipt = dict(active_revision=revision, pack=str(pack_path.relative_to(ROOT)),
                   sha256=hashlib.sha256(pack_path.read_bytes()).hexdigest(),
                   identity=pack['identity']['content_hash'], forest_blocks=171,
                   solid_pine_border=True, incidental_cut_trees=0,
                   all_service_doors_and_both_port_lanes_reachable=True,
                   reciprocal_connections=connections,
                   evidence_scope='Read-only native-art and actor-aware static walking audit. '
                                  'Production controller and snowfall evidence remains separately documented in PINES.md.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
