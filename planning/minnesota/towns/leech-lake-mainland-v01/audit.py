#!/usr/bin/env python3
"""Read-only full-parent and whole-town preservation audit for the pine trail."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from render_reference import load_pack

cli = argparse.ArgumentParser(description=__doc__)
for key in ['parent', 'child', 'source', 'output']:
    cli.add_argument('--' + key, type=Path, required=True)
a = cli.parse_args()
assert not a.output.exists()
p, q = load_pack(a.parent), load_pack(a.child)
before, after = p['data'], q['data']
name = 'MnLeechLake'
trainer = 'MN_EXTRA_LEECH_MAINLAND_FLY_TIER'
item = 'MN_LEECH_MAINLAND_REPEL'
hint = 'MN_LEECH_SHORE_WALKERText'
source = json.loads(a.source.read_text())
assert before.keys() == after.keys()
changed = {'maps', 'map_scripts', 'map_blocks', 'npcs', 'wild_encounters',
           'trainers', 'story_event_script_constants'}
for key in before:
    if key not in changed:
        assert after[key] == before[key], key
assert before['maps'].keys() == after['maps'].keys()
for key, value in before['maps'].items():
    if key != name:
        assert after['maps'][key] == value, key
old, new = before['maps'][name], after['maps'][name]
assert old.keys() == new.keys()
allowed = {'blocks', 'objects', 'scripts', 'map_event_section_commands'}
allowed.update(key for key in old if key.startswith('script_'))
allowed.update({'trainer_scripts', 'scripted_trainer_battles', 'gift_pokemon_scripts'})
for key, value in old.items():
    if key not in allowed:
        assert new[key] == value, key
assert new['objects'][:len(old['objects'])] == old['objects']
assert [o['script'] for o in new['objects'][len(old['objects']):]] == [item, trainer]
assert new['blocks'] == [int(v, 16) for row in source['rows'] for v in row.split()]
changed_tiles = []
for i, (x, y) in enumerate(zip(old['blocks'], new['blocks'])):
    if x != y:
        xx, yy = i % 32, i // 32
        assert 3 <= xx <= 18 and 4 <= yy <= 22
        assert x in [2, 4, 5] and y in [2, 3, 4, 5, 0x3c, 0x3d, 0x3e, 0x3f]
        changed_tiles.append([xx, yy])
for key in ['scripts', 'script_text_bodies']:
    for label, value in old[key].items():
        if label != hint:
            assert new[key][label] == value, (key, label)
    assert all(label.startswith(trainer) or label == item for label in new[key] if label not in old[key])
assert old['scripts'][hint][:-1] == new['scripts'][hint][:len(old['scripts'][hint])-1]
assert old['scripts'][hint][-1] == new['scripts'][hint][-1]
for key in allowed - {'blocks', 'objects', 'scripts', 'script_text_bodies', 'map_event_section_commands'}:
    if isinstance(old[key], list):
        assert new[key][:len(old[key])] == old[key], key
        for row in new[key][len(old[key]):]:
            assert row.get('source_script', '').startswith(trainer) or row.get('source_script') == item, (key, row)
for key, exceptions in [('map_scripts', {hint, 'MnLeechLake_MapEvents'}),
                        ('map_blocks', {'MnLeechLake_Blocks'}), ('npcs', {name})]:
    for label, value in before[key].items():
        if label not in exceptions:
            assert after[key][label] == value, (key, label)
    if key == 'map_scripts':
        assert set(after[key]) - set(before[key]) == set(new['scripts']) - set(old['scripts'])
    else:
        assert after[key].keys() == before[key].keys()
assert before['wild_encounters'].keys() == after['wild_encounters'].keys()
for key, value in before['wild_encounters'].items():
    expected = deepcopy(value)
    if key == name:
        expected['zones'].append(source['grass_zone'])
    assert after['wild_encounters'][key] == expected, key
for key, value in before['trainers'].items():
    if key == 'trainers':
        assert set(after['trainers'][key]) - set(value) == {trainer}
        for identity, original in value.items():
            assert after['trainers'][key][identity] == original, identity
    else:
        assert after['trainers'][key] == value, key
for key, value in before['story_event_script_constants'].items():
    current = after['story_event_script_constants'][key]
    if key == 'global':
        assert set(current) - set(value) == {trainer, 'EVENT_' + trainer, 'EVENT_MN_LEECH_MAINLAND_REPEL'}
        for label, original in value.items():
            assert current[label] == original, label
    else:
        assert current == value, key
assert q['runtime_files'] == p['runtime_files']
assert q['audio_manifest'] == p['audio_manifest'] and q['compiled_audio'] == p['compiled_audio']
for key, value in p['report'].items():
    assert q['report'][key] == (value + ['minnesota-leech-mainland-pines-v1'] if key == 'manifests' else value), key
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
result = dict(status='PASS', parent=str(a.parent), parent_sha256=sha(a.parent),
              pack=str(a.child), pack_sha256=sha(a.child), content_identity=q['identity']['content_hash'],
              unchanged_complete_maps=len(before['maps'])-1, changed_map=name,
              changed_metatiles=changed_tiles, source_sha256=sha(a.source),
              original_island_current_cabins_ports_programs_services_assets_audio_and_wildlife_retained=True)
a.output.write_text(json.dumps(result, indent=2) + '\n')
print('PASS', result['unchanged_complete_maps'], 'other maps;', len(changed_tiles), 'mainland metatiles')
