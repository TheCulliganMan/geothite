#!/usr/bin/env python3
"""Replay the reached haulage valve, crosscut and all five shaft pairs over MCP."""
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
from check_north_shore_story_mcp import choose, finish
from render_reference import load_pack

ROOM = 'MnIronHaulage'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'save', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    bindings = {key: sha(getattr(args, key)) for key in ['pack', 'save', 'client']}
    pack = load_pack(args.pack)
    module = pack['data']['maps'][ROOM]
    assert module['attributes']['tileset_name'] == 'mn_iron_haulage_rock'
    assert module['blocks'][8 * 18 + 9] == 0x41
    constants = {m['attributes']['map_constant']: name for name, m in pack['data']['maps'].items()}
    calls, pages = [], []
    def walk(c, pack, target):
        for _ in range(512):
            status = c.status()
            if status['dialogue']:
                pages.append(finish(c))
                continue
            assert status['mode'] != 'battle', status
            name = status['map']
            start = status['position']['x'], status['position']['y']
            if start == target:
                return
            m = pack['data']['maps'][name]
            a = m['attributes']
            collision = pack['data']['tilesets'][a['tileset_name']]['collision']
            bodies = {(o['x'], o['y']) for o in m['objects']}
            queue, paths = deque([start]), {start: []}
            while queue and target not in paths:
                x, y = queue.popleft()
                for dx, dy, direction in [(1,0,'right'),(-1,0,'left'),(0,1,'down'),(0,-1,'up')]:
                    at = x + dx, y + dy
                    if at in paths or at in bodies or not (0 <= at[0] < a['width'] * 2 and 0 <= at[1] < a['height'] * 2):
                        continue
                    block = m['blocks'][at[1] // 2 * a['width'] + at[0] // 2]
                    if collision[f'{block:02x}'][at[1] % 2 * 2 + at[0] % 2] == 'FLOOR':
                        paths[at] = paths[(x, y)] + [direction]
                        queue.append(at)
            assert target in paths, (name, start, target)
            c.move(paths[target][0])
        raise AssertionError('bounded native walking did not arrive')
    receipt = dict(status='IN_PROGRESS', bindings=bindings, calls=calls,
                   fixture='Actual controller-reached declined-valve save from a declared local party/access fixture.')
    live = args.output / 'live.crystalsave'
    try:
        for continued in [False, True]:
            with (args.output / f'stderr-{int(continued)}.log').open('w+') as stderr:
                process = subprocess.Popen([str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                    '--load', str((live if continued else args.save).resolve()), '--save', str(live.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
                try:
                    c = LiveRecorded(process, calls)
                    c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                          clientInfo=dict(name='haulage-rock', version='1')))
                    if not continued:
                        c.at(ROOM, 29, 12)
                        walk(c, pack, (20, 17))
                        c.move('left')
                        c.at(ROOM, 20, 17)
                        walk(c, pack, (29, 12))
                        c.press('up')
                        c.press('a')
                        choose(c, False)
                        pages.append(finish(c))
                        walk(c, pack, (20, 17))
                        c.move('left')
                        c.at(ROOM, 20, 17)
                        walk(c, pack, (29, 12))
                        c.press('up')
                        c.press('a')
                        choose(c, True)
                        pages.append(finish(c))
                        # Expected geometry for read-only path planning follows
                        # the valve just operated through native input. This
                        # changes no game state or pack file.
                        module['blocks'][8 * 18 + 9] = 13
                        walk(c, pack, (20, 17))
                        for x in [19, 18, 17]:
                            c.move('left')
                            c.at(ROOM, x, 17)
                        c.press('start')
                        assert 'SAVE' in c.observe()['content'][0]['text']
                        c.press('b')
                    else:
                        c.at(ROOM, 17, 17)
                        for x in [18, 19, 20]:
                            c.move('right')
                            c.at(ROOM, x, 17)
                        for index in [1, 3, 5, 7, 9]:
                            port = next(w for w in module['events']['warps'] if w['index'] == index)
                            name = constants[port['target_map_constant']]
                            target = pack['data']['maps'][name]
                            arrival = next(w for w in target['events']['warps'] if w['index'] == port['target_warp_id'])
                            walk(c, pack, (port['x'], port['y'] + 1))
                            c.move('up')
                            c.at(name, arrival['x'], arrival['y'])
                            c.call('hold_button', button='b', frames=40)
                            c.at(name, arrival['x'], arrival['y'])
                            back = next(w for w in target['events']['warps'] if w['index'] == arrival['index'] - 1)
                            walk(c, pack, (back['x'], back['y'] + 1))
                            c.move('up')
                            home = next(w for w in module['events']['warps'] if w['index'] == index + 1)
                            c.at(ROOM, home['x'], home['y'])
                        walk(c, pack, (5, 23))
                        c.move('up')
                        c.at('MnIronDrillApproach', 24, 5)
                        c.press('start')
                        assert 'SAVE' in c.observe()['content'][0]['text']
                        c.press('b')
                finally:
                    process.stdin.close()
                    assert process.wait(timeout=30) == 0
                    stderr.flush()
                    assert stderr.tell() == 0, 'gameplay stderr must stay empty'
        assert all(sha(getattr(args, key)) == value for key, value in bindings.items())
        receipt.update(status='PASS', rpc_calls=len(calls), observed_valve_pages=pages,
                       fresh_process_continue=True, live_save_sha256=sha(live))
        print('PASS', len(calls), 'RPC')
    finally:
        (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
