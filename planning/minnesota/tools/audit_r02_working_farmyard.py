#!/usr/bin/env python3
"""Independently constrain the complete farmyard publication to its owned delta."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack
from review_r02_working_farmyard import compose


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'candidate', 'source', 'output']:
        cli.add_argument('--' + name, type=Path, required=True)
    args = cli.parse_args()
    inputs = {str(p): sha(p) for p in [args.parent, args.candidate, args.source]}
    source = json.loads(args.source.read_text())
    assert sha(args.source) == '1348ff5a00f57b5b2add40945585f68fdf4c1e4c2cda638e840c4be1cf4cc875'
    assert sha(args.parent) == source['parent_pack_sha256']
    base, child = load_pack(args.parent), load_pack(args.candidate)
    original, preview, geometry = compose(base, source)
    name = source['map']
    module = child['data']['maps'][name]
    labels = set(module['scripts']) - set(original['scripts'])
    expected_labels = {'MN_R02_GARDEN_GROWER', 'MnR02GardenFull', 'MnR02GardenRepeat',
                       'MnR02GardenOfferText', 'MnR02GardenFullText',
                       'MnR02GardenReceivedText', 'MnR02GardenRepeatText'}
    assert labels == expected_labels
    assert module['blocks'] == preview['blocks']
    actor, = module['objects'][len(original['objects']):]
    assert (actor['x'], actor['y']) == tuple(source['new_actor']['tile'])
    assert actor['sprite'] == source['new_actor']['sprite']
    assert actor['script'] == actor['object_identifier'] == source['new_actor']['id']
    assert actor['spritemovedata'] == 'SPRITEMOVEDATA_STANDING_DOWN'
    assert actor['object_type'] == 'OBJECTTYPE_SCRIPT' and actor['event_flag'] == '-1'
    proposed = deepcopy(original)
    proposed['blocks'] = preview['blocks']
    proposed['objects'].append(actor)
    for key, value in original.items():
        if key == 'scripts' or key == 'script_text_bodies':
            assert all(module[key].get(k) == v for k, v in value.items()), ('old script changed', key)
            assert set(module[key]) - set(value) <= labels
            proposed[key].update({k: v for k, v in module[key].items() if k in labels})
        elif key.startswith('script_') and isinstance(value, list):
            assert module[key][:len(value)] == value, ('old command metadata changed', key)
            additions = module[key][len(value):]
            assert all(row['source_script'] in labels for row in additions), ('unowned command', key)
            proposed[key].extend(additions)
    old_wire = original['map_event_section_commands']
    wire = module['map_event_section_commands']
    assert wire[:-1] == old_wire and wire[-1]['command'] == 'object_event'
    assert wire[-1]['args'][:3] == ['9', '19', 'SPRITE_GRANNY']
    assert 'MN_R02_GARDEN_GROWER' in wire[-1]['args']
    proposed['map_event_section_commands'] = wire
    assert module == proposed, 'unowned map data differs'
    root = deepcopy(base)
    data = root['data']
    data['maps'][name] = proposed
    data['npcs'][name] = module['objects']
    data['map_blocks'][module['attributes']['blocks_label']] = base64.b64encode(bytes(module['blocks'])).decode()
    event_label = module['attributes']['map_events_label']
    data['map_scripts'][event_label] = [dict(command=r['command'], args=r['args']) for r in wire]
    for label in labels:
        assert child['data']['map_scripts'][label] == module['scripts'][label]
        data['map_scripts'][label] = module['scripts'][label]
    flag = source['reward']['event_flag']
    constants = data['story_event_script_constants']['global']
    assert flag not in constants
    constants[flag] = max(v for k, v in constants.items() if k.startswith('EVENT_')) + 1
    marker = 'minnesota-r02-working-farmyard-v1'
    root['report']['manifests'].append(marker)
    root['identity']['runtime_modpack_id'] += '+' + marker
    root['identity']['content_hash'] = child['identity']['content_hash']
    assert root == child, 'unowned complete-pack content changed'
    assert all(sha(Path(p)) == digest for p, digest in inputs.items())
    args.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(status='PASS_COMPLETE_PRESERVATION', inputs=inputs,
                   candidate_content_hash=child['identity']['content_hash'], changed_metatiles=17,
                   new_actors=1, new_claim_flags=1, original_actors_and_programs_exact=True,
                   complete_other_maps_preserved=len(base['data']['maps'])-1, geometry=geometry,
                   scope='Entire root is exact outside seventeen metatiles, one new grower, its seven programs/typed metadata, one event flag and required native mirrors/identity. Original parties, wild/fishing tables, services, attributes, assets and audio are exact. Reward execution requires separate controller evidence.')
    (args.output/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
