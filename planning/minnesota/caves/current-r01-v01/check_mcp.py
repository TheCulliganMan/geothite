#!/usr/bin/env python3
"""Play the complete hollow cycle, both returns and original guide over stdio."""
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

ROOM = 'MnR01Cave1F'
ITEM = 'MN_R01_ITEM_03'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--client', type=Path, default=ROOT / 'target/runtime-fps/wasm-dev/geothite')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    plan = json.loads(Path(__file__).with_name('input-plan.json').read_text())
    for key, steps in [('complete_loop_core', 26), ('short_east_return', 6),
                       ('long_guided_return', 18)]:
        assert len(plan[key]) == steps + 1
    module = pack['data']['maps'][ROOM]
    assert [module['attributes'][key] for key in ['width', 'height']] == [8, 7]
    collision = pack['data']['tilesets']['cave']['collision']
    bodies = {(actor['x'], actor['y']) for actor in module['objects']}
    floor = {(x, y) for y in range(14) for x in range(16)
             if collision[f"{module['blocks'][y // 2 * 8 + x // 2]:02x}"][y % 2 * 2 + x % 2] == 'FLOOR'
             and (x, y) not in bodies}
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
                           clientInfo=dict(name='limestone-hollows', version='1')))
                client.at(ROOM, 9, 10)

                def course(points):
                    client.at(ROOM, *points[0])
                    for start, target in zip(points, points[1:]):
                        dx, dy = target[0] - start[0], target[1] - start[1]
                        assert abs(dx) + abs(dy) == 1
                        client.move('right' if dx > 0 else 'left' if dx < 0 else 'down' if dy > 0 else 'up')
                        client.at(ROOM, *target)
                        if not continued and target == [7, 3] and not (args.output / 'upper-cycle.crystalsave').exists():
                            shutil.copy2(live, args.output / 'upper-cycle.crystalsave')

                def walk(target):
                    status = client.status()
                    assert status['map'] == ROOM
                    start = status['position']['x'], status['position']['y']
                    parents, queue = {start: None}, deque([start])
                    while queue:
                        x, y = queue.popleft()
                        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                            if at in floor and at not in parents:
                                parents[at] = (x, y)
                                queue.append(at)
                    assert target in parents
                    points, at = [], target
                    while at is not None:
                        points.append(at)
                        at = parents[at]
                    course(list(reversed(points)))

                if not continued:
                    course(plan['complete_loop_core'])
                    walk((10, 5))
                    course(plan['short_east_return'])
                    walk((10, 5))
                    course(plan['long_guided_return'])
                    walk((4, 5))
                    client.press('right')
                    client.press('a')
                    client.move('down')
                    client.at(ROOM, 4, 5)
                    pages = client.finish_dialogue()
                    assert all(line in pages for line in ['Two small hollows', 'meet by the exit.',
                                                          'Take either path.', 'The little ball',
                                                          'holds an ANTIDOTE.'])
                    walk((10, 5))
                    assert ITEM in json.dumps(client.call('map_info'))
                    client.press('right')
                    client.at(ROOM, 10, 5)
                    client.press('a')
                    client.move('down')
                    client.at(ROOM, 10, 5)
                    pages = client.finish_dialogue()
                    assert 'ANTIDOTE' in pages and 'ITEM POCKET' in pages
                    assert ITEM not in json.dumps(client.call('map_info'))
                    floor.add((11, 5))
                else:
                    assert ITEM not in json.dumps(client.call('map_info'))
                walk((11, 5))
                client.press('a')
                client.finish_dialogue()
                assert ITEM not in json.dumps(client.call('map_info'))
                walk((9, 10))
                if not continued:
                    client.move('down')
                    client.at('MnR01C00R00', 18, 18)
                    client.move('down')
                    client.at('MnR01C00R00', 18, 19)
                    client.move('up')
                    client.at('MnR01C00R00', 18, 18)
                    shutil.copy2(live, args.output / 'passive-exterior.crystalsave')
                    client.move('up')
                    client.at(ROOM, 9, 11)
                    client.move('up')
                    client.at(ROOM, 9, 10)
                client.press('start')
                assert 'PACK' in client.observe()['content'][0]['text']
                client.press('b')
                client.at(ROOM, 9, 10)
            finally:
                process.stdin.close()
                assert process.wait(timeout=30) == 0
            stderr.seek(0)
            assert stderr.read() == b'', 'Production gameplay stderr must remain byte-empty.'
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    receipt = dict(status='PASS', pack_sha256=digest(args.pack), client_sha256=digest(args.client),
                   reached_save_sha256=digest(args.save), complete_cycle_inputs=26,
                   short_return_inputs=6, guide_side_return_inputs=18,
                   original_guide_pages_and_input_lock=True, item_notice_and_body_removal=True,
                   true_passive_mouth_and_same_mouth_reentry=True,
                   continued_collected_tile_walkable=True, production_start_menu=True,
                   stderr_byte_empty=True,
                   scope='Actual stdio inputs from explicit local empty-party setup. '
                         'Native Antidote quantity/status/flag effects are separately controller-tested; regional access is not earned here.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_mcp.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
