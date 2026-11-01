#!/usr/bin/env python3
"""Read-only whole-town source loop for Mankato's terrace and pond garden."""
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
from render_reference import load_pack
from review_selection_lineage import require_selected_ancestor

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/towns/mankato-terrace-pond-v01/source-v04.json'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def rows(source):
    result = [[int(word, 16) for word in row.split()] for row in source]
    assert len(result) == 18 and all(len(row) == 24 for row in result)
    return [value for row in result for value in row]


def flood(start, allowed):
    found, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if at in allowed and at not in found:
                found.add(at)
                queue.append(at)
    return found


def compose(pack, source):
    original = pack['data']['maps'][source['map']]
    canonical = json.dumps(original, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
    assert sha(canonical) == source['native_module_sha256']
    assert original['blocks'] == rows(source['before_rows'])
    assert source['dimensions_metatiles'] == [24, 18] and source['tileset'] == 'johto'
    after = deepcopy(original)
    after['blocks'] = rows(source['rows'])
    assert after['blocks'][15 * 24 + 16:15 * 24 + 21] == [0x40, 0x41, 0x41, 0x41, 0x42]
    for move in source['building_moves']:
        x, y, width, height = move['source_rect_metatiles']
        tx, ty = move['destination_top_left_metatiles']
        for dy in range(height):
            assert original['blocks'][(y + dy) * 24 + x:(y + dy) * 24 + x + width] == after['blocks'][(ty + dy) * 24 + tx:(ty + dy) * 24 + tx + width]
    for y in range(18):
        for x in range(24):
            if x in [0, 23] or y in [0, 1, 17] or (4 <= x < 13 and 8 <= y < 12):
                assert original['blocks'][y * 24 + x] == after['blocks'][y * 24 + x], (x, y)
    for key, field, identity in [('warp_moves', 'warps', 'index'),
                                  ('sign_moves', 'bg_events', 'script')]:
        for move in source[key]:
            record, = [r for r in after['events'][field]
                       if r[identity] == move.get('id', move.get('index'))]
            assert [record['x'], record['y']] == move['from']
            record['x'], record['y'] = move['to']
    assert not source['actor_moves'] and after['objects'] == original['objects']
    assert after['attributes']['connections'] == source['preserved_connections']
    collision = pack['data']['tilesets']['johto']['collision']

    def kind(at):
        x, y = at
        return collision[f"{after['blocks'][y // 2 * 24 + x // 2]:02x}"][y % 2 * 2 + x % 2]

    terrain = {(x, y): kind((x, y)) for y in range(36) for x in range(48)}
    assert not {'ICE', 'CUT_TREE'} & set(terrain.values())
    bodies = {(o['x'], o['y']) for o in after['objects']}
    floor = {p for p, k in terrain.items() if k == 'FLOOR'} - bodies
    reached = flood((46, 18), floor)
    for spawn in pack['data']['runtime_spawn_points'].values():
        if spawn['mapName'] == source['map']:
            assert (spawn['tileX'], spawn['tileY']) in reached, 'native arrival spawn disconnected'
    assert floor - reached == {(24, 0), (25, 0), (24, 1), (25, 1)}
    assert len(floor) + len(bodies) == source['expected_floor_quadrants']
    for at in bodies:
        assert terrain[at] == 'FLOOR'
    for at in source['public_door_returns'].values():
        assert tuple(at) in reached
    for warp in after['events']['warps']:
        assert terrain[warp['x'], warp['y']] == ('DOOR' if warp['index'] % 2 else 'FLOOR')
    for sign in after['events']['bg_events']:
        x, y = sign['x'], sign['y']
        assert terrain[x, y] == 'WALL'
        assert any(at in reached for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)])
    for promenade in source['public_promenades']:
        x, y, width, height = promenade['rectangle_core']
        assert all((xx, yy) in reached for yy in range(y, y + height)
                   for xx in range(x, x + width)), promenade['id']
    water = {p for p, k in terrain.items() if k == 'WATER'}
    assert len(water) == source['expected_water_quadrants']
    assert flood(min(water), water) == water
    assert all(32 <= x <= 41 and 24 <= y <= 29 for x, y in water)
    for bank in source['fishing_banks']:
        assert tuple(bank['stand']) in reached and tuple(bank['water']) in water
    assert sum(a != b for a, b in zip(original['blocks'], after['blocks'])) == source['expected_changed_metatiles']
    return original, after


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=SOURCE)
    cli.add_argument('--installed', type=Path)
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    raw = args.source.read_bytes()
    source = json.loads(raw)
    world_raw = (ROOT / 'planning/minnesota/world.json').read_bytes()
    world = json.loads(world_raw)
    require_selected_ancestor(world, source['selected_ancestor_revision'], source['selected_ancestor_sha256'])
    parent = ROOT / source['parent_pack']
    assert sha(parent.read_bytes()) == source['parent_pack_sha256']
    pack = load_pack(parent)
    start = time.perf_counter()
    before, after = compose(pack, source)
    installed = load_pack(args.installed) if args.installed else None
    if installed:
        actual = installed['data']['maps'][source['map']]
        assert set(actual) == set(after)
        assert all(actual[key] == value for key, value in after.items()
                   if key != 'map_event_section_commands')
        after = actual
    args.output.mkdir(exist_ok=False, parents=True)
    (args.output / 'source-used.json').write_bytes(raw)
    images = []
    for label, module, render_pack in [('before', before, pack), ('after', after, installed or pack)]:
        archive = args.output / label
        archive.mkdir()
        image = native_room(module, render_pack, archive=archive, native_origin=True)
        save_render(image, args.output / (label + '-whole-original.png'))
        images.append(image)
    pair = Image.new('RGB', (images[0].width * 2, images[0].height))
    for i, image in enumerate(images):
        pair.paste(image, (i * images[0].width, 0))
    save_render(pair, args.output / 'before-after-original.png')
    report = dict(status='PASS_INSTALLED_TOWN' if installed else 'PASS_SOURCE_ONLY',
                  source=str(args.source), source_sha256=sha(raw),
                  installed_pack=str(args.installed) if args.installed else None,
                  changed_metatiles=source['expected_changed_metatiles'],
                  floor_quadrants=source['expected_floor_quadrants'],
                  water_quadrants=source['expected_water_quadrants'],
                  public_casting_banks=source['fishing_banks'],
                  seconds_after_load=round(time.perf_counter() - start, 3),
                  scope='Source-only geometry and native whole-town render; gameplay, story, '
                        'services, fishing and visual acceptance require separate evidence.')
    assert (ROOT / 'planning/minnesota/world.json').read_bytes() == world_raw
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
