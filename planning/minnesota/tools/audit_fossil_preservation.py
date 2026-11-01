#!/usr/bin/env python3
"""Read-only comparison of a fossil publication and its exact parent."""
import argparse
import json

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    before, after = load_pack(args.parent), load_pack(args.child)
    for key in before:
        if key not in ['identity', 'report', 'data']:
            assert before[key] == after[key], key
    a, b = before['data'], after['data']
    allowed = {'maps', 'map_scripts', 'npcs', 'items', 'story_event_script_constants'}
    for key in a:
        if key not in allowed:
            assert a[key] == b[key], key
    changed = {'MnR01C00R00': 'MnFossilBluff', 'MnRochester': 'MnFossilResearcher'}
    assert a['maps'].keys() == b['maps'].keys()
    text_rows = 0
    for name, old in a['maps'].items():
        new = b['maps'][name]
        if name not in changed:
            assert old == new, name
            continue
        for key in ['attributes', 'blocks', 'events']:
            assert old[key] == new[key], (name, key)
        assert new['objects'][:-1] == old['objects'], name
        assert new['objects'][-1]['script'] == changed[name], name
        for label, commands in old['scripts'].items():
            if not label.endswith(('_MapEvents', '_ObjectEvents')):
                assert commands == new['scripts'][label], (name, label)
        for label, commands in new['scripts'].items():
            if label.startswith('MnFossil') and label.endswith('Text'):
                for command in commands:
                    if command['command'] in ['text', 'line', 'cont', 'para']:
                        for line in command['args']:
                            assert len(line.strip('"')) <= 18, (label, line)
                            text_rows += 1
    assert b['items'].keys() - a['items'].keys() == {'LENS_FOSSIL', 'SPIRE_FOSSIL'}
    for item, value in a['items'].items():
        assert value == b['items'][item], item
    for key, value in a['story_event_script_constants']['global'].items():
        assert value == b['story_event_script_constants']['global'][key], key
    print(json.dumps(dict(maps=len(a['maps']), other_maps_exact=len(a['maps']) - 2,
        terrain_warps_old_actors_exact=True, encounters_teams_field_rules_rods_exact=True,
        original_items_constants_exact=True, source_art_audio_exact=True,
        new_text_rows=text_rows, content_hash=after['identity']['content_hash'])))


if __name__ == '__main__':
    main()
