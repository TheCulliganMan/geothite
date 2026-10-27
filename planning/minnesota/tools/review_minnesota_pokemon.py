#!/usr/bin/env python3
"""Check the original Minnesota design catalog; never reads or writes packs."""
import argparse
from collections import Counter
import json
import html
import hashlib
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'pokemon'
TYPES = {'NORMAL', 'FIRE', 'WATER', 'GRASS', 'ELECTRIC', 'ICE', 'FIGHTING',
         'POISON', 'GROUND', 'FLYING', 'PSYCHIC', 'BUG', 'ROCK', 'GHOST',
         'DRAGON', 'DARK', 'STEEL'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(catalog):
    require(catalog['schema_version'] == 2, 'expected design schema 2')
    total = catalog['species_count']
    require(total in (153,250,258,266), 'expected an approved 153, 250, 258 or 266-species milestone')
    expected = {153:(25,31,16,5,1),250:(41,52,23,7,2),258:(41,56,23,7,2),266:(43,57,23,7,2)}[total]
    mix_keys = ('three_stage_families','two_stage_families','single_stage_species','legendary_species','mythical_species')
    require(catalog['evolution_mix'] == dict(zip(mix_keys,expected)), 'declared evolution mix is stale')
    families = catalog['families']
    species = [s for f in families for s in f['members']]
    by_id = {s['id']: s for s in species}
    require(len(species) == total and set(by_id) == {f'MN_{i:03}' for i in range(1,total+1)}, f'exactly MN_001..MN_{total:03} required')
    require(len({s['name'].upper() for s in species}) == total, 'duplicate species name')
    require(all(s['name'].isascii() and 1 <= len(s['name']) <= 10 for s in species), 'names must fit ten native cells')
    require(len({f['id'] for f in families}) == len(families), 'duplicate family identity')
    require(Counter(len(f['members']) for f in families) == dict(zip((3,2,1),expected[:3])), 'evolution-family counts differ from the approved mix')
    require(Counter(f.get('rarity') for f in families if f.get('rarity')) == {'legendary':expected[3],'mythical':expected[4]}, 'legendary/mythical counts differ from the approved mix')
    require(all(not f.get('rarity') or len(f['members']) == 1 for f in families), 'rare species must stay single-stage')
    if total >= 250:
        base = [{k:f[k] for k in ('id','region','theme','members')}
                for f in families if int(f['id'][4:]) <= 72]
        fingerprint = hashlib.sha256(json.dumps(base,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        require(fingerprint == '9814d5d14902390cbedfbaf83348215255ccdf5dce94c194a86218108ede05d6', 'the approved original 153 identities/themes/types/evolutions changed')
    if total >= 258:
        checkpoint = [{k:f[k] for k in ('id','region','theme','members')}
                      for f in families if int(f['id'][4:]) <= 116]
        fingerprint = hashlib.sha256(json.dumps(checkpoint,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        require(fingerprint == 'cef26c30bc42b92ac9feb8466c0d9a3322ba33015e3e59ce47b4b7e60b17864b', 'the approved 250 identities/themes/types/evolutions changed')
    for f in families:
        require(f['theme'] and f['region'], f'{f["id"]}: missing theme or region')
        for index,s in enumerate(f['members']):
            require(1 <= len(s['types']) <= 2 and len(set(s['types'])) == len(s['types']) and set(s['types']) <= TYPES, f'{s["id"]}: unsupported/duplicate type')
            require(s['stage'] == index+1, f'{s["id"]}: wrong stage')
            target = f['members'][index+1]['id'] if index+1 < len(f['members']) else None
            require(s['evolves_to'] == target, f'{s["id"]}: evolution leaves family')
    require(by_id['MN_089']['name'] == 'Lutebrisk' and by_id['MN_089']['types'] == ['ICE'], 'preserve butter-filled pure Ice Lutebrisk')
    additions = Counter(f['region'] for f in families for s in f['members'] if 100 < int(s['id'][3:]) <= 153)
    require(additions == {'prairie':40,'iron_range':10,'peatlands':3}, 'expansion must contain 40 prairie, 10 Iron Range and 3 peatland species')
    pseudo = [f for f in families if f.get('classification') == 'pseudo_legendary']
    require(len(pseudo) == 1 and pseudo[0]['id'] == 'MN_F072' and not pseudo[0].get('rarity'), 'one separate mosquito pseudo-legendary family required')
    require(pseudo[0]['growth_rate'] == 'SLOW', 'pseudo-legendary must use Slow growth')
    require([s['name'] for s in pseudo[0]['members']] == ['Wrigglet', 'Pupump', 'Mosquitan'], 'preserve mosquito line')
    stat_keys = {'hp', 'attack', 'defense', 'speed', 'special_attack', 'special_defense'}
    for s,stat_total in zip(pseudo[0]['members'], [300,420,600]):
        stats = s['base_stats']
        require(set(stats) == stat_keys and all(isinstance(v,int) and 1 <= v <= 255 for v in stats.values()), 'six valid native base stats required')
        require(sum(stats.values()) == s['base_stat_total'] == stat_total, 'mosquito base-stat total mismatch')
    require([s.get('evolution_condition') for s in pseudo[0]['members']] == [{'method':'LEVEL','level':30},{'method':'LEVEL','level':55},None], 'mosquito must evolve at levels 30 and 55')
    if total >= 250:
        fossils = json.loads((ROOT/'fossil-choice.json').read_text())['options']
        require(len(fossils) == 2 and {f['family_id'] for f in fossils} == {'MN_F092','MN_F093'}, 'two separate fossil-choice families required')
        for fossil in fossils:
            require(fossil['item_name'].isascii() and len(fossil['item_name']) <= 12, 'fossil item must fit twelve native cells')
            family = next(f for f in families if f['id'] == fossil['family_id'])
            require(len(family['members']) == 2 and not family.get('rarity'), 'each fossil choice must have two ordinary evolutionary stages')
            for design,member in zip(fossil['species'],family['members']):
                require(all(design[k] == member[k] for k in ('id','name','types')), 'fossil-choice identities diverge from the roster')
            require(fossil['revives_species_id'] == family['members'][0]['id'], 'fossil must revive its first stage')
            require(family['fossil_item'] == fossil['item_name'] and family['members'][0]['evolution_condition'] == fossil['evolution'], 'fossil item/evolution condition diverges from its choice design')
    counts = Counter(t for s in species for t in s['types'])
    policy = catalog['type_balance_policy']
    require(set(counts) == TYPES and all(policy['minimum_species_per_type'] <= counts[t] <= policy['maximum_species_per_type'] for t in TYPES), 'type presence outside the declared coverage range')
    ordinary = {t:[f for f in families if not f.get('rarity') and any(t in s['types'] for s in f['members'])] for t in TYPES}
    require(all(len(fs) >= policy['minimum_ordinary_families_per_type'] for fs in ordinary.values()), 'a type depends on too few ordinary families')
    return counts, ordinary


def label(species):
    return species['name']+' — '+'/'.join(t.title() for t in species['types'])


def route_labels(family):
    return ', '.join(route['id'] if isinstance(route,dict) else route
                     for route in family.get('proposed_routes',[]))


def write_review(catalog, counts, ordinary):
    total, mix = catalog['species_count'], catalog['evolution_mix']
    ordinary_singles = mix['single_stage_species']-mix['legendary_species']-mix['mythical_species']
    summary = f"**{mix['three_stage_families']} three-stage families, {mix['two_stage_families']} two-stage families, {mix['single_stage_species']} single-stage species.** {mix['legendary_species']} legendaries and {mix['mythical_species']} mythical species; {ordinary_singles} ordinary singletons."
    rows = [f'# Minnesota {total} roster', '', 'Original design catalog. Names, types, themes and evolutionary relationships are authored; the mosquito family also has proposed stats and level evolutions. Runtime data and native art remain separate work.', '', summary, '', 'MN_101–MN_150 add forty prairie and ten Iron Range species; MN_151–MN_153 form the giant-mosquito pseudo-legendary line. The original 153 species retain their identities in the expansion.', '']
    for length,title in [(3,'Three-stage families'),(2,'Two-stage families'),(1,'Single-stage species')]:
        rows += [f'## {title}', '', '| IDs | Species and types | Theme and body transformation |', '| --- | --- | --- |']
        for f in catalog['families']:
            if len(f['members']) != length:
                continue
            ids = ', '.join(s['id'][3:] for s in f['members'])
            rare = ' **'+f['rarity'].title()+'**.' if f.get('rarity') else ''
            if f.get('classification') == 'pseudo_legendary':
                rare += ' **Pseudo-legendary**; Slow growth; evolves at levels 30 and 55; base-stat totals 300 → 420 → 600.'
            location = ' Proposed location: '+f['location']+'.' if f.get('location') else ''
            rows.append('| '+ids+' | '+' → '.join(label(s) for s in f['members'])+' | '+f['theme']+rare+location+' |')
        rows.append('')
    rows += ['## Grounding', '', 'Regional sources ground the inspirations; creature powers, anatomy and discovery scenes are original fiction.', '']
    rows += ['- ['+s['title']+']('+s['url']+'): '+s['scope'] for s in catalog['sources']]
    (ROOT/'ROSTER.md').write_text('\n'.join(rows)+'\n')
    rows = ['# Type coverage', '', 'Counts include both types of dual-type species. This is coverage across the roster, not a claim that matchups, moves or progression are balanced.', '', f'| Type | Species / {total} | Independent ordinary families |', '| --- | ---: | ---: |']
    rows += [f'| {t.title()} | {counts[t]} | {len(ordinary[t])} |' for t in sorted(TYPES)]
    policy = catalog['type_balance_policy']
    rows += ['', f"Every type appears in {policy['minimum_species_per_type']}–{policy['maximum_species_per_type']} species and at least {policy['minimum_ordinary_families_per_type']} nonlegendary families. No type is confined to a legendary encounter.", '', 'Prairie plants use varied typing: pasqueflower thaw-seers use Ice/Psychic, while milkweed latex/parachute creatures use Poison/Flying. The magnetic taconite wyrm and prairie rope-dragon introduce ordinary Dragon families. Mine fumes, milkweed and living wetland filters broaden Poison coverage.', '', 'The original 153 retain their names, types and evolutionary families, including pure Ice Lutebrisk and the giant-mosquito pseudo-legendary. Ordinary availability, early/mid/late distribution, abilities, move coverage and encounter registration are not asserted by this authoring check.', '']
    (ROOT/'COVERAGE.md').write_text('\n'.join(rows))



def native_png(path):
    """Read the Rust exporter's RGB8, filter-zero PNGs independently."""
    data = path.read_bytes()
    require(data[:8] == b'\x89PNG\r\n\x1a\n', f'{path}: PNG signature')
    offset, payloads, header = 8, [], None
    while offset < len(data):
        count = struct.unpack_from('>I', data, offset)[0]
        kind = data[offset+4:offset+8]
        chunk = data[offset+8:offset+8+count]
        crc = struct.unpack_from('>I', data, offset+8+count)[0]
        require(zlib.crc32(kind+chunk) == crc, f'{path}: PNG CRC')
        if kind == b'IHDR':
            header = struct.unpack('>IIBBBBB', chunk)
        elif kind == b'IDAT':
            payloads.append(chunk)
        elif kind == b'IEND':
            break
        offset += count+12
    require(header is not None and header[2:] == (8,2,0,0,0), f'{path}: RGB8 PNG required')
    width,height = header[:2]
    raw = zlib.decompress(b''.join(payloads))
    stride = width*3+1
    require(len(raw) == height*stride, f'{path}: PNG byte count')
    pixels = []
    for y in range(height):
        require(raw[y*stride] == 0, f'{path}: expected native exporter scanline filter')
        row = raw[y*stride+1:(y+1)*stride]
        pixels += [tuple(row[i:i+3]) for i in range(0,len(row),3)]
    return width,height,pixels


def review_sprites(catalog, require_all, registry=None, publish=True):
    """Independently verify exports, optionally without changing published views."""
    if registry is None:
        registry = json.loads((ROOT/'sprite-generation.json').read_text())
    require(registry['schema_version'] == 3, 'sprite registry schema 3 required')
    families = {f['id']:f for f in catalog['families']}
    palette_plans = json.loads((ROOT/'palettes.json').read_text())['pokemon']
    project = ROOT.parents[2]
    completed, frames, family_views = set(), {}, {}
    family_artifacts = {}
    filenames = set()
    seen_families = set()
    for reference in registry['families']:
        fid = reference['id']
        require(fid in families and fid not in seen_families, f'{fid}: unknown/duplicate art family')
        seen_families.add(fid)
        source = Path(reference['manifest'])
        require(source.parts[0] == 'sprites' and '..' not in source.parts, 'manifest outside sprites')
        manifest = json.loads((ROOT/source).read_text())
        family_artifacts[fid] = (source, manifest)
        require(manifest['family_id'] == fid, f'{fid}: manifest identity')
        members = {s['id'] for s in families[fid]['members']}
        require(manifest.get('prompt') or (ROOT/manifest['prompt_file']).is_file(), f'{fid}: missing generation prompt')
        species_palettes = {}
        outline_profiles = set()
        views = {}
        out = project/'target/minnesota-pokemon/native'/fid
        native_sheet = reference.get('native_sheet','line.native.png')
        review_sheet = reference.get('review_sheet','line.review.png')
        require(all(Path(name).name == name and name.endswith('.png') for name in (native_sheet,review_sheet)), f'{fid}: invalid candidate sheet filename')
        sw,sh,sheet_pixels = native_png(out/native_sheet)
        require((sw,sh) == (len(members)*56,112), f'{fid}: contact sheet dimensions')
        for sprite in manifest['sprites']:
            sid,view = sprite['species_id'],sprite['view']
            require(sid in members and view in ('front','back') and (sid,view) not in frames, f'{fid}: invalid/duplicate view')
            key = (fid,sprite['id'],view)
            require(key not in filenames, f'{sid}: duplicate output filename')
            filenames.add(key)
            art = Path(sprite['source'])
            require(art.parts[0] == 'sprites' and '..' not in art.parts and (ROOT/art).is_file(), f'{sid}: missing owned source')
            palette = sprite['normal_palette_rgb555']
            require(len(set(palette)) == 4 and palette[0] == 32767 and palette[3] == 0 and all(0 <= v <= 32767 for v in palette), f'{sid}: RGB555 palette')
            plan = palette_plans[sid]
            colors = plan['gbc_rgb5']
            require(len(colors) == 4 and all(len(c) == 3 and all(isinstance(v,int) and 0 <= v <= 31 for v in c) for c in colors), f'{sid}: invalid GBC RGB5 definition')
            require([c[0] | c[1]<<5 | c[2]<<10 for c in colors] == palette == plan['gbc_rgb555_words'] == manifest['pokemon_palettes'][sid]['gbc_rgb555_words'], f'{sid}: canonical palette mismatch')
            expected_rgb = [[(v<<3)|(v>>2) for v in c] for c in colors]
            require(plan['rgb8'] == expected_rgb and [h.upper() for h in plan['hex']] == ['#'+''.join(f'{v:02X}' for v in c) for c in expected_rgb], f'{sid}: palette RGB8/hex definitions disagree')
            expected_palette = species_palettes.setdefault(sid,palette)
            require(palette == expected_palette, f'{sid}: front and back palettes must match')
            rgb = [tuple((((word >> shift)&31)<<3)|(((word >> shift)&31)>>2) for shift in (0,5,10)) for word in palette]
            canvas = 56 if view == 'front' else 48
            out = project/'target/minnesota-pokemon/native'/fid
            stem = str(out/(sprite['id']+'.'+view))
            w,h,pixels = native_png(Path(stem+'.png'))
            data = Path(stem+'.2bpp').read_bytes()
            indices = json.loads(Path(stem+'.json').read_text())
            outline_profiles.add(indices.get('outline_profile', 'legacy-area-vote'))
            require(indices['source_sha256'] == hashlib.sha256((ROOT/art).read_bytes()).hexdigest() and indices['import_crop'] == sprite['crop'] and indices['import_fit'] == sprite['fit'], f'{sid}: stale export; rebuild this family')
            require(indices.get('quantization_palette_rgb555', palette) == sprite.get('source_palette_rgb555', palette), f'{sid}: stale source palette mapping')
            require(indices.get('white_detail_min_samples', 16) == sprite.get('white_detail_min_samples', 16), f'{sid}: stale white detail threshold')
            require(indices.get('interior_black_min_samples', 32) == sprite.get('interior_black_min_samples', 32), f'{sid}: stale interior black threshold')
            require(indices.get('mirror_horizontal', False) == sprite.get('mirror_horizontal', False), f'{sid}: stale native orientation')
            for field in ('source_archive', 'crop_archive'):
                archive = Path(indices[field])
                require(not archive.is_absolute() and '..' not in archive.parts and archive.parts[:3] == ('target','minnesota-pokemon','native') and 'versions' in archive.parts, f'{sid}: invalid immutable archive path')
                saved = project/archive
                require(saved.is_file() and saved.name.split('.')[0] == hashlib.sha256(saved.read_bytes()).hexdigest(), f'{sid}: missing/corrupt archived {field}')
            require((project/indices['source_archive']).read_bytes() == (ROOT/art).read_bytes(), f'{sid}: archived original differs from source')
            cw,ch,_ = native_png(project/indices['crop_archive'])
            require([cw,ch] == sprite['crop'][2:], f'{sid}: archived crop resolution changed')
            for extension in ('png','json','2bpp','review.png'):
                latest = Path(stem+'.'+extension)
                digest = hashlib.sha256(latest.read_bytes()).hexdigest()
                require((out/'versions'/(digest+'.'+latest.name)).is_file(), f'{sid}: missing immutable {extension} version')
            require((w,h) == (canvas,canvas) and indices['size'] == [canvas,canvas] and indices['view'] == view, f'{sid}: native geometry/view')
            require(indices['palette_rgb555'] == palette and len(data) == canvas*canvas//4, f'{sid}: native bytes/palette')
            require(set(pixels) == set(rgb), f'{sid}: expected exact four RGB555 colors')
            require(len(indices['rows']) == canvas and all(len(row) == canvas for row in indices['rows']), f'{sid}: source grid shape')
            column = sorted(members).index(sid)
            left = column*56+(56-canvas)//2
            top = 60 if view == 'back' else 0
            for y in range(canvas):
                for x in range(canvas):
                    require(sheet_pixels[(top+y)*sw+left+x] == pixels[y*canvas+x], f'{sid}: contact sheet differs from native frame')
                    tile = (y//8)*(canvas//8)+x//8 if view == 'front' else (x//8)*(canvas//8)+y//8
                    offset = tile*16+(y%8)*2
                    bit = 7-x%8
                    index = ((data[offset]>>bit)&1)|(((data[offset+1]>>bit)&1)<<1)
                    require(pixels[y*canvas+x] == rgb[index] and indices['rows'][y][x] == '.123'[index], f'{sid}: 2bpp/grid/PNG mismatch')
                    if x in (0,canvas-1) or y == 0 or (view == 'front' and y == canvas-1):
                        require(index == 0, f'{sid}: clipped native border')
            frames[sid,view] = sprite
            views[sid,view] = True
        require(set(views) == {(sid,v) for sid in members for v in ('front','back')}, f'{fid}: registered family requires every front AND back')
        review = manifest.get('visual_review',{})
        individual_palettes = len({tuple(p) for p in species_palettes.values()}) == len(members)
        current_outline = outline_profiles == {'source-contour-v1'}
        reviewed = current_outline and review.get('outline_profile') == 'source-contour-v1' and individual_palettes and manifest.get('back_framing') == 'gen2-close-rear-portrait' and review.get('status') == 'reviewed' and review.get('sheet_sha256') == hashlib.sha256((out/review_sheet).read_bytes()).hexdigest()
        if reviewed:
            completed.update(members)
        family_views[fid] = (reviewed,len(members),individual_palettes,current_outline)
    total = catalog['species_count']
    if not publish:
        print(f'PASS: {len(frames)} candidate native views; {len(completed)}/{total} candidate-family species visually reviewed. Published gallery unchanged.')
        if require_all:
            require(len(completed) == total and len(frames) == total*2, 'candidate visual reviews remain incomplete')
        return
    rows = ['# Sprite progress', '', f'**{len(completed)} / {total} species have native front/back pairs with visual review.**', '', 'Each Pokémon requires its own four-color GBC palette; only its front and back share it.', '', f'{len(frames)} / {total*2} native views verified independently against their palette-index JSON and 2bpp tiles. Fronts are 56×56 / 784 bytes; backs 48×48 / 576 bytes. Runtime packs are validated separately; the Rust species compiler supplies static frame-0 animation programs.', '', '| Family | Species | Front/back status |', '| --- | --- | --- |']
    gallery = ['<!doctype html><meta charset="utf-8"><title>Minnesota sprite progress</title><style>body{font:16px system-ui;background:#edf0ed;color:#13251d;margin:28px}article{background:white;padding:20px;margin:20px 0;border-radius:12px}img{image-rendering:pixelated;max-width:100%;height:auto}small{color:#536159}</style>', f'<h1>Minnesota sprites: {len(completed)} / {total} pairs reviewed</h1><p>Front row above, separately drawn back row below. Every exported view is palette and 2bpp verified.</p>']
    for f in catalog['families']:
        names = ' → '.join(s['name'] for s in f['members'])
        state = 'pending'
        if f['id'] in family_views:
            source, manifest = family_artifacts[f['id']]
            state = 'native verified; visually reviewed' if family_views[f['id']][0] else 'native verified; visual review pending'
            if not family_views[f['id']][2]:
                state = 'native pairs exist; individual palette revision pending'
            if not family_views[f['id']][3] or manifest.get('visual_review',{}).get('status') == 'outline-revision-pending':
                state = 'native verified; outline revision/review pending'
            if manifest.get('back_framing') != 'gen2-close-rear-portrait':
                state = 'native verified; close back portraits pending'
            links = [f'<a href="../../../planning/minnesota/pokemon/{html.escape(str(source),quote=True)}">Authoring manifest</a>']
            for raw in sorted({s['source'] for s in manifest['sprites']}):
                links.append(f'<a href="../../../planning/minnesota/pokemon/{html.escape(raw,quote=True)}" download>Original {html.escape(Path(raw).name)}</a>')
            for sprite in manifest['sprites']:
                stem = f'../native/{f["id"]}/{sprite["id"]}.{sprite["view"]}'
                links.append(f'<span>{sprite["species_id"]} {sprite["view"]}: <a href="{stem}.png" download>PNG</a> · <a href="{stem}.2bpp" download>2bpp</a> · <a href="{stem}.json">Provenance</a></span>')
            description = html.escape(f['theme'])
            if f.get('rarity'):
                description += ' '+html.escape(f['rarity'].title())+'.'
            if f.get('classification') == 'pseudo_legendary':
                description += ' Pseudo-legendary.'
            if f.get('location'):
                description += ' Proposed location: '+html.escape(f['location'])+'.'
            if f.get('proposed_routes'):
                description += ' Proposed habitats: '+html.escape(route_labels(f))+'.'
            type_labels = html.escape(' → '.join(label(s) for s in f['members']))
            gallery += [f'<article><h2>{html.escape(names)}</h2><small>{f["id"]}: {state}</small><p>{type_labels}</p><p>{description}</p><p><img loading="lazy" decoding="async" src="../../native/{f["id"]}/line.review.png" alt="{html.escape(names)} front and back sprites"></p><details><summary>Saved sources and native exports</summary><p>'+ '<br>'.join(links)+'</p></details></article>']
        else:
            description = html.escape(f['theme'])
            if f.get('location'):
                description += ' Proposed location: '+html.escape(f['location'])+'.'
            if f.get('proposed_routes'):
                description += ' Proposed habitats: '+html.escape(route_labels(f))+'.'
            type_labels = html.escape(' → '.join(label(s) for s in f['members']))
            gallery.append(f'<article><h2>{html.escape(names)}</h2><small>{f["id"]}: artwork pending</small><p>{type_labels}</p><p>{description}</p></article>')
        rows.append(f'| {f["id"]} | {names} | {state} |')
    (ROOT/'SPRITE-PROGRESS.md').write_text('\n'.join(rows)+'\n')
    gallery_path = project/'target/minnesota-pokemon/gallery/index.html'
    gallery_path.parent.mkdir(parents=True,exist_ok=True)
    # The gallery lives one directory below the native directory's parent.
    gallery_path.write_text('\n'.join(gallery).replace('../../native/','../native/'))
    print(f'PASS: {len(frames)} native views; {len(completed)}/{total} front/back pairs reviewed. Gallery: {gallery_path}')
    if require_all:
        require(len(completed) == total and len(frames) == total*2, f'full {total}-species sprite goal remains incomplete')


def register_manifest(relative_path):
    """Publish one authored family's metadata; this never grants review credit."""
    path = ROOT/relative_path
    require(relative_path.startswith('sprites/') and path.resolve().is_relative_to(ROOT.resolve()), 'manifest must stay in owned sprites directory')
    manifest = json.loads(path.read_text())
    catalog = json.loads((ROOT/'roster.json').read_text())
    family = next((f for f in catalog['families'] if f['id'] == manifest['family_id']),None)
    require(family is not None, 'manifest family is not in the locked roster')
    members = {s['id']:s for s in family['members']}
    require(set(manifest['pokemon_palettes']) == set(members), 'manifest palette members differ from the locked family')
    require({(s['species_id'],s['view']) for s in manifest['sprites']} == {(sid,v) for sid in members for v in ('front','back')}, 'every member needs both authored views')
    registry_path, palette_path = ROOT/'sprite-generation.json', ROOT/'palettes.json'
    registry, palettes = json.loads(registry_path.read_text()), json.loads(palette_path.read_text())
    for sid,palette in manifest['pokemon_palettes'].items():
        require(palette['species_id'] == sid and palette['name'] == members[sid]['name'], 'palette identity differs from the locked roster')
        if sid in palettes['pokemon']:
            require(palettes['pokemon'][sid] == palette, 'registering a family cannot silently replace a palette plan')
        palettes['pokemon'][sid] = palette
    existing = next((f for f in registry['families'] if f['id'] == family['id']),None)
    if existing:
        require(existing['manifest'] == relative_path, 'family already uses another manifest')
    else:
        registry['families'].append({'id':family['id'],'manifest':relative_path})
    registry['families'].sort(key=lambda f:f['id'])
    palettes['pokemon'] = dict(sorted(palettes['pokemon'].items()))
    registry_path.write_text(json.dumps(registry,indent=2)+'\n')
    palette_path.write_text(json.dumps(palettes,indent=2)+'\n')


def record_visual_review(family_id, note):
    """Record an explicit review of the current sheet; changed pixels invalidate it."""
    require(note, 'a visual review note is required')
    registry = json.loads((ROOT/'sprite-generation.json').read_text())
    reference = next((f for f in registry['families'] if f['id'] == family_id), None)
    require(reference is not None, 'unknown review family')
    path = ROOT/reference['manifest']
    manifest = json.loads(path.read_text())
    preview = ROOT.parents[2]/'target/minnesota-pokemon/native'/family_id/'line.review.png'
    manifest['status'] = 'native-front-back-reviewed'
    manifest['visual_review'] = {'status':'reviewed', 'reviewer':'root', 'notes':note, 'sheet_sha256':hashlib.sha256(preview.read_bytes()).hexdigest(), 'outline_profile':'source-contour-v1'}
    path.write_text(json.dumps(manifest,indent=2)+'\n')


def write_candidate_gallery(catalog, references):
    """Show new wildlife and saved redesign comparisons in a separate gallery."""
    project = ROOT.parents[2]
    families = {f['id']:f for f in catalog['families']}
    registered = {f['id']:f for f in json.loads((ROOT/'sprite-generation.json').read_text())['families']}
    rows = ['<!doctype html><meta charset="utf-8"><title>Minnesota new wildlife and redesigns</title>',
            '<style>body{font:16px system-ui;background:#eef2ee;color:#183026;margin:28px;max-width:1500px}article{background:white;padding:22px;margin:24px 0;border-radius:12px}img{image-rendering:pixelated;max-width:100%;height:auto}.compare{display:grid;gap:24px;grid-template-columns:1fr 1fr}small{color:#536159}@media(max-width:800px){.compare{grid-template-columns:1fr}}</style>',
            f'<h1>New wildlife and six redesigns</h1><p><a href="index.html">Full {catalog["species_count"]}-species gallery</a>. Front views above; separately drawn close backs below. Every individual uses its own GBC palette.</p>',
            '<h2>New fish, flying squirrel and warning-rattle snake</h2>']
    for fid,family in families.items():
        if int(fid[4:]) <= 116 or fid not in registered:
            continue
        names = ' → '.join(label(s) for s in family['members'])
        source = registered[fid]['manifest']
        rows += [f'<article><h3>{html.escape(names)}</h3><p>{html.escape(family["theme"])}</p><img src="../native/{fid}/line.review.png" alt="{html.escape(names)} native fronts and backs"><p><a href="../../../planning/minnesota/pokemon/{html.escape(source,quote=True)}">Saved manifest and sources</a></p></article>']
    rows.append('<h2>Redesign comparisons</h2><p>Original renders and all tested alternatives are retained. These candidate variants have separate native files and visual-review records.</p>')
    for reference in references:
        fid = reference['id']
        manifest = json.loads((ROOT/reference['manifest']).read_text())
        before = manifest['comparison_before']
        require(Path(before).name == before and (project/'target/minnesota-pokemon/native'/fid/before).is_file(), 'missing saved comparison baseline')
        names = ' → '.join(s['name'] for s in families[fid]['members'])
        rows += [f'<article><h3>{html.escape(names)}</h3><div class="compare"><section><h4>Before</h4><img src="../native/{fid}/{html.escape(before,quote=True)}" alt="Original {html.escape(names)}"></section><section><h4>Redesign</h4><img src="../native/{fid}/{html.escape(reference["review_sheet"],quote=True)}" alt="Redesigned {html.escape(names)}"></section></div><p>{html.escape(manifest["visual_review"]["notes"])}</p>',
                 f'<details><summary>Saved prompt, sources and native exports</summary><p><a href="../../../planning/minnesota/pokemon/{html.escape(reference["manifest"],quote=True)}">Candidate manifest</a> · <a href="../../../planning/minnesota/pokemon/{html.escape(manifest["prompt_file"],quote=True)}">Exact prompt</a></p>']
        for raw in sorted({s['source'] for s in manifest['sprites']}):
            rows.append(f'<p><a href="../../../planning/minnesota/pokemon/{html.escape(raw,quote=True)}">Original {html.escape(Path(raw).name)}</a></p>')
        for sprite in manifest['sprites']:
            stem = f'../native/{fid}/{sprite["id"]}.{sprite["view"]}'
            rows.append(f'<p>{sprite["species_id"]} {sprite["view"]}: <a href="{stem}.png">PNG</a> · <a href="{stem}.2bpp">2bpp</a> · <a href="{stem}.json">Palette and provenance</a></p>')
        rows.append('</details></article>')
    path = project/'target/minnesota-pokemon/gallery/redesigns.html'
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text('\n'.join(rows)+'\n')
    print(f'Gallery: {path}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-review', action='store_true')
    parser.add_argument('--sprites', action='store_true', help='verify exported pairs and refresh the progress gallery')
    parser.add_argument('--require-all-sprites', action='store_true', help='require every catalog species to have visually reviewed front/back pairs')
    parser.add_argument('--review-family', help='record a visual inspection of the current exported family sheet')
    parser.add_argument('--review-note', help='concrete observations from the visual inspection')
    parser.add_argument('--register-manifest', help='register an authored family manifest and its individual palette plans')
    parser.add_argument('--candidate-manifest', action='append', default=[], help='verify a redesign manifest and its candidate_exports sheets without publishing; repeat for multiple families')
    parser.add_argument('--write-candidate-gallery', action='store_true', help='write a separate new-wildlife/redesign gallery after candidate verification')
    args = parser.parse_args()
    require(not args.write_candidate_gallery or args.candidate_manifest, 'candidate gallery needs candidate manifests')
    catalog = json.loads((ROOT/'roster.json').read_text())
    counts, ordinary = validate(catalog)
    if args.write_review:
        write_review(catalog, counts, ordinary)
    if args.register_manifest:
        register_manifest(args.register_manifest)
    if args.review_family:
        record_visual_review(args.review_family,args.review_note)
    if args.candidate_manifest:
        references = []
        selected = []
        for relative_path in args.candidate_manifest:
            path = ROOT/relative_path
            require(relative_path.startswith('sprites/') and path.resolve().is_relative_to(ROOT.resolve()), 'candidate manifest outside sprites')
            manifest = json.loads(path.read_text())
            family = next((f for f in catalog['families'] if f['id'] == manifest['family_id']),None)
            require(family is not None, 'candidate family is not in the catalog')
            sheets = manifest['candidate_exports']
            require(set(sheets) == {'native_sheet','review_sheet'}, 'candidate requires exactly its native/review sheet filenames')
            references.append({'id':family['id'],'manifest':relative_path,**sheets})
            selected.append(family)
        candidate_catalog = {'families':selected,'species_count':sum(len(f['members']) for f in selected)}
        review_sprites(candidate_catalog,args.require_all_sprites,{'schema_version':3,'families':references},publish=False)
        if args.write_candidate_gallery:
            write_candidate_gallery(catalog,references)
    elif args.sprites or args.require_all_sprites or args.review_family or args.register_manifest:
        review_sprites(catalog, args.require_all_sprites)
    mix, policy = catalog['evolution_mix'], catalog['type_balance_policy']
    print(f"PASS: {catalog['species_count']} species; {mix['three_stage_families']} triples / {mix['two_stage_families']} pairs / {mix['single_stage_species']} singletons; {mix['legendary_species']} legendaries / {mix['mythical_species']} mythical species.")
    print(f"PASS: all 17 types occur in {policy['minimum_species_per_type']}–{policy['maximum_species_per_type']} species and at least {policy['minimum_ordinary_families_per_type']} ordinary families. This is an authoring check, not runtime acceptance.")


if __name__ == '__main__':
    main()
