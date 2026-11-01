#!/usr/bin/env python3
"""Exercise the real shelter circuit and fresh-process Continue over stdio MCP."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from check_mina_battle_mcp import LiveRecorded
from render_reference import load_pack

ROOM = 'MnR17Cave1F'
OUTSIDE = 'MnR17C00R00'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'save', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    bindings = {key: digest(getattr(args, key)) for key in ['pack', 'save', 'client']}
    pack = load_pack(args.pack)
    module = pack['data']['maps'][ROOM]
    source = json.loads((Path(__file__).parent / 'after.json').read_text())
    assert module['blocks'] == [int(v, 16) for row in source['rows'] for v in row.split()]
    assert [module['attributes'][key] for key in ['width', 'height']] == [7, 8]
    collision = pack['data']['tilesets']['cave']['collision']
    bodies = {(actor['x'], actor['y']) for actor in module['objects']}
    floor = {(x, y) for y in range(16) for x in range(14)
             if collision[f"{module['blocks'][y // 2 * 7 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    live = args.output / 'live.crystalsave'
    records, text = [], []
    receipt = dict(status='IN_PROGRESS', bindings=bindings, calls=records,
                   fixture='Declared local empty-party outside placement; no earned regional access claim.')
    try:
        for continued in [False, True]:
            with (args.output / f'stderr-{int(continued)}.log').open('w+') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str((live if continued else args.save).resolve()),
                     '--save', str(live.resolve())], stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
                try:
                    client = LiveRecorded(process, records)
                    client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                               clientInfo=dict(name='r17-loop-review', version='1')))

                    def close():
                        # Native item PocketText can own input while compact
                        # status.dialogue is false. Prove release with Start.
                        for _ in range(128):
                            text.append(client.observe()['content'][0]['text'])
                            if client.status()['dialogue']:
                                client.press('a')
                                continue
                            client.press('start')
                            view = client.observe()['content'][0]['text']
                            if 'SAVE' in view:
                                client.press('b')
                                return
                            client.press('a')
                        raise AssertionError('native conversation did not release input')

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
                        assert target in parents, (start, target)
                        route, at = [], target
                        while parents[at] is not None:
                            prior, key = parents[at]
                            route.append((at, key))
                            at = prior
                        for at, key in reversed(route):
                            client.move(key)
                            client.at(ROOM, *at)

                    if not continued:
                        client.at(OUTSIDE, 10, 44)
                        client.move('up')
                        client.at(ROOM, 9, 13)
                        client.move('up')
                        walk((7, 10))
                        client.press('left')
                        client.at(ROOM, 7, 10)
                        client.press('a')
                        assert client.status()['dialogue']
                        client.move('right')
                        client.at(ROOM, 7, 10)
                        close()
                        assert 'lichen' in '\n'.join(text).lower()
                        for at in [(7, 9), (5, 9), (5, 7), (5, 6)]:
                            walk(at)
                        client.move('right')
                        client.at(ROOM, 5, 6)
                        client.move('up')
                        client.at(ROOM, 5, 6)
                        client.press('a')
                        client.move('down')
                        client.at(ROOM, 5, 6)
                        close()
                        floor.add((5, 5))
                        for at in [(5, 5), (6, 5), (8, 5), (8, 6)]:
                            walk(at)
                        client.move('left')
                        client.at(ROOM, 8, 6)
                        for at in [(8, 7), (8, 8), (7, 8), (7, 9), (7, 10)]:
                            walk(at)
                    else:
                        client.at(ROOM, 7, 10)
                        walk((5, 5))
                        client.press('a')
                        close()
                        walk((9, 12))
                        client.move('down')
                        client.at(OUTSIDE, 10, 44)
                        client.call('hold_button', button='b', frames=80)
                        client.at(OUTSIDE, 10, 44)
                        client.move('down')
                        client.at(OUTSIDE, 10, 45)
                        client.move('up')
                        client.at(OUTSIDE, 10, 44)
                        client.move('up')
                        client.at(ROOM, 9, 13)
                        client.move('up')
                        walk((5, 5))
                        close()
                finally:
                    process.stdin.close()
                    assert process.wait(timeout=30) == 0
                    stderr.flush()
                    assert stderr.tell() == 0, 'stdio stderr must stay empty'
        assert all(digest(getattr(args, key)) == value for key, value in bindings.items())
        receipt.update(status='PASS', rpc_calls=len(records), observed_text=text,
                       live_save_sha256=digest(live), fresh_process_continue=True)
        print('PASS', len(records), 'RPC')
    finally:
        (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
