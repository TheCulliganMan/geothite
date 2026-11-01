#!/usr/bin/env python3
"""Play the actual fitted bend, directional roof and once-only item over stdio."""
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

ROOM = 'MnR17Cave1F'
ITEM = 'MN_R17_ITEM_04'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    module = pack['data']['maps'][ROOM]
    assert [module['attributes'][key] for key in ['width', 'height']] == [7, 8]
    collision = pack['data']['tilesets']['cave']['collision']
    bodies = {(actor['x'], actor['y']) for actor in module['objects']}
    floor = {(x, y) for y in range(16) for x in range(14)
             if collision[f"{module['blocks'][y // 2 * 7 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    args.output.mkdir(parents=True, exist_ok=False)
    live = args.output / 'live.crystalsave'
    phases = []
    for continued in [False, True]:
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                 '--load', str((live if continued else args.save).resolve()), '--save', str(live.resolve())],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
            try:
                client = Client(process)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                           clientInfo=dict(name='talus-den', version='1')))
                client.at(ROOM, 9, 12)

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
                    for at in [(7, 12), (5, 11), (5, 9), (4, 5), (6, 5)]:
                        walk(at)
                    walk((5, 3))
                    for key, at in [('up', (5, 2)), ('right', (6, 2)), ('down', (6, 3))]:
                        client.move(key)
                        client.at(ROOM, *at)
                    walk((3, 5))
                    client.move('up')
                    client.at(ROOM, 3, 4)
                    for _ in range(3):
                        client.move('up')
                        client.at(ROOM, 3, 4)
                    client.move('down')
                    client.at(ROOM, 3, 5)
                    walk((5, 6))
                    assert ITEM in json.dumps(client.call('map_info'))
                    client.press('up')
                    client.at(ROOM, 5, 6)
                    client.press('a')
                    client.move('down')
                    client.at(ROOM, 5, 6)
                    pages = client.finish_dialogue()
                    assert 'REVIVE' in pages and 'ITEM POCKET' in pages
                    assert ITEM not in json.dumps(client.call('map_info'))
                    floor.add((5, 5))
                else:
                    assert ITEM not in json.dumps(client.call('map_info'))
                walk((5, 5))
                client.press('a')
                client.finish_dialogue()
                assert ITEM not in json.dumps(client.call('map_info'))
                walk((9, 12))
                if not continued:
                    shutil.copy2(live, args.output / 'collected-mouth.crystalsave')
                    client.move('down')
                    client.at('MnR17C00R00', 10, 44)
                    client.move('down')
                    client.at('MnR17C00R00', 10, 45)
                    client.move('up')
                    client.at('MnR17C00R00', 10, 44)
                    shutil.copy2(live, args.output / 'passive-exterior.crystalsave')
                    client.move('up')
                    client.at(ROOM, 9, 13)
                    client.move('up')
                    client.at(ROOM, 9, 12)
                client.press('start')
                assert 'PACK' in client.observe()['content'][0]['text']
                client.press('b')
                client.at(ROOM, 9, 12)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b'', 'Production gameplay stderr must remain byte-empty.'
        phases.append('continued collected tile' if continued else 'roof/item/passive mouth')
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), all_six_authored_views_reached=True,
                   all_three_native_directional_roof_cells_and_blocked_up=True,
                   item_notice_input_lock_and_collected_body_removal=True,
                   continued_collected_tile_walkable=True, true_passive_mouth_and_reentry=True,
                   production_start_menu=True, phases=phases, stderr_byte_empty=True,
                   scope='Actual stdio inputs from an explicit empty-party local reached save. '
                         'Native quantity, flags and complete resource persistence are separately controller-tested; access is not earned here.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
