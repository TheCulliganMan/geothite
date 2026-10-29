#!/usr/bin/env python3
"""Replay both real Rock Cut mouths and its stair through production stdio MCP."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from check_minneapolis_mcp import Client
from render_reference import load_pack

ROOM = 'MnR05RockCut'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--save', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    native = load_pack(args.pack)
    room = native['data']['maps'][ROOM]
    assert [room['attributes'][k] for k in ['width', 'height']] == [8, 10]
    collisions = native['data']['tilesets']['cave']['collision']
    bodies = {(o['x'], o['y']) for o in room['objects']}
    floor = {(x, y) for y in range(20) for x in range(16)
             if collisions[f"{room['blocks'][y // 2 * 8 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    start, end = (11, 4), (5, 16)
    queue, previous = deque([start]), {start: None}
    while queue:
        x, y = queue.popleft()
        for dx, dy, key in [(1, 0, 'right'), (-1, 0, 'left'), (0, 1, 'down'), (0, -1, 'up')]:
            tile = x + dx, y + dy
            if tile in floor and tile not in previous:
                previous[tile] = ((x, y), key)
                queue.append(tile)
    assert end in previous, 'Actual ordinary-floor passage must connect the clear arrivals.'
    course, tile = [], end
    while previous[tile] is not None:
        prior, key = previous[tile]
        course.append((key, tile))
        tile = prior
    course.reverse()
    live = args.output / 'live.crystalsave'
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
             '--load', str(args.save.resolve()), '--save', str(live.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
        try:
            client = Client(process)
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                       clientInfo=dict(name='r05-two-mouths', version='2')))
            client.at(ROOM, *start)

            def cross(key, name, at):
                for _ in range(6):
                    if client.status()['map'] == name:
                        break
                    client.move(key)
                client.at(name, *at)

            cross('down', 'MnR05C00R00', (30, 28))
            shutil.copy2(live, args.output / 'north-exterior.crystalsave')
            cross('up', ROOM, (11, 5))
            client.move('up')
            client.at(ROOM, *start)
            for key, tile in course:
                client.move(key)
                client.at(ROOM, *tile)
            cross('down', 'MnR05C00R01', (16, 30))
            shutil.copy2(live, args.output / 'south-exterior.crystalsave')
            cross('up', ROOM, (5, 17))
            client.move('up')
            client.at(ROOM, *end)
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
            client.press('b')
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'Production stdio stderr must stay empty.'
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), both_mouths_exit_reenter_correctly=True,
                   native_ladder_arrivals=[[11, 5], [5, 17]],
                   ordinary_floor_stair_course_inputs=len(course), production_start_pack_menu=True,
                   stderr_byte_empty=True,
                   scope='Production stdio input from an explicit local reached empty-party save; not earned progression.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
