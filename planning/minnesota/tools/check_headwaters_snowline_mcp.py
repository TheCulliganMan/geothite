#!/usr/bin/env python3
"""Walk the native Headwaters snowline and exercise its existing lake-pier Cut gate.

Initial position, legal Minnesota partner, Cut, Hive Badge and supplies are
explicit local setup. All later actions use the production controller via MCP.
"""
import argparse
from copy import deepcopy
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import close_ui, find_cases, inspect, open_cut, session, sha
from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded
from check_river_story_mcp import road
from render_reference import load_pack


def tree_override(state):
    return next((v['block'] for v in state['overrides'] if v['map'] == 'MnR08C00R00'
                 and v['metatile'] == [3, 10]), None)


def actual_tree_floor(pack):
    result = deepcopy(pack)
    module = result['data']['maps']['MnR08C00R00']
    module['blocks'][10 * module['attributes']['width'] + 3] = 0x3f
    return result


def route(client, pack, continuing=False):
    if not continuing:
        client.at('MnR08C00R00', 24, 1)
    road(client, pack, (24, 0), 'MnR08', 2, 2, escape_wild)
    client.move('up')
    client.at('MnBemidji', 24, pack['data']['maps']['MnBemidji']['attributes']['height'] * 2 - 1)
    client.move('down')
    client.at('MnR08C00R00', 24, 0)
    for goal in [(35, 28), (35, 25), (35, 28), (70, 12), (73, 72), (26, 95)]:
        road(client, pack, goal, 'MnR08', 2, 2, escape_wild)
    client.move('down')
    client.at('MnLeechLake', 32, 0)
    client.move('up')
    client.at('MnR08C00R01', 26, 47)
    road(client, pack, (24, 1), 'MnR08', 2, 2, escape_wild)
    client.press('start')
    assert 'PACK' in json.dumps(client.observe())
    client.press('b')


def run(args, pack, mode, source=None):
    folder = args.output / mode
    folder.mkdir(exist_ok=False)
    if source is None:
        source = folder / 'initial.crystalsave'
        pos = ['24', '1', 'down'] if mode == 'route' else ['6', '22', 'up']
        cmd = [str(args.fixture_writer), 'fixture', str(args.pack), str(source), 'MnR08C00R00',
               *pos, '0' if mode == 'no-cut' else '1', '0' if mode == 'no-hive' else '1']
        result = subprocess.run(cmd, capture_output=True, text=True)
        assert result.returncode == 0, result.stderr
    initial_hash = sha(source)
    records, error = [], None
    actual = folder / 'actual.crystalsave'
    with (folder / 'stderr.log').open('x') as err:
        proc = subprocess.Popen([str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=err, text=True,
                                env={**os.environ, 'RUST_MIN_STACK': '67108864'}, bufsize=1)
        c = LiveRecorded(proc, records)
        try:
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                    clientInfo=dict(name='headwaters-snowline', version='1')))
            if mode.startswith('route'):
                route(c, pack, mode == 'route-continue')
            elif mode == 'gate-continue':
                c.at('MnR08C00R00', 10, 18)
                c.press('start'); c.press('b')
                # Native trees regrow on Continue. Walk to the inside face and
                # cut through the real party menu before returning to the road.
                assert tree_override(inspect(args.fixture_writer, args.pack, actual)) is None
                road(c, pack, (6, 20), 'MnR08', 2, 2, escape_wild)
                c.press('down')
                c.at('MnR08C00R00', 6, 20)
                open_cut(c, folder); close_ui(c)
                assert tree_override(inspect(args.fixture_writer, args.pack, actual)) == 0x3f
                road(c, actual_tree_floor(pack), (6, 22), 'MnR08', 2, 2, escape_wild)
            else:
                c.at('MnR08C00R00', 6, 22)
                c.move('up'); c.at('MnR08C00R00', 6, 22)
                open_cut(c, folder, no_cut=mode == 'no-cut'); close_ui(c)
                state = inspect(args.fixture_writer, args.pack, actual)
                if mode == 'gate':
                    assert tree_override(state) == 0x3f
                    road(c, actual_tree_floor(pack), (10, 18), 'MnR08', 2, 2, escape_wild)
                else:
                    assert tree_override(state) is None
                    c.move('up'); c.at('MnR08C00R00', 6, 22)
            (folder / 'final-observe.json').write_text(json.dumps(c.observe(), indent=2) + '\n')
            proc.stdin.close()
            assert proc.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if proc.poll() is None:
                proc.terminate(); proc.wait(timeout=30)
    report = dict(status='FAIL' if error else 'PASS', mode=mode, error=error,
                  rpc_calls=len(records), records=records, initial_save=str(source), actual_save=str(actual),
                  initial_unchanged=sha(source) == initial_hash, stderr_bytes=(folder / 'stderr.log').stat().st_size)
    (folder / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    assert error is None and report['initial_unchanged'] and not report['stderr_bytes'], report['error']
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'fixture-writer', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.client = args.client.resolve()
    args.fixture_writer = args.fixture_writer.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    hashes = {str(p): sha(p) for p in [args.pack, args.fixture_writer, args.client]}
    world = Path('planning/minnesota/world.json').read_bytes()
    results = []
    for mode in ['route', 'no-cut', 'no-hive', 'gate']:
        result = run(args, pack, mode)
        results.append(result)
        print(mode, 'PASS', result['rpc_calls'], flush=True)
        if mode in ['route', 'gate']:
            result = run(args, pack, mode + '-continue', Path(result['actual_save']))
            results.append(result)
            print(mode + '-continue', 'PASS', result['rpc_calls'], flush=True)
    grass = find_cases(pack, {'rules': [
        {'tileset': 'mn_headwaters_thaw', 'grass_block': block} for block in ['88', '9f']
    ]}, expected_cases=3)
    for case in grass:
        for mode in ['no-cut', 'no-hive', 'cut-and-hive']:
            folder = args.output / ('grass-' + case['id'] + '-' + mode)
            result = session(args, case, mode, folder, None, create=True)
            results.append(result)
            print(case['id'], mode, 'PASS', result['rpc_calls'], flush=True)
            if mode == 'cut-and-hive':
                folder = args.output / ('grass-' + case['id'] + '-continue')
                result = session(args, case, 'continue', folder, Path(result['actual_save']))
                results.append(result)
                print(case['id'], 'continue', 'PASS', result['rpc_calls'], flush=True)
    assert all(sha(Path(p)) == digest for p, digest in hashes.items())
    assert world == Path('planning/minnesota/world.json').read_bytes()
    report = dict(status='PASS', inputs=hashes, inputs_unchanged=True, world_unchanged=True,
                  sessions=len(results), rpc_calls=sum(r['rpc_calls'] for r in results), results=results,
                  scope='Explicit local party/move/Hive/position setup; every later walk, menu, Cut and Continue uses production input. No earned campaign, Surf or reward-collection claim.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
