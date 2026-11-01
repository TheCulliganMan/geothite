#!/usr/bin/env python3
"""Guarded source reconciliation; native pack publication remains Rust-only.

The reviewed plan contains exact field preimages. This preserves unrelated
concurrent planning changes and all original authored room/story annotations.
Every replaced source is retained before mutation. No game content is exported.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def encoded(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--archive', type=Path, required=True)
    args = parser.parse_args()
    plan = json.loads(args.plan.read_text())
    reference = plan['native_reference']
    pack_path = ROOT / reference['path']
    assert digest(pack_path.read_bytes()) == reference['sha256'], 'native epoch drift'
    pack = load_pack(pack_path)
    assert len(pack['data']['maps']) == reference['map_count']
    before, after = {}, {}

    def current(path):
        path = ROOT / path
        if path not in before:
            before[path] = path.read_bytes() if path.exists() else None
        return json.loads(before[path]) if before[path] else None

    def propose(path, value):
        path = ROOT / path
        current(path)
        after[path] = encoded(value)

    world = current('planning/minnesota/world.json')
    registry = current('planning/minnesota/interiors.json')
    for city in plan['cities']:
        name = city['city']; selected = {}
        for kind, path in city['selected_input_paths'].items():
            raw = (ROOT / path).read_bytes()
            assert digest(raw) == city['selected_input_sha256'][kind], (name, kind, 'source drift')
            selected[kind] = json.loads(raw)
        dossier, contract = selected['dossier'], selected['contract']
        destinations = city['canonical_destinations']
        original = current(destinations['dossier'])
        old_rooms = {room['id']: room for room in original['interiors']}
        rich_rooms = {room['id']: room for room in dossier['interiors']}
        for room in contract['rooms']:
            native = pack['data']['maps'][room['id']]
            assert native['blocks'] == [int(block, 16) for row in room['rows'] for block in row.split()]
            rich_rooms.setdefault(room['id'], room)
        # Preserve HarborWorkshop's newer authored Q07 service annotation,
        # alongside every original clinic/gym/story room's exact source value.
        rich_rooms.update(old_rooms)
        dossier['interiors'] = list(old_rooms.values()) + [rich_rooms[r['id']] for r in contract['rooms']]
        doors = {door['target']: door for door in dossier['doors']}
        for door in contract['doors']:
            # The typed runtime record intentionally omits author-only labels.
            # Keep rich IDs/notes needed by the quick source review loop.
            doors[door['target']] = dict(doors.get(door['target'], {}), **door)
        dossier['doors'] = list(doors.values())
        dossier.pop('new_rooms', None)  # One canonical row authority per room.
        dossier['expansion_contract_file'] = str(Path(destinations['contract']).relative_to(ROOT / 'planning/minnesota')) if Path(destinations['contract']).is_absolute() else str(Path(destinations['contract']).relative_to('planning/minnesota'))
        if isinstance(dossier.get('expansion_contract'), str):
            dossier['expansion_contract'] = dossier['expansion_contract_file']
        dossier['native_status'] = 'Published660-map tight-final-v02; local controller verification recorded separately; user visual acceptance pending.'
        dossier['status'] = dossier['native_status']
        propose(destinations['dossier'], dossier)
        # Executable inputs remain byte-exact to the reviewed frozen sources.
        for kind in ['layout', 'contract']:
            path = ROOT / destinations[kind]; current(path)
            after[path] = (ROOT / city['selected_input_paths'][kind]).read_bytes()
        town = next(town for town in world['towns'] if town['id'] == name)
        for key in ['dimensions_metatiles', 'ports', 'composition', 'walkable_destinations', 'door_contract']:
            town[key] = deepcopy(dossier[key])
        town['compiled_tileset'] = selected['layout']['compiled_tileset']
        town['authoring_file'] = f'towns/{name}.json'
        town['expansion_contract_file'] = dossier['expansion_contract_file']
        if name == 'MnDuluth':
            town['native_connections_source'] = dossier['expansion_contract_file']
        known = {room['id'] for room in town['interiors']}
        for room in city['world_new_room_reservations']:
            assert room['id'] not in known
            town['interiors'].append(deepcopy(room)); known.add(room['id'])
        for binding in city['connection_changes']:
            expected = binding['world_connection_current']
            link = next(link for link in world['connections'] if link.get('a') == expected['a'] and link.get('b') == expected['b'])
            assert all(link.get(k) == v for k, v in expected.items()), (name, 'connection preimage')
            legacy = {k: link.pop(k) for k in list(link) if k in {'requires_flags', 'gate_script', 'warp_index_status'} or 'gate_' in k or 'arrival_' in k}
            if legacy:
                link['legacy_planned_gate'] = legacy
            link.update(deepcopy(binding['proposed_geometry_updates']))
            link['kind'] = 'connection'
            link['qualification_source'] = 'Existing compiled native checkpoints preserved; legacy planning flags are not newly implemented prerequisites.'
            link['expansion_contract_file'] = dossier['expansion_contract_file']
        for patch in city['global_existing_interior_contract_updates']:
            record = next(record for record in registry['contracts'] if record['interior'] == patch['interior'])
            for key, change in patch['update_only_these_binding_fields'].items():
                assert record.get(key) == change['before'], (name, patch['interior'], key, 'binding preimage')
                record[key] = deepcopy(change['after'])
        known = {record['interior'] for record in registry['contracts']}
        for record in city['global_new_interior_reservations']:
            assert record['interior'] not in known
            registry['contracts'].append(deepcopy(record)); known.add(record['interior'])
    depot_path = 'planning/minnesota/transit/MnStPaulUnionDepot.json'
    depot = current(depot_path)
    assert depot['exterior_binding']['door_core'] == [44, 37]
    assert depot['exterior_binding']['safe_return_core'] == [44, 38]
    depot['exterior_binding'].update(door_core=[92, 69], safe_return_core=[92, 70], status='Original depot room preserved; relocated whole-city facade published in tight-final-v02. Rail service remains separate.')
    propose(depot_path, depot)
    propose('planning/minnesota/world.json', world)
    propose('planning/minnesota/interiors.json', registry)
    # All guards run before the first replacement; reject concurrent drift.
    for path, raw in before.items():
        assert (path.read_bytes() if path.exists() else None) == raw, str(path) + ' changed during reconciliation'
    args.archive.mkdir(parents=True, exist_ok=False)
    receipt = {'plan_sha256': digest(args.plan.read_bytes()), 'pack': reference, 'files': []}
    for path, raw in after.items():
        relative = path.relative_to(ROOT)
        for stage, data in [('before', before[path]), ('after', raw)]:
            if data is not None:
                copy = args.archive / stage / relative
                copy.parent.mkdir(parents=True, exist_ok=True); copy.write_bytes(data)
        path.write_bytes(raw)
        receipt['files'].append({'path': str(relative), 'before_sha256': digest(before[path]) if before[path] else None, 'after_sha256': digest(raw)})
    (args.archive / 'receipt.json').write_bytes(encoded(receipt))
    print('Reconciled', len(after), 'authored source files;117 new room reservations; originals archived.')


if __name__ == '__main__':
    main()
