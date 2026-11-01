#!/usr/bin/env python3
"""Read-only audit of every regional encounter source and the journey curve.

Writes optional verification output outside source. Never constructs a pack.
Surface reachability and earned story arrival require their separate checks.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('pack', type=Path)
    parser.add_argument('plan', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    before = load_pack(args.parent)
    pack = load_pack(args.pack)
    data = pack['data']
    plan = json.loads(args.plan.read_text())
    assert before['identity']['content_hash'] == plan['parent_content_hash']
    groups = {}
    for group in plan['groups']:
        for name in group['maps']:
            assert name not in groups
            groups[name] = group
    assert set(groups) == {n for n in data['wild_encounters'] if n.startswith('Mn')}
    assert len(groups) == 111
    # The revision owns encounter tables only. This also binds every prior
    # story program, retry flag, gift, shop, trainer and native geography.
    for key, value in before['data'].items():
        if key not in {'wild_encounters', 'field_encounters', 'trainers'}:
            assert value == data[key], 'unrelated data changed: ' + key
    assert before['runtime_files'] == pack['runtime_files']
    assert before['audio_manifest'] == pack['audio_manifest']
    trainer_changes = []
    trainer_floors = {}
    for name, group in groups.items():
        room = data['maps'][name]
        for request in list(room.get('trainer_scripts', {}).values()) + [b['request'] for b in room.get('scripted_trainer_battles', [])]:
            key = request['trainer_id']
            floor = [2,9,13,18,23,27,30,35,40][group['minimum_badges']]
            trainer_floors[key] = min(trainer_floors.get(key, floor), floor)
    for key, old in before['data']['trainers']['trainers'].items():
        new = data['trainers']['trainers'][key]
        if key not in trainer_floors:
            assert old == new, 'unowned opponent changed: ' + key
        else:
            assert min(p['level'] for p in new['party']) >= trainer_floors[key]
        if old == new:
            continue
        assert key.startswith('MN_')
        assert {k:v for k,v in old.items() if k != 'party'} == {k:v for k,v in new.items() if k != 'party'}
        assert len(old['party']) == len(new['party'])
        for a,b in zip(old['party'], new['party']):
            assert a['level'] <= b['level']
            assert a['item'] == b['item'] and a['dvs'] == b['dvs']
        trainer_changes.append(dict(id=key, before=old['party'], after=new['party']))
    for key in ['wild_encounters', 'field_encounters']:
        for name, value in before['data'][key].items():
            if not name.startswith('Mn'):
                assert value == data[key][name], 'original-region table changed: ' + name

    active = {key.removeprefix('MN_DEX_MEMBER_') for key in data['story_event_script_constants']['global'] if key.startswith('MN_DEX_MEMBER_')}
    assert len(active) == 225
    minimum = defaultdict(lambda: 1)
    for name in active:
        for edge in data['evolutions'].get(name, []):
            assert edge['method'] == 'LEVEL' and edge['species'] in active
            assert 1 <= edge['level'] <= 55
            minimum[edge['species']] = max(minimum[edge['species']], edge['level'])
    rows = []
    def add(name, method, species, level, **details):
        assert species in active, (name, method, species)
        assert minimum[species] <= level <= 100, (name, method, species, level, minimum[species])
        profile = data['pokemon'][species]
        assert 0 < profile['catch_rate'] <= 255
        known = [move for at, move in data['learnsets'][species] if at <= level]
        assert known and all(move in data['moves'] and data['moves'][move]['pp'] > 0 for move in known)
        row = dict(map=name, method=method, species=species, level=level,
                   bst=sum(profile['base_stats'].values()), catch_rate=profile['catch_rate'],
                   minimum_badges=groups[name]['minimum_badges'] if name in groups else None,
                   **details)
        rows.append(row)

    for name, group in groups.items():
        old = before['data']['wild_encounters'][name]
        wild = data['wild_encounters'][name]
        assert old['grass_rates'] == wild['grass_rates']
        assert old['water_rate'] == wild['water_rate']
        tables = [('grass', wild.get('grass')), ('surf', wild.get('water'))]
        for zone, prior in zip(wild.get('zones', []), old.get('zones', [])):
            assert {k:v for k,v in zone.items() if k != 'grass'} == {k:v for k,v in prior.items() if k != 'grass'}
            tables.append(('zone:' + zone['id'], zone['grass']))
        assert len(old.get('zones', [])) == len(wild.get('zones', []))
        for swarm, table in wild.get('swarm_overrides', {}).items():
            tables.append(('swarm:' + swarm, table['grass']))
        for method, table in tables:
            if not table:
                continue
            for period, slots in table.items():
                weights = [60, 30, 10] if method == 'surf' else [30, 30, 20, 10, 5, 4, 1]
                assert len(slots) == len(weights)
                for slot, (entry, weight) in enumerate(zip(slots, weights)):
                    if method == 'surf':
                        assert entry['level'] >= max(20, [2,6,10,15,19,23,27,32,38][group['minimum_badges']])
                    else:
                        assert [2,6,10,15,19,23,27,32,38][group['minimum_badges']] <= entry['level'] <= [8,13,18,23,30,32,38,44,49][group['minimum_badges']]
                    add(name, method, entry['species'], entry['level'], period=period, slot=slot, weight_percent=weight)

    for name, field in data['field_encounters'].items():
        if not name.startswith('Mn'):
            continue
        assert name in groups
        old = before['data']['field_encounters'][name]
        for method, table in field['tables'].items():
            for bucket in ['common', 'rare']:
                prior = old['tables'][method][bucket]
                assert len(prior) == len(table[bucket])
                for entry, original in zip(table[bucket], prior):
                    assert entry['weight'] == original['weight'] > 0
                    assert entry.get('sleep_turns_by_time', {}) == original.get('sleep_turns_by_time', {})
                    add(name, method, entry['species'], entry['level'], bucket=bucket, native_weight=entry['weight'])

    # Each active header and its swarm/time indirection is checked, even on dry
    # maps. The surface audit separately excludes dry/inaccessible rod claims.
    for name, room in data['maps'].items():
        if not name.startswith('Mn'):
            continue
        raw = set()
        typed = set()
        for label, body in room['scripts'].items():
            if not isinstance(body, list):
                continue
            for index, command in enumerate(body):
                if command.get('command') in {'givepoke', 'giveegg', 'loadwildmon'}:
                    species, level = command['args'][:2]
                    raw.add((label, index, species, int(level), command['command']))
        for gift in room.get('gift_pokemon_scripts', []):
            typed.add((gift['source_script'], gift['command_index'], gift['species_id'], gift['level'], 'giveegg' if gift['egg'] else 'givepoke'))
            add(name, 'egg' if gift['egg'] else 'gift', gift['species_id'], gift['level'], script=gift['source_script'])
        for battle in room.get('scripted_wild_battles', []):
            request = battle['request']
            typed.add((battle['source_script'], battle['loadwildmon_command_index'], request['species'], request['level'], 'loadwildmon'))
            add(name, 'static', request['species'], request['level'], script=battle['source_script'])
        assert raw == typed, ('unaccounted or stale scripted acquisition', name, raw ^ typed)
        base = room['attributes'].get('fishing_group')
        headers = [(base, False)] + [(r['swarm_group'], True) for r in data['fishing']['swarm_rules'].values() if r['base_group'] == base]
        for header, swarm in headers:
            if header not in data['fishing']['groups']:
                assert header in {None, 'FISHGROUP_NONE'}
                continue
            fish = data['fishing']['groups'][header]
            assert 0 < fish['bite_threshold'] <= 255
            for rod, table in fish['rod_tables'].items():
                prior_threshold = -1
                for slot in table['slots']:
                    assert prior_threshold < slot['threshold'] <= 255
                    prior_threshold = slot['threshold']
                    if slot.get('species'):
                        add(name, rod, slot['species'], slot['level'], swarm=swarm, header=header)
                    else:
                        timed = data['fishing']['time_groups'][slot['time_group']]
                        for period in ['day', 'night']:
                            add(name, rod, timed[period+'_species'], timed[period+'_level'], period=period, swarm=swarm, header=header)
                assert prior_threshold == 255

    summaries = []
    for group in plan['groups']:
        land = [r for r in rows if r['map'] in group['maps'] and (r['method'] == 'grass' or r['method'].startswith('zone:'))]
        surf = [r for r in rows if r['map'] in group['maps'] and r['method'] == 'surf']
        summaries.append(dict(id=group['id'], minimum_badges=group['minimum_badges'], access=group['access'], maps=group['maps'],
                              walking_levels=[min(r['level'] for r in land), max(r['level'] for r in land)] if land else None,
                              surf_levels=[min(r['level'] for r in surf), max(r['level'] for r in surf)] if surf else None,
                              walking_species=sorted({r['species'] for r in land})))
    result = dict(pack=str(args.pack), identity=pack['identity']['content_hash'], reviewed_maps=len(groups),
                  rows_by_method=dict(Counter(r['method'] for r in rows)), reviewed_local_trainers=len(trainer_floors),
                  changed_local_trainers=len(trainer_changes), trainer_changes=trainer_changes, groups=summaries, sources=rows,
                  scope='Every installed table, time period, rod indirection, field bucket, typed gift and static encounter checked. Separate surface/controller evidence establishes actual local access; no earned full-campaign claim.')
    if args.output:
        assert not args.output.exists(), 'preserve earlier verification output'
        args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in {'sources','groups','trainer_changes'}}, indent=2))


if __name__ == '__main__':
    main()
