#!/usr/bin/env python3
"""Play both cleft drops, both real mouths and the original cache over stdio."""
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

ROOM = 'MnR20Cave1F'
ITEM = 'MN_R20_ITEM_01'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    contract = json.loads(Path(__file__).with_name('native.contract.json').read_text())
    pack = load_pack(args.pack)
    module = pack['data']['maps'][ROOM]
    assert [module['attributes'][key] for key in ['width', 'height']] == [9, 8]
    collision = pack['data']['tilesets']['cave']['collision']
    bodies = {(actor['x'], actor['y']) for actor in module['objects']}
    floor = {(x, y) for y in range(16) for x in range(18)
             if collision[f"{module['blocks'][y // 2 * 9 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    args.output.mkdir(parents=True, exist_ok=False)
    live = args.output / 'live.crystalsave'
    returns = []
    for continued in [False, True]:
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(
                [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                 '--load', str((live if continued else args.save).resolve()), '--save', str(live.resolve())],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
            try:
                client = Client(process)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                           clientInfo=dict(name='two-mouth-cleft', version='1')))
                client.at(ROOM, 5, 12)

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
                    assert target in parents, ('ordinary return disconnected', target)
                    steps, at = [], target
                    while parents[at] is not None:
                        prior, key = parents[at]
                        steps.append((at, key))
                        at = prior
                    for at, key in reversed(steps):
                        client.move(key)
                        client.at(ROOM, *at)
                    return len(steps)

                if not continued:
                    for at in [(13, 12), (4, 9), (4, 7), (4, 5), (7, 9),
                               (10, 7), (11, 11), (12, 9)]:
                        walk(at)
                    walk((10, 7))
                    shutil.copy2(live, args.output / 'upper-lip.crystalsave')
                    for y, expected in [(8, 15), (9, 13)]:
                        walk((10, y))
                        for _ in range(6):
                            if client.status()['position'] == dict(x=7, y=y):
                                break
                            client.move('left')
                        client.at(ROOM, 7, y)
                        for _ in range(3):
                            client.move('right')
                            client.at(ROOM, 7, y)
                        length = walk((10, y))
                        assert length == expected
                        returns.append(length)
                    for mouth in contract['mouths']:
                        x, y = mouth['inside_core']
                        walk((x, y - 1))
                        client.move('down')
                        px, py = mouth['passive_core']
                        client.at(mouth['exterior'], px, py)
                        client.move('right')
                        client.at(mouth['exterior'], px + 1, py)
                        client.move('left')
                        client.at(mouth['exterior'], px, py)
                        shutil.copy2(live, args.output / (mouth['label'] + '-exterior.crystalsave'))
                        client.move('up')
                        client.at(ROOM, x, y)
                        client.move('up')
                        client.at(ROOM, x, y - 1)
                    walk((4, 5))
                    assert ITEM in json.dumps(client.call('map_info'))
                    client.press('right')
                    client.at(ROOM, 4, 5)
                    client.press('a')
                    client.move('down')
                    client.at(ROOM, 4, 5)
                    pages = client.finish_dialogue()
                    assert 'HYPER POTION' in pages and 'ITEM POCKET' in pages
                    assert ITEM not in json.dumps(client.call('map_info'))
                    floor.add((5, 5))
                else:
                    assert ITEM not in json.dumps(client.call('map_info'))
                walk((5, 5))
                client.press('a')
                client.finish_dialogue()
                assert ITEM not in json.dumps(client.call('map_info'))
                walk((5, 12))
                client.press('start')
                assert 'PACK' in client.observe()['content'][0]['text']
                client.press('b')
                client.at(ROOM, 5, 12)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b'', 'Production gameplay stderr must remain byte-empty.'
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), all_nine_authored_views_reached=True,
                   both_native_left_drops_and_blocked_reverse=True, ordinary_return_inputs=returns,
                   both_true_passive_mouths_and_same_mouth_reentry=True,
                   original_item_notice_input_lock_and_body_removal=True,
                   continued_collected_tile_walkable=True, production_start_menu=True,
                   stderr_byte_empty=True,
                   scope='Actual stdio inputs from explicit local empty-party setup. '
                         'Exact Hyper Potion HP/quantity/flag effects are separately controller-tested; regional access is not earned here.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
