#!/usr/bin/env python3
"""Review Rochester's explicit native court without writing a game pack."""
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


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def kind(pack, module, point):
    x, y = point
    attrs = module['attributes']
    block = module['blocks'][y // 2 * attrs['width'] + x // 2]
    return pack['data']['tilesets'][attrs['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]


def distances(pack, module, start):
    attrs = module['attributes']
    occupied = {(o['x'], o['y']) for o in module['objects']}
    found = {start: 0}
    todo = deque([start])
    while todo:
        x, y = todo.popleft()
        for point in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if (point in found or point in occupied or not
                    (0 <= point[0] < attrs['width'] * 2 and 0 <= point[1] < attrs['height'] * 2)):
                continue
            if kind(pack, module, point) == 'FLOOR':
                found[point] = found[x, y] + 1
                todo.append(point)
    return found


def compose(pack, source):
    original = pack['data']['maps'][source['map']]
    assert digest(json.dumps(original, sort_keys=True, separators=(',', ':')).encode()) == source['native_module_sha256']
    width, height = source['dimensions_metatiles']
    def parse(rows):
        assert len(rows) == height
        assert all(len(row.split()) == width for row in rows)
        return [int(word, 16) for row in rows for word in row.split()]
    assert original['blocks'] == parse(source['before_rows'])
    assert original['attributes']['connections'] == source['preserved_connections']
    assert [original['attributes']['width'], original['attributes']['height']] == source['dimensions_metatiles']
    assert original['attributes']['tileset_name'] == source['tileset']
    assert source['source_pixels_modified'] is False
    result = deepcopy(original)
    result['blocks'] = parse(source['rows'])
    assert len(result['blocks']) == len(original['blocks'])
    for move in source['building_moves']:
        x, y, w, h = move['source_rect_metatiles']
        tx, ty = move['destination_top_left_metatiles']
        for dy in range(h):
            assert (original['blocks'][(y + dy) * width + x:(y + dy) * width + x + w]
                    == result['blocks'][(ty + dy) * width + tx:(ty + dy) * width + tx + w]), move
    for moves, records, key in [
        (source['warp_moves'], result['events']['warps'], 'index'),
        (source['actor_moves'], result['objects'], 'object_identifier'),
        (source['sign_moves'], result['events']['bg_events'], 'script'),
    ]:
        for move in moves:
            identity = move.get('id', move.get('index'))
            record = next(row for row in records if row[key] == identity)
            assert [record['x'], record['y']] == move['from']
            record['x'], record['y'] = move['to']
    for actor in result['objects']:
        assert kind(pack, result, (actor['x'], actor['y'])) == 'FLOOR', actor['object_identifier']
    # Each read event now occupies the native post's real solid quadrant.
    for sign in result['events']['bg_events']:
        assert kind(pack, result, (sign['x'], sign['y'])) == 'WALL', sign['script']
    reachable = distances(pack, result, (4, 18))
    returns = source['public_door_returns']
    for name, point in returns.items():
        assert tuple(point) in reachable, name
    for point in source['route_ports'][2:]:
        assert tuple(point) in reachable, 'free east approach'
    for sign in result['events']['bg_events']:
        x, y = sign['x'], sign['y']
        assert any(p in reachable for p in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)])
    for index in [1, 3, 5, 7, 9, 11]:
        door = next(row for row in result['events']['warps'] if row['index'] == index)
        assert kind(pack, result, (door['x'], door['y'])) == 'DOOR', door
    for block in result['blocks']:
        collisions = pack['data']['tilesets'][source['tileset']]['collision'][f'{block:02x}']
        assert not set(collisions).intersection({'ICE', 'WATER', 'CUT_TREE', 'TALL_GRASS'})
    old_returns = {name: (warp['x'], warp['y']) for name, index in zip(returns, [2, 4, 6, 8, 10, 12])
                   for warp in original['events']['warps'] if warp['index'] == index}
    comparisons = []
    for trip in source['travel_comparisons']:
        a, b = trip['from'], trip['to']
        before = distances(pack, original, old_returns[a])[old_returns[b]]
        after = distances(pack, result, tuple(returns[a]))[tuple(returns[b])]
        assert after < before, trip
        comparisons.append(dict(trip, before=before, after=after))
    return original, result, comparisons


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=Path('planning/minnesota/towns/rochester-care-green-v01/source-v01.json'))
    cli.add_argument('--installed', type=Path,
                     help='Render the actual published town after comparing its source projection')
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    raw = args.source.read_bytes()
    source = json.loads(raw)
    world_raw = (ROOT / 'planning/minnesota/world.json').read_bytes()
    world = json.loads(world_raw)
    require_selected_ancestor(world, source['selected_ancestor_revision'], source['selected_ancestor_sha256'])
    parent = ROOT / source['parent_pack']
    assert digest(parent.read_bytes()) == source['parent_pack_sha256']
    pack = load_pack(parent)
    started = time.perf_counter()
    before, after, trips = compose(pack, source)
    installed_pack = None
    installed_sha256 = None
    if args.installed:
        installed_sha256 = digest(args.installed.read_bytes())
        installed_pack = load_pack(args.installed)
        actual = installed_pack['data']['maps'][source['map']]
        # The independent full-root audit separately projects the refreshed
        # native event wire. All other typed map data must match this preview.
        assert set(actual) == set(after)
        assert all(actual[key] == value for key, value in after.items()
                   if key != 'map_event_section_commands')
        after = actual
    args.output.mkdir(exist_ok=False, parents=True)
    (args.output / 'source-used.json').write_bytes(raw)
    images = []
    for name, module, render_pack in [('before', before, pack),
                                      ('after', after, installed_pack or pack)]:
        archive = args.output / name
        archive.mkdir()
        image = native_room(module, render_pack, archive=archive, native_origin=True)
        save_render(image, args.output / (name + '-whole-original.png'))
        images.append(image)
    pair = Image.new('RGB', (images[0].width * 2, images[0].height))
    pair.paste(images[0], (0, 0))
    pair.paste(images[1], (images[0].width, 0))
    save_render(pair, args.output / 'before-after-original.png')
    report = dict(status='PASS_INSTALLED_TOWN' if args.installed else 'PASS_SOURCE_ONLY',
                  source=str(args.source), source_sha256=digest(raw),
                  installed_pack=str(args.installed) if args.installed else None,
                  installed_pack_sha256=installed_sha256,
                  parent_sha256=source['parent_pack_sha256'], selected_revision=world['active_local_review_revision'],
                  travel_comparisons=trips,
                  sign_events_moved=len(source['sign_moves']),
                  previously_misaligned_signs=sum(
                      kind(pack, before, (sign['x'], sign['y'])) != 'WALL'
                      for sign in before['events']['bg_events']),
                  changed_metatiles=sum(a != b for a, b in zip(before['blocks'], after['blocks'])),
                  seconds_after_load=round(time.perf_counter() - started, 3),
                  scope='Whole-town native art and static collision/actor/door approaches. '
                        'Installed mode compares the published typed map with the source projection. '
                        'Complete-root preservation and native input/Continue have separate receipts.')
    if args.installed:
        assert digest(args.installed.read_bytes()) == installed_sha256
    assert world_raw == (ROOT / 'planning/minnesota/world.json').read_bytes()
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
