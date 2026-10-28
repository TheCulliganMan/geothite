#!/usr/bin/env python3
"""Read-only native R11 warm-restoration proposal, proof and source-art review.

Only --freeze-proposal writes the detached authoring bundle. Neither mode can
write a crystalpack, canonical route, game program, catalog or runtime asset.
"""
import argparse
import base64
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import sys

from PIL import Image

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from audit_outdoor_connections import collision
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_major_city import ReviewArt, player_camera, render_map

BASE_SHA = '3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
WARM_SHA = '2b2887fd056d507d85ac5d2b8861e92559d8679cf11902f63c42f270d617842d'
WARM_PACK = ROOT / 'content-packs/minnesota-winter-mixed-art-v01.crystalpack'
COLD = 'mn_winter_north_shore'
WARM = 'mn_north_shore'
NAMES = [f'MnR11C{c:02}R{r:02}' for c in range(2) for r in range(4)]
LAYOUT = ROOT / 'planning/minnesota/routes/MN_R11.layout.json'
DOSSIER = ROOT / 'planning/minnesota/routes/MN_R11.json'
V03 = ROOT / 'planning/minnesota/regions/ice/sections/source-v03'
SOURCE = HERE / 'source-v01'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def write(path, value):
    assert not path.exists(), ('Retain all proposal versions', path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def whole(pack):
    blocks = [None] * (56 * 96)
    objects = []
    for name in NAMES:
        m = pack['data']['maps'][name]
        column, row = map(int, re.fullmatch(r'MnR11C(\d+)R(\d+)', name).groups())
        a = m['attributes']
        for y in range(a['height']):
            i = (row * 24 + y) * 56 + column * 32
            blocks[i:i + a['width']] = m['blocks'][y*a['width']:(y+1)*a['width']]
        for o in m['objects']:
            objects.append(dict(id=o.get('object_identifier') or o['script'], sprite=o['sprite'],
                tile=[column*64+o['x'], row*48+o['y']],
                facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(
                    o['spritemovedata'].rsplit('_', 1)[-1], 'south'),
                native_palette_override=o.get('pal', 0)))
    assert None not in blocks
    return blocks, objects


def freeze(base, warm):
    SOURCE.mkdir(parents=True, exist_ok=False)
    rows = []
    for name in NAMES:
        m, old = base['data']['maps'][name], warm['data']['maps'][name]
        column, row = map(int, re.fullmatch(r'MnR11C(\d+)R(\d+)', name).groups())
        a = m['attributes']
        cells = []
        for i, (before, after) in enumerate(zip(m['blocks'], old['blocks'])):
            if before == after:
                continue
            x, y = i % a['width'], i // a['width']
            cells.append(dict(metatile=[x,y], source_metatile=[column*32+x,row*24+y],
                before=before, after=after,
                before_quadrants=base['data']['tilesets'][a['tileset_name']]['collision'][f'{before:02x}'],
                after_quadrants=warm['data']['tilesets'][WARM]['collision'][f'{after:02x}']))
        attr = dict(a, tileset_name=WARM)
        assert dict(m, blocks=old['blocks'], attributes=attr) == old
        rows.append(dict(map=name, dimensions_metatiles=[a['width'],a['height']],
            module_sha256=sha(canonical(m)), warm_reference_module_sha256=sha(canonical(old)),
            old_tileset=a['tileset_name'], new_tileset=WARM,
            blocks_label=a['blocks_label'], old_blocks_sha256=sha(bytes(m['blocks'])),
            new_blocks_sha256=sha(bytes(old['blocks'])), cells=cells))
    assert [len(r['cells']) for r in rows] == [499,329,0,0,0,0,0,0]
    current = json.loads(LAYOUT.read_bytes())
    original = json.loads((V03/'MN_R11.layout.json.before').read_bytes())
    assert current['layers'][:218] == original['layers'] and len(current['layers']) == 1331
    assert layout_blocks(current) == whole(base)[0]
    assert all(l['name'] == 'Preserve exact original fitted native shoreline/cap'
               for l in current['layers'][218:503])
    assert Counter(l['name'] for l in current['layers'][503:]) == {
        'Continuous winter section, revised geographic snowline and safe fork stops':820,
        'Exact native mixed winter edge: collision and canopy/wood shapes preserved':6,
        'Exact cold-roof native canopy, facade pixels and original collisions retained':2}
    candidate = deepcopy(current)
    candidate['layers'] = candidate['layers'][:503]
    candidate['compiled_tileset'] = WARM
    candidate.pop('winter_section_review')
    review = dict(status='PROPOSED_SOURCE_ONLY_NO_NATIVE_PUBLICATION_OR_CONTROLLER',
        instruction='North Shore and Grand Marais stay warm on Lake Superior; winter moves inland near Ely.',
        reference_pack_sha256=WARM_SHA, preimage_baseline_pack_sha256=BASE_SHA,
        preserved='All current native terrain except exact828 winter cells; actors, programs, ports, falls, stairs, ledges, buoys and art exact.',
        source_policy='Keep original218 authored strokes and all285 explicit native fitted shore/cap overrides; remove only828 winter/roof tail points.',
        publication_identity=None)
    candidate['warm_coast_restoration_review'] = review
    assert layout_blocks(candidate) == whole(warm)[0]
    dossier = json.loads(DOSSIER.read_bytes())
    candidate_dossier = deepcopy(dossier)
    candidate_dossier['compiled_tileset'] = WARM
    candidate_dossier.pop('winter_section_review')
    candidate_dossier['warm_coast_restoration_review'] = review
    for path, data in [(SOURCE/'MN_R11.warm.candidate.layout.json',candidate),
                       (SOURCE/'MN_R11.warm.candidate.json',candidate_dossier)]:
        write(path, data)
    inputs = [LAYOUT, DOSSIER, V03/'integration.contract.json',
        V03/'MN_R11.layout.json.before',
        ROOT/'planning/minnesota/regions/ice/roof-fidelity-v01/roof-fidelity.contract.json']
    contract = dict(schema_version=1, id='MN_R11_WARM_COAST_RESTORATION',
        status='READ_ONLY_PROPOSAL_NOT_APPLIED', source_preimage_pack_sha256=BASE_SHA,
        source_preimage_identity=base['identity']['content_hash'], warm_reference_pack_sha256=WARM_SHA,
        warm_reference_identity=warm['identity']['content_hash'], expected_map_count=683,
        source_files_sha256={str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in inputs},
        candidates_sha256={p.name:sha(p.read_bytes()) for p in SOURCE.glob('*.json')},
        changed_metatiles=828, changed_maps=2, retargeted_attributes=2,
        modules=rows, art_changes=[], flag_changes=[], program_changes=[],
        inverse='Swap each listed before/after; restore old_tileset and matching attribute/block mirrors. No whole historical pack or program replacement.',
        future_native_identity=None)
    write(SOURCE/'native.contract.json', contract)
    return contract


def preflight(base, contract):
    assert len(base['data']['maps']) == contract['expected_map_count']
    for row in contract['modules']:
        m=base['data']['maps'][row['map']]
        assert sha(canonical(m)) == row['module_sha256'], row['map']
        assert base['data']['map_attributes'][row['map']] == m['attributes']
        assert base64.b64decode(base['data']['map_blocks'][row['blocks_label']]) == bytes(m['blocks'])
        assert all(n==row['map']or other['attributes']['blocks_label']!=row['blocks_label']
            for n,other in base['data']['maps'].items())
    for path, expected in contract['source_files_sha256'].items():
        assert sha((ROOT/path).read_bytes()) == expected, path
    for name, expected in contract['candidates_sha256'].items():
        assert sha((SOURCE/name).read_bytes()) == expected, name
    assert layout_blocks(json.loads(LAYOUT.read_bytes())) == whole(base)[0]
    current, proposed = json.loads(DOSSIER.read_bytes()), json.loads((SOURCE/'MN_R11.warm.candidate.json').read_bytes())
    excluded = {'compiled_tileset','winter_section_review','warm_coast_restoration_review'}
    assert {k:v for k,v in current.items()if k not in excluded} == {k:v for k,v in proposed.items()if k not in excluded}


def clone_maps(pack):
    return dict(pack, data=dict(pack['data'], maps=dict(pack['data']['maps']),
        map_attributes=dict(pack['data']['map_attributes']), map_blocks=dict(pack['data']['map_blocks'])))


def derive(base, contract):
    preflight(base, contract)
    candidate = clone_maps(base)
    for row in contract['modules']:
        if not row['cells']:
            continue
        m = deepcopy(base['data']['maps'][row['map']])
        width = m['attributes']['width']
        for cell in row['cells']:
            x, y = cell['metatile']; i = y*width+x
            assert m['blocks'][i] == cell['before']
            m['blocks'][i] = cell['after']
        m['attributes']['tileset_name'] = row['new_tileset']
        candidate['data']['maps'][row['map']] = m
        candidate['data']['map_attributes'][row['map']] = deepcopy(m['attributes'])
        candidate['data']['map_blocks'][row['blocks_label']] = base64.b64encode(bytes(m['blocks'])).decode()
    return candidate


def validate(base, candidate, contract):
    residual = clone_maps(candidate)
    for row in contract['modules']:
        if not row['cells']:
            continue
        m = deepcopy(candidate['data']['maps'][row['map']])
        assert m['attributes']['tileset_name'] == row['new_tileset']
        assert candidate['data']['map_attributes'][row['map']] == m['attributes']
        assert base64.b64decode(candidate['data']['map_blocks'][row['blocks_label']]) == bytes(m['blocks'])
        width = m['attributes']['width']
        for cell in row['cells']:
            x, y = cell['metatile']; i = y*width+x
            assert m['blocks'][i] == cell['after']
            m['blocks'][i] = cell['before']
        m['attributes']['tileset_name'] = row['old_tileset']
        residual['data']['maps'][row['map']] = m
        residual['data']['map_attributes'][row['map']] = base['data']['map_attributes'][row['map']]
        residual['data']['map_blocks'][row['blocks_label']] = base['data']['map_blocks'][row['blocks_label']]
    assert residual == base, 'unexpected terrain, art, metadata, actors, programs, flags, audio or other whole-pack drift'


def proof(base, candidate, warm, contract):
    validate(base, candidate, contract)
    changes = Counter()
    protected = Counter()
    for name in NAMES:
        m, new = base['data']['maps'][name], candidate['data']['maps'][name]
        assert new == warm['data']['maps'][name], ('not exact actual warm native module', name)
        before, after = collision(base,m), collision(candidate,new)
        assert set(before) == set(after)
        for point, a in before.items():
            b = after[point]
            assert ('FLOOR' if a == 'ICE' else a) == b, (name,point,a,b)
            if a != b:
                changes[f'{a}->{b}'] += 1
            if a not in {'FLOOR','TALL_GRASS','ICE','WALL','HEADBUTT_TREE'}:
                protected[a] += 1
            if point[0] in {0,m['attributes']['width']*2-1} or point[1] in {0,m['attributes']['height']*2-1}:
                assert a == b, ('outer seam collision changed', name, point)
        assert 'ICE' not in after.values()
        assert all(block < 0x83 for block in new['blocks'])
    assert changes == {'ICE->FLOOR':52}
    art = {}
    warm_art, current_art = SourceArt(warm), SourceArt(base)
    for period in ['morn','day','nite','dark']:
        ref = warm_art.blocks(WARM,period)
        now = current_art.blocks(WARM,period)
        cold = current_art.blocks(COLD,period)
        assert len(ref) == len(now) == 131
        assert all(x.tobytes()==y.tobytes()==z.tobytes() for x,y,z in zip(ref,now,cold[:131]))
        art[period] = 'All131 original native warm recipes pixel-exact against actual prewinter pack and cold alias prefix'
    negatives = []
    def reject(label, broken, preimage=False):
        try:
            preflight(broken,contract) if preimage else validate(base,broken,contract)
        except AssertionError:
            negatives.append(dict(case=label,rejected=True))
        else:
            raise AssertionError('negative was accepted: '+label)
    owned = 'MnR11C00R00'
    for label, name, index, value in [
        ('Leftover sliding ICE',owned,next(i for i,b in enumerate(base['data']['maps'][owned]['blocks']) if b==0x85),0x85),
        ('Wrong original lawn restoration',owned,next(i for i,b in enumerate(candidate['data']['maps'][owned]['blocks']) if b==2),1),
        ('Snowy roof left installed',owned,299,0x89),
        ('Unrelated R11 southern cell drift','MnR11C00R03',0,1),
        ('Unrelated approved city changed','MnMinneapolis',0,1)]:
        broken=clone_maps(candidate); broken['data']['maps'][name]=deepcopy(candidate['data']['maps'][name])
        broken['data']['maps'][name]['blocks'][index]=value
        label=broken['data']['maps'][name]['attributes']['blocks_label']
        broken['data']['map_blocks'][label]=base64.b64encode(bytes(broken['data']['maps'][name]['blocks'])).decode()
        reject(label,broken)
    broken=clone_maps(candidate);broken['data']['maps'][owned]=deepcopy(candidate['data']['maps'][owned])
    broken['data']['maps'][owned]['objects'][0]['x']+=1;reject('Moved original actor',broken)
    broken=clone_maps(candidate);broken['data']['maps'][owned]=deepcopy(candidate['data']['maps'][owned])
    key=next(iter(broken['data']['maps'][owned]['scripts']))
    broken['data']['maps'][owned]['scripts'][key].append(dict(command='end',args=[]));reject('Old native program changed',broken)
    broken=clone_maps(base);broken['data']['maps'][owned]=deepcopy(base['data']['maps'][owned])
    broken['data']['maps'][owned]['blocks'][0]=0;reject('Stale owned parent',broken,True)
    return dict(status='STATIC_NATIVE_DATA_PROPOSAL_PASS_NOT_PUBLISHED_OR_CONTROLLER',
        exact_actual_warm_reference_modules=8, changed_cells=828, winter_terrain_cells=826,
        roof_cells=2, collision_changes=dict(changes), all_other_quadrants_exact=True,
        protected_special_quadrants=dict(protected), outer_seam_quadrants_exact=True,
        all_original_actors_programs_events_flags_encounters_field_moves_and_rooms_exact=True,
        whole_parent_residual_exact_after_inverse_828_cells_2_aliases_and_mirrors=True,
        art_unchanged_and_original_warm_recipes=art, negative_cases=negatives,
        installed_winter_inventory=inventory(base,contract),
        limits=['Static data/source-art proposal only; no pack, identity, controller or GPU result is produced.',
            'R11 connection records remain exact. Direct Grand Marais and inland Ely geography is a separate coordinated proposal.',
            'Removing ICE intentionally removes52 forced-slide quadrants; all other native collision tokens and old warm modules match exactly.',
            'This does not establish earned campaign access or all-route Johto-quality acceptance.'])


def inventory(base,contract):
    tileset=base['data']['tilesets'][COLD]
    files=base['runtime_files']
    recipes=bytes(files[f'data/tilesets/{COLD}_metatiles.bin'])
    foregrounds=json.loads(bytes(files[f'data/tilesets/{COLD}_foregrounds.json']))
    counts=Counter(c['before']for r in contract['modules']for c in r['cells'])
    installed=[]
    for block,count in sorted(counts.items()):
        glyphs=list(recipes[block*16:(block+1)*16])
        installed.append(dict(block=f'{block:02x}',installed_metatiles=count,
            quadrants=tileset['collision'][f'{block:02x}'],glyphs=glyphs,
            palette_attributes={f'{g:02x}':tileset['palette_map'][g]for g in sorted(set(glyphs))},
            foreground_rgb={f'{g:02x}':foregrounds[str(g)]for g in sorted(set(glyphs))if str(g)in foregrounds}))
    coast=[n for n in base['data']['maps']if n.startswith(('MnR11','MnR12','MnGrandMarais'))or n=='MnMaraisCedarApproach']
    return dict(cold_alias=COLD,warm_alias=WARM,changed_recipes=installed,
        alias_users=[n for n,m in base['data']['maps'].items()if m['attributes']['tileset_name']==COLD],
        whole_coast_map_aliases={n:base['data']['maps'][n]['attributes']['tileset_name']for n in sorted(coast)},
        installed_definition_sha256=sha(canonical(tileset)),
        runtime_art_files_sha256={p:sha(bytes(v))for p,v in files.items()if COLD in p or WARM in p},
        installed_global_art_manifests_retained=[m for m in base['report']['manifests']if 'winter'in m],
        private_snow_tree_87_installed_but_unused_on_R11=True,
        cliff_note='No snowy cliff-face recipe is installed. Exactly27 native rock-top71 Floor cells were snow83; restore71. All directional cliff faces, stairs and ledges remain native-exact.',
        flag_note='R11 warm/current full modules differ only by tileset name and828 terrain cells. No R11 actor/event/program/flag command was added by winter; restore no flags.',
        presentation_note='Retargeting to mn_north_shore restores ordinary grass rustle cache. Both aliases already use identical original water/flower/waterfall animation. Leave shared Rust behavior and all winter art available for inland regions.')


VIEWS = [('north-coast-exit',[40,8]),('roof-and-falls',[23,22]),
    ('upper-ordinary-bank',[36,24]),('old-grass-habitat',[46,29]),
    ('superior-shore-cove',[48,33]),('palisade-ledge',[19,51]),
    ('rock-terrace',[25,64]),('second-stream',[33,78]),
    ('birch-stair-return',[37,91]),('stoney-full-falls',[25,152]),
    ('duluth-approach',[47,189])]


def render(pack, alias, label, output):
    blocks, objects = whole(pack)
    module=dict(id='MN_R11', attributes=dict(pack['data']['maps'][NAMES[0]]['attributes'],
        width=56,height=96,tileset_name=alias),blocks=blocks)
    art=ReviewArt(pack)
    ground,scene,_=render_map(art,module,objects,'day',output/label)
    save_render(scene.resize((560,960),Image.Resampling.NEAREST),output/label/'whole-display.png')
    occupied={tuple(o['tile'])for o in objects}
    kinds=collision(pack,module)
    frames=[]; permissions={}
    for name,point in VIEWS:
        assert tuple(point) not in occupied
        token=kinds[tuple(point)]
        assert token in {'FLOOR','TALL_GRASS'},(label,name,point,token)
        permissions[name]=token
        x,y=point; left=max(0,min(scene.width-160,x*16-80));top=max(0,min(scene.height-144,y*16-72))
        x0,y0=left//32,top//32;x1,y1=(left+191)//32,(top+175)//32
        x1,y1=min(56,x1),min(96,y1)
        sub=dict(module,attributes=dict(module['attributes'],width=x1-x0,height=y1-y0),
            blocks=[blocks[yy*56+xx]for yy in range(y0,y1)for xx in range(x0,x1)])
        folder=output/label/'views'/name;box=(x0*32,y0*32,x1*32,y1*32)
        g=render_stage(ground,'crop',folder,'exact-terrain-window',box)
        s=render_stage(scene,'crop',folder,'exact-actor-window',box)
        frame,_=player_camera(art,sub,[],g,s,'day',dict(core=[x-x0*2,y-y0*2]),folder)
        frames.append((name+' / '+label,frame))
    return scene,frames,permissions


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-river-long-v02.crystalpack')
    parser.add_argument('--expected-parent-sha256',default=BASE_SHA)
    parser.add_argument('--freeze-proposal',action='store_true')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert not args.output.exists(),'Retain every proof/render epoch'
    assert sha(args.pack.read_bytes())==args.expected_parent_sha256
    assert sha(WARM_PACK.read_bytes())==WARM_SHA
    base,warm=load_pack(args.pack),load_pack(WARM_PACK)
    contract=freeze(base,warm)if args.freeze_proposal else json.loads((SOURCE/'native.contract.json').read_bytes())
    candidate=derive(base,contract)
    report=proof(base,candidate,warm,contract)
    args.output.mkdir(parents=True)
    scene_frames=[];whole_images=[];permissions={}
    for label,pack,alias in [('actual-current-cold',base,COLD),
        ('actual-prewinter-warm',warm,WARM),('candidate-warm',candidate,WARM)]:
        scene,frames,p=render(pack,alias,label,args.output)
        scene_frames.append(frames);whole_images.append((label,scene.resize((448,768),Image.Resampling.NEAREST)))
        permissions[label]=p
    assert whole_images[1][1].tobytes()==whole_images[2][1].tobytes()
    save_render(room_sheet(whole_images,3),args.output/'whole-route-three-state.png')
    contact=[scene_frames[state][view]for view in range(len(VIEWS))for state in range(3)]
    save_render(room_sheet(contact,3),args.output/'player-views-three-state.png')
    report.update(parent_pack_sha256=args.expected_parent_sha256,parent_identity=base['identity']['content_hash'],
        warm_reference_pack_sha256=WARM_SHA,contract_sha256=sha((SOURCE/'native.contract.json').read_bytes()),
        tool_sha256=sha(Path(__file__).read_bytes()),views=[dict(id=n,core=p)for n,p in VIEWS],
        view_collision_permissions=permissions,
        rendered_files_sha256={str(p.relative_to(args.output)):sha(p.read_bytes())for p in args.output.rglob('*.png')})
    write(args.output/'proof.json',report)
    (args.output/'prepare_review.snapshot.py').write_bytes(Path(__file__).read_bytes())
    assert sha(args.pack.read_bytes())==args.expected_parent_sha256
    print(json.dumps({k:report[k]for k in ['status','changed_cells','collision_changes','contract_sha256']}))


if __name__=='__main__':
    main()
