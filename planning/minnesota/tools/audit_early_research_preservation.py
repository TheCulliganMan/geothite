#!/usr/bin/env python3
"""Audit the three research-service maps against their complete campaign parent."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack

parser = argparse.ArgumentParser(description=__doc__)
for name in ('parent', 'child', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
assert not args.output.exists(), 'retain every audit'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
parent_sha, child_sha = sha(args.parent), sha(args.child)
p, q = load_pack(args.parent), load_pack(args.child)
a, b = p['data'], q['data']
rooms = {'MnWinonaFieldStation', 'MnR01C00R00', 'MnRochesterFossilLab'}
replaced = {
    'MnKindlitResearchAide', 'MnWTamarackDeliveryReturnText',
    'MnFossilExchange', 'MnFossilExchangeText', 'MnFossilDirectionsText',
    'MnRochesterFossilLabAssistant', 'MnRochesterFossilLabAssistantText',
    'MnLabFossilRestoredText',
}
flags = {'EVENT_MN_POUCHID_RESEARCH_RECEIVED', 'EVENT_MN_BIRCHIT_RESEARCH_RECEIVED'}
assert a.keys() == b.keys()
for section in a:
    if section not in {'maps', 'map_scripts', 'story_event_script_constants'}:
        assert a[section] == b[section], section
old_flags = a['story_event_script_constants']['global']
new_flags = b['story_event_script_constants']['global']
assert set(new_flags) - set(old_flags) == flags
assert all(new_flags.get(k) == v for k, v in old_flags.items())
constants = deepcopy(b['story_event_script_constants'])
for flag in flags:
    del constants['global'][flag]
assert constants == a['story_event_script_constants']
assert a['maps'].keys() == b['maps'].keys()
script_fields = {
    'scripts', 'script_item_grants', 'gift_pokemon_scripts',
    'script_flag_commands', 'script_text_commands', 'script_text_bodies',
    'script_variable_commands', 'script_control_commands',
    'script_runtime_commands', 'script_object_commands',
}
changed_labels = set()
for name, before in a['maps'].items():
    after = b['maps'][name]
    if name not in rooms:
        assert before == after, name
        continue
    assert before.keys() == after.keys()
    for section in before:
        if section not in script_fields:
            assert before[section] == after[section], (name, section)
    for label, body in before['scripts'].items():
        if label not in replaced:
            assert after['scripts'].get(label) == body, (name, label)
    changed_labels.update(set(after['scripts']) - set(before['scripts']))
    changed_labels.update(replaced & set(before['scripts']))
    old_objects = [v for v in before['script_object_commands'] if v['source_script'] != 'MnRochesterFossilLabAssistant']
    new_objects = [v for v in after['script_object_commands'] if v['source_script'] != 'MnRochesterFossilLabAssistant']
    assert old_objects == new_objects, (name, 'unrelated object commands')
assert set(b['map_scripts']) - set(a['map_scripts']) <= changed_labels
for label, body in a['map_scripts'].items():
    if label not in changed_labels:
        assert b['map_scripts'].get(label) == body, ('global script', label)
for section in ('runtime_files', 'audio_manifest', 'compiled_audio'):
    assert p[section] == q[section], section
for key, value in p['report'].items():
    expected = value + ['minnesota-early-research-access-v1'] if key == 'manifests' else value
    assert q['report'][key] == expected, ('report', key)
assert sha(args.parent) == parent_sha and sha(args.child) == child_sha
receipt = dict(status='PASS', parent=str(args.parent), parent_sha256=parent_sha,
               pack=str(args.child), pack_sha256=child_sha,
               content_identity=q['identity']['content_hash'],
               changed_maps=sorted(rooms), unchanged_complete_maps=len(a['maps']) - 3,
               original_terrain_actors_encounters_assets_audio_and_unrelated_programs_exact=True,
               scope='Preservation audit; native transactions and earned access require separate controller evidence.')
args.output.write_text(json.dumps(receipt, indent=2) + '\n')
print('PASS', receipt['unchanged_complete_maps'], 'other complete maps; research service preservation')
