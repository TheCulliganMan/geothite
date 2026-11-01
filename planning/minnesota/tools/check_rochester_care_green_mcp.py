#!/usr/bin/env python3
"""Review tighter Rochester over stdio from its actual earned Care Badge save."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from check_mina_battle_mcp import LiveRecorded
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack


CITY = 'MnRochester'
PACK_SHA256 = '47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(c, pack, point, facing, words):
    walk(c, pack, point)
    c.press(facing)
    c.at(CITY, *point)
    c.press('a')
    assert c.status()['dialogue'], 'authored town interaction must own input'
    c.move('down')
    c.at(CITY, *point)
    pages = finish(c)
    assert words in pages, pages
    return pages


def course(c, pack, first):
    c.at(CITY, *(26, 18) if first else (19, 12))
    assert c.status()['badges'] == 1, 'use the separately earned first-badge checkpoint'
    pages = []
    if first:
        for point, facing, words in [((31, 16), 'left', 'VAREL'),
                                      ((35, 16), 'left', 'ROCHESTER'),
                                      ((23, 18), 'up', 'south gym')]:
            pages.append(read(c, pack, point, facing, words))
        # Both relocated doors and their original native passive arrivals.
        walk(c, pack, (7, 18))
        travel(c, 'up', 'MnRochesterPatientHouse', (5, 6))
        travel(c, 'down', CITY, (7, 18))
        walk(c, pack, (26, 18))
        travel(c, 'up', 'MnRochesterGym', (6, 17))
        walk(c, pack, (6, 18))
        travel(c, 'down', CITY, (26, 18))
        walk(c, pack, (15, 8))
        travel(c, 'up', 'MnRochesterPokecenter1F', (10, 13))
        walk(c, pack, (10, 14))
        travel(c, 'down', CITY, (15, 8))
        pages.append(read(c, pack, (19, 12), 'up', 'open to everyone'))
    else:
        pages.append(read(c, pack, (19, 12), 'up', 'open to everyone'))
        for lane in [18, 19]:
            walk(c, pack, (39, lane))
            travel(c, 'right', 'MnR01C00R00', (0, lane + 6))
            travel(c, 'left', CITY, (39, lane))
    assert c.status()['badges'] == 1
    c.press('start')
    assert 'PACK' in json.dumps(c.observe()), 'production Start menu'
    c.press('b')
    return pages


def session(args, pack, save, output, first):
    output.mkdir(exist_ok=False)
    actual = output / 'actual.crystalsave'
    records, pages, error = [], [], None
    with (output / 'stderr.log').open('x') as stderr:
        process = subprocess.Popen(
            [str(args.client), 'mcp', str(args.pack), '--load', str(save), '--save', str(actual)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
            text=True, bufsize=1, env={**os.environ, 'RUST_MIN_STACK': '67108864'})
        c = LiveRecorded(process, records)
        try:
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                    clientInfo=dict(name='rochester-care-green', version='1')))
            pages = course(c, pack, first)
            (output / 'final-observe.json').write_text(json.dumps(c.observe(), indent=2) + '\n')
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', error=error, records=records,
                  rpc_calls=len(records), pages=pages, actual_save=str(actual),
                  stderr_bytes=(output / 'stderr.log').stat().st_size)
    (output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    assert not error and not result['stderr_bytes'], error
    return result


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'save', 'client', 'output']:
        cli.add_argument('--' + name, type=Path, required=True)
    args = cli.parse_args()
    for name in ['pack', 'save', 'client']:
        setattr(args, name, getattr(args, name).resolve())
    assert sha(args.pack) == PACK_SHA256, 'reviewed Rochester bytes required'
    inputs = {str(path): sha(path) for path in [args.pack, args.save, args.client]}
    args.output.mkdir(exist_ok=False, parents=True)
    pack = load_pack(args.pack)
    results = [session(args, pack, args.save, args.output / 'play', True)]
    results.append(session(args, pack, Path(results[0]['actual_save']),
                           args.output / 'continue', False))
    assert all(sha(Path(path)) == value for path, value in inputs.items())
    report = dict(status='PASS', inputs=inputs, inputs_unchanged=True,
                  sessions=len(results), rpc_calls=sum(row['rpc_calls'] for row in results),
                  results=results,
                  scope='Continue an actual earned Care Badge save, then use only production '
                        'input for signs, guide, relocated doors, clinic return, garden, both '
                        'first-route lanes and Start across two fresh processes. No state grants '
                        'or saved-terrain migration; full campaign acceptance remains separate.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS', report['sessions'], 'sessions', report['rpc_calls'], 'calls', flush=True)


if __name__ == '__main__':
    main()
