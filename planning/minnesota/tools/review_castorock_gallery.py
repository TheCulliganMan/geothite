#!/usr/bin/env python3
"""Read-only whole-gallery review with the real Strength gate and native art."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_passage_native import actors

ROOM = 'MnR03Cave1F'
BOULDER = 'MN_R03_JAW_BOULDER'
CREATURE = 'MN_R03_JAW_CASTOROCK'


def review(pack, art, before, site, output, published=False):
    started = time.perf_counter()
    assert {k: v for k, v in before.items() if k != 'rows'} == {
        k: v for k, v in site.items() if k != 'rows'}
    module = pack['data']['maps'][ROOM]
    old = [int(s, 16) for row in before['rows'] for s in row.split()]
    blocks = [int(s, 16) for row in site['rows'] for s in row.split()]
    assert (blocks if published else old) == module['blocks'] and len(blocks) == 14 * 8
    collision = pack['data']['tilesets']['cave']['collision']

    def kind(at):
        x, y = at
        return collision[f'{blocks[y // 2 * 14 + x // 2]:02x}'][y % 2 * 2 + x % 2]

    objects = module['objects']
    assert len(objects) == 4
    bodies = {(o['x'], o['y']) for o in objects}
    boulder = tuple(site['boulder'])
    floor = {(x, y) for y in range(16) for x in range(28) if kind((x, y)) == 'FLOOR'}
    water = {(x, y) for y in range(16) for x in range(28) if kind((x, y)) == 'WATER'}
    assert all(kind((x, y)) == 'WALL' for y in range(16) for x in range(28)
               if x in (0, 27) or y in (0, 15))
    assert all(tuple(at) in floor for at in site['clue_approaches'])
    assert all(kind(tuple(at)) == 'WALL' for at in site['clues'])
    assert bodies <= floor

    def flood(start, blocked, surf=False):
        allowed = (floor | (water if surf else set())) - blocked
        assert start in allowed
        seen, todo = {start}, deque([start])
        while todo:
            x, y = todo.popleft()
            for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if at in allowed and at not in seen:
                    seen.add(at)
                    todo.append(at)
        return seen

    arrival = tuple(site['initial_arrival'])
    closed = flood(arrival, bodies, surf=True)
    assert tuple(site['boulder_approach']) in closed
    assert not any(tuple(at) in closed for at in site['clue_approaches'][1:])
    assert tuple(site['creature_approach']) not in closed
    opened = flood(arrival, bodies - {boulder})
    assert opened == floor - (bodies - {boulder}), sorted(floor - bodies - opened)
    for x in range(10, 16):
        assert (x, 6) in floor
    for x, start in [(10, (21, 10)), (15, (14, 6))]:
        seen = flood(start, bodies - {boulder} | {(x, 6)})
        assert arrival in seen and all(tuple(at) in seen for at in site['clue_approaches'])
    output.mkdir(parents=True, exist_ok=False)
    scenes = []
    palette_pack = npc_palette_pack(pack, 'nite')
    for label, values in [('before', old), ('after', blocks)]:
        terrain = art.render('cave', 14, 8, values, 'nite')
        terrain.save(output / f'{label}-terrain-original.png')
        unrevealed = dict(module, objects=[o for o in objects
                                        if o['object_identifier'] != CREATURE])
        scene = actors(palette_pack, terrain, unrevealed)
        scene.save(output / f'{label}-initial-actors-original.png')
        scene.resize((1344, 768), Image.Resampling.NEAREST).save(output / f'{label}-large.png')
        scenes.append((label + ' initial visit; Castorock remains hidden', scene.resize(
            (896, 512), Image.Resampling.NEAREST)))
        actors(palette_pack, terrain, module).save(output / f'{label}-revealed-anchors-original.png')
    room_sheet(scenes, columns=1).save(output / 'whole-before-after.png')
    ledger = [dict(metatile=[i % 14, i // 14], before=f'{a:02x}', after=f'{b:02x}')
              for i, (a, b) in enumerate(zip(old, blocks)) if a != b]
    assert all(change['metatile'][0] >= 8 for change in ledger), 'preserve original west cave'
    record = dict(status='PASS_SOURCE_REVIEW', changed_metatiles=len(ledger), ledger=ledger,
                  ordinary_floor=len(floor), closed_surf_reachable=len(closed),
                  opened_actor_aware_floor=len(opened), water_tiles=len(water),
                  original_western_cave_exact=True, both_pushed_returns_and_clues=True,
                  published_rows_match=published,
                  output=str(output), render_seconds=round(time.perf_counter() - started, 3),
                  scope='Source geometry/visual review; no publication or production input.')
    (output / 'source-proof.json').write_text(json.dumps(record, indent=2) + '\n')
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'before', 'site', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    parser.add_argument('--published', action='store_true',
                        help='Require installed native blocks to match the revised site.')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    before = json.loads(args.before.read_text())
    args.output.mkdir(parents=True, exist_ok=False)
    previous = None
    while True:
        raw = args.site.read_bytes()
        digest = hashlib.sha256(raw).hexdigest()
        if digest != previous:
            destination = args.output / (digest[:12] + '-' + str(time.time_ns()))
            snapshot = args.output / (destination.name + '-site.json')
            snapshot.write_bytes(raw)
            try:
                result = review(pack, art, before, json.loads(raw), destination, args.published)
            except (AssertionError, ValueError, KeyError) as error:
                if not args.watch:
                    raise
                result = dict(status='SOURCE_ERROR', error=str(error), output=str(destination))
            result.update(source_sha256=digest, source_snapshot=str(snapshot))
            (args.output / 'latest.json').write_text(json.dumps(result, indent=2) + '\n')
            print(json.dumps(result), flush=True)
            previous = digest
        if not args.watch:
            break
        time.sleep(0.25)


if __name__ == '__main__':
    main()
