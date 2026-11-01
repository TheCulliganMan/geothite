#!/usr/bin/env python3
"""Read-only native-art preview and collision contracts for story lairs."""
import argparse
import json
import time
from collections import deque
from pathlib import Path

from render_reference import SourceArt, load_pack
from render_archive import save_render


def distance(pack, source, blocks, start, target, occupied=()):
    width, height = source['dimensions_metatiles']
    collision = pack['data']['tilesets'][source['tileset']]['collision']
    todo = deque([(tuple(start), 0)])
    seen = {tuple(start), *map(tuple, occupied)}
    while todo:
        (x, y), steps = todo.popleft()
        if (x, y) == tuple(target):
            return steps
        for xx, yy in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)):
            if not (0 <= xx < width*2 and 0 <= yy < height*2) or (xx, yy) in seen:
                continue
            block = blocks[(yy//2)*width+xx//2]
            if collision[f'{block:02x}'][(yy%2)*2+xx%2] not in ('FLOOR','LADDER'):
                continue
            seen.add((xx, yy))
            todo.append(((xx, yy), steps+1))
    return None


def review(pack_path):
    pack = load_pack(pack_path)
    art = SourceArt(pack)
    output = Path('planning/minnesota/renders/story-events-v2') / f'ice-source-{time.time_ns()}'
    output.mkdir(parents=True)
    root = Path('planning/minnesota/lairs')
    cargo = json.loads((root/'minneapolis-cargo.json').read_text())
    relay = json.loads((root/'minneapolis-relay.json').read_text())
    iron = json.loads((root/'iron-approach.json').read_text())
    gallery = json.loads((root/'iron-gallery.json').read_text())
    core = json.loads((root/'iron-core.json').read_text())
    for source in (cargo, relay, iron, gallery, core):
        width, height = source['dimensions_metatiles']
        rows = [r.split() for r in source['rows']]
        assert len(rows) == height and all(len(r) == width for r in rows)
        blocks = [int(v,16) for row in rows for v in row]
        save_render(art.render(source['tileset'], width, height, blocks, 'nite' if source['tileset'] == 'cave' else 'day'), output / f'{source["id"]}-whole.png')
        start = source['entrance']['passive']
        if source in (cargo, gallery):
            breaker = source['cargo_breaker' if source is cargo else 'feed']['approach']
            stairs = source['relay_stair' if source is cargo else 'core_stair']['active']
            occupied = [(21,17), (12,12)] if source is cargo else []
            closed = distance(pack, source, blocks, start, breaker, occupied)
            assert closed is not None
            assert distance(pack, source, blocks, start, stairs) is None
            opened = blocks.copy()
            for change in source['block_changes']:
                x,y = change['metatile']
                assert opened[y*width+x] == int(change['closed'],16)
                opened[y*width+x] = int(change['open'],16)
            short = distance(pack, source, opened, start, breaker, occupied)
            assert short is not None and short < closed, (source['id'], closed, short)
            assert distance(pack, source, opened, start, stairs, occupied) is not None
            save_render(art.render(source['tileset'], width, height, opened, 'day'), output/f'{source["id"]}-open-whole.png')
            print(f'{source["id"]}: feed {closed} steps closed, {short} after shortcut; next stair sealed then reachable.')
        elif source is relay:
            reed = source['reed']['tile']
            approach = source['reed']['approach']
            steps = distance(pack, source, blocks, start, approach, [reed, (16,11)])
            assert steps is not None and steps > abs(start[0]-approach[0])+abs(start[1]-approach[1]), (steps, start, approach)
            assert distance(pack, source, blocks, start, source['holding_switch']['approach'], [reed, (16,11)]) is not None
            print(f'Relay rival approach: {steps} steps; controls reachable for explicit battle denial.')
        elif source is iron:
            target = source['gallery_stair']['active']
            long = distance(pack, source, blocks, start, target)
            assert long is not None, 'Iron approach stair disconnected'
            print(f'Iron crosscut: gallery stair {long} steps; no power-operated boulder shortcut.')
        else:
            occupied = [source['reed']['tile'], source['foreman']['tile']]
            assert distance(pack, source, blocks, start, source['witness_pose'], occupied) is not None
            assert distance(pack, source, blocks, start, source['controls']['approach'], occupied) is not None
            piston = source['piston']
            for block in (piston['extended'], piston['retracted']):
                assert set(pack['data']['tilesets'][source['tileset']]['collision'][block]) == {'WALL'}
            print('Drill core: witnessed pose and separate controls reachable; moving piston preserves solid collision.')
        # No reachable FLOOR/LADDER route may escape the authored outer border.
        for edge in [(x,y) for x in range(width*2) for y in (0,height*2-1)] + [(x,y) for y in range(height*2) for x in (0,width*2-1)]:
            assert distance(pack, source, blocks, start, edge) is None, edge
    print(output)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True)
    review(parser.parse_args().pack)
