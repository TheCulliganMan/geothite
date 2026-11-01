#!/usr/bin/env python3
"""Read-only exact-parent audit of the Winona research-partner wrapper."""
import argparse
import copy
import json

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    before, after = load_pack(args.parent), load_pack(args.child)
    for key in before.keys() | after.keys():
        if key not in {'data', 'identity', 'report'}:
            assert before.get(key) == after.get(key), key
    a, b = before['data'], after['data']
    for key in a.keys() | b.keys():
        if key not in {'maps', 'map_scripts', 'npcs', 'story_event_script_constants'}:
            assert a.get(key) == b.get(key), key
    room = 'MnWinonaFieldStation'
    for key in ['maps', 'npcs']:
        assert a[key].keys() == b[key].keys()
        for name in a[key]:
            if name != room:
                assert a[key][name] == b[key][name], (key, name)
    old, new = a['maps'][room], b['maps'][room]
    for key in ['attributes', 'blocks', 'events', 'scenes']:
        assert old[key] == new[key], key
    objects = copy.deepcopy(old['objects'])
    aide = next(o for o in objects if o['object_identifier'] == 'MN_W_AIDE')
    original = aide['script']
    aide['script'] = 'MnKindlitResearchAide'
    assert new['objects'] == objects
    assert new['scripts'][original] == old['scripts'][original]
    for label, commands in a['map_scripts'].items():
        if label not in {room + '_MapEvents', room + '_ObjectEvents'}:
            assert commands == b['map_scripts'][label], label
    for namespace, constants in a['story_event_script_constants'].items():
        result = b['story_event_script_constants'][namespace]
        assert all(result[k] == v for k, v in constants.items())
        assert result.keys() - constants.keys() == (
            {'EVENT_MN_KINDLIT_RESEARCH_RECEIVED'} if namespace == 'global' else set())
    rows = 0
    for label, commands in new['scripts'].items():
        if label.startswith('MnKindlit') and label.endswith('Text'):
            for command in commands:
                if command['command'] in {'text', 'line', 'cont', 'para'}:
                    for line in command['args']:
                        assert len(line.strip('"')) <= 18, (label, line)
                        rows += 1
    print(json.dumps(dict(maps=len(a['maps']), other_maps_exact=len(a['maps']) - 1,
        terrain_warps_scenes_and_actor_positions_exact=True,
        original_research_programs_exact=True, existing_constants_exact=True,
        encounters_teams_items_art_audio_exact=True, new_text_rows=rows,
        content_identity=after['identity']['content_hash'])))


if __name__ == '__main__':
    main()
