#!/usr/bin/env python3
"""Read-only complete-parent preservation audit for the native spring grotto."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from render_reference import load_pack

cli = argparse.ArgumentParser(description=__doc__)
for name in ['parent', 'child', 'source', 'output']:
    cli.add_argument('--' + name, type=Path, required=True)
a = cli.parse_args()
assert not a.output.exists(), 'retain every audit'
p, q = load_pack(a.parent), load_pack(a.child)
old, new = p['data'], q['data']
room, hidden, flag = 'MnR12Cave1F', 'MN_R12_GROTTO_REVIVE', 'EVENT_MN_R12_GROTTO_REVIVE'
source = json.loads(a.source.read_text())
allowed = {'maps', 'map_scripts', 'map_attributes', 'map_dimensions', 'map_blocks',
           'runtime_map_metadata', 'story_event_script_constants', 'npcs'}
assert old.keys() == new.keys()
for section in old:
    if section not in allowed:
        assert old[section] == new[section], section
assert old['maps'].keys() == new['maps'].keys()
for name, module in old['maps'].items():
    if name != room:
        assert new['maps'][name] == module, name
before, after = old['maps'][room], new['maps'][room]
assert before.keys() == after.keys()
changed = {'attributes', 'scripts', 'script_field_pickups', 'map_event_section_commands',
           'events', 'objects', 'blocks'}
for section in before:
    if section not in changed:
        assert after[section] == before[section], section
attributes = deepcopy(before['attributes'])
attributes.update(width=20, height=14)
assert after['attributes'] == attributes
assert after['blocks'] == [int(v, 16) for row in source['rows'] for v in row.split()]
objects = deepcopy(before['objects'])
objects[1].update(x=35, y=3)
assert after['objects'] == objects
events = deepcopy(before['events'])
events['bg_events'].append(dict(x=28, y=16, event_type='BGEVENT_ITEM', script=hidden))
assert after['events'] == events
assert set(after['scripts']) - set(before['scripts']) == {hidden}
for label, program in before['scripts'].items():
    assert after['scripts'][label] == program, label
assert after['script_field_pickups'][:len(before['script_field_pickups'])] == before['script_field_pickups']
assert len(after['script_field_pickups']) == len(before['script_field_pickups']) + 1
pickup = after['script_field_pickups'][-1]
assert pickup['command'] == 'hiddenitem' and pickup['item_id'] == 'REVIVE'
assert pickup['event_flag'] == flag and pickup['quantity'] == 1
for section, exceptions in [
    ('map_scripts', {room + '_MapEvents', hidden}),
    ('map_attributes', {room}), ('map_dimensions', {'MN_R12_CAVE_1F'}),
    ('map_blocks', {room + '_Blocks'}), ('runtime_map_metadata', {'MN_R12_CAVE_1F'}),
    ('npcs', {room}), ('story_event_script_constants', {'global'}),
]:
    assert set(new[section]) - set(old[section]) == ({hidden} if section == 'map_scripts' else set())
    for label, original in old[section].items():
        if label not in exceptions:
            assert new[section][label] == original, (section, label)
constants = old['story_event_script_constants']['global']
current = new['story_event_script_constants']['global']
assert set(current) - set(constants) == {flag}
assert all(current[label] == value for label, value in constants.items())
assert p['runtime_files'] == q['runtime_files']
assert p['audio_manifest'] == q['audio_manifest'] and p['compiled_audio'] == q['compiled_audio']
for name, original in p['report'].items():
    expected = original + ['minnesota-gunflint-spring-grotto-v1'] if name == 'manifests' else original
    assert q['report'][name] == expected, name
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
receipt = dict(status='PASS', parent=str(a.parent), parent_sha256=sha(a.parent),
               pack=str(a.child), pack_sha256=sha(a.child), content_identity=q['identity']['content_hash'],
               changed_map=room, unchanged_complete_maps=len(old['maps'])-1,
               source_sha256=sha(a.source), native_art_audio_catalogs_wildlife_and_original_programs_exact=True,
               original_mouth_otta_and_cache_claim_flag_preserved=True)
a.output.write_text(json.dumps(receipt, indent=2) + '\n')
print('PASS', receipt['unchanged_complete_maps'], 'other complete maps; original art/audio/catalogs/programs/wildlife exact')
