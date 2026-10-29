#!/usr/bin/env python3
"""Play the bank cave, original walker, roof and once-only ball over stdio."""
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

ROOM = 'MnR03Cave1F'
OUTSIDE = 'MnR03C00R00'
ITEM = 'MN_R03_ITEM_02'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    module = pack['data']['maps'][ROOM]
    assert [module['attributes'][key] for key in ['width', 'height']] == [8, 6]
    collision = pack['data']['tilesets']['cave']['collision']
    bodies = {(actor['x'], actor['y']) for actor in module['objects']}
    floor = {(x, y) for y in range(12) for x in range(16)
             if collision[f"{module['blocks'][y // 2 * 8 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    assert (5, 6) in floor, 'Retain the original dry walker approach.'
    args.output.mkdir(parents=True, exist_ok=False)
    live = args.output / 'live.crystalsave'
    for continued in [False, True]:
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                 '--load', str((live if continued else args.save).resolve()), '--save', str(live.resolve())],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
            try:
                client = Client(process)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                           clientInfo=dict(name='bank-cave', version='1')))
                client.at(ROOM, 7, 8)

                def walk(target):
                    status = client.status()
                    assert status['map'] == ROOM
                    start = status['position']['x'], status['position']['y']
                    parents, queue = {start: None}, deque([start])
                    while queue:
                        x, y = queue.popleft()
                        for dx, dy, key in [(1, 0, 'right'), (-1, 0, 'left'), (0, 1, 'down'), (0, -1, 'up')]:
                            at = x + dx, y + dy
                            if at in floor and at not in parents:
                                parents[at] = ((x, y), key)
                                queue.append(at)
                    assert target in parents
                    steps, at = [], target
                    while parents[at] is not None:
                        prior, key = parents[at]
                        steps.append((at, key))
                        at = prior
                    for at, key in reversed(steps):
                        client.move(key)
                        client.at(ROOM, *at)

                if not continued:
                    walk((5, 6))
                    client.press('up')
                    client.press('a')
                    client.move('down')
                    client.at(ROOM, 5, 6)
                    pages = client.finish_dialogue()
                    for line in ['A pocket of dry', 'stone by the bank.', 'The path curves',
                                 'around that rock.', 'Take the far cache', 'and circle back.']:
                        assert line in pages, (line, pages)
                    walk((6, 7))
                    shutil.copy2(live, args.output / 'seep-bank.crystalsave')
                    for at in [(6, 5), (6, 3), (7, 3), (8, 3)]:
                        walk(at)
                    for key, at in [('up', (8, 2)), ('right', (9, 2))]:
                        client.move(key)
                        client.at(ROOM, *at)
                        for _ in range(3):
                            client.move('up')
                            client.at(ROOM, *at)
                    client.move('down')
                    client.at(ROOM, 9, 3)
                    walk((10, 3))
                    shutil.copy2(live, args.output / 'blind-bend.crystalsave')
                    walk((11, 5))
                    assert ITEM in json.dumps(client.call('map_info'))
                    client.press('right')
                    client.at(ROOM, 11, 5)
                    client.press('a')
                    client.move('down')
                    client.at(ROOM, 11, 5)
                    pages = client.finish_dialogue()
                    assert 'GREAT BALL' in pages and 'BALL POCKET' in pages, pages
                    assert ITEM not in json.dumps(client.call('map_info'))
                    floor.add((12, 5))
                else:
                    assert ITEM not in json.dumps(client.call('map_info'))
                walk((12, 5))
                client.press('a')
                client.finish_dialogue()
                assert ITEM not in json.dumps(client.call('map_info'))
                walk((7, 8))
                if not continued:
                    client.move('down')
                    client.at(OUTSIDE, 8, 38)
                    client.move('right')
                    client.at(OUTSIDE, 9, 38)
                    client.move('left')
                    client.at(OUTSIDE, 8, 38)
                    shutil.copy2(live, args.output / 'passive-exterior.crystalsave')
                    client.move('up')
                    client.at(ROOM, 7, 9)
                    client.move('up')
                    client.at(ROOM, 7, 8)
                client.press('start')
                assert 'PACK' in client.observe()['content'][0]['text']
                client.press('b')
                client.at(ROOM, 7, 8)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b'', 'Production gameplay stderr must remain byte-empty.'
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), original_walker_pages_and_input_lock=True,
                   original_walker_approach_dry=True, both_native_roof_cells_and_up_block=True,
                   item_notice_input_lock_and_body_removal=True, continued_collected_tile_walkable=True,
                   true_passive_mouth_and_same_mouth_reentry=True, production_start_menu=True,
                   processes=2, stderr_byte_empty=True,
                   scope='Actual stdio inputs from an explicit empty-party local reached save. '
                         'Native quantity, flags and full resource persistence are separately controller-tested; regional access is not earned here.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
