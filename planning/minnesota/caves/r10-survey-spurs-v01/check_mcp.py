#!/usr/bin/env python3
"""Walk the revised rock spur and supply bend with production stdio input.

Starts from the controller's actual reached save, with both caches collected.
No party, position, flags, inventory or RNG is edited. Existing controller
cases separately prove actual rewards, both battles, full pocket and Continue.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded
from check_north_shore_story_mcp import choose, finish
from render_reference import load_pack

ROOM = 'MnR10Cave1F'
DIRECTIONS = [(1, 0, 'right'), (-1, 0, 'left'), (0, 1, 'down'), (0, -1, 'up')]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'save', 'client', 'verifier', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    bindings = {key: sha(getattr(args, key)) for key in ['pack', 'save', 'client', 'verifier']}
    pack = load_pack(args.pack)
    module = pack['data']['maps'][ROOM]
    source = json.loads((Path(__file__).parent / 'after.json').read_text())
    assert module['blocks'] == [int(v, 16) for row in source['rows'] for v in row.split()]
    collision = pack['data']['tilesets']['cave']['collision']
    collected = {'MN_R10_ITEM_04', 'MN_R10_ESCAPE_ROPE'}
    bodies = {(o['x'], o['y']) for o in module['objects'] if o['script'] not in collected}
    floor = {(x, y) for y in range(36) for x in range(48)
             if collision[f'{module["blocks"][y//2*24+x//2]:02x}'][y % 2 * 2 + x % 2] == 'FLOOR'} - bodies
    idfile = args.output / 'modpack-id.txt'
    idfile.write_text(pack['identity']['runtime_modpack_id'])
    live = args.output / 'live.crystalsave'
    calls, native, pages = [], [], []
    process = None
    receipt = dict(status='IN_PROGRESS', bindings=bindings, calls=calls, scope=__doc__)

    def verify(path):
        result = subprocess.run([str(args.verifier.resolve()), str(idfile.resolve()),
                                 bindings['pack'], pack['identity']['content_hash'],
                                 str(path.resolve()), '-', 'verify', 'READ_ONLY'],
                                capture_output=True, text=True)
        assert result.returncode == 0, result.stderr
        assert 'ROPE=1 REVIVE=1 TAKEN=true SILAS=false NORA=false' in result.stdout, result.stdout
        native.append(result.stdout)

    def point(client):
        state = client.status()
        assert state['map'] == ROOM, state
        return state['position']['x'], state['position']['y']

    def settle(client):
        for _ in range(128):
            state = client.status()
            if state['mode'] == 'wildbattle':
                escape_wild(client)
                continue
            view = client.observe()['structuredContent']
            if (state['mode'] == 'overworld' and not state['dialogue']
                    and not view['dialogue'] and not view['prompt'] and not view['menu']
                    and not client.call('flow_state')['structuredContent']['animating']):
                return
            client.press('a')
        raise AssertionError('ambient native notice or encounter did not release input')

    def walk(client, target):
        settle(client)
        for _ in range(1024):
            start = point(client)
            if start == target:
                return
            parents, queue = {start: None}, deque([start])
            while queue and target not in parents:
                x, y = queue.popleft()
                for dx, dy, key in DIRECTIONS:
                    at = x + dx, y + dy
                    if at in floor and at not in parents:
                        parents[at] = ((x, y), key)
                        queue.append(at)
            assert target in parents, (start, target)
            at = target
            while parents[at][0] != start:
                at = parents[at][0]
            key = parents[at][1]
            for _ in range(8):
                client.move(key)
                settle(client)
                actual = point(client)
                assert actual in [start, at], (start, at, actual)
                if actual == at:
                    break
            assert point(client) == at
        raise AssertionError(('native walk exceeded course', target))

    try:
        verify(args.save)
        for phase in range(2):
            with (args.output / f'stderr-{phase}.log').open('x') as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str((args.save if phase == 0 else live).resolve()),
                     '--save', str(live.resolve())], stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
                client = LiveRecorded(process, calls)
                client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                           clientInfo=dict(name='r10-survey-spurs', version='1')))
                if phase == 0:
                    client.at(ROOM, 33, 25)
                    for target in [(25, 21), (20, 13)]:
                        walk(client, target)
                    for _ in range(3):
                        client.move('right')
                        client.at(ROOM, 20, 13)
                    for target in [(8, 8), (8, 5)]:
                        walk(client, target)
                    client.press('right')
                    client.press('a')
                    assert client.status()['dialogue']
                    client.move('left')
                    client.at(ROOM, 8, 5)
                    pages.append(finish(client))
                    for target in [(8, 8), (20, 9), (20, 6), (20, 9), (19, 15)]:
                        walk(client, target)
                    client.press('left')
                    client.press('a')
                    choose(client, False)
                    pages.append(finish(client))
                    walk(client, (35, 8))
                    for _ in range(3):
                        client.move('up')
                        client.at(ROOM, 35, 8)
                    walk(client, (38, 5))
                    shutil.copy2(live, args.output / 'actual-supply-crook.crystalsave')
                    for x in [14, 15]:
                        walk(client, (x, 15))
                        for _ in range(8):
                            client.move('down')
                            if point(client) == (x, 18):
                                break
                        client.at(ROOM, x, 18)
                        for _ in range(3):
                            client.move('up')
                            client.at(ROOM, x, 18)
                        walk(client, (x, 15))
                    for target in [(28, 23), (33, 25)]:
                        walk(client, target)
                    client.press('up')
                    client.press('a')
                    choose(client, False)
                    pages.append(finish(client))
                    walk(client, (36, 28))
                    shutil.copy2(live, args.output / 'actual-unchanged-fishing-bank.crystalsave')
                    walk(client, (3, 12))
                    for _ in range(8):
                        client.move('down')
                        if client.status()['map'] != ROOM:
                            break
                    client.at('MnR10C00R00', 36, 18)
                else:
                    client.at('MnR10C00R00', 36, 18)
                    for _ in range(8):
                        client.move('up')
                        if client.status()['map'] == ROOM:
                            break
                    client.at(ROOM, 3, 13)
                    client.move('up')
                    client.at(ROOM, 3, 12)
                    walk(client, (35, 9))
                    walk(client, (3, 12))
                    client.press('start')
                    assert any('PACK' in v['text'] for v in client.observe()['structuredContent']['menu'])
                    client.press('b')
                verify(live)
                process.stdin.close()
                assert process.wait(timeout=30) == 0
                process = None
            assert (args.output / f'stderr-{phase}.log').stat().st_size == 0
        assert all(sha(getattr(args, key)) == value for key, value in bindings.items())
        receipt.update(status='PASS', rpc_calls=len(calls), native=native, pages=pages,
                       fresh_process_continue=True,
                       fixture='Actual controller reached save with declared prior seven badges/depot access, original Minnesota party, supplies and both caches collected. No later state edits or new rewards; native notices, encounters and Repel reuse use input. Not earned regional arrival or a new fishing-bite replay.')
    except BaseException as error:
        receipt.update(status='FAIL', error=repr(error), native=native, pages=pages)
        raise
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            process.wait(timeout=30)
        (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('PASS survey spur, supply bend, stairs/drops, native refusals, mouth and Continue;', len(calls), 'RPC')


if __name__ == '__main__':
    main()
