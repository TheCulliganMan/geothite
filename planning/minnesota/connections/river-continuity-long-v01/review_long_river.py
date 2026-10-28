#!/usr/bin/env python3
"""Render explicit detached river strokes against the exact written parent.

No terrain inference/randomness or pack writing. Original native catalogs stay
inside the input pack. Output is authoring JSON, bounded deltas and source art.
"""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
sys.path.insert(0, str(ROOT / 'planning/minnesota/connections/river-continuity-v01'))
from audit_city_expansions import module_hash
from audit_outdoor_connections import collision
from audit_water_edges import production_permissions, numeric
from render_archive import save_render
from render_reference import load_pack, layout_blocks, room_sheet
from render_progress_atlas import native_room
from review_north_shore import oriented_sprite
from review_first_bend import composite, layout, write_json

PLAN = Path(__file__).resolve().parent
PARENT_SHA = '92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def full_region(pack, manifest, prefix):
    rows = [r for r in manifest['new_maps'] if r['id'].startswith(prefix)]
    grid = [5] * (64 * 68)
    for row in rows:
        m = pack['data']['maps'][row['id']]
        x0, y0 = row['source_origin_metatiles']
        w, h = row['dimensions_metatiles']
        for y in range(h):
            grid[(y+y0)*64+x0:(y+y0)*64+x0+w] = m['blocks'][y*w:(y+1)*w]
    return rows, grid


def region_camera(pack, rows, global_core, facing):
    width=max(r['source_origin_metatiles'][0]+r['dimensions_metatiles'][0]for r in rows)
    height=max(r['source_origin_metatiles'][1]+r['dimensions_metatiles'][1]for r in rows)
    world = Image.new('RGBA', (width*32, height*32), '#121f2b')
    permission=None
    for row in rows:
        x, y = row['source_origin_metatiles']
        world.alpha_composite(native_room(pack['data']['maps'][row['id']], pack,
                                         native_origin=True).convert('RGBA'), (x*32, y*32))
        gx,gy=global_core
        w,h=row['dimensions_metatiles']
        if x*2<=gx<(x+w)*2 and y*2<=gy<(y+h)*2:
            permission=collision(pack,pack['data']['maps'][row['id']])[gx-x*2,gy-y*2]
    x, y = global_core
    crop = world.crop((x*16-72, y*16-64, x*16+88, y*16+80))
    # Existing native standing/Surf sprites give source scale only; these are
    # not native animation, GPU or reached-gameplay claims.
    if permission in {'FLOOR','TALL_GRASS','WATER'}:
        sprite='SPRITE_SURF' if permission=='WATER' else 'SPRITE_CHRIS'
        crop.alpha_composite(oriented_sprite(pack, sprite, facing), (72,64))
    return crop


def water_length(pack, rows, start, end):
    _, grid = full_region(pack, {'new_maps':rows}, rows[0]['id'].split('C00')[0])
    tileset = pack['data']['tilesets']['johto']['collision']
    tokens, water, _, _ = production_permissions()
    wet = {(x,y) for y in range(136) for x in range(128)
           if numeric(tileset[f'{grid[(y//2)*64+x//2]:02x}'][(y%2)*2+x%2], tokens) in water}
    todo = deque([start]); previous = {start:None}
    while todo:
        p = todo.popleft()
        if p == end:
            path = []
            while p is not None:
                path.append(list(p)); p = previous[p]
            return list(reversed(path))
        for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
            q = p[0]+dx,p[1]+dy
            if q in wet and q not in previous:
                previous[q]=p;todo.append(q)
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack', type=Path, required=True)
    ap.add_argument('--geography', type=Path, default=PLAN/'geography-v01.json')
    ap.add_argument('--preimage-contract',type=Path,
                    help='Frozen92bb source contract; permits only the actual reviewed57ee combined preview parent.')
    ap.add_argument('--output', type=Path, required=True)
    args=ap.parse_args()
    assert not args.output.exists(), 'Retain every source/render epoch.'
    args.output.mkdir(parents=True)
    actual_parent=digest(args.pack.read_bytes())
    baseline=None
    if args.preimage_contract:
        assert actual_parent=='57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62'
        baseline=json.loads(args.preimage_contract.read_text())
        assert baseline['parent_pack_sha256']==PARENT_SHA
    else:
        assert actual_parent == PARENT_SHA
    before=load_pack(args.pack)
    if baseline:
        for entry in baseline['changes']:
            assert module_hash(before['data']['maps'][entry['map']])==entry['native_module_sha256'],('Combined parent changed an owned preimage',entry['map'])
    after=dict(before);after['data']=dict(before['data']);after['data']['maps']=dict(before['data']['maps'])
    manifest=json.loads((ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json').read_text())
    design=json.loads(args.geography.read_text())
    write_json(args.output/'geography.snapshot.json', design)
    regions={}
    for prefix, authored in design['regions'].items():
        rows, old_grid = full_region(before, manifest, prefix)
        authored_layout=dict(schema_version=2,id=prefix,tileset='johto',compiled_tileset='johto',
            dimensions_metatiles=[64,68],source_pixels_modified=False,
            shore_fitting='johto_8_neighbors',reject_cliff_cuts=True,layers=authored['layers'])
        grid=layout_blocks(authored_layout)
        # The original two trainer bank clearings, including their grass, stay
        # exact; the new channel must fit around them rather than move actors.
        for y in range(68):
            for x in range(64):
                i=y*64+x
                if old_grid[i] in {2,3} or min(x,y,63-x,67-y)<2:
                    grid[i]=old_grid[i]
        if authored.get('landings'):
            # Explicit refitted original banks. Keep their actual actors and
            # approach tiles, not the old unprogrammed square lawns.
            layers=[dict(name='Explicit watercourse rows',kind='block_rect',bounds=[0,0,64,68],block='05')]
            for y in range(68):
                for x in range(64):
                    layers.append(dict(name='Authored source cell',kind='point',position=[x,y],block=f'{grid[y*64+x]:02x}'))
            for bank in authored['landings']:
                layers.append(dict(name=bank['name'],kind='block_rect',bounds=bank['old_bounds'],block='05'))
                layers.extend(bank['layers'])
            grid=layout_blocks(dict(authored_layout,layers=layers,shore_fitting='explicit_source_edges'))
        # Bounded author-reviewed shoreline corrections inside the mouths.
        # Only these explicit cells may supersede old two-column margin tiles;
        # the actual outer seam remains untouched and is checked separately.
        for cell in authored.get('interior_seam_tapers', []):
            x, y = cell['metatile']
            assert x > 0 and y > 0 and x < 63 and y < 67
            i = y*64+x
            assert grid[i] == int(cell['before'], 16), (prefix, cell, grid[i])
            grid[i] = int(cell['after'], 16)
        for row in rows:
            name=row['id'];old=before['data']['maps'][name];m=deepcopy(old)
            x0,y0=row['source_origin_metatiles'];w,h=row['dimensions_metatiles']
            m['blocks']=[grid[(y+y0)*64+x+x0] for y in range(h) for x in range(w)]
            after['data']['maps'][name]=m
        regions[prefix]=dict(rows=rows,old_grid=old_grid,grid=grid)
        full=dict(id=prefix,attributes={'width':64,'height':68,'tileset_name':'johto'},blocks=grid)
        write_json(args.output/'source'/f'{prefix}.candidate.layout.json',layout(full,'DETACHED_SOURCE_NOT_NATIVE'))

    # R05 stays the exact old road/trainer/ledge/room map. Close only native
    # all-Water metatiles in the declared band; no land/bridge/gate can be erased.
    tokens,water,wall,engine_hashes=production_permissions()
    for row_index in range(3):
        name=f'MnR05C00R{row_index:02}'
        old=before['data']['maps'][name];m=deepcopy(old);a=m['attributes'];w=a['width']
        for closure in design['r05_water_only_closures']:
            x0,y0,x1,y1=closure['bounds_metatiles']
            for ly in range(a['height']):
                y=ly+row_index*24
                for x in range(w):
                    i=ly*w+x;b=old['blocks'][i]
                    quadrants=before['data']['tilesets'][a['tileset_name']]['collision'][f'{b:02x}']
                    kinds=[numeric(q,tokens)for q in quadrants]
                    allowed=water|wall if closure.get('allow_original_wall_quadrants') else water
                    if x0<=x<x1 and y0<=y<y1 and any(q in water for q in kinds) and all(q in allowed for q in kinds):
                        m['blocks'][i]=int(closure['replacement'],16)
        for barrier in design.get('r05_exact_barrier_replacements',[]):
            if barrier['map']!=name:continue
            for (x,y),expected_block in zip(barrier['metatiles'],barrier['before']):
                i=y*w+x;assert old['blocks'][i]==int(expected_block,16)
                m['blocks'][i]=int(barrier['after'],16)
        after['data']['maps'][name]=m

    owned=[n for n in after['data']['maps'] if after['data']['maps'][n] is not before['data']['maps'][n]]
    changes=[];anchors=[]
    for name in owned:
        old=before['data']['maps'][name];m=after['data']['maps'][name];expected=deepcopy(old);expected['blocks']=m['blocks'];assert expected==m
        cells=[dict(metatile=[i%m['attributes']['width'],i//m['attributes']['width']],before=f'{a:02x}',after=f'{b:02x}')for i,(a,b)in enumerate(zip(old['blocks'],m['blocks']))if a!=b]
        changes.append(dict(map=name,native_module_sha256=module_hash(old),dimensions_metatiles=[m['attributes']['width'],m['attributes']['height']],cells=cells))
        write_json(args.output/'source'/f'{name}.before.layout.json',layout(old,'FROZEN_NATIVE_PREIMAGE'))
        write_json(args.output/'source'/f'{name}.candidate.layout.json',layout(m,'DETACHED_SOURCE_NOT_NATIVE'))
        before_kind=collision(before,old);after_kind=collision(after,m)
        for group,entries in [('objects',old['objects']),*old['events'].items()]:
            for entry in entries:
                p=entry['x'],entry['y'];assert before_kind[p]==after_kind[p],(name,group,p,before_kind[p],after_kind[p])
                anchors.append(dict(map=name,group=group,core=list(p),permission=before_kind[p]))
                if group=='objects' and 'Mississippi' in name:
                    for dx,dy in [(-1,0),(1,0),(0,-1),(0,1)]:
                        q=p[0]+dx,p[1]+dy
                        assert before_kind[q]==after_kind[q],('Original actor approach',name,q,before_kind[q],after_kind[q])

    placements=json.loads((ROOT/'planning/minnesota/renders/ice/native-geography-v03/atlas/native-connection-audit.json').read_text())['placements']
    all_names={r['id']for r in manifest['new_maps']}|{'MnMinneapolis','MnStPaul','MnR04C00R00','MnR03C00R00','MnWinona','MnStCloud','MnR05C00R00','MnR05C00R01','MnR05C00R02'}
    for epoch,pack in [('before',before),('candidate',after)]:
        composite(pack,placements,all_names,12,args.output/f'whole-corridor-{epoch}.png',f'DETACHED LONG RIVER / {epoch.upper()}')
        upper=set(r['id']for r in regions['MnMississippiUpstreamWoods']['rows'])|{'MnR05C00R00','MnR05C00R01','MnR05C00R02','MnStCloud'}
        composite(pack,placements,upper,16,args.output/f'upper-river-{epoch}.png',f'UPPER WOODS + R05 / {epoch.upper()}')
        for prefix,info in regions.items():
            # Full64×68 source rectangle, original native scale, actors retained.
            world=Image.new('RGB',(64*32,68*32),'#121f2b')
            for row in info['rows']:
                x,y=row['source_origin_metatiles']
                world.paste(native_room(pack['data']['maps'][row['id']],pack,native_origin=True),(x*32,y*32))
            save_render(world,args.output/f'{prefix}-{epoch}-whole-original.png')
        views=[]
        for label,prefix,point,facing in [
            ('upper-south-landing','MnMississippiUpstreamWoods',(32,131),'east'),
            ('upper-quiet-pool','MnMississippiUpstreamWoods',(112,82),'north'),
            ('upper-pine-tongue','MnMississippiUpstreamWoods',(87,64),'south'),
            ('upper-keeper-bank','MnMississippiUpstreamWoods',(68,23),'east'),
            ('lower-angler-landing','MnMississippiLowerWoods',(20,61),'east'),
            ('lower-still-reach','MnMississippiLowerWoods',(68,34),'east'),
            ('lower-pine-bank','MnMississippiLowerWoods',(102,83),'east'),
            ('lower-south-descent','MnMississippiLowerWoods',(44,115),'south')]:
            im=region_camera(pack,regions[prefix]['rows'],point,facing)
            save_render(im,args.output/'cameras'/f'{label}-{epoch}-native160.png')
            save_render(im.resize((640,576),Image.Resampling.NEAREST),args.output/'cameras'/f'{label}-{epoch}-4x.png')
            views.append((label+' '+epoch,im.convert('RGB')))
        r05rows=[dict(id=f'MnR05C00R{i:02}',source_origin_metatiles=[0,i*24],
            dimensions_metatiles=[32,pack['data']['maps'][f'MnR05C00R{i:02}']['attributes']['height']])for i in range(3)]
        r05whole=Image.new('RGB',(32*32,56*32),'#121f2b')
        for row in r05rows:
            r05whole.paste(native_room(pack['data']['maps'][row['id']],pack,native_origin=True),(0,row['source_origin_metatiles'][1]*32))
        save_render(r05whole,args.output/f'R05-{epoch}-whole-original.png')
        for label,point,facing in [('r05-lookout-water',(54,21),'west'),('r05-fisher-pier',(56,89),'east'),('r05-reward-bank',(42,81),'west')]:
            im=region_camera(pack,r05rows,point,facing)
            save_render(im,args.output/'cameras'/f'{label}-{epoch}-native160.png')
            save_render(im.resize((640,576),Image.Resampling.NEAREST),args.output/'cameras'/f'{label}-{epoch}-4x.png')
            views.append((label+' '+epoch,im.convert('RGB')))
        save_render(room_sheet(views,columns=4),args.output/f'player-cameras-{epoch}.png')

    paths=[]
    for prefix,entry in regions.items():
        start,end=((0,40),(0,132))if'Upstream'in prefix else((0,72),(36,135))
        old=water_length(before,entry['rows'],start,end);new=water_length(after,entry['rows'],start,end)
        paths.append(dict(region=prefix,from_core=list(start),to_core=list(end),before_steps=None if old is None else len(old)-1,candidate_steps=None if new is None else len(new)-1,candidate_path=new))
    if baseline:
        assert changes==baseline['changes'],'Combined preview changed the frozen river cell deltas'
    write_json(args.output/'candidate.contract.json',dict(schema_version=1,status='DETACHED_SOURCE_REVIEW_NOT_NATIVE',parent_pack_sha256=actual_parent,parent_identity=before['identity']['content_hash'],source_native_preimage_parent_sha256=PARENT_SHA,source_preimage_contract_sha256=digest(args.preimage_contract.read_bytes())if args.preimage_contract else None,geography_sha256=digest(args.geography.read_bytes()),changes=changes,unchanged_native_anchor_permissions=anchors,retained_native_connections=True,world_rectangles_unchanged=True,programs_actors_rooms_rewards_art_audio_unchanged=True,source_only_paths=paths,engine_source_hashes=engine_hashes))
    print(json.dumps({'output':str(args.output),'changed_cells':sum(len(r['cells'])for r in changes),'paths':[{k:v for k,v in p.items()if k!='candidate_path'}for p in paths]},indent=2))


if __name__=='__main__':
    main()
