#!/usr/bin/env python3
"""Prove supporting-story integration and exact preservation of the whole campaign."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

MARKER = 'comprehensive-story-support-v27-integration-v1'
TRANSFER = 'MnIceTransfer'


def expected_data(current, common, latest):
    expected = deepcopy(current['data'])
    adopted = []
    changed_maps = set()
    for name, donor in latest['data']['maps'].items():
        before = common['data']['maps'].get(name)
        if before is None:
            continue
        labels = {key for key, value in donor['scripts'].items()
                  if before['scripts'].get(key) != value}
        if not labels:
            continue
        assert before['scripts'].keys() <= donor['scripts'].keys(), ('removed labels', name)
        checked = deepcopy(donor)
        for field in ['scripts', 'script_text_bodies', 'script_text_commands']:
            checked[field] = before[field]
        if name == TRANSFER:
            for field in ['objects', 'map_event_section_commands']:
                checked[field] = before[field]
        assert checked == before, ('unreviewed donor map fields', name)
        room = expected['maps'][name]
        for label in labels:
            assert room['scripts'].get(label) == before['scripts'].get(label), (name, label)
            assert current['data']['map_scripts'].get(label) == common['data']['map_scripts'].get(label), ('raw preimage', label)
            assert latest['data']['map_scripts'][label] == donor['scripts'][label], ('raw donor', label)
            room['scripts'][label] = donor['scripts'][label]
            expected['map_scripts'][label] = donor['scripts'][label]
            if label in donor['script_text_bodies']:
                room['script_text_bodies'][label] = donor['script_text_bodies'][label]
            else:
                room['script_text_bodies'].pop(label, None)
            adopted.append((name, label))
        room['script_text_commands'] = [row for row in room['script_text_commands']
                                        if row['source_script'] not in labels] + [
            row for row in donor['script_text_commands'] if row['source_script'] in labels]
        changed_maps.add(name)
    assert len(adopted) == 134 and len(changed_maps) == 19, (len(adopted), len(changed_maps))
    room = expected['maps'][TRANSFER]
    before = common['data']['maps'][TRANSFER]
    donor = latest['data']['maps'][TRANSFER]
    assert room['objects'] == before['objects']
    tess = next(actor for actor in room['objects']
                if actor['object_identifier'] == 'MN_ICE_HQ_TESS')
    assert (tess['x'], tess['y'], tess['event_flag']) == (25, 5, '-1')
    tess['event_flag'] = 'EVENT_MN_ICE_CAPTIVE_RESCUED'
    assert room['objects'] == donor['objects'], 'only Tess departure may change'
    assert room['map_event_section_commands'] == before['map_event_section_commands']
    room['map_event_section_commands'] = donor['map_event_section_commands']
    for field, key in [('npcs', TRANSFER), ('map_scripts', 'MnIceTransfer_MapEvents')]:
        assert current['data'][field][key] == common['data'][field][key]
        assert latest['data'][field][key] != common['data'][field][key]
        expected[field][key] = latest['data'][field][key]
    return expected, adopted


def check(current, common, latest, child):
    for pack, content in [(current, '4a56ea4c8e94bfb9c2c18c8350775d22776e1b7bf7d1fdb3df533c12329443a2'),
                          (common, '74bfb7af0c8147c13812005770425d82a4dbf5a38655ac3e563d49708c7c11eb'),
                          (latest, 'ccf84221d047e958736a4ca5cbecc917713a4496e03ffe89527e72e2d2593914')]:
        assert pack['identity']['content_hash'] == content
    expected, adopted = expected_data(current, common, latest)
    assert child['data'] == expected, 'unapproved campaign delta'
    report = deepcopy(current['report'])
    report['manifests'].append(MARKER)
    assert child['report'] == report
    for field in current.keys() | child.keys():
        if field not in {'data', 'report', 'identity'}:
            assert current.get(field) == child.get(field), field
    assert child['identity'] != current['identity']
    return adopted


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['current', 'common', 'latest', 'child']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    current, common, latest, child = map(load_pack, [args.current, args.common, args.latest, args.child])
    adopted = check(current, common, latest, child)
    rejected = []
    for case in ['buoys', 'verdigrim', 'research-threshold', 'audio', 'tess-pose', 'tess-departure', 'text-cache']:
        bad = deepcopy(child)
        if case == 'buoys':
            bad['data']['maps']['MnTallTaleTrail']['blocks'][63] = 7
        elif case == 'verdigrim':
            bad['data']['story_event_script_constants']['global'].pop('EVENT_MN_VERDIGRIM_CAUGHT')
        elif case == 'research-threshold':
            rows = bad['data']['maps']['MnWinonaFieldStation']['scripts']['MnRewardDexTier4']
            next(row for row in rows if row['command'] == 'ifless')['args'][0] = '224'
        elif case == 'audio':
            bad['audio_manifest'] = None
        elif case in {'tess-pose', 'tess-departure'}:
            tess = next(actor for actor in bad['data']['maps'][TRANSFER]['objects']
                        if actor['object_identifier'] == 'MN_ICE_HQ_TESS')
            if case == 'tess-pose':
                tess['x'] -= 1
            else:
                tess['event_flag'] = '-1'
        else:
            name, label = next((name, label) for name, label in adopted
                               if label in bad['data']['maps'][name]['script_text_bodies'])
            bad['data']['maps'][name]['script_text_bodies'][label]['commands'] = []
        try:
            check(current, common, latest, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('accepted deliberate corruption: ' + case)
    print(json.dumps(dict(adopted_programs=sorted(adopted), captive_departure_mirrors=4,
                         negative_cases_rejected=rejected,
                         scope='Whole-pack preservation and exact supporting-story delta. Native behavior and earned ending remain separate.'), indent=2))


if __name__ == '__main__':
    main()
