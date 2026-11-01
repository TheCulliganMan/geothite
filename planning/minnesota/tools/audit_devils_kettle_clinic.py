#!/usr/bin/env python3
"""Check only the existing clinic's nurse and one native recovery registration."""
import argparse
from copy import deepcopy
import json

from render_reference import load_pack

ROOM = 'MnGrandMaraisPokecenter'
CONSTANT = 'MN_GRAND_MARAIS_POKECENTER'
NURSE = 'MnGrandMaraisNurse'
MANIFEST = 'minnesota-marais-clinic-recovery-v1'


def audit(before, after):
    for key in before.keys() | after.keys():
        if key not in {'data', 'identity', 'report'}:
            assert before.get(key) == after.get(key), key
    report = deepcopy(after['report'])
    assert report['manifests'].count(MANIFEST) == 1
    report['manifests'].remove(MANIFEST)
    assert report == before['report'], 'unrelated report drift'
    old, new = before['data'], after['data']
    previous, clinic = old['maps'][ROOM], new['maps'][ROOM]
    expected_program = [dict(command='blackoutmod', args=[CONSTANT]),
                        dict(command='jumpstd', args=['PokecenterNurseScript'])]
    assert previous['scripts'][NURSE] == expected_program[1:]
    assert clinic['scripts'] == dict(previous['scripts'], **{NURSE: expected_program})
    changed_parsers = {'script_runtime_commands', 'script_control_commands'}
    for key in previous.keys() | clinic.keys():
        if key in changed_parsers:
            assert [r for r in previous[key] if r['source_script'] != NURSE] == [
                r for r in clinic[key] if r['source_script'] != NURSE], key
        elif key != 'scripts':
            assert previous.get(key) == clinic.get(key), key
    assert [r for r in clinic['script_runtime_commands'] if r['source_script'] == NURSE] == [
        dict(command='blackoutmod', args=[CONSTANT], source_script=NURSE, command_index=0)]
    assert [r for r in clinic['script_control_commands'] if r['source_script'] == NURSE] == [
        dict(r, command_index=r['command_index']+1)
        for r in previous['script_control_commands'] if r['source_script'] == NURSE]
    additions = new['runtime_spawn_points'].keys() - old['runtime_spawn_points'].keys()
    assert len(additions) == 1
    identifier = next(iter(additions))
    spawn = new['runtime_spawn_points'][identifier]
    metadata = old['runtime_map_metadata'][CONSTANT]
    def field(snake, camel):
        value = metadata.get(snake, metadata.get(camel))
        assert value is not None
        return value
    assert spawn == dict(identifier=int(identifier), mapConstant=CONSTANT,
                         mapName=ROOM, groupId=field('group_id','groupId'),
                         mapId=field('map_id','mapId'), groupName=field('group_name','groupName'),
                         tileX=3, tileY=5, metatileX=1, metatileY=2, subtileX=1, subtileY=1)
    assert not any(s['mapConstant'] == CONSTANT for s in old['runtime_spawn_points'].values())
    residual = deepcopy(new)
    residual['maps'][ROOM] = previous
    residual['map_scripts'][NURSE] = old['map_scripts'][NURSE]
    assert new['map_scripts'][NURSE] == expected_program
    del residual['runtime_spawn_points'][identifier]
    assert residual == old, 'unrelated compiled content drift'
    return dict(status='PASS', other_maps_exact=len(old['maps'])-1,
                all_art_audio_species_and_terrain_exact=True,
                prior_spawns_exact=True, new_clinic_recovery_points=1,
                native_nurse_registration=True, gameplay_acceptance=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before'); parser.add_argument('after')
    args = parser.parse_args()
    before, after = load_pack(args.before), load_pack(args.after)
    report = audit(before, after)
    negatives = []
    new_key = next(iter(after['data']['runtime_spawn_points'].keys() - before['data']['runtime_spawn_points'].keys()))
    for label in ['wrong recovery floor', 'old recovery drift', 'native art drift', 'extra compiled nurse operation']:
        bad = dict(after, data=dict(after['data']))
        if label == 'native art drift':
            bad['runtime_files'] = dict(after['runtime_files'])
            bad['runtime_files'][next(iter(bad['runtime_files']))] = b'unreviewed source art'
        elif label == 'extra compiled nurse operation':
            bad['data']['maps'] = dict(after['data']['maps'])
            room = deepcopy(after['data']['maps'][ROOM])
            room['script_runtime_commands'].append(dict(command='blackoutmod',args=['MN_WINONA_HOME_2F'],source_script=NURSE,command_index=2))
            bad['data']['maps'][ROOM] = room
        else:
            bad['data']['runtime_spawn_points'] = dict(after['data']['runtime_spawn_points'])
            key = new_key if label == 'wrong recovery floor' else next(iter(before['data']['runtime_spawn_points']))
            spawn = dict(after['data']['runtime_spawn_points'][key]); spawn['tileY'] += 1
            bad['data']['runtime_spawn_points'][key] = spawn
        try:
            audit(before,bad)
        except AssertionError:
            negatives.append(dict(mutation=label,rejected=True))
        else:
            raise AssertionError('accepted corruption: '+label)
    print(json.dumps(dict(report,negative_cases=negatives)))
