#!/usr/bin/env python3
"""Compare an Ely Challenge archive to its Badge Battles parent, including parsed native text."""
import argparse
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / 'tools'))
from render_reference import load_pack


def changes(a, b):
    return {key for key in a.keys() | b.keys() if key not in a or key not in b or a[key] != b[key]}


def commands(pages):
    result = []
    for page, lines in enumerate(pages):
        assert 1 <= len(lines) <= 2
        for row, line in enumerate(lines):
            assert line.isascii() and 1 <= len(line) <= 18
            result.append({'command': 'line' if row else 'para' if page else 'text', 'args': [json.dumps(line)]})
    return result + [{'command': 'done', 'args': []}]


def verify(parent, child):
    edits = json.loads((HERE / 'registration.json').read_text())
    assert changes(parent, child) == {'data', 'identity', 'report'}
    assert changes(parent['identity'], child['identity']) == {'content_hash', 'runtime_modpack_id'}
    assert changes(parent['report'], child['report']) == {'manifests'}
    history = parent['report']['manifests']
    assert history[-1] == 'northstar-badge-battles-v1'
    assert child['report']['manifests'] == history + ['northstar-ely-challenge-v1']
    assert child['identity']['runtime_modpack_id'] == '+'.join(child['report']['manifests'])
    before, after = parent['data'], child['data']
    assert changes(before, after) == {'maps', 'map_scripts'}
    assert changes(before['maps'], after['maps']) == edits.keys() == {'MnEly', 'MnLeagueLobby'}
    expected = {'MnWayLeagueAgainText', 'MnWayLeagueHoldText', 'MnElyRegistrationText'}
    assert changes(before['map_scripts'], after['map_scripts']) == expected
    assert {label for labels in edits.values() for label in labels} == expected
    for map_id, labels in edits.items():
        a, b = before['maps'][map_id], after['maps'][map_id]
        assert changes(a, b) == {'scripts', 'script_text_bodies'}
        for key in ('scripts', 'script_text_bodies'):
            assert changes(a[key], b[key]) == labels.keys()
        for label, change in labels.items():
            for data, room, side in ((before, a, 'before'), (after, b, 'after')):
                rows = commands(change[side])
                assert room['scripts'][label] == data['map_scripts'][label] == rows, label
                assert room['script_text_bodies'][label] == {
                    'label': label,
                    'commands': [dict(row, command_index=i) for i, row in enumerate(rows)],
                }, label
    print(f"Verified all {len(after['maps'])} maps. The three text bodies and their compiled copies are the only gameplay changes.")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    verify(load_pack(args.parent), load_pack(args.child))
