#!/usr/bin/env python3
"""Check the current five-stop overlay without writing game content."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import load_pack
from audit_regional_species_policy import audit


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(parent, child, source):
    old, new = parent['data'], child['data']
    assert len(source['stations']) == 5
    expected = deepcopy(new)
    assert source['base_content_hash'] == parent['identity']['content_hash']
    assert source['service_event'] == 'EVENT_MN_COURIER_DEFEATED'
    changed = {key for key in old['maps'] if old['maps'][key] != new['maps'][key]}
    assert changed == {stop['map'] for stop in source['stations']}
    assert old['maps'].keys() == new['maps'].keys()
    raw_owned = set()
    rides = 0
    for origin, stop in enumerate(source['stations']):
        a, b = old['maps'][stop['map']], new['maps'][stop['map']]
        prefix = 'MnGlacier' + stop['id']
        for field in ['blocks', 'events', 'objects']:
            assert a[field] == b[field], (stop['map'], field)
        attributes = dict(a['attributes'], fishing_group='FISHGROUP_NONE')
        assert b['attributes'] == attributes
        expected['map_attributes'][stop['map']] = a['attributes']
        for label, body in a['scripts'].items():
            target = prefix + 'Original' if label == stop['conductor_script'] else label
            assert b['scripts'][target] == body, label
        for label, body in a['script_text_bodies'].items():
            assert b['script_text_bodies'][label] == body, label
        before = a['map_script_section_commands']
        assert b['map_script_section_commands'][:len(before)] == before
        callbacks = [r for r in b['map_script_section_commands']
                     if r['command'] == 'callback' and r['args'][0] == 'MAPCALLBACK_NEWMAP']
        assert len(callbacks) == 1
        assert b['scripts'][callbacks[0]['args'][1]] == [
            {'command': 'setevent', 'args': [stop['visited_flag']]},
            {'command': 'endcallback', 'args': []}]
        for index, destination in enumerate(source['stations']):
            if index == origin:
                continue
            label = prefix + 'Ride' + str(index)
            rows = b['scripts'][label]
            assert rows[0] == {'command': 'checkevent', 'args': [source['service_event']]}
            assert rows[1]['command'] == 'iffalse'
            assert rows[2] == {'command': 'checkevent', 'args': [destination['visited_flag']]}
            assert rows[3]['command'] == 'iffalse'
            special = next(i for i, row in enumerate(rows) if row['command'] == 'special')
            assert rows[special]['args'] == ['MagnetTrain']
            assert rows[special - 1] == {'command': 'setval', 'args': ['TRUE' if index < origin else 'FALSE']}
            warp = next(i for i, row in enumerate(rows) if row['command'] == 'warpfacing')
            x, y = destination['ride_arrival_core']
            assert rows[warp]['args'] == [destination['constant'], str(x), str(y), 'DOWN']
            assert rows[warp + 1] == {'command': 'newloadmap', 'args': ['MAPSETUP_TRAIN']}
            assert special < warp
            rides += 1
        assert len(b['script_vertical_menus']) == 15
        raw_owned.update([stop['conductor_script'], a['attributes']['map_scripts_label']])
        expected['maps'][stop['map']] = a
    assert rides == 20
    for label in set(new['map_scripts']) - set(old['map_scripts']):
        assert label.startswith('MnGlacier'), label
        del expected['map_scripts'][label]
    for label in raw_owned:
        expected['map_scripts'][label] = old['map_scripts'][label]
    flags = new['story_event_script_constants']['global']
    added = set(flags) - set(old['story_event_script_constants']['global'])
    assert added == {s['visited_flag'] for i, s in enumerate(source['stations']) if i not in [1, 4]}
    for flag in added:
        del expected['story_event_script_constants']['global'][flag]
    assert expected == old, 'unrelated native content changed'
    report = deepcopy(child['report'])
    report['manifests'].remove('minnesota-glacier-line-service-v1')
    assert report == parent['report']
    for field in parent:
        if field not in ['data', 'identity', 'report']:
            assert parent[field] == child[field], field
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {family['id'] for family in curation['retired_families']}
    active = {member['name'].upper() for family in roster['families']
              if family['id'] not in retired for member in family['members']}
    regional = audit(dict(child, data=dict(new, maps={key: new['maps'][key] for key in changed})), active)
    return dict(status='PASS', maps=len(new['maps']), unchanged_other_maps=len(new['maps'])-5,
                directed_rides=rides, public_visit_callbacks=5, new_visit_flags=3,
                regional_policy=regional,
                all_station_terrain_actors_doors_and_original_story_exact=True,
                all_unrelated_content_art_audio_exact=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--source', type=Path, default=Path(__file__).with_name('service.json'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'retain prior audit versions'
    parent, child = load_pack(args.parent), load_pack(args.pack)
    result = check(parent, child, json.loads(args.source.read_text()))
    result.update(parent=str(args.parent), parent_sha256=sha(args.parent),
                  pack=str(args.pack), pack_sha256=sha(args.pack),
                  content_identity=child['identity']['content_hash'], source_sha256=sha(args.source))
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
