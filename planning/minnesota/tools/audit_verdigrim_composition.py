#!/usr/bin/env python3
"""Strict current-parent comparison for Verdigrim, authored cries and Dex sync.

This verifies composition, not scene integration or earned acquisition.
"""
import argparse
import copy
import json
from pathlib import Path

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    before, after = load_pack(args.parent), load_pack(args.child)
    root = Path(__file__).resolve().parents[1] / 'pokemon'
    roster = json.loads((root / 'roster.json').read_text())
    retired = {f['id'] for f in json.loads((root / 'roster-curation.json').read_text())['retired_families']}
    voices = {m['name'].upper(): 'CRY_' + m['id'] for f in roster['families']
              if f['id'] not in retired for m in f['members']}
    for key in before.keys() | after.keys():
        if key not in {'data', 'identity', 'report', 'runtime_files', 'audio_manifest'}:
            assert before.get(key) == after.get(key), key
    for key, value in before['runtime_files'].items():
        assert after['runtime_files'].get(key) == value, key
    additions = after['runtime_files'].keys() - before['runtime_files'].keys()
    assert len(additions) == 12
    assert all(k.startswith('gfx/pokemon/verdigrim/') or k == 'gfx/footprints/verdigrim.png'
               for k in additions)
    for group, entries in before['audio_manifest'].items():
        result = after['audio_manifest'][group]
        assert all(result.get(k) == v for k, v in entries.items()), group
        assert result.keys() - entries.keys() == (set(voices.values()) if group == 'cries' else set())
    a, b = before['data'], after['data']
    catalog_fields = {'pokemon', 'pokedex_entries', 'menu_icons', 'learnsets', 'egg_moves', 'evolutions'}
    changed = catalog_fields | {'pokemon_frontpic_anim', 'pokemon_cries', 'audio', 'maps',
        'map_scripts', 'story_event_script_constants', 'oak_ratings', 'capture_rules'}
    for key in a.keys() | b.keys():
        if key not in changed:
            assert a.get(key) == b.get(key), key
    for key in catalog_fields | {'pokemon_frontpic_anim'}:
        assert all(b[key].get(k) == v for k, v in a[key].items()), key
        assert b[key].keys() - a[key].keys() == (
            {'VERDIGRIM', 'VERDIGRIM_IDLE'} if key == 'pokemon_frontpic_anim' else {'VERDIGRIM'}), key
    for species, binding in b['pokemon_cries'].items():
        assert binding == (dict(cry=voices[species], pitch=0, length=256)
                           if species in voices else a['pokemon_cries'][species]), species
    assert set(b['pokemon_cries']) == set(a['pokemon_cries']) | {'VERDIGRIM'}
    old_audio = {v['id']: v for v in a['audio']}
    new_audio = {v['id']: v for v in b['audio']}
    assert all(new_audio.get(k) == v for k, v in old_audio.items())
    assert new_audio.keys() - old_audio.keys() == set(voices.values())
    expected = copy.deepcopy(a['oak_ratings'])
    expected[-1]['caughtCountLimit'] = len(b['pokemon'])
    assert b['oak_ratings'] == expected
    expected = copy.deepcopy(a['capture_rules'])
    expected['heavy_ball_modifiers']['VERDIGRIM'] = 30
    assert b['capture_rules'] == expected
    expected = copy.deepcopy(a['story_event_script_constants'])
    expected['global']['MN_DEX_MEMBER_VERDIGRIM'] = 1
    assert b['story_event_script_constants'] == expected
    room = 'MnWinonaFieldStation'
    assert a['maps'].keys() == b['maps'].keys()
    for name in a['maps']:
        if name != room:
            assert a['maps'][name] == b['maps'][name], name
    old, new = a['maps'][room], b['maps'][room]
    labels = {'MnRewardDexTier4', 'MnRewardDexClaim4Text', 'MnRewardDexClaim2',
              'MnRewardDexClaim2Text', 'MnRewardDexNeedText'}
    for label, program in old['scripts'].items():
        if label not in labels:
            assert new['scripts'][label] == program, label
    assert old['scripts'].keys() == new['scripts'].keys()
    for key in old.keys() | new.keys():
        if key not in {'scripts', 'script_control_commands', 'script_item_grants', 'script_text_bodies'}:
            assert old.get(key) == new.get(key), key
    expected = copy.deepcopy(old['scripts']['MnRewardDexTier4'])
    next(c for c in expected if c['command'] == 'ifless')['args'][0] = str(len(voices))
    assert new['scripts']['MnRewardDexTier4'] == expected
    expected = copy.deepcopy(old['scripts']['MnRewardDexClaim2'])
    next(c for c in expected if c['command'] == 'verbosegiveitem')['args'] = ['ULTRA_BALL', '10']
    assert new['scripts']['MnRewardDexClaim2'] == expected
    expected = copy.deepcopy(old['script_control_commands'])
    next(c for c in expected if c['source_script'] == 'MnRewardDexTier4'
         and c['command'] == 'ifless')['compare_value'] = str(len(voices))
    assert new['script_control_commands'] == expected
    expected = copy.deepcopy(old['script_item_grants'])
    reward = next(c for c in expected if c['source_script'] == 'MnRewardDexClaim2')
    reward.update(item_id='ULTRA_BALL', quantity=10)
    assert new['script_item_grants'] == expected
    text_labels = labels & new['script_text_bodies'].keys()
    for label, body in old['script_text_bodies'].items():
        if label not in text_labels:
            assert new['script_text_bodies'][label] == body, label
    for label in text_labels:
        assert new['script_text_bodies'][label] == dict(label=label, commands=[
            dict(**instruction, command_index=i) for i, instruction in enumerate(new['scripts'][label])])
        assert all(len(row.strip('"')) <= 18 for c in new['scripts'][label] for row in c['args'])
    for label, program in a['map_scripts'].items():
        assert b['map_scripts'][label] == (new['scripts'][label] if label in labels else program), label
    assert a['map_scripts'].keys() == b['map_scripts'].keys()
    print(json.dumps(dict(content_identity=after['identity']['content_hash'],
        other_maps_exact=len(a['maps']) - 1, previous_species_and_runtime_art_exact=True,
        original_audio_programs_exact=len(old_audio), authored_minnesota_cries=len(voices),
        total_audio_programs=len(new_audio), new_runtime_files=len(additions),
        existing_reward_flags_and_other_programs_exact=True)))


if __name__ == '__main__':
    main()
