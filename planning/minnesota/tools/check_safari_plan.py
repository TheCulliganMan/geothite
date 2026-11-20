"""Resolve the preserve's encounter rates, slot weights and selected regional species."""
import json


def safari_tables(zone_file, zone, lineup_file):
    habitats = zone['candidate_habitats']
    sources = {(zone_file.parent / h['encounters']['source']).resolve() for h in habitats}
    assert len(sources) == 1, 'the four preserve districts must reference one runtime zone'
    native = json.loads(sources.pop().read_text())
    assert native['id'] == zone['id']
    by_id = {h['id']: h for h in native['habitats']}
    lineup = json.loads(lineup_file.read_text())['safari']
    assert len(by_id) == len(native['habitats']) and by_id.keys() == lineup.keys()
    for habitat in habitats:
        ref = habitat['encounters']
        assert ref['habitat_id'] == habitat['id'] and ref['habitat_id'] in by_id
        assert ref['tables'] == ['grass', 'water']

    tables = {}
    for name, habitat in by_id.items():
        tables[name] = {}
        for kind in ('grass', 'water'):
            assert len(habitat[kind]) == len(lineup[name][kind]), (name, kind)
            slots = [dict(base, species=species) for base, species in
                     zip(habitat[kind], lineup[name][kind])]
            assert sum(s['weight'] for s in slots) == 100, (name, kind)
            assert all(1 <= s['level'] <= 100 and 0 < s['weight'] <= 100 and
                       0 <= s['flee_threshold'] <= 255 for s in slots), (name, kind)
            tables[name][kind] = slots
    return tables
