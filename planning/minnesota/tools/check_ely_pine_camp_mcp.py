#!/usr/bin/env python3
"""Walk the integrated Ely pine town from a retained native initial checkpoint.

The controller fixture declares one injured Minnesota partner and the initial
town position. All subsequent movement, collection and Continue use production
stdio input; the course never rewrites position, party, flags or inventory.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import sha
from check_mina_battle_mcp import LiveRecorded
from check_winona_mcp import finish, travel, walk
from render_reference import load_pack


def session(args, pack, source, output, first):
    output.mkdir(exist_ok=False)
    actual, records, pages, error = output / 'actual.crystalsave', [], [], None
    with (output / 'stderr.log').open('x') as stderr:
        process = subprocess.Popen(
            [str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
            text=True, bufsize=1, env={**os.environ, 'RUST_MIN_STACK': '67108864'})
        c = LiveRecorded(process, records)
        try:
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                    clientInfo=dict(name='ely-pine-camp', version='3')))
            if first:
                c.at('MnEly', 24, 2)
                for approach, direction, words in [((21, 4), 'down', 'pines'),
                                                   ((33, 10), 'left', 'supply nook'),
                                                   ((29, 17), 'up', 'FULL HEAL')]:
                    walk(c, pack, approach)
                    c.press(direction); c.press('a')
                    text = finish(c)
                    pages.append(text)
                    assert words in text, text
                # The original sign is walkable: face it from the lower floor.
                walk(c, pack, (20, 5))
                c.move('up'); c.at('MnEly', 20, 4)
                c.press('a')
                text = finish(c)
                pages.append(text)
                assert 'eight badges' in text.lower(), text
                walk(c, pack, (34, 5))
                c.press('right'); c.press('a')
                text = finish(c)
                pages.append(text)
                assert 'FULL RESTORE' in ' '.join(text.upper().split()), text
            else:
                c.at('MnEly', 34, 5)
                c.move('right'); c.at('MnEly', 35, 5)
                c.move('left'); c.at('MnEly', 34, 5)
                for y in [14, 15]:
                    walk(c, pack, (39, y))
                    travel(c, 'right', 'MnElyShorePass', (0, y))
                    travel(c, 'left', 'MnEly', (39, y))
                for x in [24, 25]:
                    walk(c, pack, (x, 0))
                    travel(c, 'up', 'MnR13C00R02', (x + 10, 31))
                    travel(c, 'down', 'MnEly', (x, 0))
                for point, room, arrival in [((11, 22), 'MnElyPokecenter1F', (3, 5)),
                                              ((27, 28), 'MnElyMart', (2, 5))]:
                    walk(c, pack, point)
                    travel(c, 'up', room, arrival)
                    walk(c, pack, (3, 6) if room.endswith('1F') else (2, 6))
                    travel(c, 'down', 'MnEly', point)
                walk(c, pack, (25, 14))
                travel(c, 'up', 'MnLeagueLobby', (1, 4))
                travel(c, 'left', 'MnEly', (25, 14))
                c.press('start')
                assert 'PACK' in json.dumps(c.observe())
                c.press('b')
            (output / 'final-observe.json').write_text(json.dumps(c.observe(), indent=2) + '\n')
            process.stdin.close()
            assert process.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if process.poll() is None:
                process.terminate(); process.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', error=error, records=records,
                  rpc_calls=len(records), pages=pages, actual_save=str(actual),
                  stderr_bytes=(output / 'stderr.log').stat().st_size)
    (output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    assert not error and not result['stderr_bytes'], error
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'save', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    for key in ['pack', 'save', 'client']:
        setattr(args, key, getattr(args, key).resolve())
    args.output.mkdir(exist_ok=False, parents=True)
    world = Path('planning/minnesota/world.json')
    inputs = {str(p): sha(p) for p in [args.pack, args.save, args.client, world]}
    pack = load_pack(args.pack)
    results = [session(args, pack, args.save, args.output / 'play', True)]
    results.append(session(args, pack, Path(results[0]['actual_save']), args.output / 'continue', False))
    assert all(sha(Path(p)) == digest for p, digest in inputs.items())
    report = dict(status='PASS', inputs=inputs, inputs_unchanged=True, sessions=len(results),
                  rpc_calls=sum(r['rpc_calls'] for r in results), results=results,
                  scope='Declared local initial state, then production conversations, supply, once-only claim persistence, four regional seam lanes, three building returns, Start and Continue. Native controller separately proves medicine effects, capacity retry and free healing; no earned campaign claim.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS', report['sessions'], 'sessions', report['rpc_calls'], 'calls', flush=True)


if __name__ == '__main__':
    main()
