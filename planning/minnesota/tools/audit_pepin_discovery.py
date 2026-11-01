#!/usr/bin/env python3
"""Independently check bounded native Pepin publication; never writes a pack."""
import argparse
from copy import deepcopy
import json
from render_reference import load_pack

MAPS = {'MnMississippiLowerWoodsC00R01', 'MnMississippiLowerWoodsC01R01',
        'MnMississippiLowerWoodsC01R00'}
FLAGS = {'EVENT_MN_PEPIN_' + suffix for suffix in [
    'DOCK_SKETCH_READ', 'EDDY_OBSERVED', 'REEDS_OBSERVED',
    'SAFE_INLET_IDENTIFIED', 'CAUGHT']}
ACTORS = {
    'MnMississippiLowerWoodsC00R01': {('MnPepinDock', 'SPRITE_FISHER', 22, 10)},
    'MnMississippiLowerWoodsC01R01': {('MnPepinOperator', 'SPRITE_SAILOR', 50, 16)},
    'MnMississippiLowerWoodsC01R00': {('MN_PEPIN_PEPIVERN', 'SPRITE_PEPIVERN', 24, 21)}}
READS = {
    'MnMississippiLowerWoodsC00R01': [(24, 10, 'MnPepinEddy')],
    'MnMississippiLowerWoodsC01R01': [(50, 20, 'MnPepinReeds')],
    'MnMississippiLowerWoodsC01R00': []}


def check(parent, child):
    old, new = parent['data'], child['data']
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'report'}:
            assert parent.get(key) == child.get(key), key
    report = deepcopy(parent['report'])
    report['manifests'].append('minnesota-pepin-three-wakes-v1')
    assert child['report'] == report, 'unreviewed report change'
    for key in old.keys() | new.keys():
        if key not in {'maps', 'npcs', 'map_scripts', 'story_event_script_constants'}:
            assert old.get(key) == new.get(key), key
    assert old['maps'].keys() == new['maps'].keys()
    for name, before in old['maps'].items():
        after = new['maps'][name]
        if name not in MAPS:
            assert after == before, name
            continue
        for key in ['attributes', 'blocks', 'scenes']:
            assert after[key] == before[key], (name, key)
        assert after['events']['warps'] == before['events']['warps']
        assert after['events']['coord_events'] == before['events']['coord_events']
        reads = list(before['events']['bg_events']) + [
            dict(x=x, y=y, event_type='BGEVENT_READ', script=label)
            for x, y, label in READS[name]]
        assert after['events']['bg_events'] == reads
        original_count = len(before['objects'])
        assert after['objects'][:original_count] == before['objects']
        added = after['objects'][original_count:]
        assert len(added) == 1
        assert {(o['script'], o['sprite'], o['x'], o['y']) for o in added} == ACTORS[name]
        assert all(after['scripts'].get(k) == v for k, v in before['scripts'].items())
        assert all(k.startswith(('MnPepin', 'MN_PEPIN_'))
                   for k in after['scripts'].keys() - before['scripts'].keys())
        added_labels = after['scripts'].keys() - before['scripts'].keys()
        for field in before.keys() | after.keys():
            if field in {'id', 'attributes', 'blocks', 'scenes', 'objects', 'events', 'scripts'}:
                continue
            previous, current = before[field], after[field]
            if field in {'map_script_section_commands', 'map_event_section_commands'}:
                def without_new_rows(rows):
                    return [dict(command=row['command'], args=row['args']) for row in rows
                            if not any(arg in added_labels for arg in row['args'])]
                assert without_new_rows(current) == without_new_rows(previous), (name, field)
            elif isinstance(previous, dict):
                assert all(current.get(label) == value for label, value in previous.items()), (name, field)
                assert current.keys() - previous.keys() <= added_labels, (name, field)
            elif isinstance(previous, list):
                def origin(row):
                    return next((row[key] for key in ['source_script','script_label','script_name'] if key in row), None)
                retained = [row for row in current if origin(row) not in added_labels]
                assert retained == previous, (name, field)
            else:
                assert current == previous, (name, field)
        added_text = {label for label in added_labels if label.startswith('MnPepinText_')}
        assert after['script_text_bodies'].keys() - before['script_text_bodies'].keys() == added_text
        for label in added_text:
            expected = dict(label=label, commands=[dict(command=row['command'], args=row['args'], command_index=index)
                            for index, row in enumerate(after['scripts'][label])])
            assert after['script_text_bodies'][label] == expected, (name, label)

        wild = [(c['args'][0], c['args'][1]) for program in after['scripts'].values()
                if isinstance(program, list) for c in program if c['command'] == 'loadwildmon']
        assert wild == ([('PEPIVERN', '55')] if name.endswith('C01R00') else [])
        if name.endswith('C01R00'):
            callbacks = [row['args'] for row in after['map_script_section_commands']
                         if row['command'] == 'callback' and row['args'][1].startswith('MnPepin')]
            assert callbacks == [['MAPCALLBACK_OBJECTS', 'MnPepinObjects'],
                                 ['MAPCALLBACK_NEWMAP', 'MnPepinNewVisit']]
        else:
            assert after['map_script_section_commands'] == before['map_script_section_commands']
    assert old['npcs'].keys() == new['npcs'].keys()
    for name, npcs in old['npcs'].items():
        if name not in MAPS:
            assert new['npcs'][name] == npcs, name
        else:
            assert new['npcs'][name] == new['maps'][name]['objects'], name
    permitted = set()
    for name in MAPS:
        room = new['maps'][name]
        programs = dict(room['scripts'])
        for attr, section in [('map_scripts_label', 'map_script_section_commands'),
                              ('map_events_label', 'map_event_section_commands')]:
            programs[room['attributes'][attr]] = [
                dict(command=row['command'], args=row['args']) for row in room[section]]
        permitted.update(programs)
        for label, commands in programs.items():
            assert new['map_scripts'][label] == commands, label
    for label in old['map_scripts'].keys() | new['map_scripts'].keys():
        if label not in permitted:
            assert old['map_scripts'].get(label) == new['map_scripts'].get(label), label
    before, after = old['story_event_script_constants'], new['story_event_script_constants']
    assert before.keys() == after.keys()
    for key in before:
        if key != 'global':
            assert before[key] == after[key], key
    a, b = before['global'], after['global']
    assert all(b.get(k) == v for k, v in a.items())
    assert b.keys() - a.keys() == FLAGS
    ids = {b[k] for k in FLAGS}
    assert len(ids) == 5 and not ids & {v for k, v in a.items() if k.startswith('EVENT_')}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    check(parent, child)
    rejected = []
    for case in ['terrain', 'old-program', 'old-text-body', 'foreign-encounter', 'audio']:
        changed = deepcopy(child)
        if case == 'terrain':
            changed['data']['maps']['MnMississippiLowerWoodsC01R01']['blocks'][0] ^= 1
        elif case == 'old-program':
            name = 'MnMississippiLowerWoodsC00R01'
            label = next(iter(parent['data']['maps'][name]['scripts']))
            changed['data']['maps'][name]['scripts'][label] = []
        elif case == 'old-text-body':
            name = next(name for name in MAPS if parent['data']['maps'][name]['script_text_bodies'])
            label = next(iter(parent['data']['maps'][name]['script_text_bodies']))
            changed['data']['maps'][name]['script_text_bodies'][label]['commands'] = []
        elif case == 'foreign-encounter':
            room = changed['data']['maps']['MnMississippiLowerWoodsC01R00']
            command = next(c for p in room['scripts'].values() if isinstance(p, list)
                           for c in p if c['command'] == 'loadwildmon')
            command['args'][0] = 'MEW'
        else:
            changed['audio_manifest'] = None
        try:
            check(parent, changed)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(maps=len(child['data']['maps']),
        preserved_other_complete_maps=len(parent['data']['maps']) - 3,
        original_terrain_connections_catalog_audio_runtime_files_exact=True,
        rejected_negative_cases=rejected,
        caution='No runtime, visual, earned access or full-game acceptance is implied.'), indent=2))


if __name__ == '__main__':
    main()
