#!/usr/bin/env python3
"""Exercise major-city planning bindings, including rejected metadata drift.

This is read-only source/native metadata verification, not controller gameplay.
Corrupted fixtures exist only in memory and never change a pack or source file.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from check_city_expansion_plan import validate
from render_reference import load_pack


ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads(path.read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    pack = load_pack(args.pack)
    world = read(ROOT / 'world.json')
    registry = {room['interior']: room for room in read(ROOT / 'interiors.json')['contracts']}
    results = []
    for town in world['towns']:
        if not town.get('expansion_contract_file'):
            continue
        city = town['id']
        authored = read(ROOT / town['authoring_file'])
        plan = read(ROOT / town['expansion_contract_file'])
        room_names, links = validate(ROOT, town, authored, registry, world['connections'], pack)
        results.append(dict(city=city, check='unchanged source/native indexed bindings', passed=True,
                            new_rooms=len(room_names), reciprocal_roads=len(links)))
        name = next(name for name in room_names if registry[name]['parent'] != city)
        facade = plan['doors'][0]['target']
        changes = [
            ('stale upper-floor entry', lambda t, a, r, c: r[name]['paired_bindings'][0]['parent_entry_core'].__setitem__(0, -1)),
            ('stale upper-floor return', lambda t, a, r, c: r[name]['paired_bindings'][0]['parent_safe_returns_core'][0].__setitem__(0, -1)),
            ('wrong indexed arrival', lambda t, a, r, c: r[name]['paired_bindings'][0].__setitem__('arrival_warp_index', 255)),
            ('missing upper-floor pair', lambda t, a, r, c: r[name].__setitem__('paired_bindings', [])),
            ('wrong room source', lambda t, a, r, c: r[name].__setitem__('source_file', 'stale.json')),
            ('missing room reservation', lambda t, a, r, c: t.__setitem__('interiors', [room for room in t['interiors'] if room['id'] != name])),
            ('stale facade target index', lambda t, a, r, c: next(door for door in a['doors'] if door['target'] == facade).__setitem__('target_warp', 255)),
            ('wrong road offset', lambda t, a, r, c: next(link for link in c if (link['a'], link['b']) in links).__setitem__('offset_a', 999)),
            ('wrong road lane', lambda t, a, r, c: next(link for link in c if (link['a'], link['b']) in links)['a_edge_core'][0].__setitem__(0, -1)),
        ]
        for label, corrupt in changes:
            t, a, r, c = deepcopy((town, authored, registry, world['connections']))
            corrupt(t, a, r, c)
            try:
                validate(ROOT, t, a, r, c, pack)
            except (AssertionError, KeyError, StopIteration) as error:
                results.append(dict(city=city, check='reject '+label, passed=True, reason=str(error) or type(error).__name__))
            else:
                raise AssertionError(city+': accepted '+label)
        original = pack['data']['maps'][name]
        native_changes = [
            ('native room terrain drift', lambda m: m['blocks'].__setitem__(0, (m['blocks'][0]+1)%256)),
            ('native extra warp', lambda m: m['events']['warps'].append(deepcopy(m['events']['warps'][0]))),
            ('native wrong destination', lambda m: m['events']['warps'][0].__setitem__('target_warp_id', 255)),
            ('native wrong extent', lambda m: m['attributes'].__setitem__('width', 255)),
        ]
        for label, corrupt in native_changes:
            pack['data']['maps'][name] = deepcopy(original)
            corrupt(pack['data']['maps'][name])
            try:
                validate(ROOT, town, authored, registry, world['connections'], pack)
            except (AssertionError, KeyError, StopIteration) as error:
                results.append(dict(city=city, check='reject '+label, passed=True, reason=str(error) or type(error).__name__))
            else:
                raise AssertionError(city+': accepted '+label)
            finally:
                pack['data']['maps'][name] = original
    assert len(results) == 42 and all(result['passed'] for result in results)
    raw = (json.dumps(dict(scope=__doc__, pack=str(args.pack), pack_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                           results=results, checks=len(results)), indent=2)+'\n').encode()
    args.output.mkdir(parents=True, exist_ok=False)
    output = args.output / ('proof-'+hashlib.sha256(raw).hexdigest()+'.json')
    output.write_bytes(raw)
    print(f'{len(results)} planning/native checks passed, including 39 rejected corruptions: {output}')


if __name__ == '__main__':
    main()
