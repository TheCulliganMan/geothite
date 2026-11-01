#!/usr/bin/env python3
"""Check the published Cedar gate registration, not canonical-map rebasing."""
from copy import deepcopy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROOM = 'MnGrandMaraisShoreGate'
SOURCE = 'towns/MnGrandMaraisShoreGate.json'
BASE = 'd09cff71ce8b879b5226347e6f430fd7d3ddc2458348a242b5ef8eb36581bf14'


def validate(root, town, entry, source, contract):
    """Reject lost mouths, mixed source lineages and stale room registration."""
    assert town['id'] == contract['parent'] == 'MnGrandMarais'
    assert entry['id'] == contract['interior'] == source['id'] == ROOM
    assert entry['authoring_file'] == contract['source_file'] == SOURCE
    assert entry['access_kind'] == contract['access_kind'] == 'published_road_gatehouse'
    assert entry['kind'] == source['kind'] == 'gatehouse'
    assert source['dimensions_metatiles'] == entry['dimensions_metatiles'] == [5, 4]
    assert source['template'] == contract['reference'] == 'IlexForestAzaleaGate'
    assert source['base_content_hash'] == contract['base_content_hash'] == BASE
    assert source['canonical_rebase_required'] is contract['canonical_rebase_required'] is True
    assert source['canonical_rebase_note']
    assert contract['integration_scope'] == 'preserved_winona_lineage'
    binding = source['exterior_binding']
    assert binding['map'] == contract['exterior_map'] == 'MnMaraisCedarApproach'
    assert binding['expected_dimensions_metatiles'] == contract['exterior_dimensions_metatiles'] == [32, 19]
    assert binding['tileset'] == contract['exterior_tileset'] == 'johto'
    assert contract['exterior_sides'] == binding['sides'] and len(binding['sides']) == 2
    assert contract['passive_arrivals'] == source['passive_arrivals']
    assert source['passive_arrival'] == {'index': 5, 'tile': [1, 4]}
    assert [a['index'] for a in source['passive_arrivals']] == [5, 6, 7, 8]
    assert [e['index'] for e in source['exits']] == [1, 2, 3, 4]
    assert contract['return_triggers_core'] == [e['tile'] for e in source['exits']]
    assert contract['native_identity'] == {
        'group_id': 90, 'map_id': 1, 'constant': source['native_constant']}
    for side_index, (name, x, return_x) in enumerate([('town', 39, 38), ('shore', 44, 45)]):
        side = binding['sides'][side_index]
        assert side == dict(name=name, entrances=[[x, 26], [x, 27]],
                            returns=[[return_x, 26], [return_x, 27]],
                            active_indices=[1 + side_index * 4, 2 + side_index * 4],
                            passive_indices=[3 + side_index * 4, 4 + side_index * 4],
                            room_passive_indices=[5 + side_index * 2, 6 + side_index * 2])
        for lane in range(2):
            arrival = source['passive_arrivals'][side_index * 2 + lane]
            exit = source['exits'][side_index * 2 + lane]
            assert arrival['tile'] == [1 if side_index == 0 else 8, 4 + lane]
            assert exit['tile'] == [0 if side_index == 0 else 9, 4 + lane]
            assert arrival['target'] == exit['target'] == binding['map']
            assert arrival['target_warp'] == exit['target_warp'] == side['passive_indices'][lane]
    actors = json.loads((root / 'actors.json').read_text())['actors']
    for resident in source['residents']:
        records = [a for a in actors if a.get('native_object_id', a['id']) == resident['id']]
        assert len(records) == 1, resident['id'] + ': missing or duplicate resident registration'
        actor = records[0]
        assert actor['map'] == ROOM and actor['source_file'] == SOURCE
        assert actor['anchor_core'] == resident['tile']
        assert actor['approach_core'] == resident['approach']
        assert actor['scene'] == resident['id'] and actor['requires'] == []


def check():
    world = json.loads((ROOT / 'world.json').read_text())
    town = next(t for t in world['towns'] if t['id'] == 'MnGrandMarais')
    entries = [r for r in town['interiors'] if r['id'] == ROOM]
    assert len(entries) == 1, 'missing or duplicate Grand Marais gate'
    contracts = json.loads((ROOT / 'interiors.json').read_text())['contracts']
    contracts = [c for c in contracts if c['interior'] == ROOM]
    assert len(contracts) == 1, 'missing or duplicate gate contract'
    source = json.loads((ROOT / SOURCE).read_text())
    entry, contract = entries[0], contracts[0]
    validate(ROOT, town, entry, source, contract)
    rejected = 0
    for field, value in [('parent', 'MnWinona'), ('source_file', 'towns/MnGrandMarais.json'),
                         ('base_content_hash', 'different-lineage'),
                         ('canonical_rebase_required', False),
                         ('exterior_dimensions_metatiles', [20, 12]),
                         ('exterior_sides', [])]:
        bad = deepcopy(contract)
        bad[field] = value
        try:
            validate(ROOT, town, entry, source, bad)
        except AssertionError:
            rejected += 1
    assert rejected == 6
    print('Published gate registration passes; six unsafe registrations rejected. Canonical Cedar rebase remains explicit.')


if __name__ == '__main__':
    check()
