#!/usr/bin/env python3
"""Use native party-menu Cut on every placed grass variant in the winter repair.

Requires a built current geothite client, the Rust inland_grass_cut_fixture
example and the completed external candidate. Fixtures are explicit local setup;
all subsequent actions use VisibleShellController through production stdio MCP.
"""
import argparse
from collections import deque
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded
from render_reference import load_pack

VECTORS = {'right': (1, 0), 'left': (-1, 0), 'down': (0, 1), 'up': (0, -1)}
OPPOSITE = {'right': 'left', 'left': 'right', 'up': 'down', 'down': 'up'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def find_cases(pack, plan, expected_cases=12):
    wanted = [('mn_tamarack_thaw', 3)] + [
        (r['tileset'], int(r['grass_block'], 16)) for r in plan['rules']]
    cases = []
    for alias, grass in wanted:
        chosen = None
        for name, module in pack['data']['maps'].items():
            a = module['attributes']
            if a['tileset_name'] != alias or grass not in module['blocks']:
                continue
            collision = pack['data']['tilesets'][alias]['collision']
            occupied = {(o['x'], o['y']) for o in module['objects']}
            def kind(x, y):
                block = module['blocks'][y // 2 * a['width'] + x // 2]
                return collision[f'{block:02x}'][y % 2 * 2 + x % 2]
            for i, block in enumerate(module['blocks']):
                if block != grass:
                    continue
                mx, my = i % a['width'], i // a['width']
                for dx, dy, ax, ay, facing in [
                        (0, 0, -1, 0, 'right'), (1, 0, 2, 0, 'left'),
                        (0, 0, 0, -1, 'down'), (0, 1, 0, 2, 'up')]:
                    target, approach = (mx * 2 + dx, my * 2 + dy), (mx * 2 + ax, my * 2 + ay)
                    if not (0 <= approach[0] < a['width'] * 2 and 0 <= approach[1] < a['height'] * 2):
                        continue
                    if kind(*target) != 'TALL_GRASS' or kind(*approach) not in {'FLOOR', 'TALL_GRASS'}:
                        continue
                    if approach in occupied or target in occupied:
                        continue
                    if any(abs(o['x'] - approach[0]) + abs(o['y'] - approach[1]) <= o['radius'] + 3
                           for o in module['objects'] if o['object_type'] == 'OBJECTTYPE_TRAINER'):
                        continue
                    replacement = pack['data']['field_moves']['cut']['replacements'][alias][grass]
                    chosen = dict(id=f'{alias}-{grass:02x}', map=name, tileset=alias,
                                  grass_block=grass, metatile=[mx, my], approach=list(approach),
                                  target=list(target), facing=facing, collision=collision[f'{grass:02x}'],
                                  replacement=replacement['replacement_block_id'])
                    break
                if chosen:
                    break
            if chosen:
                break
        placed = any(m['attributes']['tileset_name'] == alias and grass in m['blocks']
                     for m in pack['data']['maps'].values())
        assert chosen or not placed, ('no safe native approach', alias, grass)
        if chosen:
            cases.append(chosen)
    assert len(cases) == expected_cases, 'native green control and every requested placed variant required'
    return cases


def inspect(writer, pack, save):
    result = subprocess.run([str(writer), 'inspect', str(pack), str(save)], capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    return json.loads(result.stdout)


def override(state, case):
    entries = [v['block'] for v in state['overrides']
               if v['map'] == case['map'] and v['metatile'] == case['metatile']]
    assert len(entries) <= 1
    return entries[0] if entries else None


def symbol(client, x, y):
    rows = client.observe()['structuredContent']['viewport']
    columns = [int(v) for v in rows[0]['text'].split()]
    line = next(row['text'].split()[1:] for row in rows[1:]
                if int(row['text'].split()[0]) == y)
    return line[columns.index(x)]


def open_cut(client, folder, no_cut=False):
    client.press('start')
    menu = client.observe()['structuredContent']['menu']
    assert any(v['kind'] == 'selected' and v['text'].strip() == '> #MON' for v in menu), menu
    client.press('a')
    menu = client.observe()['structuredContent']['menu']
    assert any(v['kind'] == 'selected' and 'VOYAGEAR' in v['text'] for v in menu), menu
    client.press('a')
    view = client.observe()
    (folder / 'party-submenu.json').write_text(json.dumps(view, indent=2) + '\n')
    menu = view['structuredContent']['menu']
    has_cut = any(v['kind'] == 'selected' and v['text'].strip().upper() == '> CUT' for v in menu)
    if no_cut:
        assert not any(v['text'].strip(' >').upper() == 'CUT' for v in menu)
    else:
        assert has_cut, menu
        client.press('a')
        (folder / 'cut-response.json').write_text(json.dumps(client.observe(), indent=2) + '\n')


def close_ui(client):
    client.finish_dialogue()
    for _ in range(12):
        s = client.status()
        if s['mode'] == 'overworld' and s['menu'] is None and not s['dialogue']:
            return
        client.press('b')
    raise AssertionError('native Cut/party UI did not release input')


def step(client, case, direction, target):
    client.move(direction)
    if client.status()['mode'] != 'overworld':
        escape_wild(client)
    if client.status()['dialogue']:
        client.finish_dialogue()
    client.at(case['map'], *target)


def verify_cleared_wall_cells(client, case):
    if 'WALL' not in case['collision']:
        return
    mx, my = case['metatile']
    floor = {(mx * 2 + q % 2, my * 2 + q // 2) for q, kind in enumerate(case['collision'])
             if kind == 'TALL_GRASS'}
    walls = {(mx * 2 + q % 2, my * 2 + q // 2) for q, kind in enumerate(case['collision'])
             if kind == 'WALL'}
    at = tuple(case['target'])
    for goal in sorted(floor) + [tuple(case['target'])]:
        queue, paths = deque([at]), {at: []}
        while queue and goal not in paths:
            x, y = queue.popleft()
            for button, (dx, dy) in VECTORS.items():
                to = x + dx, y + dy
                if to in floor and to not in paths:
                    paths[to] = paths[x, y] + [(button, to)]
                    queue.append(to)
        assert goal in paths
        for button, to in paths[goal]:
            step(client, case, button, to)
        at = goal
        for button, (dx, dy) in VECTORS.items():
            if (at[0] + dx, at[1] + dy) in walls:
                step(client, case, button, at)


def session(args, case, mode, folder, source, create=False):
    folder.mkdir(exist_ok=False)
    if create:
        source = folder / 'initial.crystalsave'
        cmd = [str(args.fixture_writer), 'fixture', str(args.pack), str(source), case['map'],
               *map(str, case['approach']), case['facing'], '0' if mode == 'no-cut' else '1',
               '0' if mode == 'no-hive' else '1']
        result = subprocess.run(cmd, capture_output=True, text=True)
        assert result.returncode == 0, result.stderr
    initial_hash = sha(source)
    initial_state = inspect(args.fixture_writer, args.pack, source)
    actual, records, error = folder / 'actual.crystalsave', [], None
    with (folder / 'stderr.log').open('x') as err:
        proc = subprocess.Popen([str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=err, text=True, bufsize=1,
                                env={**os.environ, 'RUST_MIN_STACK': '67108864'})
        client = LiveRecorded(proc, records)
        try:
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                         clientInfo=dict(name='native-inland-grass-cut', version='1')))
            client.at(case['map'], *case['approach'])
            if mode == 'continue':
                client.press('start')
                client.press('b')
                continued = inspect(args.fixture_writer, args.pack, actual)
                continued_override = override(continued, case)
                assert continued_override in [None, case['replacement']]
                reset = continued_override is None
                assert symbol(client, *case['target']) == ('"' if reset else '.')
                if reset:
                    # Normal input re-enters the regrown block and faces within it.
                    step(client, case, case['facing'], case['target'])
                    open_cut(client, folder)
                    close_ui(client)
                    step(client, case, OPPOSITE[case['facing']], case['approach'])
                else:
                    step(client, case, case['facing'], case['target'])
                    step(client, case, OPPOSITE[case['facing']], case['approach'])
            else:
                assert symbol(client, *case['target']) == '"'
                open_cut(client, folder, no_cut=mode == 'no-cut')
                if mode == 'cut-and-hive' and client.status()['dialogue']:
                    # The native use notice owns input; it cannot move the player.
                    client.move(case['facing'])
                    client.at(case['map'], *case['approach'])
                close_ui(client)
                client.at(case['map'], *case['approach'])
                if mode == 'cut-and-hive':
                    assert symbol(client, *case['target']) == '.'
                    shutil.copy2(actual, folder / 'after-cut-before-walk.crystalsave')
                    step(client, case, case['facing'], case['target'])
                    verify_cleared_wall_cells(client, case)
                    step(client, case, OPPOSITE[case['facing']], case['approach'])
                else:
                    assert symbol(client, *case['target']) == '"'
            final = client.status()
            proc.stdin.close()
            assert proc.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if proc.poll() is None:
                proc.terminate()
                proc.wait(timeout=30)
    report = dict(status='FAIL' if error else 'PASS', case=case, mode=mode, error=error,
                  records=records, rpc_calls=len(records), initial_save=str(source), actual_save=str(actual),
                  initial_unchanged=sha(source) == initial_hash, stderr_bytes=(folder / 'stderr.log').stat().st_size)
    if not error:
        saved = inspect(args.fixture_writer, args.pack, actual)
        expected = None if mode in ['no-cut', 'no-hive'] else case['replacement']
        assert override(saved, case) == expected, saved
        assert saved['party'] == initial_state['party'], 'field Cut must preserve native HP and PP'
        assert saved['money'] == initial_state['money'] and saved['badges'] == initial_state['badges']
        report.update(native_saved_state=saved, final=final)
        if mode == 'continue':
            report['regrew_on_continue'] = reset
    assert report['initial_unchanged'] and report['stderr_bytes'] == 0, report
    (folder / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    assert not error, error
    return report


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'fixture-writer', 'client', 'output']:
        cli.add_argument('--' + key, type=Path, required=True)
    cli.add_argument('--source', type=Path, default=Path(
        'planning/minnesota/revisions/inland-grass-cut-v01/source-v02.json'))
    args = cli.parse_args()
    for key in ['pack', 'fixture_writer', 'client']:
        setattr(args, key, getattr(args, key).resolve())
    args.output.mkdir(exist_ok=False, parents=True)
    inputs = {str(p): sha(p) for p in [args.pack, args.fixture_writer, args.client, args.source]}
    plan, pack = json.loads(args.source.read_text()), load_pack(args.pack)
    cases = find_cases(pack, plan)
    (args.output / 'cases.json').write_text(json.dumps(cases, indent=2) + '\n')
    results, denied_aliases = [], set()
    try:
        for case in cases:
            if case['tileset'] not in denied_aliases:
                for mode in ['no-cut', 'no-hive']:
                    r = session(args, case, mode, args.output / (case['id'] + '-' + mode), None, create=True)
                    results.append(r)
                    print('PASS', case['id'], mode, flush=True)
                denied_aliases.add(case['tileset'])
            r = session(args, case, 'cut-and-hive', args.output / (case['id'] + '-cut'), None, create=True)
            results.append(r)
            r = session(args, case, 'continue', args.output / (case['id'] + '-continue'), Path(r['actual_save']))
            results.append(r)
            assert r['regrew_on_continue'] == results[3]['regrew_on_continue'], 'native green/icy reset behavior differs'
            print('PASS', case['id'], 'native Cut, walls and Continue', flush=True)
    finally:
        unchanged = all(sha(Path(p)) == expected for p, expected in inputs.items())
        report = dict(status='PASS' if len(results) == 32 and unchanged else 'INCOMPLETE_OR_FAILED',
                      cases=len(cases), sessions=len(results), rpc_calls=sum(r['rpc_calls'] for r in results),
                      inputs_unchanged=unchanged, results=results,
                      scope='Independent initial position/VOYAGEAR60/move/Hive/repel/held-item fixtures. All later Cut, movement, walls, notices and Continue use production MCP input. Native green grass is the reset control; no earned campaign claim.')
        (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    assert report['status'] == 'PASS'


if __name__ == '__main__':
    main()
