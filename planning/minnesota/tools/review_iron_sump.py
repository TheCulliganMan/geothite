#!/usr/bin/env python3
"""Retain whole native-art sump previews and check dry/Surf compartment graphs."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def render(pack, art, source, output):
    started = time.perf_counter()
    raw = source.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    version = output / (digest[:12] + '-' + str(time.time_ns()))
    version.mkdir(parents=True, exist_ok=False)
    (version / 'source.json').write_bytes(raw)
    spec = json.loads(raw)
    width, height = spec['dimensions_metatiles']
    rows = [[int(v, 16) for v in row.split()] for row in spec['rows']]
    assert len(rows) == height and all(len(row) == width for row in rows)
    blocks = [v for row in rows for v in row]
    module = pack['data']['maps'][spec['id']]
    alias = spec.get('base_tileset', spec['tileset'])
    scene = actors(npc_palette_pack(pack, 'nite'), art.render(alias, width, height, blocks, 'nite', derived=spec.get('derived_blocks')), module)
    scene.save(version / 'whole-original.png')
    scene.resize((scene.width * 2, scene.height * 2), Image.Resampling.NEAREST).save(version / 'whole-2x.png')
    collision = dict(pack['data']['tilesets'][alias]['collision'])
    for recipe in spec.get('derived_blocks', []):
        collision[recipe['id']] = recipe['collision']
    bodies = {(o['x'], o['y']) for o in module['objects']}

    def kind(at, opened=False):
        x, y = at
        b = blocks[y // 2 * width + x // 2]
        if opened and (x // 2, y // 2) == (16, 3):
            b = 13
        return collision[f'{b:02x}'][y % 2 * 2 + x % 2]

    def flood(seed, nodes):
        assert seed in nodes
        seen, queue = {seed}, deque([seed])
        while queue:
            x, y = queue.popleft()
            for p in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
                if p in nodes and p not in seen:
                    seen.add(p)
                    queue.append(p)
        return seen

    reports = []
    for opened in [False, True]:
        floor = {(x, y) for y in range(height * 2) for x in range(width * 2)
                 if kind((x, y), opened) == 'FLOOR' and (x, y) not in bodies}
        water = {(x, y) for y in range(height * 2) for x in range(width * 2)
                 if kind((x, y), opened) == 'WATER'}
        west, east, pocket = [flood(p, floor) for p in [(4,19), (30,19), (34,5)]]
        assert west.isdisjoint(east | pocket)
        assert (east == pocket) if opened else east.isdisjoint(pocket)
        assert west | east | pocket == floor, ('orphan floor', sorted(floor - west - east - pocket))
        surf = flood((12,12), floor | water)
        assert surf == west | water, 'Surf must remain in the western drift'
        reports.append(dict(opened=opened, west=len(west), east=len(east), pocket=len(pocket), water=len(water)))
    for warp in module['events']['warps']:
        assert kind((warp['x'], warp['y'])) == ('LADDER' if warp['index'] % 2 else 'FLOOR'), warp
    assert all(kind(p) == 'FLOOR' for p in bodies)
    assert all(kind((x,y)) == 'WALL' for y in range(height * 2) for x in range(width * 2)
               if x in [0,width*2-1] or y in [0,height*2-1])
    bank = spec['fishing_bank']
    assert kind(tuple(bank['tile'])) == 'FLOOR' and kind(tuple(bank['water'])) == 'WATER'
    receipt = dict(status='SOURCE_ONLY', source_sha256=digest,
                   whole=str(version / 'whole-2x.png'),
                   warm_render_seconds=round(time.perf_counter() - started,3), states=reports)
    (version / 'review.json').write_text(json.dumps(receipt,indent=2)+'\n')
    (output / 'latest.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt),flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack','source','output']:
        parser.add_argument('--'+key,type=Path,required=True)
    parser.add_argument('--watch',action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        digest = hashlib.sha256(args.source.read_bytes()).hexdigest()
        if digest != previous:
            previous = digest
            try:
                render(pack,art,args.source,args.output)
            except (AssertionError,KeyError,ValueError) as error:
                receipt = dict(status='SOURCE_ERROR',source_sha256=digest,error=str(error))
                version = max(args.output.glob(digest[:12]+'-*'),key=lambda p:p.stat().st_mtime_ns)
                (version/'review.json').write_text(json.dumps(receipt,indent=2)+'\n')
                (args.output/'latest.json').write_text(json.dumps(receipt,indent=2)+'\n')
                print(json.dumps(receipt),flush=True)
                if not args.watch:
                    raise
        if not args.watch:
            return
        time.sleep(0.2)


if __name__ == '__main__':
    main()
