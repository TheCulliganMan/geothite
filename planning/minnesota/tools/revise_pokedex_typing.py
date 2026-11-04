#!/usr/bin/env python3
"""Apply roster typing to an existing Pokédex release without changing its roster."""
import html
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys


def once(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f'Expected exactly one occurrence: {old[:100]}')
    return text.replace(old, new, 1)


def main():
    source, output = map(Path, sys.argv[1:])
    if output.exists() or not source.is_dir():
        raise ValueError('Provide an existing release and a fresh output directory')
    subprocess.run(['git', 'check-ignore', '--quiet', '--no-index', str(output)], check=True)
    if any(p.is_symlink() for p in source.rglob('*')):
        raise ValueError('Source release must not contain symlinks')
    catalog = json.loads((source / 'pokedex.json').read_text())
    roster = json.loads(Path('planning/minnesota/pokemon/roster.json').read_text())
    families = {f['id']: f for f in roster['families']}
    members = {m['id']: m for family in roster['families'] for m in family['members']}
    changes = {}
    for species in catalog['species']:
        identity = species['identity']
        if identity['id'] in members:
            desired = members[identity['id']]['types']
            if desired != identity['types']:
                changes[identity['id']] = (identity['types'], desired)
    expected = {'MN_049', 'MN_139', 'MN_145', 'MN_215'}
    if set(changes) != expected:
        raise ValueError(f'Unexpected release or roster differences: {set(changes) ^ expected}')
    groups = {}
    for species in catalog['species']:
        identity = species['identity']
        new = changes.get(identity['id'], (None, identity['types']))[1]
        groups.setdefault(species['family'], []).append((identity['stage'], identity['name'], new))
    for family, line in groups.items():
        line.sort()
        types = [set(item[2]) for item in line]
        if sum(a != b for a, b in zip(types, types[1:])) > 1 or any(not a.issubset(b) for a, b in zip(types, types[1:])):
            raise ValueError(f'{family}: typing continuity failed')
    shutil.copytree(source, output)
    path = output / 'index.html'
    index = path.read_text()
    badges = {kind.upper(): badge for badge, kind in re.findall(r'(<span class="type-badge" aria-label="([A-Za-z]+) type"[^>]*>[^<]+</span>)', index)}
    for species in catalog['species']:
        identity = species['identity']
        if identity['id'] not in changes:
            continue
        old, new = changes[identity['id']]
        identity['types'] = new
        pattern = r'<article class="pokemon-card" data-id="' + re.escape(identity['id']) + r'".*?</article>'
        match = re.search(pattern, index, flags=re.S)
        if not match:
            raise ValueError(f'Missing card: {identity["id"]}')
        card = match.group()
        card = once(card, 'data-types="' + ' '.join(old) + '"', 'data-types="' + ' '.join(new) + '"')
        card = once(card, 'data-search="' + identity['name'] + ' ' + ' '.join(old) + ' ', 'data-search="' + identity['name'] + ' ' + ' '.join(new) + ' ')
        old_badges, new_badges = (''.join(badges[t] for t in group) for group in (old, new))
        card = once(card, '<div class="type-badges">' + old_badges + '</div>', '<div class="type-badges">' + new_badges + '</div>')
        index = index[:match.start()] + card + index[match.end():]
        page_path = output / 'species' / (identity['name'].lower() + '.html')
        page = page_path.read_text()
        page = once(page, '<div class="entry-types">' + old_badges + '</div>', '<div class="entry-types">' + new_badges + '</div>')
        page_path.write_text(page)
    for family_id in ('MN_F064',):
        line = [s for s in catalog['species'] if s['family'] == family_id]
        old, new = line[0]['theme'], families[family_id]['theme']
        old_html, new_html = html.escape(old, quote=True), html.escape(new, quote=True)
        if index.count(old_html) != len(line) + 1:
            raise ValueError(f'Unexpected search text for {family_id}')
        index = index.replace(old_html, new_html)
        for species in line:
            species['theme'] = new
            page_path = output / 'species' / (species['identity']['name'].lower() + '.html')
            page = page_path.read_text()
            page_path.write_text(once(page, old_html, new_html))
    path.write_text(index)
    (output / 'pokedex.json').write_text(json.dumps(catalog, ensure_ascii=False, separators=(',', ':')))
    old_base = '/pokedex/releases/' + source.name
    new_base = '/pokedex/releases/' + output.name
    for file in output.rglob('*'):
        if file.is_file() and file.suffix in {'.html', '.json', '.js', '.css', '.svg'}:
            text = file.read_text()
            if old_base in text:
                file.write_text(text.replace(old_base, new_base))
    print(f'Updated {len(changes)} species; checked {len(catalog["species"])} species in {len(groups)} families')


if __name__ == '__main__':
    main()
