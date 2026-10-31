#!/usr/bin/env python3
"""Read-only proof that the installed cycling revision changes text alone."""
import argparse
from copy import deepcopy
import json

from render_reference import load_pack

CITY = 'MnMinneapolis'


def audit(before, after, source):
    assert before.keys() == after.keys()
    for key in before:
        if key not in {'data', 'identity'}:
            assert before[key] == after[key], key
    expected = deepcopy(before['data'])
    city = expected['maps'][CITY]
    ids = {'MN_EXTRA_MPLS_CYCLE_%02d' % n for n in range(1, 19)}
    assert len(source['trainers']) == 18
    assert {r['id'] for r in source['trainers']} == ids
    for row in source['trainers']:
        for suffix, field in [('Seen', 'before_lines'), ('Won', 'after_lines'), ('Lost', 'loss_lines')]:
            label = row['id'] + suffix
            lines = row[field]
            assert lines and all(s and s.isascii() and len(s) <= 18 for s in lines)
            commands = [dict(command='text' if i == 0 else 'para' if i % 2 == 0 else 'line',
                             args=['"' + line + '"']) for i, line in enumerate(lines)]
            commands.append(dict(command='done', args=[]))
            assert label in city['scripts'] and label in city['script_text_bodies']
            city['scripts'][label] = commands
            city['script_text_bodies'][label] = dict(label=label, commands=[
                dict(command, command_index=i) for i, command in enumerate(commands)])
            expected['map_scripts'][label] = commands
    assert after['data'] == expected, 'content outside the 54 text programs changed'
    return dict(status='PASS', riders=18, text_programs=54,
                other_maps_exact=len(expected['maps']) - 1,
                teams_patrols_optional_battles_workshop_rewards_art_audio_exact=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before'); parser.add_argument('after'); parser.add_argument('source')
    args = parser.parse_args()
    before, after = load_pack(args.before), load_pack(args.after)
    with open(args.source) as handle:
        source = json.load(handle)
    report = audit(before, after, source)
    negatives = []
    for label in ['patrol', 'battle program', 'reward program', 'compiled text']:
        bad = dict(after, data=dict(after['data'], maps=dict(after['data']['maps'])))
        city = deepcopy(after['data']['maps'][CITY])
        bad['data']['maps'][CITY] = city
        if label == 'patrol':
            next(o for o in city['objects'] if o['script'] == 'MN_EXTRA_MPLS_CYCLE_01')['x'] += 1
        elif label == 'battle program':
            city['scripts']['MN_EXTRA_MPLS_CYCLE_01'][0]['command'] = 'end'
        elif label == 'compiled text':
            city['script_text_bodies']['MN_EXTRA_MPLS_CYCLE_01Seen']['commands'][0]['args'] = ['"Wrong text."']
        else:
            room = deepcopy(after['data']['maps']['MnMinneapolisToolWorkshop'])
            bad['data']['maps']['MnMinneapolisToolWorkshop'] = room
            room['scripts']['MnRewardBike'][0]['command'] = 'end'
        try:
            audit(before, bad, source)
        except AssertionError:
            negatives.append(dict(mutation=label, rejected=True))
        else:
            raise AssertionError('accepted corruption: ' + label)
    print(json.dumps(dict(report, negative_cases=negatives)))
