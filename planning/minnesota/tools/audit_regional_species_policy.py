#!/usr/bin/env python3
"""Check Minnesota placement references against the active original roster.

This is a content policy audit, not a claim that every acquisition is reachable.
Original regions and unreferenced global fishing/trade tables are outside scope.
"""
import argparse
from collections import Counter
import json
from pathlib import Path

from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
HUMANS_AND_PROPS = set('''OFFICER FISHER GRAMPS TEACHER POKEFAN_M GYM_GUIDE
COOLTRAINER_M LASS BLACK_BELT CLERK GENTLEMAN POKEFAN_F NURSE GRANNY RIVAL
ROCKET SCIENTIST YOUNGSTER SWIMMER_GIRL COOLTRAINER_F FISHING_GURU SAILOR TWIN
SUPER_NERD LANCE SAGE ROCKER BEAUTY KIMONO_GIRL FALKNER BUG_CATCHER RECEPTIONIST
GAMEBOY_KID ELM MOM MN_CITY_CYCLIST_M MN_CITY_CYCLIST_F MN_PROF_TAMARACK
MN_RIVAL_REED ROCK BOULDER PAPER POKE_BALL'''.split())
FIELDS = {'species', 'species_id', 'requestedSpecies', 'offeredSpecies',
          'day_species', 'night_species'}


def audit(pack, active):
    data = pack['data']
    counts = Counter()
    failures = []

    def check(species, source):
        counts['species_references'] += 1
        if species not in active:
            failures.append([source, species])

    def scan(value, source):
        if isinstance(value, dict):
            for key, child in value.items():
                if key in FIELDS and isinstance(child, str):
                    check(child, source + ':' + key)
                else:
                    scan(child, source + ':' + key)
        elif isinstance(value, list):
            for index, child in enumerate(value):
                scan(child, source + ':' + str(index))

    trainers = {key for key in data['trainers']['trainers'] if key.startswith('MN_')}
    groups = set()
    trades = set()
    for name, room in data['maps'].items():
        if not name.startswith('Mn'):
            continue
        counts['regional_maps'] += 1
        for kind in ['wild_encounters', 'field_encounters']:
            if name in data[kind]:
                scan(data[kind][name], name + ':' + kind)
        for kind in ['gift_pokemon_scripts', 'scripted_wild_battles']:
            scan(room[kind], name + ':' + kind)
        requests = list(room['trainer_scripts'].values())
        requests += [battle['request'] for battle in room['scripted_trainer_battles']]
        trainers.update(request['trainer_id'] for request in requests)
        for label, body in room['scripts'].items():
            if not isinstance(body, list):
                continue
            for row in body:
                command, args = row.get('command'), row.get('args', [])
                if command in ['givepoke', 'giveegg', 'loadwildmon']:
                    assert args, (name, label, command)
                    check(args[0], name + ':' + label + ':' + command)
                elif command == 'trade':
                    assert args, (name, label, command)
                    trades.add(args[0])
        for actor in room['objects']:
            sprite = actor['sprite'].removeprefix('SPRITE_')
            if sprite not in HUMANS_AND_PROPS:
                check(sprite, name + ':visible-actor:' + actor['script'])
        group = room['attributes'].get('fishing_group')
        if group and group != 'FISHGROUP_NONE':
            groups.add(group)
    for identity in sorted(trainers):
        assert identity in data['trainers']['trainers'], ('missing trainer', identity)
        scan(data['trainers']['trainers'][identity]['party'], 'trainer:' + identity)
    counts['regional_trainers'] = len(trainers)
    fishing = data['fishing']
    while True:
        expanded = groups | {rule['swarm_group'] for rule in fishing['swarm_rules'].values()
                             if rule['base_group'] in groups}
        if expanded == groups:
            break
        groups = expanded
    times = set()
    for group in sorted(groups):
        assert group in fishing['groups'], ('missing fishing group', group)
        table = fishing['groups'][group]
        scan(table, 'fishing:' + group)
        for rod in table['rod_tables'].values():
            times.update(slot['time_group'] for slot in rod['slots'] if slot['time_group'])
    for group in sorted(times):
        assert group in fishing['time_groups'], ('missing fishing time group', group)
        scan(fishing['time_groups'][group], 'fishing-time:' + group)
    for trade in sorted(trades):
        assert trade in data['npc_trades'], ('missing regional trade', trade)
        scan(data['npc_trades'][trade], 'trade:' + trade)
    for species in sorted(active):
        for edge in data['evolutions'].get(species, []):
            check(edge['species'], 'evolution:' + species)
    counts.update(fishing_groups=len(groups), fishing_time_groups=len(times),
                  regional_trades=len(trades), active_species=len(active))
    assert not failures, json.dumps(failures, indent=2)
    return dict(counts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    args = parser.parse_args()
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {family['id'] for family in curation['retired_families']}
    active = {member['name'].upper() for family in roster['families']
              if family['id'] not in retired for member in family['members']}
    print(json.dumps(audit(load_pack(args.pack), active), indent=2))


if __name__ == '__main__':
    main()
