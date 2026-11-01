#!/usr/bin/env python3
"""Replay real Safari command ownership, Run and Continue through production MCP.

The input is the retained controller-generated wildlife-menu save. No wildlife,
position, money, party, flags or budgets are injected by this replay.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import sha
from check_mina_battle_mcp import LiveRecorded


def view(client):
    return client.observe()['structuredContent']


def selected(display):
    return next(row['text'].strip()[2:] for row in display['menu']
                if row['text'].strip().startswith('> '))


def session(args, source, folder, encounter):
    folder.mkdir(exist_ok=False)
    actual, records, error = folder / 'actual.crystalsave', [], None
    with (folder / 'stderr.log').open('x') as stderr:
        process = subprocess.Popen(
            [str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True,
            env={**os.environ, 'RUST_MIN_STACK': '67108864'}, bufsize=1)
        client = LiveRecorded(process, records)
        try:
            client.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                         clientInfo=dict(name='current-safari-review', version='1')))
            before = client.status()
            assert before['money'] == 1000
            if encounter:
                initial = view(client)
                assert initial['viewport_title'] == 'Safari' and selected(initial) == 'RUN'
                assert all(command in json.dumps(initial['menu'])
                           for command in ['BALL', 'BAIT', 'ROCK', 'RUN'])
                for button in ['start', 'b', 'select']:
                    client.press(button)
                    current = view(client)
                    assert current['menu'] == initial['menu']
                    assert current['viewport'] == initial['viewport']
                client.press('left')
                assert selected(view(client)) == 'ROCK'
                client.press('right')
                assert selected(view(client)) == 'RUN'
                client.press('a')
                assert 'Got away safely!' in json.dumps(view(client))
                client.press('a')
            client.press('start')
            assert 'PACK' in json.dumps(view(client)['menu'])
            client.press('b')
            after = client.status()
            assert after['position'] == before['position'] and after['map'] == before['map']
            assert after['money'] == before['money']
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', error=error, initial_save=str(source),
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
    paths = [args.pack, args.save, args.client, Path('planning/minnesota/world.json')]
    inputs = {str(path): sha(path) for path in paths}
    first = session(args, args.save, args.output / 'wildlife-menu', True)
    results = [first, session(args, Path(first['actual_save']), args.output / 'continue-field', False)]
    assert all(sha(Path(path)) == digest for path, digest in inputs.items())
    report = dict(status='PASS', sessions=len(results),
                  rpc_calls=sum(r['rpc_calls'] for r in results), inputs=inputs,
                  inputs_unchanged=True, results=results,
                  scope='Actual reached wildlife, command cursor/ownership, native Run/result, '
                        'normal Start and fresh Continue. Initial ranger-course partner, money, '
                        'Surf/badge and deterministic divider are declared setup fixtures.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS', report['sessions'], 'sessions', report['rpc_calls'], 'calls', flush=True)


if __name__ == '__main__':
    main()
