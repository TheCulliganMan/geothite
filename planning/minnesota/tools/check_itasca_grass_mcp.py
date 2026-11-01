#!/usr/bin/env python3
"""Exercise native cold/melt grass Cut and Continue against a green control."""
import argparse
import json
from pathlib import Path

from check_inland_grass_cut_mcp import find_cases, session, sha
from render_reference import load_pack


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
    pack = load_pack(args.pack)
    cases = find_cases(pack, {'rules': [
        {'tileset': 'mn_itasca_thaw', 'grass_block': block} for block in ['88', 'b1']
    ] + [{'tileset': 'mn_winter_itasca_trail', 'grass_block': '03'}]}, expected_cases=4)
    (args.output / 'cases.json').write_text(json.dumps(cases, indent=2) + '\n')
    results = []
    for case in cases:
        for mode in ['no-cut', 'no-hive', 'cut-and-hive']:
            folder = args.output / (case['id'] + '-' + mode)
            result = session(args, case, mode, folder, None, create=True)
            results.append(result)
            print(case['id'], mode, 'PASS', flush=True)
            if mode == 'cut-and-hive':
                folder = args.output / (case['id'] + '-continue')
                result = session(args, case, 'continue', folder, Path(result['actual_save']))
                results.append(result)
                print(case['id'], 'continue PASS', flush=True)
    assert all(sha(Path(p)) == value for p, value in inputs.items())
    assert world == Path('planning/minnesota/world.json').read_bytes()
    (args.output / 'receipt.json').write_text(json.dumps(dict(status='PASS', inputs=inputs,
        inputs_unchanged=True, world_unchanged=True, sessions=len(results),
        rpc_calls=sum(r['rpc_calls'] for r in results), results=results,
        scope='Explicit local Minnesota partner/Cut/Hive/repel/position setup. Actual party-menu Cut, move/badge refusals, walking, saved overrides, native regrowth and re-cut after Continue. Original green control included. No state changes after input or earned campaign claim.'), indent=2) + '\n')


if __name__ == '__main__':
    main()
