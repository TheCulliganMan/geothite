#!/usr/bin/env python3
"""Replay both shelter climbs, Harper and the real passive mouth over stdio."""
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

ROOM = 'MnR13Cave1F'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    native = load_pack(args.pack)
    module = native['data']['maps'][ROOM]
    assert [module['attributes'][key] for key in ['width', 'height']] == [10, 8]
    collision = native['data']['tilesets']['cave']['collision']
    bodies = {(actor['x'], actor['y']) for actor in module['objects']}
    floor = {(x, y) for y in range(16) for x in range(20)
             if collision[f"{module['blocks'][y // 2 * 10 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
    args.output.mkdir(parents=True, exist_ok=False)
    live = args.output / 'live.crystalsave'
    with tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
             '--load', str(args.save.resolve()), '--save', str(live.resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
        try:
            client = Client(process)
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                       clientInfo=dict(name='cedar-spring-shelter', version='1')))
            client.at(ROOM, 7, 14)

            def walk(target):
                status = client.status()
                assert status['map'] == ROOM
                start = status['position']['x'], status['position']['y']
                previous, queue = {start: None}, deque([start])
                while queue:
                    x, y = queue.popleft()
                    for dx, dy, key in [(1, 0, 'right'), (-1, 0, 'left'), (0, 1, 'down'), (0, -1, 'up')]:
                        at = x + dx, y + dy
                        if at in floor and at not in previous:
                            previous[at] = ((x, y), key)
                            queue.append(at)
                assert target in previous
                steps, at = [], target
                while previous[at] is not None:
                    prior, key = previous[at]
                    steps.append((at, key))
                    at = prior
                for at, key in reversed(steps):
                    client.move(key)
                    client.at(ROOM, *at)

            walk((9, 11))
            shutil.copy2(live, args.output / 'spring-bank.crystalsave')
            courses = [[(11, 5), (11, 6), (11, 7), (11, 8)],
                       [(12, 10), (12, 11), (12, 12)]]
            for course in courses:
                for at in course + list(reversed(course)):
                    walk(at)
            walk((13, 6))
            client.press('up')
            client.press('a')
            client.move('down')
            client.at(ROOM, 13, 6)
            pages = client.finish_dialogue()
            assert all(text in pages for text in ['Rest in the rock.', 'I will tend to',
                                                  'your whole team.', 'Ely is not far.'])
            walk((14, 11))
            shutil.copy2(live, args.output / 'cache-approach.crystalsave')
            walk((7, 14))
            for _ in range(6):
                if client.status()['map'] != ROOM:
                    break
                client.move('down')
            client.at('MnR13C00R01', 14, 6)
            client.move('down')
            client.at('MnR13C00R01', 14, 7)
            client.move('up')
            client.at('MnR13C00R01', 14, 6)
            shutil.copy2(live, args.output / 'passive-exterior.crystalsave')
            client.press('start')
            assert 'PACK' in client.observe()['content'][0]['text']
            client.press('b')
            client.move('up')
            client.at(ROOM, 7, 15)
            client.move('up')
            client.at(ROOM, 7, 14)
        finally:
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        stderr.seek(0)
        assert stderr.read() == b'', 'Production gameplay stderr must stay byte-empty.'
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), both_native_stairs_bidirectional=True,
                   spring_bank_and_cache_reachable=True, original_harper_pages_and_input_lock=True,
                   true_passive_mouth_and_safe_exterior_step=True, same_mouth_reentry=True,
                   production_start_pack_menu=True, stderr_byte_empty=True,
                   scope='Production input from an explicit empty-party local reached save. '
                         'Actual healing/item PP/Continue are separately controller-tested; regional access is not earned here.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
