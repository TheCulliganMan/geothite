#!/usr/bin/env python3
"""Play the shelter fork, both drops and compact mouth through production MCP."""
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

ROOM = 'MnR12Cave1F'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--save', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    native = load_pack(args.pack)
    module = native['data']['maps'][ROOM]
    assert [module['attributes'][k] for k in ['width', 'height']] == [16, 12]
    collision = native['data']['tilesets']['cave']['collision']
    bodies = {(o['x'], o['y']) for o in module['objects']}
    floor = {(x, y) for y in range(24) for x in range(32)
             if collision[f"{module['blocks'][y // 2 * 16 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    live = args.output / 'live.crystalsave'
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
             '--load', str(args.save.resolve()), '--save', str(live.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
        try:
            client = Client(process)
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                       clientInfo=dict(name='gunflint-shelter', version='2')))
            client.at(ROOM, 17, 22)

            def walk(target):
                status = client.status()
                assert status['map'] == ROOM
                start = (status['position']['x'], status['position']['y'])
                previous, queue = {start: None}, deque([start])
                while queue:
                    x, y = queue.popleft()
                    for dx, dy, key in [(1, 0, 'right'), (-1, 0, 'left'), (0, 1, 'down'), (0, -1, 'up')]:
                        tile = x + dx, y + dy
                        if tile in floor and tile not in previous:
                            previous[tile] = ((x, y), key)
                            queue.append(tile)
                assert target in previous, 'Actual ordinary floor must reach the destination.'
                course, tile = [], target
                while previous[tile] is not None:
                    prior, key = previous[tile]
                    course.append((key, tile))
                    tile = prior
                for key, tile in reversed(course):
                    client.move(key)
                    client.at(ROOM, *tile)
                return len(course)

            walk((6, 5))
            client.press('left')
            client.press('a')
            client.move('down')
            client.at(ROOM, 6, 5)
            pages = client.finish_dialogue()
            assert 'A dry rock roof' in pages and 'Two paths curl' in pages
            walk((26, 5))
            shutil.copy2(live, args.output / 'cache-approach.crystalsave')
            returns = []
            for x in [14, 15]:
                walk((x, 13))
                for _ in range(6):
                    status = client.status()
                    if status['position'] == dict(x=x, y=16):
                        break
                    client.move('down')
                client.at(ROOM, x, 16)
                for _ in range(3):
                    client.move('up')
                    client.at(ROOM, x, 16)
                returns.append(walk((x, 13)))
            walk((17, 22))
            for _ in range(6):
                if client.status()['map'] != ROOM:
                    break
                client.move('down')
            status = client.status()
            assert status['map'] == 'MnR12C00R00'
            assert status['position'] in [dict(x=40, y=15), dict(x=40, y=16)]
            if status['position']['y'] == 15:
                client.move('down')
            client.at('MnR12C00R00', 40, 16)
            shutil.copy2(live, args.output / 'compact-exterior.crystalsave')
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
            client.press('b')
            client.at('MnR12C00R00', 40, 16)
            client.move('up')
            client.at(ROOM, 17, 23)
            client.move('up')
            client.at(ROOM, 17, 22)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'Production stdio stderr must stay empty.'
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), otta_pages_and_input_lock=True,
                   cache_approach=True, both_native_drops_and_reverse_denial=True,
                   ordinary_return_inputs=returns, compact_mouth_exit_reentry=True,
                   actual_exterior='MnR12C00R00', active=[40, 15], safe_floor=[40, 16],
                   production_start_pack_menu=True, stderr_byte_empty=True,
                   scope='Production input from an explicit local reached empty-party save; item PP effects are separately controller-tested, not earned progression.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
