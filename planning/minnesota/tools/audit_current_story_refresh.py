#!/usr/bin/env python3
"""Read-only independent current story merge and preservation check."""
import argparse
from copy import deepcopy
import json
from render_reference import load_pack

BASE = 'f416ff3c899c316a63eef38f65e0546d369eafc4922981969031d95b834e3782'
DONOR = '74bfb7af0c8147c13812005770425d82a4dbf5a38655ac3e563d49708c7c11eb'
KEEP = {('MnBoundaryIntake', 'MnArcIntakeFinnText'),
        ('MnBoundaryIntake', 'MnArcIntakeIvyText'),
        ('MnWinonaFieldStation', 'MnRewardDexTier4')}
AWARD = ('MnWinonaFieldStation', 'MnRewardDexClaim4Text')


def origin(row):
    return next((row[k] for k in ['source_script', 'script_label', 'script_name'] if k in row), None)


def check(parent, common, donor, child):
    assert common['identity']['content_hash'] == BASE
    assert donor['identity']['content_hash'] == DONOR
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'report'}:
            assert parent.get(key) == child.get(key), key
    report = deepcopy(parent['report'])
    report['manifests'].append('northstar-current-story-refresh-v2')
    assert child['report'] == report, 'report'
    old, new = parent['data'], child['data']
    for key in old.keys() | new.keys():
        if key not in {'maps', 'map_scripts'}:
            assert old.get(key) == new.get(key), key
    assert old['maps'].keys() == new['maps'].keys(), 'map set'
    changed_maps = {}
    counts = dict(adopted=0, retained=0, already_current=0)
    for name, before in old['maps'].items():
        after = new['maps'][name]
        expected = deepcopy(before['scripts'])
        source = donor['data']['maps'].get(name)
        base = common['data']['maps'].get(name)
        changed = set()
        if source is not None and base is None:
            # These interiors belong to the current-world composition. The
            # story refresh explicitly retains their complete current modules.
            assert name in {'MnNorthShorePieShop', 'MnNorthShoreSmokehouse'}, name
            assert after == before, (name, 'retained donor-only current map')
            continue
        if source is not None:
            assert base is not None
            assert base['scripts'].keys() <= source['scripts'].keys()
            for label, wanted in source['scripts'].items():
                prior = base['scripts'].get(label)
                now = before['scripts'].get(label)
                if wanted == prior:
                    continue
                if (name, label) in KEEP:
                    counts['retained'] += 1
                elif now == wanted:
                    counts['already_current'] += 1
                else:
                    assert now == prior, (name, label, 'merge conflict')
                    expected[label] = deepcopy(wanted)
                    if (name, label) == AWARD:
                        assert expected[label][0]['args'] == ['"You\'ve caught 224"']
                        expected[label][0]['args'] = ['"You\'ve caught 225"']
                    changed.add(label)
                    counts['adopted'] += 1
        assert after['scripts'] == expected, (name, 'programs')
        if not changed:
            assert after == before, (name, 'unchanged complete map')
            continue
        changed_maps[name] = changed
        for field in before.keys() | after.keys():
            if field == 'scripts':
                continue
            a, b, d = before[field], after[field], source[field]
            if isinstance(a, dict):
                assert b.keys() == a.keys() | (d.keys() & changed), (name, field, 'keys')
                for label in b:
                    if label not in changed:
                        assert b[label] == a[label], (name, field, label)
                    elif field == 'script_text_bodies':
                        body = dict(label=label, commands=[dict(command=row['command'], args=row['args'], command_index=index)
                                    for index, row in enumerate(expected[label])])
                        assert b[label] == body, (name, field, label)
                    else:
                        assert b[label] == d[label], (name, field, label)
            elif isinstance(a, list) and all(isinstance(row, dict) for row in a + b + d):
                assert [row for row in b if origin(row) not in changed] == [row for row in a if origin(row) not in changed], (name, field, 'preserved metadata')
                assert [row for row in b if origin(row) in changed] == [row for row in d if origin(row) in changed], (name, field, 'revised metadata')
            else:
                assert a == b, (name, field)
    assert counts in [dict(adopted=217, retained=3, already_current=2), dict(adopted=209, retained=3, already_current=10)], counts
    revised_labels = {label for labels in changed_maps.values() for label in labels}
    assert new['map_scripts'].keys() == old['map_scripts'].keys() | revised_labels
    for label in old['map_scripts']:
        if label not in revised_labels:
            assert new['map_scripts'][label] == old['map_scripts'][label], ('global program', label)
    for name, labels in changed_maps.items():
        for label in labels:
            assert new['map_scripts'][label] == new['maps'][name]['scripts'][label], (name, 'program mirror', label)
    return dict(changed_maps=len(changed_maps), **counts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['parent', 'common', 'donor', 'child']:
        parser.add_argument(key)
    args = parser.parse_args()
    parent, common, donor, child = [load_pack(getattr(args, key)) for key in ['parent', 'common', 'donor', 'child']]
    result = check(parent, common, donor, child)
    rejected = []
    for case in ['terrain', 'roster', 'audio', 'threshold', 'intake-directions', 'award-count', 'metadata-order', 'actor-event-wire', 'raw-actor-table', 'current-shop', 'event-program-mirror', 'program-mirror']:
        changed = deepcopy(child)
        if case == 'terrain':
            changed['data']['maps']['MnBoundaryIntake']['blocks'][0] ^= 1
        elif case == 'roster':
            changed['data']['pokemon'].pop('VERDIGRIM')
        elif case == 'audio':
            changed['audio_manifest'] = None
        elif case == 'threshold':
            changed['data']['maps']['MnWinonaFieldStation']['scripts']['MnRewardDexTier4'] = []
        elif case == 'intake-directions':
            changed['data']['maps']['MnBoundaryIntake']['scripts']['MnArcIntakeFinnText'] = []
        elif case == 'award-count':
            changed['data']['maps']['MnWinonaFieldStation']['scripts']['MnRewardDexClaim4Text'][0]['args'] = ['"You\'ve caught 224"']
        elif case == 'metadata-order':
            rows = changed['data']['maps']['MnGrandMarais']['script_control_commands']
            assert len(rows) > 1 and rows != list(reversed(rows))
            rows.reverse()
        elif case == 'actor-event-wire':
            rows = changed['data']['maps']['MnR09C00R00']['map_event_section_commands']
            row = next(row for row in rows if row['command'] == 'object_event'
                       and row['args'][:2] == ['41', '37'])
            assert row['args'][3] == 'SPRITEMOVEDATA_STANDING_UP'
            row['args'][3] = 'SPRITEMOVEDATA_STANDING_RIGHT'
        elif case == 'raw-actor-table':
            changed['data']['npcs']['MnR09C00R00'] = []
        elif case == 'current-shop':
            changed['data']['maps']['MnNorthShorePieShop']['blocks'][0] ^= 1
        elif case == 'event-program-mirror':
            assert changed['data']['map_scripts']['MnR09C00R00_MapEvents']
            changed['data']['map_scripts']['MnR09C00R00_MapEvents'] = []
        else:
            changed['data']['map_scripts']['MnRewardDexClaim2'] = []
        try:
            check(parent, common, donor, changed)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(**result, rejected_negative_cases=rejected,
        caution='Static preservation only; runtime, presentation and earned-campaign acceptance remain separate.'), indent=2))


if __name__ == '__main__':
    main()
