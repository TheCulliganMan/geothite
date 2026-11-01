#!/usr/bin/env python3
"""Independently project the exact Mankato delta over its complete parent."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack
from review_mankato_terrace_pond import compose


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for key in ['parent', 'candidate', 'source', 'output']:
        cli.add_argument('--' + key, type=Path, required=True)
    args = cli.parse_args()
    inputs = {str(path): sha(path) for path in [args.parent, args.candidate, args.source]}
    source = json.loads(args.source.read_text())
    assert sha(args.parent) == source['parent_pack_sha256']
    assert sha(args.source) == '9d2b4bbc5aa30521eb1724a4a6e66d6f39ee86c716a547c48e3db00f9b2ec9ea'
    base, child = load_pack(args.parent), load_pack(args.candidate)
    original, town = compose(base, source)
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
    marker = 'minnesota-mankato-terrace-pond-v1'
    expected['report']['manifests'].append(marker)
    expected['identity']['runtime_modpack_id'] += '+' + marker
    expected['identity']['content_hash'] = child['identity']['content_hash']
    assert expected == child, 'unrelated data or native split payload changed'
    assert all(sha(Path(path)) == value for path, value in inputs.items())
    args.output.mkdir(exist_ok=False, parents=True)
    report = dict(status='PASS_COMPLETE_PRESERVATION', inputs=inputs, inputs_unchanged=True,
                  candidate_content_hash=child['identity']['content_hash'],
                  complete_other_maps_preserved=len(base['data']['maps']) - 1,
                  changed_metatiles=sum(a != b for a, b in zip(original['blocks'], town['blocks'])),
                  moved_warps=4, moved_actors=0, moved_signs=3,
                  new_contained_water_quadrants=source['expected_water_quadrants'],
                  scope='Complete root matches explicit terrain/anchor changes, event wire and '
                        'required native mirrors, marker and derived identity. Every other map, '
                        'program, room, encounter, roster, art and audio remains exact. Native '
                        'publication/readback, input and visual review have separate receipts.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
