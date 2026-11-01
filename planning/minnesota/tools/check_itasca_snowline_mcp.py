#!/usr/bin/env python3
"""Play the complete Itasca snowline from an actual retained controller save.

Use the reached Cut save from the native Headwaters controller regression.
Party, six prior badges and original position began as explicit local setup;
this course never edits game state after that test's production inputs.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from check_inland_grass_cut_mcp import sha
from check_lake_country_mcp import escape_wild
from check_mina_battle_mcp import LiveRecorded
from check_river_story_mcp import road
from check_winona_mcp import travel, walk as town_walk
from render_reference import load_pack


def session(args, pack, source, folder):
    folder.mkdir(exist_ok=False)
    original_hash = sha(source)
    actual, records, error, wild, hops = folder / 'actual.crystalsave', [], None, [], []
    with (folder / 'stderr.log').open('x') as err:
        proc = subprocess.Popen([str(args.client), 'mcp', str(args.pack), '--load', str(source), '--save', str(actual)],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=err, text=True,
                                env={**os.environ, 'RUST_MIN_STACK': '67108864'}, bufsize=1)
        c = LiveRecorded(proc, records)
        try:
            c.rpc('initialize', dict(protocolVersion='2025-06-18', capabilities={},
                                    clientInfo=dict(name='native-itasca-snowline', version='1')))
            initial = c.status()
            assert initial['map'].startswith('MnR14')
            def flee(client):
                wild.append(client.status())
                escape_wild(client)
            def walk(at):
                road(c, pack, at, prefix='MnR14', rows=2, columns=2, encounter=flee)
            walk((15, 24))
            for lane in range(28, 32):
                walk((lane, 23))
                lip_seen = False
                for _ in range(8):
                    c.move('down')
                    state = c.status()
                    assert state['mode'] == 'overworld', state
                    lip_seen |= state['position'] == dict(x=lane, y=24)
                    if state['position'] == dict(x=lane, y=26):
                        break
                c.at('MnR14C00R00', lane, 26)
                for _ in range(4):
                    c.move('up')
                    c.at('MnR14C00R00', lane, 26)
                walk((lane, 23))
                hops.append(dict(lane=lane, native_drop_to_26=True, lip_observed=lip_seen,
                                 reverse_refusals=4, ordinary_grassy_return=True))
            for target in [(15, 56), (88, 56), (92, 26), (95, 26)]:
                walk(target)
            travel(c, 'right', 'MnBemidji', (0, 24))
            travel(c, 'left', 'MnR14C01R00', (31, 26))
            walk((6, 42))
            travel(c, 'left', 'MnItascaGatehouse', (8, 4))
            town_walk(c, pack, (2, 4))
            travel(c, 'left', 'MnLakeItasca', (42, 24))
            travel(c, 'right', 'MnItascaGatehouse', (1, 4))
            town_walk(c, pack, (7, 4))
            travel(c, 'right', 'MnR14C00R00', (5, 42))
            c.move('right'); c.at('MnR14C00R00', 6, 42)
            walk((15, 24))
            c.press('start')
            assert 'PACK' in json.dumps(c.observe())
            c.press('b')
            final = c.status()
            assert final['money'] == initial['money'] and final['badges'] == initial['badges']
            (folder / 'final-observe.json').write_text(json.dumps(c.observe(), indent=2) + '\n')
            proc.stdin.close()
            assert proc.wait(timeout=30) == 0
        except BaseException as exc:
            error = repr(exc)
        finally:
            if proc.poll() is None:
                proc.terminate(); proc.wait(timeout=30)
    result = dict(status='FAIL' if error else 'PASS', error=error, initial_save=str(source), actual_save=str(actual),
                  rpc_calls=len(records), records=records, wild=wild, hops=hops,
                  initial_unchanged=sha(source) == original_hash, stderr_bytes=(folder / 'stderr.log').stat().st_size)
    (folder / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    assert not error and result['initial_unchanged'] and not result['stderr_bytes'], error
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'save', 'client', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.client = args.client.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = {str(p): sha(p) for p in [args.pack, args.save, args.client]}
    world = Path('planning/minnesota/world.json').read_bytes()
    pack = load_pack(args.pack)
    results = []
    for label in ['play', 'continue']:
        source = args.save if not results else Path(results[-1]['actual_save'])
        result = session(args, pack, source, args.output / label)
        results.append(result)
        print(label, 'PASS', result['rpc_calls'], flush=True)
    assert all(sha(Path(p)) == value for p, value in inputs.items())
    assert world == Path('planning/minnesota/world.json').read_bytes()
    (args.output / 'receipt.json').write_text(json.dumps(dict(status='PASS', inputs=inputs,
        inputs_unchanged=True, world_unchanged=True, sessions=len(results),
        rpc_calls=sum(r['rpc_calls'] for r in results), results=results,
        scope='Actual saved native-controller Cut checkpoint with explicit original local six-badge/Minnesota party setup. Production input covers all chunks, four one-way drops/reverse refusals/grass returns, actual Bemidji and Itasca gatehouse/town pairs, wild Run, Start and Continue. No state rewrites, Surf or earned campaign claim.'), indent=2) + '\n')


if __name__ == '__main__':
    main()
