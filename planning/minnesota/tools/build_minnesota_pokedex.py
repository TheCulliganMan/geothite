#!/usr/bin/env python3
"""Assemble authored Minnesota data; Rust remains the pack compiler."""
import argparse
import hashlib
import itertools
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'pokemon'
STAT_KEYS = ('hp', 'attack', 'defense', 'speed', 'special_attack', 'special_defense')


def three_lines(text):
    """Balance complete words across exactly three native text lines."""
    assert text.isascii() and not any(c in text for c in '\n@')
    words = text.split()
    choices = []
    for left, right in itertools.combinations(range(1, len(words)), 2):
        lines = [' '.join(words[:left]), ' '.join(words[left:right]),
                 ' '.join(words[right:])]
        if max(map(len, lines)) <= 18:
            choices.append((sum((len(line) - len(text) / 3) ** 2 for line in lines), lines))
    if not choices:
        raise ValueError(f'dex text cannot fit three native lines: {text}')
    return ' @ '.join(min(choices)[1])


def rows(filename):
    return [line.split('|') for line in (ROOT / filename).read_text().splitlines()
            if line and not line.startswith('#')]


def create_metadata():
    catalog = json.loads((ROOT / 'roster.json').read_text())
    vocabulary_path = ROOT.parents[2] / 'target/minnesota-pokemon/profile-vocabulary.json'
    vocabulary = json.loads(vocabulary_path.read_text()) if vocabulary_path.is_file() else None
    palettes = json.loads((ROOT / 'palettes.json').read_text())['pokemon']
    dex = {f'MN_{int(number):03}': (kind, [three_lines(a), three_lines(b)])
           for number, kind, a, b in rows('dex-authoring.txt')}
    design = {f'MN_F{int(row[0]):03}': row[1:] for row in rows('profile-authoring.txt')}
    species = []
    for family in catalog['families']:
        vector, heights, weights, donors, groups, levels, growth = design[family['id']]
        final_stats = list(map(int, vector.split(',')))
        heights, weights = [list(map(int, values.split(','))) for values in (heights, weights)]
        icon, cry, footprint = donors.split(',')
        levels = list(map(int, levels.split(','))) if levels else []
        assert len(heights) == len(weights) == len(family['members'])
        assert len(levels) == len(family['members']) - 1
        rare = bool(family.get('rarity'))
        for index, member in enumerate(family['members']):
            identity = member['id']
            stats = member.get('base_stats')
            if stats is None:
                # Distinct family roles persist during growth; full explicit rows
                # are published in the complete species-profiles.json.
                factor = [1.0] if len(heights) == 1 else ([0.56, 1.0] if len(heights) == 2 else [0.54, 0.76, 1.0])
                if int(family['id'][4:]) <= 3:
                    factor = [0.60, 0.76, 1.0]
                stats = dict(zip(STAT_KEYS, [max(15, round(n * factor[index])) for n in final_stats]))
            classification, pages = dex[identity]
            assert 1 <= len(classification) <= 11
            eggs = ['EGG_' + value for value in groups.split(',')]
            genderless = 'MINERAL' in groups and all(g in ('MINERAL', 'INDETERMINATE') for g in groups.split(','))
            gender = 255 if rare or genderless else (31 if int(family['id'][4:]) <= 3 else 127)
            if family['id'] == 'MN_F008':
                gender = 254
            catch = 3 if rare else (45 if int(family['id'][4:]) <= 3 or family['id'] == 'MN_F072' else [190, 90, 45][min(index, 2)])
            if len(heights) == 1 and not rare:
                catch = 60
            if family['id'] in ('MN_F092', 'MN_F093'):
                catch = 45
            normal = palettes[identity]['gbc_rgb5']
            # Rotate the two chromatic inks, preserving luminosity order and
            # the black/white endpoints. Every species keeps its own palette.
            shiny = [[color[2], color[0], color[1]] for color in normal[1:3]]
            if shiny == normal[1:3]:
                shiny = [[min(30, c[0] + 3), max(1, c[1] - 4), c[2]] for c in normal[1:3]]
            evolution = member.get('evolution_condition')
            if member['evolves_to'] and evolution is None:
                evolution = {'method': 'LEVEL', 'level': levels[index]}
            profile = {
                'id': identity, 'classification': classification,
                'height_inches': heights[index], 'weight_tenths_lb': weights[index],
                'dex_pages': pages, 'base_stats': stats,
                'catch_rate': catch,
                'base_exp': 255 if rare else min(254, max(40, round(sum(stats.values()) * (0.42 if index == len(heights)-1 else 0.22)))),
                'gender_ratio': gender, 'hatch_cycles': 120 if rare else (40 if growth == 'SLOW' else 20),
                'growth_rate': 'GROWTH_' + growth, 'egg_groups': eggs,
                'evolution': evolution, 'held_items': [None, None],
                'menu_icon': 'ICON_' + icon, 'footprint_species': footprint,
                'cry': {'cry': 'CRY_' + cry, 'pitch': (index - len(heights) + 1) * 24,
                        'length': 144 + index * 16},
                'shiny_rgb5': shiny,
            }
            # This optional local index catches authoring typos early. The Rust
            # compiler always resolves every reference against the actual base.
            if vocabulary is not None:
                for key in ('menu_icon', 'growth_rate', 'footprint_species'):
                    vocabulary_key = {'menu_icon': 'menu_icons', 'growth_rate': 'growth_rates'}.get(key, key)
                    assert profile[key] in vocabulary[vocabulary_key], (identity, key, profile[key])
                assert profile['cry']['cry'] in vocabulary['cry_ids'], (identity, profile['cry'])
                assert set(eggs) <= set(vocabulary['egg_groups']), (identity, eggs)
            species.append(profile)
    species.sort(key=lambda item: item['id'])
    assert len(species) == catalog['species_count'] == len(dex)
    return catalog, {'schema_version': 1, 'species': species}


def write_guide(catalog, batch):
    profiles = {row['id']: row for row in batch['species']}
    lines = ['# Minnesota Pokédex', '',
             f'{len(profiles)} original species; all descriptions and powers are fictional. '
             'Move IDs resolve against the supplied base pack, which uses a physical/special split per move.', '',
             'Stats are HP / Attack / Defense / Speed / Special Attack / Special Defense. '
             'Fronts are 56×56; close rear portraits are 48×48. '
             'Normal and shiny colors each use four GBC inks.', '']
    for family in catalog['families']:
        lines += [f'## {family["id"]}: ' + ' → '.join(m['name'] for m in family['members']), '', family['theme'], '']
        lines += ['Habitat: ' + family.get('habitat', family['region'].replace('_', ' ')) + '.', '']
        if family.get('location'):
            lines += ['Proposed encounter location: ' + family['location'] + '.', '']
        for key in ('legendary_narrative', 'mythical_narrative'):
            if family.get(key):
                lines += [family[key], '']
        for member in family['members']:
            p = profiles[member['id']]
            lines += [f'### {member["id"]} {member["name"]} — ' + '/'.join(member['types']), '',
                      f'{p["classification"]} Pokémon. {p["height_inches"]//12}′{p["height_inches"]%12}″; '
                      f'{p["weight_tenths_lb"]/10:g} lb. Stats: ' + ' / '.join(str(p['base_stats'][key]) for key in STAT_KEYS) +
                      f' (total {sum(p["base_stats"].values())}).', '',
                      ' '.join(page.replace(' @ ', ' ') for page in p['dex_pages']), '']
            if p['evolution']:
                lines += [f'Evolves to {member["evolves_to"]} at level {p["evolution"]["level"]}.', '']
            lines += ['Level moves: ' + '; '.join(f'{level}: {move}' for level, move in p['level_moves']) + '.', '',
                      'TM/HM/tutor: ' + ', '.join(p['tmhm_moves']) + '.', '',
                      'Egg moves: ' + (', '.join(p['egg_moves']) or 'none') + '.', '',
                      f'Catch rate {p["catch_rate"]}; base EXP {p["base_exp"]}; '
                      f'{p["growth_rate"]}; egg groups ' + '/'.join(p['egg_groups']) +
                      f'; hatch cycles {p["hatch_cycles"]}.', '']
    (ROOT / 'POKEDEX.md').write_text('\n'.join(lines))


def write_pack_index(catalog):
    project = ROOT.parents[2]
    directory = project / 'content-packs/minnesota-pokemon-families'
    expected = {family['id'].lower() + '.crystalpack' for family in catalog['families']}
    actual = {path.name for path in directory.glob('*.crystalpack')}
    assert actual == expected, 'publish all complete family packs before indexing'
    aggregate = project / 'content-packs/minnesota-pokemon.crystalpack'
    paths = [aggregate] + sorted(directory.glob('*.crystalpack'))
    checksums = [hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + str(path.relative_to(project))
                 for path in paths]
    (project / 'content-packs/minnesota-pokemon.sha256').write_text('\n'.join(checksums) + '\n')
    lines = ['# Local Minnesota family packs', '',
             f'{len(paths)} compiled local packs: one aggregate plus {len(expected)} complete evolution families. '
             'Every family snapshot includes the original 251 species and its entire new family. '
             'Load one snapshot at a time; these are standalone packs.', '',
             f'[Aggregate: {251 + catalog["species_count"]} species](../../content-packs/minnesota-pokemon.crystalpack) · '
             '[Checksums](../../content-packs/minnesota-pokemon.sha256) · '
             '[Full Pokédex and moves](../../planning/minnesota/pokemon/POKEDEX.md)', '',
             'These links refer to generated local files, which stay ignored by Git.', '',
             '| Family | New species | Types | Local pack |',
             '| --- | --- | --- | --- |']
    for family in catalog['families']:
        names = ' → '.join(member['name'] for member in family['members'])
        types = ' → '.join('/'.join(member['types']) for member in family['members'])
        name = family['id'].lower() + '.crystalpack'
        lines.append(f'| {family["id"]} | {names} | {types} | [Pack](../../content-packs/minnesota-pokemon-families/{name}) |')
    (project / 'modpacks/minnesota-pokemon/PACKS.md').write_text('\n'.join(lines) + '\n')
    print(f'Indexed {len(paths)} compiled packs and wrote local SHA-256 checksums')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--metadata-only', action='store_true')
    parser.add_argument('--pack-index', action='store_true', help='index completed Rust publisher outputs')
    args = parser.parse_args()
    catalog, batch = create_metadata()
    cache = ROOT.parents[2] / 'target/minnesota-pokemon/species-metadata.json'
    cache.parent.mkdir(parents=True, exist_ok=True)
    cache.write_text(json.dumps(batch, indent=2) + '\n')
    if args.metadata_only:
        print(f'Authored metadata: {len(batch["species"])} species, 532 native dex pages')
        return
    moves = json.loads((ROOT / 'movesets.json').read_text())
    by_id = {row['id']: row for row in moves['species']}
    assert len(by_id) == len(batch['species'])
    for profile in batch['species']:
        row = by_id.pop(profile['id'])
        assert set(row) == {'id', 'level_moves', 'tmhm_moves', 'egg_moves'}
        profile.update({key: row[key] for key in ('level_moves', 'tmhm_moves', 'egg_moves')})
    assert not by_id
    (ROOT / 'species-profiles.json').write_text(json.dumps(batch, indent=2) + '\n')
    write_guide(catalog, batch)
    if args.pack_index:
        write_pack_index(catalog)
    print(f'Assembled {len(batch["species"])} full profiles, learnsets and Pokédex guide')


if __name__ == '__main__':
    main()
