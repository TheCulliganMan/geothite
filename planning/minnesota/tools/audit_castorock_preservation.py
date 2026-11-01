#!/usr/bin/env python3
"""Read-only exact-parent audit for the hand-authored incisor gallery."""
import argparse
import json
from pathlib import Path

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    parser.add_argument('site')
    args = parser.parse_args()
    before, after = load_pack(args.parent), load_pack(args.child)
    site = json.loads(Path(args.site).read_text())
    room = site['map']
    for key in before.keys() | after.keys():
        if key not in {'data', 'report', 'identity'}:
            assert before.get(key) == after.get(key), key
    a, b = before['data'], after['data']
    mirrors = {'maps', 'npcs', 'map_attributes', 'map_dimensions', 'map_blocks',
               'runtime_map_metadata'}
    for key in a.keys() | b.keys():
        if key in mirrors:
            assert a[key].keys() == b[key].keys(), key
            for name in a[key]:
                cave_key = f'{room}_Blocks' if key == 'map_blocks' else room
                if name != cave_key:
                    assert a[key][name] == b[key][name], (key, name)
        elif key not in {'map_scripts', 'story_event_script_constants'}:
            assert a.get(key) == b.get(key), key
    changed_labels = {'MN_R03_CAVE_WALKERText', f'{room}_MapEvents',
                      f'{room}_MapScripts'}
    for name, commands in a['map_scripts'].items():
        if name not in changed_labels:
            assert commands == b['map_scripts'][name], name
    old, new = a['maps'][room], b['maps'][room]
    assert old['events']['warps'] == new['events']['warps']
    assert old['events']['coord_events'] == new['events']['coord_events']
    assert not old['events']['bg_events']
    assert new['events']['bg_events'] == [
        dict(x=x, y=y, event_type='BGEVENT_READ', script=label)
        for (x, y), label in zip(site['clues'], ['MnJawPool', 'MnJawPillar', 'MnJawBed'])
    ]
    assert new['objects'][:len(old['objects'])] == old['objects']
    assert [o['object_identifier'] for o in new['objects'][len(old['objects']):]] == [
        'MN_R03_JAW_BOULDER', 'MN_R03_JAW_CASTOROCK']
    for key in old['attributes'].keys() | new['attributes'].keys():
        if key not in {'width', 'height'}:
            assert old['attributes'].get(key) == new['attributes'].get(key), key
    width, height = site['dimensions_metatiles']
    assert (new['attributes']['width'], new['attributes']['height']) == (width, height)
    assert new['blocks'] == [int(cell, 16) for row in site['rows'] for cell in row.split()]
    changes = {(x, y): (int(was, 16), int(now, 16))
               for x, y, was, now in site['changed_original_blocks']}
    assert len(changes) == 3
    for y in range(old['attributes']['height']):
        for x in range(old['attributes']['width']):
            was = old['blocks'][y * old['attributes']['width'] + x]
            now = new['blocks'][y * width + x]
            assert (was, now) == changes.get((x, y), (was, was)), (x, y)
    constants_a = a['story_event_script_constants']
    constants_b = b['story_event_script_constants']
    assert constants_a.keys() == constants_b.keys()
    for namespace in constants_a:
        for key, value in constants_a[namespace].items():
            assert constants_b[namespace][key] == value, key
        added = constants_b[namespace].keys() - constants_a[namespace].keys()
        expected = {'EVENT_MN_CASTOROCK_' + suffix for suffix in
                    ['POOL', 'PILLAR', 'BED', 'TRACKS_READ', 'CAUGHT']}
        assert added == (expected if namespace == 'global' else set()), namespace
    print(json.dumps(dict(maps=len(a['maps']), other_maps_exact=len(a['maps']) - 1,
        old_warps_actors_pickup_exact=True, original_metatile_changes=len(changes),
        encounters_teams_items_field_rules_exact=True, source_art_audio_exact=True,
        content_hash=after['identity']['content_hash'])))


if __name__ == '__main__':
    main()
