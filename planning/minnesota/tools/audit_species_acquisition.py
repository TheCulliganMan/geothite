#!/usr/bin/env python3
"""Read-only upper-bound acquisition audit; tables never prove player reachability.

Absence from even this generous pool identifies a missing family acquisition
path. Presence still needs native route, capture, evolution and breeding proof.
Unconditional collection of fishing time groups, trades and roamers deliberately
avoids reporting a family missing merely because an indirect source was omitted.
"""
import argparse
import json
from pathlib import Path

from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    parser.add_argument('--species', action='append', default=[], type=str.upper,
                        help='Show direct source labels and incoming evolution edges for an active species.')
    parser.add_argument('--require-roster-alignment', action='store_true',
                        help='Exit unsuccessfully if species, Dex membership or completion threshold differ from the active source roster.')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    data = pack['data']
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    families = [f for f in roster['families'] if f['id'] not in retired]
    active = {m['name'].upper() for f in families for m in f['members']}
    unknown = set(args.species) - active
    if unknown:
        parser.error('Not active Minnesota species: ' + ', '.join(sorted(unknown)))
    potential = set()
    sources = {}

    def collect(value, label):
        if isinstance(value, str) and value in active:
            potential.add(value)
            sources.setdefault(value, set()).add(label)
        elif isinstance(value, dict):
            for child in value.values():
                collect(child, label)
        elif isinstance(value, list):
            for child in value:
                collect(child, label)

    for kind in ['wild_encounters', 'field_encounters']:
        for name, table in data[kind].items():
            if name.startswith('Mn'):
                collect(table, kind + ':' + name)
    for name, room in data['maps'].items():
        if not name.startswith('Mn'):
            continue
        for kind in ['gift_pokemon_scripts', 'scripted_wild_battles']:
            collect(room[kind], kind + ':' + name)
        for label, commands in room['scripts'].items():
            if not isinstance(commands, list):
                continue
            for command in commands:
                if command.get('command') in ['givepoke', 'giveegg', 'loadwildmon']:
                    collect(command.get('args', [])[:1], name + ':' + label)
        group = room['attributes'].get('fishing_group')
        if group in data['fishing']['groups']:
            collect(data['fishing']['groups'][group], 'regional-fishing:' + group)
    # These are generous upper bounds, not claims about regional activation.
    for kind in ['npc_trades', 'roaming_pokemon']:
        collect(data[kind], 'global-upper-bound:' + kind)
    collect(data['fishing'].get('time_groups', {}), 'fishing-time-upper-bound')
    # Follow every evolution regardless of item, level, time or trade access.
    # This remains an upper bound and also covers cross-family branch edges.
    reachable = set(potential)
    while True:
        evolved = {edge['species'] for species in reachable
                   for edge in data['evolutions'].get(species, [])
                   if edge['species'] in active}
        if evolved <= reachable:
            break
        reachable.update(evolved)
    missing = []
    for family in families:
        members = {m['name'].upper() for m in family['members']}
        if not members & reachable:
            missing.append(dict(id=family['id'], members=sorted(members)))
    # Acquisition and integration are different failures: a missing encounter
    # does not imply that a species is absent from the compiled catalog.
    prefix = 'MN_DEX_MEMBER_'
    dex_members = {key[len(prefix):] for key in
                   data['story_event_script_constants']['global']
                   if key.startswith(prefix)}
    station = data['maps'].get('MnWinonaFieldStation', {})
    reward_thresholds = sorted({int(command['args'][0])
        for label, commands in station.get('scripts', {}).items()
        if label == 'MnRewardDex' or label.startswith('MnRewardDexTier')
        if isinstance(commands, list)
        for command in commands
        if command.get('command') == 'ifless'
        and command.get('args', ['', ''])[0].isdigit()
        and command['args'][1:] == ['MnRewardDexNeed']})
    absent_catalog = sorted(active - set(data['pokemon']))
    absent_members = sorted(active - dex_members)
    obsolete_members = sorted(dex_members - active)
    completion_threshold = max(reward_thresholds, default=None)
    aligned = not (absent_catalog or absent_members or obsolete_members) \
        and completion_threshold == len(active)
    report = dict(
        pack=str(args.pack), content_identity=pack['identity']['content_hash'],
        active_species=len(active), active_families=len(families),
        potentially_direct_species=len(potential),
        potentially_available_after_all_evolutions=len(reachable),
        species_not_in_direct_or_evolution_pool=sorted(active - reachable),
        families_without_any_potential_source=missing,
        roster_pack_alignment=dict(
            aligned=aligned,
            active_species_absent_from_pack_catalog=absent_catalog,
            active_species_absent_from_dex_membership=absent_members,
            dex_members_outside_active_roster=obsolete_members,
            compiled_dex_member_count=len(dex_members),
            compiled_research_thresholds=reward_thresholds,
            compiled_completion_threshold=completion_threshold,
            required_completion_threshold=len(active)),
        caution='Presence is not acceptance: reachability, catchability, breeding/evolution conditions and an earned journey remain unverified.'
    )
    if args.species:
        report['requested_source_details'] = {
            species: dict(
                direct_source_labels=sorted(sources.get(species, set())),
                potentially_reachable=species in reachable,
                incoming_evolutions=[dict(parent=parent, edge=edge)
                    for parent, edges in data['evolutions'].items() if parent in active
                    for edge in edges if edge['species'] == species],
            ) for species in dict.fromkeys(args.species)
        }
    print(json.dumps(report, indent=2))
    if args.require_roster_alignment and not aligned:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
