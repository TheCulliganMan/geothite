#!/usr/bin/env python3
"""Actual stdio crossing and Gym4 facade from an explicit local empty-party save."""
import argparse
from collections import deque
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_river_story_mcp import same_map_travel
from check_winona_mcp import DIRECTIONS, travel
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def walk(client, pack, target):
    state = client.status()
    name = state['map']
    start = (state['position']['x'], state['position']['y'])
    module = pack['data']['maps'][name]
    attrs = module['attributes']
    width, height = attrs['width'], attrs['height']
    collisions = pack['data']['tilesets'][attrs['tileset_name']]['collision']

    def ground(tile):
        x, y = tile
        if not (0 <= x < width*2 and 0 <= y < height*2):
            return False
        block = module['blocks'][y//2*width+x//2]
        return collisions[f'{block:02x}'][y%2*2+x%2] in {'FLOOR', 'TALL_GRASS', 'LADDER'}

    blocked = {(actor['x'], actor['y']) for actor in module['objects']}
    for actor in module['objects']:
        if actor['object_type'] != 'OBJECTTYPE_TRAINER':
            continue
        dx, dy = {'UP': (0, -1), 'DOWN': (0, 1), 'LEFT': (-1, 0), 'RIGHT': (1, 0)}[
            actor['spritemovedata'].rsplit('_', 1)[-1]]
        for step in range(1, actor['radius']+1):
            tile = (actor['x']+step*dx, actor['y']+step*dy)
            if not ground(tile):
                break
            blocked.add(tile)
    queue, paths = deque([start]), {start: []}
    while queue and target not in paths:
        x, y = queue.popleft()
        for dx, dy, button in DIRECTIONS:
            tile = (x+dx, y+dy)
            if tile not in paths and tile not in blocked and ground(tile):
                paths[tile] = paths[(x, y)]+[button]
                queue.append(tile)
    assert target in paths, (name, start, target)
    for button in paths[target]:
        client.move(button)
        state = client.status()
        assert state['map'] == name and state['mode'] == 'overworld', state
    client.at(name, *target)


def play(client, pack):
    client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                 clientInfo=dict(name='gym-four-local-review', version='1')))
    client.at('MnMinneapolis', 58, 24)
    initial = client.status()
    assert initial['badges'] == 3
    same_map_travel(client, 'right', 'MnR04C00R00', (0, 18))
    walk(client, pack, (55, 18))
    travel(client, 'right', 'MnStPaul', (0, 24))
    same_map_travel(client, 'right', 'MnStPaul', (1, 24))
    walk(client, pack, (54, 16))
    travel(client, 'up', 'MnStPaulGym', (2, 17))
    client.press('start')
    assert 'PACK' in client.observe()['content'][0]['text']
    client.press('b')
    walk(client, pack, (2, 18))
    travel(client, 'down', 'MnStPaul', (54, 16))
    walk(client, pack, (1, 24))
    travel(client, 'left', 'MnR04C00R00', (55, 18))
    walk(client, pack, (0, 18))
    travel(client, 'left', 'MnMinneapolis', (63, 24))
    same_map_travel(client, 'left', 'MnMinneapolis', (58, 24))
    final = client.status()
    assert final['badges'] == initial['badges'] and final['money'] == initial['money']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, default=ROOT / 'content-packs/minnesota-gym-four-approach.crystalpack')
    parser.add_argument('--save', type=Path, default=ROOT / '.local/gym-four-approach/qualified.crystalsave')
    parser.add_argument('--client', type=Path, default=ROOT / 'target/gym-controller/wasm-dev/geothite')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(args.client), 'mcp', str(args.pack), '--load', str(args.save)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, bufsize=1)
        try:
            play(Client(process), pack)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'gameplay stderr was not byte-empty'
    print('PASS: local empty-party setup; actual stdio Minneapolis/R04/St Paul/Gym4 facade, Start and return; unchanged badges/money and byte-empty stderr.')
