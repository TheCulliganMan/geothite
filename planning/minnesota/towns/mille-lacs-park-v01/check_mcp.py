#!/usr/bin/env python3
"""Play park roads, people, barriers and reward persistence via production stdio."""
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

MAP = 'MnMilleLacs'
ROUTE = 'MnR18C01R00'
ITEM = 'MN_ML_PARK_ROOST'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'saves', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    module = pack['data']['maps'][MAP]
    collision = pack['data']['tilesets']['mn_headwaters']['collision']
    bodies = {(o['x'], o['y']) for o in module['objects']}
    floor = {(x, y) for x in range(48) for y in range(40)
             if collision[f"{module['blocks'][y // 2 * 24 + x // 2]:02x}"][y % 2 * 2 + x % 2]
             in ['FLOOR', 'TALL_GRASS'] and (x, y) not in bodies}
    args.output.mkdir(parents=True, exist_ok=False)
    count = 0
    for phase in ['walk', 'roost']:
        live = args.output / (phase + '-live.crystalsave')
        initial = args.saves / ('entry.crystalsave' if phase == 'walk' else 'roost.crystalsave')
        for continued in [False, True]:
            with tempfile.TemporaryFile() as stderr:
                process = subprocess.Popen(
                    [str(args.client.resolve()), 'mcp', str(args.pack.resolve()),
                     '--load', str((live if continued else initial).resolve()), '--save', str(live.resolve())],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, bufsize=1)
                try:
                    client = Client(process)
                    client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                               clientInfo=dict(name='mille-lacs-park', version='1')))

                    def walk(target):
                        status = client.status()
                        assert status['map'] == MAP
                        start = status['position']['x'], status['position']['y']
                        parents, queue = {start: None}, deque([start])
                        while queue:
                            x, y = queue.popleft()
                            for dx, dy, key in [(1, 0, 'right'), (-1, 0, 'left'),
                                                (0, 1, 'down'), (0, -1, 'up')]:
                                at = x + dx, y + dy
                                if at in floor and at not in parents:
                                    parents[at] = ((x, y), key)
                                    queue.append(at)
                        assert target in parents, (start, target)
                        course, at = [], target
                        while parents[at] is not None:
                            prior, key = parents[at]
                            course.append((at, key))
                            at = prior
                        for at, key in reversed(course):
                            client.move(key)
                            client.at(MAP, *at)

                    if phase == 'walk':
                        client.at(MAP, 4, 13)
                        if not continued:
                            for at, face, snippet in [
                                ((6, 13), 'right', 'SHORE WATCH'),
                                ((16, 16), 'right', 'flash of wings'),
                                ((26, 23), 'right', 'own weather'),
                                ((12, 30), 'left', 'wooded bend'),
                                ((39, 10), 'up', 'marked buoys'),
                            ]:
                                walk(at)
                                client.press(face)
                                client.press('a')
                                client.move('down')
                                client.at(MAP, *at)
                                assert snippet in client.finish_dialogue()
                            for x in [16, 17]:
                                walk((x, 17))
                                for _ in range(4):
                                    client.move('down')
                                    status = client.status()['position']
                                    assert (status['x'], status['y']) in [(x, 18), (x, 20)]
                                    if status == dict(x=x, y=20):
                                        break
                                client.at(MAP, x, 20)
                                client.move('up')
                                client.at(MAP, x, 20)
                                walk((x, 17))
                            for at, direction in [((24, 21), 'up'), ((24, 24), 'down'),
                                                  ((27, 22), 'right')]:
                                walk(at)
                                client.move(direction)
                                client.at(MAP, *at)
                            for y, outside_y in [(12, 10), (13, 11)]:
                                walk((0, y))
                                client.move('left')
                                client.at(ROUTE, 15, outside_y)
                                client.move('right')
                                client.at(MAP, 0, y)
                            walk((4, 13))
                        else:
                            walk((16, 16))
                            walk((24, 23))
                            walk((4, 13))
                    else:
                        client.at(MAP, 36 if continued else 37, 29)
                        if not continued:
                            assert ITEM in json.dumps(client.call('map_info'))
                            client.press('left')
                            client.at(MAP, 37, 29)
                            client.press('a')
                            client.move('down')
                            client.at(MAP, 37, 29)
                            pages = client.finish_dialogue()
                            assert 'PP UP' in pages and 'ITEM POCKET' in pages, pages
                            assert ITEM not in json.dumps(client.call('map_info'))
                            client.move('left')
                            client.at(MAP, 36, 29)
                        else:
                            assert ITEM not in json.dumps(client.call('map_info'))
                            client.move('right')
                            client.at(MAP, 37, 29)
                            client.move('left')
                            client.at(MAP, 36, 29)
                        client.press('a')
                        client.finish_dialogue()
                        assert ITEM not in json.dumps(client.call('map_info'))
                    client.press('start')
                    assert 'PACK' in client.observe()['content'][0]['text']
                    client.press('b')
                finally:
                    process.stdin.close()
                    assert process.wait(timeout=30) == 0
                stderr.seek(0)
                assert stderr.read() == b'', 'Production gameplay stderr must stay byte-empty.'
            count += 1
    receipt = dict(status='PASS', pack_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                   client_sha256=hashlib.sha256(args.client.read_bytes()).hexdigest(),
                   processes=count, both_road_lanes_and_directions=True,
                   five_resident_conversations_and_input_lock=True,
                   both_drop_lanes_reverse_blocks_and_ordinary_returns=True,
                   dock_rails_and_closed_end=True, native_roost_notice_and_collected_floor=True,
                   continued_roost_once_only=True, production_start_menu=True,
                   stderr_byte_empty=True,
                   scope='Production stdio from actual controller-reached same-pack checkpoints. '
                         'Empty-party local walking and legal regional-party Surf setup are explicit fixtures. '
                         'Full paddle, item quantity/flag effects and the native wild/RUN are separately controller-proven; access is not earned here.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
