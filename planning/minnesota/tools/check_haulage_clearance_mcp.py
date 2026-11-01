#!/usr/bin/env python3
"""Replay the reached Haulage stair, rock and cliff refusals through stdio MCP.

Starts from the controller's saved local fixture. No position, party, flags or
terrain are rewritten. A second process loads the actual first-session save.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import sha
from check_mina_battle_mcp import LiveRecorded


STEPS = [
    ('up', (10, 9)), ('up', (10, 8)), ('up', (10, 7)),
    ('left', (9, 7)), ('up', (9, 6)), ('up', (9, 6)),
    ('left', (8, 6)), ('up', (8, 6)), ('down', (8, 7)),
    ('down', (8, 8)), ('down', (8, 8)), ('right', (9, 8)),
    ('down', (9, 8)), ('up', (9, 7)), ('right', (10, 7)),
    ('down', (10, 8)), ('down', (10, 9)), ('down', (10, 10)),
]


def session(args, source, folder):
    folder.mkdir(exist_ok=False)
    actual = folder / 'actual.crystalsave'
    records, error = [], None
    with (folder / 'stderr.log').open('x') as stderr:
        process = subprocess.Popen(
            [str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True,
            env={**os.environ, 'RUST_MIN_STACK': '67108864'}, bufsize=1)
        client = LiveRecorded(process, records)
        try:
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                         clientInfo=dict(name='haulage-clearance-review', version='1')))
            client.at('MnIronHaulage', 10, 10)
            for direction, expected in STEPS:
                client.move(direction)
                client.at('MnIronHaulage', *expected)
                assert client.status()['mode'] == 'overworld'
            client.press('start')
            assert 'PACK' in json.dumps(client.observe())
            client.press('b')
            client.at('MnIronHaulage', 10, 10)
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', error=error, source=str(source),
                  actual_save=str(actual), rpc_calls=len(records), records=records,
                  stderr_bytes=(folder / 'stderr.log').stat().st_size)
    (folder / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    assert not error and not result['stderr_bytes'], error
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'save', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.client = args.client.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    world = Path('planning/minnesota/world.json')
    inputs = {str(path): sha(path) for path in [args.pack, args.save, args.client, world]}
    first = session(args, args.save, args.output / 'stair-and-shoulders')
    results = [first, session(args, Path(first['actual_save']), args.output / 'continue')]
    assert all(sha(Path(path)) == digest for path, digest in inputs.items())
    result = dict(status='PASS', sessions=len(results),
                  rpc_calls=sum(r['rpc_calls'] for r in results), inputs=inputs,
                  inputs_unchanged=True, results=results,
                  scope='Actual production steps, boulder and lower cliff refusals, stair in both '
                        'directions, Start and fresh Continue. Native local partner/position/repel '
                        'are setup fixtures; no earned mine access or regional progression claim.')
    (args.output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    print('PASS', result['sessions'], 'sessions', result['rpc_calls'], 'calls', flush=True)


if __name__ == '__main__':
    main()
