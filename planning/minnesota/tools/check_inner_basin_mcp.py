#!/usr/bin/env python3
"""Solve the native basin from a real, retained closed-arrival checkpoint.

The preceding controller actually reads and compares the exterior shore marks.
Initial local partner/position/capture supplies are declared setup. No state is
rewritten after input; the open terrain copy below only plans walking inputs.
"""
import argparse
from copy import deepcopy
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import sha
from check_mina_battle_mcp import LiveRecorded
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack


def session(args, packed, opened, source, output, first):
    output.mkdir(exist_ok=False)
    records, pages, error = [], [], None
    actual = output / 'actual.crystalsave'
    with (output / 'stderr.log').open('x') as stderr:
        proc = subprocess.Popen([str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True,
                                bufsize=1, env={**os.environ, 'RUST_MIN_STACK': '67108864'})
        c = LiveRecorded(proc, records)
        try:
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                    clientInfo=dict(name='inner-basin-native', version='1')))
            if first:
                c.at('MnInnerBasin', 2, 25)
                walk(c, packed, (14, 8))
                for _ in range(3):
                    c.move('up'); c.at('MnInnerBasin', 14, 8)
                walk(c, packed, (15, 20))
                c.press('left'); c.press('a')
                pages.append(finish(c))
                # Native low -> middle -> high and a full lower cycle back to low.
                for approach, facing, presses in [((10, 10), 'left', 2),
                                                   ((17, 14), 'right', 1),
                                                   ((12, 18), 'left', 3)]:
                    walk(c, packed, approach)
                    for _ in range(presses):
                        c.press(facing); c.press('a')
                        pages.append(finish(c))
                walk(c, packed, (15, 20))
                c.press('left'); c.press('a')
                pages.append(finish(c))
                # Only planning uses the known original drain changes. These
                # movement assertions independently require the real game to open.
                walk(c, opened, (14, 8))
                c.move('up'); c.at('MnInnerBasin', 14, 7)
                walk(c, opened, (14, 3))
            else:
                c.at('MnInnerBasin', 14, 3)
            walk(c, opened, (3, 24))
            travel(c, 'down', 'MnFormerShores', (18, 24))
            travel(c, 'up', 'MnInnerBasin', (2, 25))
            walk(c, opened, (14, 3))
            c.press('start')
            assert 'PACK' in json.dumps(c.observe())
            c.press('b')
            (output / 'final-observe.json').write_text(json.dumps(c.observe(), indent=2) + '\n')
            proc.stdin.close()
            assert proc.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if proc.poll() is None:
                proc.terminate(); proc.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', error=error, records=records, pages=pages,
                  rpc_calls=len(records), actual_save=str(actual), stderr_bytes=(output / 'stderr.log').stat().st_size)
    (output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    assert not error and not result['stderr_bytes'], error
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'client', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    for name in ['pack', 'save', 'client']:
        setattr(args, name, getattr(args, name).resolve())
    args.output.mkdir(exist_ok=False, parents=True)
    world = Path('planning/minnesota/world.json')
    inputs = {str(p): sha(p) for p in [args.pack, args.save, args.client, world]}
    packed = load_pack(args.pack)
    opened = deepcopy(packed)
    room = opened['data']['maps']['MnInnerBasin']
    for change in room['script_block_changes']:
        room['blocks'][change['y'] // 2 * 14 + change['x'] // 2] = change['block_id']
    results = [session(args, packed, opened, args.save, args.output / 'solve', True)]
    results.append(session(args, packed, opened, Path(results[0]['actual_save']), args.output / 'continue', False))
    assert all(sha(Path(p)) == digest for p, digest in inputs.items())
    report = dict(status='PASS', inputs=inputs, inputs_unchanged=True, results=results,
                  sessions=len(results), rpc_calls=sum(r['rpc_calls'] for r in results),
                  scope='Actual previously read shore marks; closed-sill refusal, original three-dial solution/lever, real opened walk, physical same-mouth return/restoration, Start and fresh-process Continue. Initial local partner/position/supplies are explicit fixtures; no earned regional acquisition claim.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS', report['sessions'], 'sessions', report['rpc_calls'], 'calls', flush=True)


if __name__ == '__main__':
    main()
