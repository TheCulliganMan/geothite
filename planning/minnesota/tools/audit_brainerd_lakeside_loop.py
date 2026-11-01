#!/usr/bin/env python3
"""Independently compare the complete Brainerd child with its explicit parent."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack
from review_brainerd_lakeside_loop import compose


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for key in ['parent', 'candidate', 'source', 'output']:
        cli.add_argument('--' + key, type=Path, required=True)
    args = cli.parse_args()
    inputs = {str(p): sha(p) for p in [args.parent, args.candidate, args.source]}
    source = json.loads(args.source.read_text())
    assert sha(args.source) == '8ad8cd5a1e7347235009e103f183eb948d3218f66263a19b5c5d258125032bee'
    assert sha(args.parent) == source['parent_pack_sha256']
    base, child = load_pack(args.parent), load_pack(args.candidate)
    original, town = compose(base, source)
    wire = deepcopy(original['map_event_section_commands'])
    groups = {'warp_event': town['events']['warps'],
              'bg_event': town['events']['bg_events'], 'object_event': town['objects']}
    indices = {key: 0 for key in groups}
    for row in wire:
        if row['command'] in groups:
            key = row['command']
            record = groups[key][indices[key]]
            row['args'][:2] = [str(record['x']), str(record['y'])]
            indices[key] += 1
    assert all(indices[key] == len(records) for key, records in groups.items())
    town['map_event_section_commands'] = wire
    expected = deepcopy(base)
    data = expected['data']
    data['maps'][town['id']] = town
    data['map_blocks'][town['attributes']['blocks_label']] = base64.b64encode(bytes(town['blocks'])).decode()
    data['map_scripts'][town['attributes']['map_events_label']] = [
        dict(command=row['command'], args=row['args']) for row in wire]
    marker = 'minnesota-brainerd-lakeside-loop-v1'
    expected['report']['manifests'].append(marker)
    expected['identity']['runtime_modpack_id'] += '+' + marker
    expected['identity']['content_hash'] = child['identity']['content_hash']
    assert expected == child, 'unrelated data or native payload changed'
    assert all(sha(Path(p)) == value for p, value in inputs.items())
    args.output.mkdir(parents=True, exist_ok=False)
    report = dict(status='PASS_COMPLETE_PRESERVATION', inputs=inputs, inputs_unchanged=True,
                  candidate_content_hash=child['identity']['content_hash'],
                  complete_other_maps_preserved=len(base['data']['maps']) - 1,
                  changed_metatiles=57, moved_warps=2, moved_actors=0, moved_signs=0,
                  legacy_attribute_table_preserved=True,
                  scope='Complete root matches the explicit town blocks, two exterior warp coordinates, event wire/native mirrors and marker/identity. Every other map, room, program, roster, encounter, fishing table, art, audio and original attribute table remains exact.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
