#!/usr/bin/env python3
"""Render Brainerd's explicit whole-town rows without exporting game content."""
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
SOURCE = ROOT / 'planning/minnesota/towns/brainerd-lakeside-loop-v01/proposal.json'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def kind(pack, town, point):
    x, y = point
    a = town['attributes']
    block = town['blocks'][y // 2 * a['width'] + x // 2]
    return pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]


def distances(pack, town, start):
    a = town['attributes']
    bodies = {(o['x'], o['y']) for o in town['objects']}
    seen, queue = {start: 0}, deque([start])
    while queue:
        x, y = queue.popleft()
        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if (at in seen or at in bodies or not
                    (0 <= at[0] < a['width'] * 2 and 0 <= at[1] < a['height'] * 2)):
                continue
            if kind(pack, town, at) == 'FLOOR':
                seen[at] = seen[x, y] + 1
                queue.append(at)
    return seen


def compose(pack, source):
    original = pack['data']['maps'][source['map']]
    assert digest(json.dumps(original, sort_keys=True, separators=(',', ':')).encode()) == source['native_module_sha256']
    assert source['source_pixels_modified'] is False
    assert source['dimensions_metatiles'] == [32, 24]
    assert original['attributes']['tileset_name'] == source['tileset']
    assert original['attributes']['connections'] == source['preserved_connections']

    def parse(rows):
        assert len(rows) == 24 and all(len(row.split()) == 32 for row in rows)
        return [int(word, 16) for row in rows for word in row.split()]

    assert original['blocks'] == parse(source['before_rows'])
    town = deepcopy(original)
    town['blocks'] = parse(source['rows'])
    assert town['blocks'][6 * 32 + 23:6 * 32 + 25] == [0x40, 0x42], 'fitted pier cap corners'
    for move in source['building_moves']:
        sx, sy, width, height = move['source_rect_metatiles']
        tx, ty = move['destination_top_left_metatiles']
        for dy in range(height):
            assert original['blocks'][(sy + dy) * 32 + sx:(sy + dy) * 32 + sx + width] == town['blocks'][(ty + dy) * 32 + tx:(ty + dy) * 32 + tx + width]
    for move in source['warp_moves']:
        warp, = [w for w in town['events']['warps'] if w['index'] == move['index']]
        assert [warp['x'], warp['y']] == move['from_tile']
        warp['x'], warp['y'] = move['to_tile']
    protected = {(x, y) for x in range(32) for y in range(24)
                 if x in [0, 31] or y in [0, 23]}
    for area in source['protected_rectangles_metatiles']:
        x, y, width, height = area['rectangle']
        protected |= {(xx, yy) for yy in range(y, y + height) for xx in range(x, x + width)}
    for x, y in protected:
        assert town['blocks'][y * 32 + x] == original['blocks'][y * 32 + x], ('protected', x, y)
    reached = distances(pack, town, (9, 22))
    for lane in source['public_lanes']:
        x, y, width, height = lane['rectangle_core']
        assert all((xx, yy) in reached for yy in range(y, y + height)
                   for xx in range(x, x + width)), ('public passing lane', lane['id'])
    for name, at in source['public_door_returns'].items():
        assert tuple(at) in reached, ('door return', name)
    for warp in town['events']['warps']:
        assert kind(pack, town, (warp['x'], warp['y'])) == ('DOOR' if warp['index'] % 2 else 'FLOOR')
    for obj in town['objects']:
        p = obj['x'], obj['y']
        assert kind(pack, town, p) == 'FLOOR'
        # Wardens intentionally stand between public streets and outer arrival pockets.
        assert any(q in reached for q in [(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)])
    for sign in town['events']['bg_events']:
        p = sign['x'], sign['y']
        assert kind(pack, town, p) == 'WALL'
        assert any(q in reached for q in [(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)])
    for spawn in pack['data']['runtime_spawn_points'].values():
        if spawn['mapName'] == town['id']:
            assert (spawn['tileX'], spawn['tileY']) in reached
    for y in range(48):
        for x in range(64):
            at = x, y
            before, after = kind(pack, original, at), kind(pack, town, at)
            assert after not in ['ICE', 'CUT_TREE']
            if before == 'WATER' or after == 'WATER':
                assert original['blocks'][y // 2 * 32 + x // 2] == town['blocks'][y // 2 * 32 + x // 2], ('water/shore', at)
    for bank in source['fishing_banks']:
        assert tuple(bank['stand']) in reached
        assert kind(pack, town, bank['water']) == 'WATER'
    assert sum(a != b for a, b in zip(original['blocks'], town['blocks'])) == source['expected_changed_metatiles']
    return original, town


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=SOURCE)
    cli.add_argument('--output', type=Path, required=True)
    cli.add_argument('--installed', type=Path)
    args = cli.parse_args()
    assert not args.output.exists(), 'retain every prior render'
    source = json.loads(args.source.read_text())
    require_selected_ancestor(json.loads((ROOT / 'planning/minnesota/world.json').read_text()),
                              source['selected_ancestor_revision'], source['selected_ancestor_pack_sha256'])
    parent = ROOT / source['parent_pack']
    assert digest(parent.read_bytes()) == source['parent_pack_sha256']
    pack = load_pack(parent)
    started = time.perf_counter()
    before, after = compose(pack, source)
    if args.installed:
        installed = load_pack(args.installed)['data']['maps'][source['map']]
        expected = deepcopy(after)
        expected['map_event_section_commands'] = installed['map_event_section_commands']
        assert expected == installed
    args.output.mkdir(parents=True)
    images = []
    for label, town in [('before', before), ('after', after)]:
        image = native_room(town, pack, archive=args.output / label, native_origin=True)
        save_render(image, args.output / f'{label}-whole-original.png')
        images.append(image)
    combined = Image.new('RGB', (images[0].width * 2, images[0].height), (18, 25, 28))
    for i, image in enumerate(images):
        combined.paste(image, (i * image.width, 0))
    save_render(combined, args.output / 'before-after-original.png')
    report = dict(status='PASS_INSTALLED_TOWN' if args.installed else 'PASS_SOURCE_ONLY',
                  source=str(args.source), source_sha256=digest(args.source.read_bytes()),
                  changed_metatiles=source['expected_changed_metatiles'],
                  public_casting_banks=source['fishing_banks'],
                  seconds_after_load=round(time.perf_counter() - started, 3),
                  scope='Explicit whole-town source/native rows and actor-aware paths. Native publication, input and visual acceptance remain separate.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
