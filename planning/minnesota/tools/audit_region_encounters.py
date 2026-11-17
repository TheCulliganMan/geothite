#!/usr/bin/env python3
"""Inventory the selected Minnesota pack's encounters before or after authoring.

Terrain counts describe collision geometry, not earned access or pathfinding.
Scripted acquisitions are reported separately from repeatable spawn tables.
Optional JSON output belongs in ignored review storage.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path

from audit_outdoor_connections import collision
from render_reference import load_pack


PLANNING = Path(__file__).resolve().parents[1]
ROOT = PLANNING.parents[1]


def active_roster():
    source = PLANNING / 'pokemon'
    curation = json.loads((source / 'roster-curation.json').read_text())
    roster = json.loads((source / 'roster.json').read_text())
    retired = {family['id'] for family in curation['retired_families']}
    return {
        member['name'].upper()
        for family in roster['families'] if family['id'] not in retired
        for member in family['members']
    }


def audit(data, active, stages):
    sites = []
    ordinary = set()
    map_geometry = {}
    fishing = data['fishing']

    def add(location, method, slots, **details):
        entries = [{'species': slot['species'], 'level': slot['level']} for slot in slots]
        if method not in {'gift', 'egg', 'static'} and details.get('tiles') != 0:
            ordinary.update(slot['species'] for slot in entries)
        sites.append(dict(location=location, method=method, slots=entries, **details))

    def timed(location, method, table, **details):
        for period, slots in (table or {}).items():
            add(location, method, slots, period=period, **details)

    for name, room in data['maps'].items():
        if not name.startswith('Mn'):
            continue
        attrs = room['attributes']
        tiles = collision({'data': data}, room)
        land = {at for at, kind in tiles.items() if kind == 'TALL_GRASS'
                or (attrs['environment'] in {'CAVE', 'DUNGEON'} and kind == 'FLOOR')}
        water = sum(kind == 'WATER' for kind in tiles.values())
        trees = sum(kind.startswith('HEADBUTT_TREE') for kind in tiles.values())
        map_geometry[name] = dict(walking=len(land), water=water, headbutt=trees,
                                  progression=stages.get(name), location=attrs['location'])
        wild = data['wild_encounters'].get(name, {})
        for zone in wild.get('zones', []):
            cells = {at for at in land
                     if zone['minX'] <= at[0] <= zone['maxX']
                     and zone['minY'] <= at[1] <= zone['maxY']}
            land -= cells
            timed(name, 'zone', zone['grass'], source=zone['id'], tiles=len(cells))
        timed(name, 'walking', wild.get('grass'), tiles=len(land))
        timed(name, 'surf', wild.get('water'), tiles=water)
        for key, swarm in wild.get('swarm_overrides', {}).items():
            timed(name, 'walking_swarm', swarm['grass'], source=key, tiles=len(land))
        for method, table in data['field_encounters'].get(name, {}).get('tables', {}).items():
            for bucket in ('common', 'rare'):
                add(name, method, table[bucket], bucket=bucket,
                    **({'tiles': trees} if method == 'headbutt' else {}))

        base = attrs.get('fishing_group')
        groups = [(base, None)] + [
            (rule['swarm_group'], key) for key, rule in fishing['swarm_rules'].items()
            if rule['base_group'] == base
        ]
        for header, swarm in groups:
            if header in (None, 'FISHGROUP_NONE'):
                continue
            for rod, table in fishing['groups'][header]['rod_tables'].items():
                for index, slot in enumerate(table['slots']):
                    metadata = dict(source=header, swarm=swarm, index=index, tiles=water)
                    if slot['species'] is not None:
                        add(name, rod, [slot], **metadata)
                        continue
                    variants = fishing['time_groups'][slot['time_group']]
                    for period in ('day', 'night'):
                        add(name, rod, [{
                            'species': variants[f'{period}_species'],
                            'level': variants[f'{period}_level'],
                        }], period=period, **metadata)
        for gift in room['gift_pokemon_scripts']:
            add(name, 'egg' if gift['egg'] else 'gift',
                [{'species': gift['species_id'], 'level': gift['level']}],
                source=gift['source_script'])
        for battle in room['scripted_wild_battles']:
            add(name, 'static', [battle['request']], source=battle['source_script'])

    safari = data.get('safari_zone') or {}
    for habitat in safari.get('habitats', []):
        (left, top), (right, bottom) = habitat['bounds_core']
        surfaces = {'grass': {}, 'water': {}}
        for name, placement in safari['maps'].items():
            origin = placement['origin_core']
            if origin is None:
                continue
            court = safari['gate_court']
            (court_left, court_top), (court_right, court_bottom) = court['free_bounds_core']
            counts = Counter(
                kind for (x, y), kind in collision({'data': data}, data['maps'][name]).items()
                if left <= x + origin[0] < right and top <= y + origin[1] < bottom
                and not (name == court['map_name'] and court_left <= x < court_right
                         and court_top <= y < court_bottom)
            )
            for method, token in [('grass', 'TALL_GRASS'), ('water', 'WATER')]:
                if counts[token]:
                    surfaces[method][name] = counts[token]
        for method, maps in surfaces.items():
            add(habitat['id'], f'safari_{method}', habitat[method],
                maps=maps, tiles=sum(maps.values()))

    def acquisition_closure(exclude_methods=()):
        obtainable = {
            slot['species'] for site in sites
            if site['method'] not in exclude_methods and site.get('tiles') != 0
            for slot in site['slots'] if slot['species'] in active
        }
        pending = list(obtainable)
        while pending:
            species = pending.pop()
            for evolution in data['evolutions'].get(species, []):
                if evolution['method'] != 'LEVEL':
                    continue
                target = evolution['species']
                if target in active and target not in obtainable:
                    obtainable.add(target)
                    pending.append(target)
        return obtainable

    obtainable = acquisition_closure()
    outside_safari = acquisition_closure({'safari_grass', 'safari_water'})

    affected = defaultdict(set)
    duplicates = defaultdict(list)
    for site in sites:
        species = [slot['species'] for slot in site['slots']]
        if set(species) - active and site.get('tiles') != 0:
            affected[site['method']].add(site['location'])
        if site['method'] == 'walking' and site['period'] == 'day' and site['tiles']:
            duplicates[tuple(species)].append(site['location'])

    return {
        'summary': {
            'regional_maps': len(map_geometry),
            'source_rows_by_method': dict(Counter(site['method'] for site in sites)),
            'maps_with_inactive_species': {method: sorted(maps) for method, maps in affected.items()},
            'duplicate_day_walk_lineups': [maps for maps in duplicates.values() if len(maps) > 1],
            'active_species_missing_from_pack': sorted(active - data['pokemon'].keys()),
            'active_species_without_known_acquisition': sorted(active - obtainable),
            'active_species_without_free_acquisition': sorted(active - outside_safari),
            'installed_active_species_missing_from_repeatable_tables':
                sorted((active & data['pokemon'].keys()) - ordinary),
        },
        'maps': map_geometry,
        'sites': sites,
    }


def verify_repeatable_delta(parent, candidate, prior_audit, current_audit, active):
    """Bind a repeatable-only release to its exact parent mechanics and scope."""
    before, after = parent['data'], candidate['data']

    def preserve_slots(old_slots, new_slots):
        assert len(old_slots) == len(new_slots), 'slot count changed'
        for old, new in zip(old_slots, new_slots):
            old['species'] = new['species']
        assert old_slots == new_slots, 'encounter mechanics changed'

    def preserve_periods(old_table, new_table):
        if old_table is None:
            assert new_table is None
            return
        assert old_table.keys() == new_table.keys()
        for period in old_table:
            preserve_slots(old_table[period], new_table[period])

    def exact_keys(field):
        assert before[field].keys() == after[field].keys(), f'{field} keys changed'
        return before[field]

    for name, source in exact_keys('wild_encounters').items():
        if not name.startswith('Mn'):
            continue
        target = after['wild_encounters'][name]
        for field in ('grass', 'water'):
            preserve_periods(source[field], target[field])
        old_zones, new_zones = source.get('zones', []), target.get('zones', [])
        assert len(old_zones) == len(new_zones), 'local zone count changed'
        for old, new in zip(old_zones, new_zones):
            preserve_periods(old['grass'], new['grass'])
        for key, old in source.get('swarm_overrides', {}).items():
            preserve_periods(old['grass'], target['swarm_overrides'][key]['grass'])

    old_field, new_field = before['field_encounters'], after['field_encounters']
    for name, source in old_field.items():
        if name.startswith('Mn'):
            for method, table in source['tables'].items():
                for bucket in ('common', 'rare'):
                    preserve_slots(table[bucket], after['field_encounters'][name]
                                   ['tables'][method][bucket])

    safari_maps = (before['safari_zone'] or {}).get('maps', {})
    expected_new_trees = {
        name for name, geometry in current_audit['maps'].items()
        if geometry['headbutt'] and name not in old_field and name not in safari_maps
    }
    assert new_field.keys() - old_field.keys() == expected_new_trees, 'Headbutt coverage drift'
    for name in expected_new_trees:
        assert set(new_field[name]['tables']) == {'headbutt'}
    old_field.update({name: new_field[name] for name in expected_new_trees})

    if before['safari_zone']:
        old_habitats = before['safari_zone']['habitats']
        new_habitats = after['safari_zone']['habitats']
        assert len(old_habitats) == len(new_habitats)
        for old, new in zip(old_habitats, new_habitats):
            for method in ('grass', 'water'):
                preserve_slots(old[method], new[method])

    prior_fishing, current_fishing = before['fishing'], after['fishing']
    old_groups, new_groups = prior_fishing['groups'], current_fishing['groups']
    old_times, new_times = prior_fishing['time_groups'], current_fishing['time_groups']
    assert old_groups.keys() == new_groups.keys(), 'rod group identities changed'
    used_clock_groups = set()
    for name, old_group in old_groups.items():
        if not name.startswith('FISHGROUP_MN_'):
            continue
        for rod in ('GOOD_ROD', 'SUPER_ROD'):
            old_slots = old_group['rod_tables'][rod]['slots']
            new_slots = new_groups[name]['rod_tables'][rod]['slots']
            assert len(old_slots) == len(new_slots), 'rod slot count changed'
            for old, new in zip(old_slots, new_slots):
                assert (old['species'] is None) == (new['species'] is None)
                if old['time_group'] != new['time_group']:
                    assert old['time_group'] in old_times
                    assert new['time_group'] not in old_times
                    prior = old_times[old['time_group']]
                    current = new_times[new['time_group']]
                    for period in ('day', 'night'):
                        assert prior[f'{period}_level'] == current[f'{period}_level']
                    used_clock_groups.add(new['time_group'])
                old['species'] = new['species']
                old['time_group'] = new['time_group']
    assert old_times.keys() | used_clock_groups == new_times.keys()
    old_times.update({name: new_times[name] for name in used_clock_groups})

    prefix = 'MN_DEX_MEMBER_'
    old_constants = before['story_event_script_constants']['global']
    new_constants = after['story_event_script_constants']['global']
    prior_members = {key for key in old_constants if key.startswith(prefix)}
    current_members = {key for key in new_constants if key.startswith(prefix)}
    assert len(prior_members) == len(current_members) == len(active), 'research threshold changed'
    assert current_members == {prefix + name for name in active}, 'regional research roster drifted'
    assert all(new_constants[key] == 1 for key in current_members)
    for key in prior_members:
        del old_constants[key]
    old_constants.update({key: new_constants[key] for key in current_members})

    for field in before:
        assert before[field] == after[field], f'unexpected change in {field}'
    for key in ('runtime_files', 'audio_manifest', 'compiled_audio'):
        assert parent[key] == candidate[key], f'{key} changed'

    prior = prior_audit['summary']
    current = current_audit['summary']
    assert not current['active_species_missing_from_pack'], 'active roster is incomplete'
    assert not current['active_species_without_known_acquisition'], 'unavailable active species'
    assert not current['active_species_without_free_acquisition'], 'paid-only active species'
    retired = set(current['maps_with_inactive_species']) - {'static', 'gift', 'egg'}
    assert not retired, f'retired species in repeatable sources: {retired}'
    lost = set(current['installed_active_species_missing_from_repeatable_tables']) - set(
        prior['installed_active_species_missing_from_repeatable_tables'])
    # The canoe overlay used Castorock as a generic substitute. Its existing
    # fossil and one-time jaw discovery are the intended acquisition paths.
    if 'CASTOROCK' in lost:
        assert any(site['location'] == 'MnR03Cave1F' and site['method'] == 'static'
                   and site['slots'][0]['species'] == 'CASTOROCK'
                   for site in current_audit['sites'])
        lost.remove('CASTOROCK')
    assert not lost, f'active repeatable species lost: {sorted(lost)}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, help='defaults to world.json local selection')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--compare', type=Path, help='verify the repeatable-only delta from this parent')
    args = parser.parse_args()
    world = json.loads((PLANNING / 'world.json').read_text())
    revision = world[world['active_local_review_revision']]
    pack = load_pack(args.pack or ROOT / revision['pack'])
    if args.pack is None and revision['content_identity'] != pack['identity']['content_hash']:
        parser.error('local pack identity differs from world.json')
    path = PLANNING / 'pokemon/acquisition-v01/encounter-progression-v01/progression.json'
    groups = json.loads(path.read_text())['groups']
    stages = {name: group['id'] for group in groups for name in group['maps']}
    active = active_roster()
    result = audit(pack['data'], active, stages)
    if args.compare:
        parent = load_pack(args.compare)
        verify_repeatable_delta(parent, pack, audit(parent['data'], active, stages), result, active)
        result['summary']['parent_preservation_verified'] = True
    result['identity'] = pack['identity']['content_hash']
    if args.output:
        with args.output.open('x') as output:
            json.dump(result, output, indent=2)
            output.write('\n')
    print(json.dumps(result['summary'], indent=2))


if __name__ == '__main__':
    main()
