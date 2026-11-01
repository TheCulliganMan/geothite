#!/usr/bin/env python3
"""Exercise the joined R10 cache/mine terrace using production MCP input.

Requires exact-pack initial saves for the requested named courses. Use cache
for the earlier open southern threshold, or west-cache for the closed terrace
with its southern stair. Initial position, Minnesota partners and Repel are
explicit fixtures. No later state or coordinates are rewritten. Every render,
RPC and native save is retained.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from check_mina_battle_mcp import LiveRecorded


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(args, name):
    output = args.output / name
    output.mkdir(exist_ok=False)
    source = args.fixtures / (name + '.crystalsave')
    original = sha(source)
    records = []
    process = None
    error = None
    try:
        with (output / 'stderr.log').open('x') as stderr:
            process = subprocess.Popen(
                [str(args.client), 'mcp', str(args.pack), '--load', str(source),
                 '--save', str(output / 'actual-autosave.crystalsave')],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                text=True, bufsize=1)
            client = LiveRecorded(process, records)
            client.rpc('initialize', dict(
                protocolVersion='2025-06-18', capabilities={},
                clientInfo=dict(name='r10-cache-terrace-check', version='1')))

            def at(x, y):
                client.at('MnR10C00R00', x, y)
                status = client.status()
                assert status['mode'] == 'overworld' and not status['dialogue'], status

            def denied(button, x, y):
                # One directional press can only turn. Subsequent presses
                # must attempt the wall while already facing it.
                for _ in range(3):
                    client.press(button)
                    at(x, y)

            def walk(button, steps, x, y):
                client.move(button, steps)
                at(x, y)

            if name == 'cache':
                at(25, 13)
                client.move('up', 4)
                at(25, 9)
                client.move('right', 1)
                at(26, 9)  # Clear approach beside the preserved cache at27,9.
                client.move('left', 1)
                client.move('down', 4)
                at(25, 13)
            elif name == 'west-cache':
                at(33, 14)
                client.move('up', 2)
                at(33, 12)
                client.move('left', 4)
                client.move('up', 3)
                at(29, 9)
                client.move('up', 1)
                client.move('left', 3)
                client.move('down', 1)
                at(26, 9)
                client.move('down', 1)
                client.move('left', 3)
                at(23, 10)
                denied('left', 23, 10)
                client.move('up', 2)
                at(23, 8)
                denied('left', 23, 8)
                client.move('down', 3)
                at(23, 11)
                denied('left', 23, 11)
                client.move('down', 1)
                at(23, 12)
                denied('left', 23, 12)
                denied('down', 23, 12)
                client.move('right', 6)
                at(29, 12)
                denied('down', 29, 12)
                client.move('right', 4)
                at(33, 12)
                client.move('down', 2)
                at(33, 14)
            elif name == 'carrier-ridge':
                at(11, 20)
                walk('right', 1, 12, 20)
                denied('down', 12, 20)
                walk('up', 2, 12, 18)
                walk('right', 2, 14, 18)
                denied('right', 14, 18)
                walk('up', 2, 14, 16)
                walk('right', 3, 17, 16)
                denied('down', 17, 16)
                walk('right', 4, 21, 16)
                denied('down', 21, 16)
                walk('right', 2, 23, 16)
                walk('down', 2, 23, 18)
                denied('left', 23, 18)
                walk('down', 2, 23, 20)
                denied('down', 23, 20)
                walk('right', 2, 25, 20)
                denied('down', 25, 20)
                walk('up', 4, 25, 16)
                walk('left', 7, 18, 16)
                walk('up', 2, 18, 14)
                denied('right', 18, 14)
                walk('down', 1, 18, 15)
                walk('right', 1, 19, 15)
                denied('up', 19, 15)
                walk('down', 1, 19, 16)
                walk('up', 1, 19, 15)
                denied('up', 19, 15)
                walk('left', 2, 17, 15)
                walk('up', 4, 17, 11)
                walk('right', 1, 18, 11)
                denied('right', 18, 11)
                walk('up', 2, 18, 9)
                denied('right', 18, 9)
                walk('up', 1, 18, 8)
                denied('up', 18, 8)
                walk('down', 1, 18, 9)
                walk('left', 2, 16, 9)
                walk('down', 2, 16, 11)
                denied('left', 16, 11)
                walk('down', 2, 16, 13)
                walk('left', 3, 13, 13)
                walk('up', 1, 13, 12)
                denied('up', 13, 12)
                walk('down', 2, 13, 14)
                walk('left', 2, 11, 14)
                walk('down', 6, 11, 20)
            elif name == 'carrier-grass':
                at(11, 22)
                walk('right', 8, 19, 22)
                walk('up', 3, 19, 19)
                walk('up', 1, 19, 18)
                denied('up', 19, 18)
                walk('left', 2, 17, 18)
                denied('up', 17, 18)
                walk('right', 4, 21, 18)
                denied('up', 21, 18)
                denied('right', 21, 18)
                walk('down', 1, 21, 19)
                denied('right', 21, 19)
                walk('left', 5, 16, 19)
                denied('left', 16, 19)
                walk('down', 2, 16, 21)
                walk('right', 3, 19, 21)
                walk('down', 1, 19, 22)
                walk('left', 8, 11, 22)
            elif name == 'east-wall':
                at(40, 8)
                denied('right', 40, 8)
                client.move('down', 4)
                at(40, 12)
                denied('right', 40, 12)
                client.move('down', 4)
                at(40, 16)
                denied('right', 40, 16)
                denied('down', 40, 16)
                client.move('up', 4)
                client.move('up', 4)
                at(40, 8)
            else:
                assert name == 'iva'
                at(36, 14)
                client.move('up', 1)
                client.at('MnR10C00R00', 36, 13)
                for _ in range(96):
                    observed = client.observe()
                    view = observed['structuredContent']
                    menu = ' '.join(row['text'] for row in view['menu']).upper()
                    if (all(word in menu for word in ['FIGHT', 'PKMN', 'PACK', 'RUN'])
                            and not view['dialogue'] and not view['prompt']):
                        (output / 'native-battle-menu.json').write_text(
                            json.dumps(observed, indent=2) + '\n')
                        break
                    client.press('a')
                else:
                    raise AssertionError('Iva did not reach the production battle menu')
            process.stdin.close()
            process.wait(timeout=30)
            assert process.returncode == 0
            if name != 'iva':
                reached = output / 'actual-autosave.crystalsave'
                reached_hash = sha(reached)
                process = subprocess.Popen(
                    [str(args.client), 'mcp', str(args.pack), '--load', str(reached),
                     '--save', str(output / 'continued-autosave.crystalsave')],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    text=True, bufsize=1)
                client = LiveRecorded(process, records)
                client.rpc('initialize', dict(
                    protocolVersion='2025-06-18', capabilities={},
                    clientInfo=dict(name='r10-reached-save-continue', version='1')))
                x, y = {'cache': (25, 13), 'west-cache': (33, 14),
                        'carrier-ridge': (11, 20), 'carrier-grass': (11, 22),
                        'east-wall': (40, 8)}[name]
                at(x, y)
                client.press('start')
                menu = client.observe()
                assert 'SAVE' in json.dumps(menu).upper(), menu
                client.press('b')
                at(x, y)
                process.stdin.close()
                process.wait(timeout=30)
                assert process.returncode == 0 and sha(reached) == reached_hash
    except BaseException as exc:
        error = repr(exc)
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            process.wait(timeout=30)
    stderr = (output / 'stderr.log').read_text()
    unchanged = sha(source) == original
    report = dict(status='PASS' if error is None and not stderr and unchanged else 'FAIL',
                  error=error, initial_save_sha256=original, initial_save_unchanged=unchanged,
                  stderr=stderr, calls=records)
    (output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(name, report['status'], error or '', flush=True)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ['pack', 'client', 'fixtures', 'output']:
        parser.add_argument('--' + field, type=Path, required=True)
    parser.add_argument('--cases', nargs='+', choices=[
        'cache', 'west-cache', 'carrier-ridge', 'carrier-grass', 'east-wall', 'iva'],
                        default=['cache', 'east-wall', 'iva'])
    args = parser.parse_args()
    assert len(set(args.cases)) == len(args.cases), 'Duplicate native courses'
    args.client = args.client.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = {str(path): sha(path) for path in [args.pack, args.client, Path(__file__)]}
    results = {name: check(args, name) for name in args.cases}
    unchanged = all(sha(Path(path)) == value for path, value in inputs.items())
    report = dict(status='PASS' if unchanged and all(
        r['status'] == 'PASS' for r in results.values()) else 'FAIL',
        inputs=inputs, inputs_unchanged=unchanged,
        cases={name: r['status'] for name, r in results.items()},
        calls=sum(len(r['calls']) for r in results.values()),
        scope='Runs the named local-fixture courses through production MCP; see cases. '
              'No cache collection, battle turn or earned progression claim.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    assert report['status'] == 'PASS', 'Inspect retained native failure receipts'


if __name__ == '__main__':
    main()
