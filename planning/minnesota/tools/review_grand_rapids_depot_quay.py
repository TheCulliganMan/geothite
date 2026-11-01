#!/usr/bin/env python3
"""Review explicit Grand Rapids depot/quay rows without serializing a game pack."""
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


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def kind(pack, module, point):
    x, y = point
    a = module['attributes']
    block = module['blocks'][y // 2 * a['width'] + x // 2]
    return pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]


def distances(pack, module, start, ignore_guards=False):
    a = module['attributes']
    occupied = {(o['x'], o['y']) for o in module['objects']
                if not (ignore_guards and o['object_identifier'].startswith('MN_GRAND_RAPIDS_'))}
    result, pending = {start: 0}, deque([start])
    while pending:
        x, y = pending.popleft()
        for point in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if (point in result or point in occupied or not
                    (0 <= point[0] < a['width'] * 2 and 0 <= point[1] < a['height'] * 2)):
                continue
            if kind(pack, module, point) == 'FLOOR':
                result[point] = result[x, y] + 1
                pending.append(point)
    return result


def compose(pack, source):
    original = pack['data']['maps'][source['map']]
    assert sha(json.dumps(original, sort_keys=True, separators=(',', ':')).encode()) == source['native_module_sha256']
    width, height = source['dimensions_metatiles']
    assert (width, height) == (24, 24)
    def parse(rows):
        assert len(rows) == height and all(len(row.split()) == width for row in rows)
        return [int(word, 16) for row in rows for word in row.split()]
    assert original['blocks'] == parse(source['before_rows'])
    assert original['attributes']['tileset_name'] == source['tileset']
    assert original['attributes']['connections'] == source['preserved_connections']
    assert source['source_pixels_modified'] is False
    after = deepcopy(original)
    after['blocks'] = parse(source['rows'])
    for move in source['building_moves']:
        x, y, w, h = move['source_rect_metatiles']
        tx, ty = move['destination_top_left_metatiles']
        for dy in range(h):
            assert (original['blocks'][(y + dy) * width + x:(y + dy) * width + x + w]
                    == after['blocks'][(ty + dy) * width + tx:(ty + dy) * width + tx + w])
    for moves, records, key in [
        (source['warp_moves'], after['events']['warps'], 'index'),
        (source['actor_moves'], after['objects'], 'object_identifier'),
        (source['sign_moves'], after['events']['bg_events'], 'script'),
    ]:
        for move in moves:
            identity = move.get('id', move.get('index'))
            record = next(row for row in records if row[key] == identity)
            assert [record['x'], record['y']] == move['from']
            record['x'], record['y'] = move['to']
    lo, hi = source['protected_rows_metatiles']
    assert original['blocks'][lo * width:(hi + 1) * width] == after['blocks'][lo * width:(hi + 1) * width]
    for y in range(height):
        for x in range(width):
            if x in [0, width - 1] or y in [0, height - 1]:
                assert original['blocks'][y * width + x] == after['blocks'][y * width + x]
    for y in range(height * 2):
        for x in range(width * 2):
            before_kind, after_kind = kind(pack, original, (x, y)), kind(pack, after, (x, y))
            for protected in [{'TALL_GRASS'}, {'WATER', 'WATERFALL', 'BUOY'}]:
                assert (before_kind in protected) == (after_kind in protected)
            assert after_kind not in ['ICE', 'ICE_2B', 'CUT_TREE']
    reached = distances(pack, after, tuple(source['preserved_spawn']))
    for point in list(source['public_door_returns'].values()) + source['river_approaches']:
        assert tuple(point) in reached and kind(pack, after, tuple(point)) == 'FLOOR', point
    for obj in after['objects']:
        assert kind(pack, after, (obj['x'], obj['y'])) == 'FLOOR', obj['object_identifier']
    for door in after['events']['warps']:
        if door['index'] % 2:
            assert kind(pack, after, (door['x'], door['y'])) == 'DOOR'
    for sign in after['events']['bg_events']:
        x, y = sign['x'], sign['y']
        assert kind(pack, after, (x, y)) == 'WALL', sign['script']
        assert any(p in reached for p in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)])
    qualified = distances(pack, after, tuple(source['preserved_spawn']), ignore_guards=True)
    assert all(tuple(point) in qualified for point in source['route_ports'])
    for y in range(34, 39):
        for x in [25, 26]:
            assert (x, y) in reached, 'both pier lanes must stay clear of bodies'
    before_points = {'west_arrival': (6, 24), 'east_arrival': (43, 24), 'pier': (26, 38),
                     'clinic': (9, 12), 'depot': (27, 14), 'mart': (39, 16)}
    after_points = dict(before_points, **{key: tuple(value) for key, value in source['public_door_returns'].items()})
    trips = []
    for trip in source['travel_comparisons']:
        a, b = trip['start'], trip['end']
        before = distances(pack, original, before_points[a])[before_points[b]]
        now = distances(pack, after, after_points[a])[after_points[b]]
        if trip['shorter_required']:
            assert now < before, (trip, before, now)
        trips.append(dict(trip, before=before, after=now))
    return original, after, trips


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=Path('planning/minnesota/towns/grand-rapids-depot-quay-v01/source-v01.json'))
    cli.add_argument('--installed', type=Path, help='Compare and render an actual published child')
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
    started = time.perf_counter()
    before, after, trips = compose(pack, source)
    installed = None
    installed_sha256 = None
    if args.installed:
        installed_sha256 = sha(args.installed.read_bytes())
        installed = load_pack(args.installed)
        actual = installed['data']['maps'][source['map']]
        assert set(actual) == set(after)
        assert all(actual[key] == value for key, value in after.items()
                   if key != 'map_event_section_commands')
        after = actual
    args.output.mkdir(exist_ok=False, parents=True)
    (args.output / 'source-used.json').write_bytes(raw)
    images = []
    for name, module, render_pack in [('before', before, pack),
                                     ('after', after, installed or pack)]:
        archive = args.output / name
        archive.mkdir()
        image = native_room(module, render_pack, archive=archive, native_origin=True)
        save_render(image, args.output / (name + '-whole-original.png'))
        images.append(image)
    pair = Image.new('RGB', (images[0].width * 2, images[0].height))
    for i, image in enumerate(images):
        pair.paste(image, (i * images[0].width, 0))
    save_render(pair, args.output / 'before-after-original.png')
    report = dict(status='PASS_INSTALLED_TOWN' if args.installed else 'PASS_SOURCE_ONLY',
                  source=str(args.source), source_sha256=sha(raw),
                  installed_pack=str(args.installed) if args.installed else None,
                  installed_pack_sha256=installed_sha256,
                  parent_sha256=source['parent_pack_sha256'], selected_revision=world['active_local_review_revision'],
                  travel_comparisons=trips, changed_metatiles=sum(a != b for a, b in zip(before['blocks'], after['blocks'])),
                  seconds_after_load=round(time.perf_counter() - started, 3),
                  scope='Explicit whole-town source and static geometry only. Original buildings, '
                        'habitat positions, water, lower shore, wardens and connections are retained. '
                        'Rust publication, native story/service/riverfront and visual acceptance remain separate.')
    assert world_raw == (ROOT / 'planning/minnesota/world.json').read_bytes()
    if args.installed:
        assert sha(args.installed.read_bytes()) == installed_sha256
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
