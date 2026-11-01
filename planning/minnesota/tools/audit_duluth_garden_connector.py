#!/usr/bin/env python3
"""Read-only exact-parent and forced-stair proof for Duluth's garden connector."""
import argparse
import base64
from collections import deque
import hashlib
import json
from pathlib import Path

from render_reference import layout_blocks, load_pack

PATCHES = [(27, 17, 0x01, 0x69), (28, 17, 0x69, 0x8c),
           (27, 18, 0x01, 0x69), (28, 18, 0x69, 0x01), (28, 19, 0x69, 0x75)]
LANES = [(24, 14), (25, 14), (84, 14), (85, 14),
         (44, 40), (45, 40), (46, 40), (47, 40), (48, 40), (49, 40)]


def audit(parent, child, layout):
    a, b = parent['data'], child['data']
    old, new = a['maps']['MnDuluth'], b['maps']['MnDuluth']
    expected = old['blocks'].copy()
    for x, y, before, after in PATCHES:
        assert expected[y * 64 + x] == before
        expected[y * 64 + x] = after
    assert new == dict(old, blocks=expected)
    assert layout_blocks(layout) == expected
    label = old['attributes']['blocks_label']
    assert list(base64.b64decode(b['map_blocks'][label], validate=True)) == expected
    normalized = dict(child, data=dict(b, maps=dict(b['maps']), map_blocks=dict(b['map_blocks'])))
    normalized['data']['maps']['MnDuluth'] = old
    normalized['data']['map_blocks'][label] = a['map_blocks'][label]
    normalized['identity'] = parent['identity']
    assert normalized == parent, 'only five blocks, their mirror and derived identity may change'
    collision = b['tilesets'][new['attributes']['tileset_name']]['collision']
    occupied = {(obj['x'], obj['y']) for obj in new['objects']}

    def kind(at, blocks=expected):
        x, y = at
        return collision[f'{blocks[y // 2 * 64 + x // 2]:02x}'][y % 2 * 2 + x % 2]

    def flood(start, blocks=expected, forbidden=frozenset()):
        seen = {start: 0}
        todo = deque([start])
        while todo:
            x, y = todo.popleft()
            for point in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if (0 <= point[0] < 128 and 0 <= point[1] < 92
                        and point not in seen and point not in occupied and point not in forbidden
                        and kind(point, blocks) == 'FLOOR'):
                    seen[point] = seen[(x, y)] + 1
                    todo.append(point)
        return seen

    before, after = flood((55, 33), old['blocks']), flood((55, 33))
    assert before[(64, 37)] == 29 and after[(64, 37)] == 13
    assert set(after) - set(before) == {(57, 36), (57, 37), (57, 39)}
    assert set(before) - set(after) == {(55, 34), (55, 35), (55, 36), (55, 37), (56, 38)}
    walls = [(55, y) for y in range(34, 38)] + [(57, 34), (57, 35), (56, 38), (57, 38)]
    assert all(kind(point) == 'WALL' for point in walls)
    assert all(kind((56, y)) == 'FLOOR' for y in range(33, 38))
    return_band = {(x, y) for x in range(54, 57) for y in range(33, 43)}
    assert (46, 39) not in flood((46, 42), forbidden=return_band)
    assert all((warp['x'], warp['y']) in after for warp in new['events']['warps'] if warp['index'] % 2 == 0)
    returns = []
    for x, y in LANES:
        steps = flood((x, y + 2))[(x, y - 1)]
        assert steps == flood((x, y + 2), old['blocks'])[(x, y - 1)]
        assert steps >= 9 and kind((x, y + 1)) == 'WALL'
        returns.append(dict(upper=[x, y - 1], landing=[x, y + 2], return_steps=steps))
    return dict(changed_metatiles=PATCHES, ordinary_trip_steps_before=29,
                ordinary_trip_steps_after=13, forced_stair_side_walls=walls,
                ledge_returns=returns, all_38_passive_returns_reachable=True,
                both_deliberate_return_stairs_required_when_return_band_sealed=True,
                all_art_audio_story_actors_and_729_other_maps_exact=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'pack', 'layout', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    inputs = [(path, path.read_bytes()) for path in [args.parent, args.pack, args.layout]]
    parent, child = load_pack(args.parent), load_pack(args.pack)
    result = audit(parent, child, json.loads(args.layout.read_text()))
    assert all(path.read_bytes() == data for path, data in inputs)
    assert not args.output.exists()
    args.output.write_text(json.dumps(dict(status='PASS', result=result,
        inputs=[dict(path=str(path), sha256=hashlib.sha256(data).hexdigest()) for path, data in inputs]), indent=2) + '\n')
    print(json.dumps(dict(status='PASS', result=result)))


if __name__ == '__main__':
    main()
