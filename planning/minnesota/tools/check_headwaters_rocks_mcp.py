#!/usr/bin/env python3
"""Prove moved Headwaters rocks and unchanged cliff/stair collision with input."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import sha
from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded

CASES = [
    ('west-upper', (26, 24), 'up', [
        ('up', (26, 24)), ('right', (27, 24)), ('up', (27, 24)),
        ('down', (27, 25)), ('down', (27, 26)), ('down', (27, 26)),
        ('left', (26, 26)), ('down', (26, 26)), ('up', (26, 25)),
        ('up', (26, 24)), ('left', (25, 24)),
        ('left', (25, 24)), ('right', (26, 24))]),
    ('east-upper', (40, 24), 'up', [
        ('up', (40, 24)), ('right', (41, 24)), ('up', (41, 24)),
        ('down', (41, 25)), ('down', (41, 26)), ('down', (41, 26)),
        ('left', (40, 26)), ('down', (40, 26)), ('up', (40, 25)),
        ('up', (40, 24))]),
    ('lower-stair', (32, 28), 'down', [
        ('down', (32, 29)), ('down', (32, 29)), ('right', (33, 29)),
        ('down', (33, 29)), ('up', (33, 28)), ('right', (34, 28)),
        ('right', (35, 28)), ('up', (35, 27)), ('up', (35, 26)),
        ('up', (35, 25)), ('down', (35, 26)), ('down', (35, 27)),
        ('down', (35, 28)), ('left', (34, 28)), ('left', (33, 28)),
        ('left', (32, 28))]),
]


def run(args, case, source, folder):
    name, start, _, steps = case
    folder.mkdir(exist_ok=False)
    initial_hash = sha(source)
    actual, records, error = folder / 'actual.crystalsave', [], None
    with (folder / 'stderr.log').open('x') as err:
        proc = subprocess.Popen([str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=err, text=True,
                                env={**os.environ, 'RUST_MIN_STACK': '67108864'}, bufsize=1)
        c = LiveRecorded(proc, records)
        try:
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                    clientInfo=dict(name='headwaters-native-rock-physics', version='1')))
            c.at('MnR08C00R00', *start)
            for button, expected in steps:
                previous = c.status()['position']
                for _ in range(8):
                    c.move(button)
                    if c.status()['mode'] != 'overworld':
                        escape_wild(c)
                    state = c.status()
                    assert state['mode'] == 'overworld', state
                    if state['position'] == dict(x=expected[0], y=expected[1]):
                        break
                    assert state['position'] == previous, (button, expected, state)
                c.at('MnR08C00R00', *expected)
            c.press('start')
            assert 'PACK' in json.dumps(c.observe())
            c.press('b')
            c.at('MnR08C00R00', *start)
            (folder / 'final-observe.json').write_text(json.dumps(c.observe(), indent=2) + '\n')
            proc.stdin.close()
            assert proc.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if proc.poll() is None:
                proc.terminate(); proc.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', case=name, error=error, initial_save=str(source),
                  actual_save=str(actual), rpc_calls=len(records), records=records,
                  initial_unchanged=sha(source) == initial_hash, stderr_bytes=(folder / 'stderr.log').stat().st_size)
    (folder / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    assert error is None and result['initial_unchanged'] and not result['stderr_bytes'], error
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'fixture-writer', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.client = args.client.resolve()
    args.fixture_writer = args.fixture_writer.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = {str(p): sha(p) for p in [args.pack, args.fixture_writer, args.client]}
    world = Path('planning/minnesota/world.json').read_bytes()
    results = []
    for case in CASES:
        name, start, facing, _ = case
        source = args.output / (name + '-initial.crystalsave')
        command = [str(args.fixture_writer), 'fixture', str(args.pack), str(source), 'MnR08C00R00',
                   *map(str, start), facing, '0', '0']
        fixture = subprocess.run(command, capture_output=True, text=True)
        assert fixture.returncode == 0, fixture.stderr
        result = run(args, case, source, args.output / name)
        results.append(result)
        result = run(args, case, Path(result['actual_save']), args.output / (name + '-continue'))
        results.append(result)
        print(name, 'and Continue PASS', flush=True)
    assert all(sha(Path(p)) == value for p, value in inputs.items())
    assert world == Path('planning/minnesota/world.json').read_bytes()
    report = dict(status='PASS', inputs=inputs, inputs_unchanged=True, world_unchanged=True,
                  sessions=len(results), rpc_calls=sum(r['rpc_calls'] for r in results), results=results,
                  scope='Explicit zero-badge/no-Cut local Minnesota partner/position/repel setup. Every subsequent move, refusal, stair step, Start and Continue uses production input. No earned campaign claim.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
