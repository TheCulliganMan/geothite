#!/usr/bin/env python3
"""Preview an explicit city revision without publishing or changing source files."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, layout_blocks, load_pack
from review_passage_native import actors


def render(pack, art, directory, output):
    started = time.perf_counter()
    sources = {name: (directory / name).read_bytes()
               for name in ['after.layout.json', 'recomposition.contract.json']}
    layout = json.loads(sources['after.layout.json'])
    contract = json.loads(sources['recomposition.contract.json'])
    original = pack['data']['maps'][contract['city']]
    module_hash = hashlib.sha256(json.dumps(
        original, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
    assert module_hash == contract['native_module_sha256'], 'Parent city differs'
    assert layout['id'] == contract['city'] and layout['source_pixels_modified'] is False
    width, height = layout['dimensions_metatiles']
    assert [width, height] == contract['dimensions_metatiles'] == contract['old_dimensions_metatiles']
    assert layout['compiled_tileset'] == original['attributes']['tileset_name']
    city = deepcopy(original)
    city['blocks'] = layout_blocks(layout)
    sections = {'object': city['objects'], 'background': city['events']['bg_events'],
                'coordinate': city['events']['coord_events'], 'warp': city['events']['warps']}
    inventory = set()
    for anchor in contract['anchor_overrides']:
        key = anchor['kind'], anchor['index']
        assert key not in inventory, 'Repeated anchor'
        inventory.add(key)
        target = sections[key[0]][key[1]]
        assert [target['x'], target['y']] == anchor['before'], 'Anchor preimage differs'
        target['x'], target['y'] = anchor['after']
    assert inventory == {(kind, i) for kind, rows in sections.items() for i in range(len(rows))}
    collision = pack['data']['tilesets'][layout['compiled_tileset']]['collision']

    def kind(module, point):
        x, y = point
        assert 0 <= x < width * 2 and 0 <= y < height * 2, 'Anchor outside map'
        return collision[f"{module['blocks'][y // 2 * width + x // 2]:02x}"][y % 2 * 2 + x % 2]

    for i, (before, after) in enumerate(zip(original['blocks'], city['blocks'])):
        x, y = i % width, i // width
        if x in [0, width - 1] or y in [0, height - 1]:
            assert before == after, 'Street revision changes perimeter'
        if any(v.startswith(('WATER', 'BRIDGE_', 'HOP_'))
               for v in collision[f'{before:02x}']):
            assert before == after, 'Street revision changes river, bridge or ledge'
        assert 'CUT_TREE' not in collision[f'{after:02x}'], 'Incidental Cut canopy'
    occupied = {(v['x'], v['y']) for v in city['objects']}
    assert len(occupied) == len(city['objects']), 'Overlapping bodies'
    start = tuple(contract['arrival_core'])
    seen = {start}
    queue = deque(seen)
    foot = {'FLOOR', 'LADDER', 'BRIDGE_NS_DECK', 'BRIDGE_EW_DECK'}
    active_warps = {(v['x'], v['y']) for v in city['events']['warps']
                    if kind(city, (v['x'], v['y'])) != 'FLOOR'}
    while queue:
        x, y = queue.popleft()
        for p in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if (0 <= p[0] < width * 2 and 0 <= p[1] < height * 2
                    and p not in seen and p not in occupied and p not in active_warps
                    and kind(city, p) in foot):
                seen.add(p)
                queue.append(p)
    retained_coordinate_events = 0
    for category, rows in sections.items():
        prior = original['objects'] if category == 'object' else original['events'][
            {'background': 'bg_events', 'coordinate': 'coord_events', 'warp': 'warps'}[category]]
        for index, (before, after) in enumerate(zip(prior, rows)):
            at = after['x'], after['y']
            assert kind(original, (before['x'], before['y'])) == kind(city, at), 'Changed anchor collision'
            if category == 'coordinate' and before == after:
                # Guard triggers can stand beyond a blocking officer or in
                # water. Preserve their whole local footprint; their access and
                # denial states need separate production-controller coverage.
                bx, by = at[0] // 2, at[1] // 2
                assert all(original['blocks'][y * width + x] == city['blocks'][y * width + x]
                           for y in range(max(0, by - 1), min(height, by + 2))
                           for x in range(max(0, bx - 1), min(width, bx + 2))), (
                    'Changed retained trigger footprint', index, at)
                retained_coordinate_events += 1
                continue
            if category == 'warp' and kind(city, at) == 'FLOOR':
                assert at in seen, 'Blocked passive return'
            else:
                assert any(p in seen for p in [(at[0] - 1, at[1]), (at[0] + 1, at[1]),
                                              (at[0], at[1] - 1), (at[0], at[1] + 1)]), (
                    'Blocked approach', category, index, at, after)
    digest = hashlib.sha256(b''.join(sources.values())).hexdigest()
    destination = output / f'{digest[:12]}-{time.time_ns()}'
    destination.mkdir(parents=True, exist_ok=False)
    terrain = art.render(layout['tileset'], width, height, city['blocks'],
                         derived=layout.get('derived_blocks', []))
    scene = actors(pack, terrain, city)
    terrain.save(destination / 'terrain-original.png')
    scene.save(destination / 'whole-town-original.png')
    scene.convert('RGB').resize((width * 16, height * 16), Image.Resampling.NEAREST).save(
        destination / 'preview.png')
    for name, raw in sources.items():
        (destination / name).write_bytes(raw)
    record = dict(status='SOURCE_PREVIEW', output=str(destination),
                  source_sha256={name: hashlib.sha256(raw).hexdigest()
                                 for name, raw in sources.items()},
                  reachable_foot_tiles=len(seen),
                  unchanged_coordinate_event_footprints=retained_coordinate_events,
                  changed_metatiles=sum(a != b for a, b in zip(original['blocks'], city['blocks'])),
                  render_seconds=round(time.perf_counter() - started, 3),
                  scope='Source and ordinary Foot graph only; no pack or gameplay state is changed.')
    (destination / 'review.json').write_text(json.dumps(record, indent=2) + '\n')
    (output / 'latest.json').write_text(json.dumps(record, indent=2) + '\n')
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'directory', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.parent)
    art = SourceArt(pack)
    previous = None
    while True:
        try:
            raw = b''.join((args.directory / name).read_bytes()
                           for name in ['after.layout.json', 'recomposition.contract.json'])
            digest = hashlib.sha256(raw).hexdigest()
            if digest != previous:
                try:
                    print(json.dumps(render(pack, art, args.directory, args.output)), flush=True)
                except (AssertionError, KeyError, ValueError) as error:
                    if not args.watch:
                        raise
                    print(json.dumps(dict(error=str(error), source_sha256=digest)), flush=True)
                previous = digest
        except FileNotFoundError:
            if not args.watch:
                raise
        if not args.watch:
            break
        time.sleep(0.25)


if __name__ == '__main__':
    main()
