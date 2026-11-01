#!/usr/bin/env python3
"""Independently compare the Rochester child with its complete source/parent."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack
from review_rochester_care_green import compose


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--parent', type=Path, required=True)
    cli.add_argument('--candidate', type=Path, required=True)
    cli.add_argument('--source', type=Path, required=True)
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    inputs = {str(p): sha(p) for p in [args.parent, args.candidate, args.source]}
    source = json.loads(args.source.read_text())
    assert inputs[str(args.parent)] == source['parent_pack_sha256']
    assert inputs[str(args.source)] == '3a58e14c9bd788e6b350c275966437b01e55199ff81d1564162bbb927e4a1296'
    base, child = load_pack(args.parent), load_pack(args.candidate)
    original, town, trips = compose(base, source)
    # Independently project only the corresponding x/y arguments into the
    # original event wire. All command names, tails and ordering must survive.
    wire = deepcopy(original['map_event_section_commands'])
    groups = {'warp_event': town['events']['warps'],
              'bg_event': town['events']['bg_events'], 'object_event': town['objects']}
    indices = {name: 0 for name in groups}
    for row in wire:
        command = row['command']
        if command in groups:
            record = groups[command][indices[command]]
            row['args'][:2] = [str(record['x']), str(record['y'])]
            indices[command] += 1
    assert all(indices[key] == len(records) for key, records in groups.items())
    town['map_event_section_commands'] = wire
    expected = deepcopy(base)
    data = expected['data']
    data['maps'][town['id']] = town
    data['npcs'][town['id']] = town['objects']
    data['map_blocks'][town['attributes']['blocks_label']] = base64.b64encode(bytes(town['blocks'])).decode()
    data['map_scripts'][town['attributes']['map_events_label']] = [
        dict(command=row['command'], args=row['args']) for row in wire]
    marker = 'minnesota-rochester-care-green-v1'
    expected['report']['manifests'].append(marker)
    expected['identity']['runtime_modpack_id'] += '+' + marker
    expected['identity']['content_hash'] = child['identity']['content_hash']
    assert expected == child, 'unrelated root data or native split payload differs'
    assert all(sha(Path(path)) == value for path, value in inputs.items())
    args.output.mkdir(exist_ok=False, parents=True)
    report = dict(status='PASS_COMPLETE_PRESERVATION', inputs=inputs, inputs_unchanged=True,
                  candidate_content_hash=child['identity']['content_hash'],
                  complete_other_maps_preserved=len(base['data']['maps']) - 1,
                  changed_metatiles=sum(a != b for a, b in zip(original['blocks'], town['blocks'])),
                  warp_positions=4, actor_positions=1, sign_positions=3,
                  travel_comparisons=trips,
                  scope='Complete root matches exact authored rows and anchor positions, event '
                        'wire, NPC/block mirrors, manifest and derived identity. Every other map, '
                        'program, room, flag, encounter, roster, art and audio remains exact. '
                        'Content hash/runtime validity is checked by Rust publication/readback; '
                        'native input and visual acceptance are separate.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
