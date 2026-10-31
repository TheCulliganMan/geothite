#!/usr/bin/env python3
"""Read-only winter-town publication, preservation and regional roster audit.

Static access is separate from the production controller/stdio/visual proofs.
No packs, copied source art or game state are written.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_regional_species_policy import audit
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
TOWN = 'MnInternationalFalls'
SKIER = 'MN_EXTRA_FALLS_SKIER'
WON = 'EVENT_' + SKIER
LABELS = {SKIER + suffix for suffix in ['', 'Seen', 'Won', 'Lost', 'After', 'Declined']}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(parent, child, source):
    old, town = parent['data']['maps'][TOWN], child['data']['maps'][TOWN]
    read = lambda name: json.loads((source / name).read_text())
    blocks = lambda terrain: [int(v, 16) for row in terrain['rows'] for v in row.split()]
    assert old['blocks'] == blocks(read('before.json'))
    assert town['blocks'] == blocks(read('after.json'))
    assert town['attributes'] == old['attributes'] and town['events'] == old['events']
    assert town['objects'][:-1] == old['objects']
    actor = town['objects'][-1]
    assert (actor['x'], actor['y'], actor['sprite'], actor['script']) == (
        17, 12, 'SPRITE_COOLTRAINER_F', SKIER)
    assert actor['spritemovedata'] == 'SPRITEMOVEDATA_STANDING_DOWN'
    assert actor['radius'] == 0 and actor['object_type'] == 'OBJECTTYPE_SCRIPT'
    assert town['scripts'].keys() - old['scripts'].keys() == LABELS
    assert all(town['scripts'][k] == v for k, v in old['scripts'].items())
    assert town['map_event_section_commands'][:-1] == old['map_event_section_commands']
    assert town['map_event_section_commands'][-1]['args'] == [
        '17', '12', 'SPRITE_COOLTRAINER_F', 'SPRITEMOVEDATA_STANDING_DOWN',
        '0', '0', '-1', '-1', '0', 'OBJECTTYPE_SCRIPT', '0', SKIER, '-1']
    expected = deepcopy(old)
    for key in ['blocks', 'objects', 'scripts', 'map_event_section_commands']:
        expected[key] = town[key]
    for key in ['script_control_commands', 'script_map_commands', 'script_text_commands',
                'script_flag_commands', 'scripted_trainer_battles', 'script_text_bodies',
                'script_object_commands', 'script_runtime_commands']:
        prior, current = old[key], town[key]
        if isinstance(prior, dict):
            assert {k: v for k, v in current.items() if k not in LABELS} == prior, key
        else:
            order = lambda v: (v['source_script'], v.get('command_index', -1))
            assert sorted((v for v in current if v['source_script'] not in LABELS), key=order) == sorted(prior, key=order), key
        expected[key] = current
    assert expected == town, 'unexpected native map field'
    data = dict(parent['data'])
    for key, name, value in [
        ('maps', TOWN, town), ('npcs', TOWN, town['objects']),
        ('map_blocks', old['attributes']['blocks_label'], base64.b64encode(bytes(town['blocks'])).decode('ascii'))]:
        data[key] = dict(data[key], **{name: value})
    scripts = dict(data['map_scripts'])
    for label in LABELS:
        assert label not in scripts
        scripts[label] = town['scripts'][label]
    scripts[old['attributes']['map_events_label']] = [
        {'command': row['command'], 'args': row['args']} for row in town['map_event_section_commands']]
    data['map_scripts'] = scripts
    constants = deepcopy(data['story_event_script_constants'])
    actual = child['data']['story_event_script_constants']['global']
    assert actual.keys() - constants['global'].keys() == {SKIER, WON}
    assert actual[SKIER] == 1 and actual[WON] not in constants['global'].values()
    constants['global'].update({name: actual[name] for name in [SKIER, WON]})
    data['story_event_script_constants'] = constants
    trainers = deepcopy(data['trainers'])
    trainer = child['data']['trainers']['trainers'][SKIER]
    placement = read('trainers.json')['trainers'][0]
    assert trainer['name'] == 'MARIT' and trainer['trainer_class'] == 'SKIER'
    assert [(p['species'], p['level']) for p in trainer['party']] == [
        (p['species'], p['level']) for p in placement['party']]
    assert SKIER not in trainers['trainers']
    trainers['trainers'][SKIER] = trainer
    data['trainers'] = trainers
    assert data == child['data'], [k for k in data if data[k] != child['data'][k]]
    assert child['runtime_files'] == parent['runtime_files'], 'all original art/program files exact'
    report = dict(parent['report'])
    report['manifests'] = [*report['manifests'], 'minnesota-international-falls-windbreak-v1']
    assert child['report'] == report
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'runtime_files', 'report'}:
            assert child.get(key) == parent.get(key), key
    tiles = data['tilesets'][town['attributes']['tileset_name']]['collision']
    kinds = {(x, y): tiles[f'{town["blocks"][y//2*16+x//2]:02x}'][y % 2*2+x % 2]
             for y in range(28) for x in range(32)}
    bodies = {(o['x'], o['y']) for o in town['objects']}
    floor = {at for at, kind in kinds.items() if kind in {'FLOOR', 'DOOR'}} - bodies
    reached, queue = {(20, 25)}, deque([(20, 25)])
    while queue:
        x, y = queue.popleft()
        for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if at in floor and at not in reached:
                reached.add(at)
                queue.append(at)
    assert floor == reached
    assert all(tuple(v['tile']) in reached for v in read('after.json')['review_contract']['viewpoints'])
    assert all(kinds[at] == 'WALL' for at in [(12, 12), (13, 19), (28, 14)])
    roster = read_roster()
    scoped = dict(child, data=dict(data, maps={TOWN: town}))
    regional = audit(scoped, roster)
    assert data['pokemon']['LUTEBRISK']['type1'] == data['pokemon']['LUTEBRISK']['type2'] == 'ICE'
    return dict(unchanged_maps=len(data['maps'])-1, reachable_ordinary_tiles=len(reached),
                regional_policy=regional, original_actors=4, original_warps=4,
                all_art_audio_runtime_files_preserved=True)


def read_roster():
    curation = json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text())
    roster = json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    return {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'child', 'source', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'retain earlier audit results'
    parent = load_pack(args.parent)
    child = load_pack(args.child)
    result = check(parent, child, args.source)
    result.update(status='PASS', parent=str(args.parent), parent_sha256=digest(args.parent),
                  pack=str(args.child), pack_sha256=digest(args.child),
                  content_identity=child['identity']['content_hash'],
                  sources={p.name: digest(p) for p in args.source.glob('*.json')})
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
