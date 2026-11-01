#!/usr/bin/env python3
"""Author exact ice/Crystal-cave-rock quadrant changes without writing a pack."""
import argparse
import collections
import copy
import hashlib
import json
import pickle
from pathlib import Path

from audit_outdoor_connections import collision, project
from render_reference import load_pack
from river_content_lineage import validate

BASE = Path('content-packs/minnesota-fruit-compact-v01.crystalpack')
SOURCE = Path('planning/minnesota/regions/ice/frozen-water-compact-v02.json')


def author():
    digest = hashlib.sha256(BASE.read_bytes()).hexdigest()
    cache = Path('.local')/f'fruit-review-{digest}.pickle'
    pack = pickle.loads(cache.read_bytes()) if cache.exists() else load_pack(BASE)
    validate(pack)
    maps = pack['data']['maps']
    names = ['MnR12C00R00', 'MnR12C00R01', 'MnR13C00R00', 'MnR13C00R01', 'MnR17C00R00', 'MnR17C00R01', 'MnRainyRiverIcefields']
    kinds = {n:collision(pack,maps[n]) for n in names}
    targets = {n:maps[n]['attributes']['tileset_name'] for n in names}
    assert set(targets.values()) == {'mn_winter_outdoor', 'mn_winter_gunflint', 'mn_winter_north_shore'}
    recipes = collections.defaultdict(list)
    lookup = {}
    rows = []
    totals = collections.Counter()
    for name in names:
        m = maps[name];a=m['attributes'];k=kinds[name];target=targets[name]
        ice={v for v,kind in k.items() if kind in ['ICE','WATER','WATER_21']}
        changes={}
        for v in ice:
            if k[v]!='ICE':changes[v]='ice'
        for (x,y),kind in k.items():
            v=(x,y)
            if kind in ['BUOY','WATERFALL']:
                changes[v]='rock'
            if kind=='WALL' and any(q in ice for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]):
                changes[v]='rock'
        # Only a real, in-range paired map connection may retain edge ice.
        w,h=a['width']*2,a['height']*2
        for v in ice:
            x,y=v
            directions=[d for d,yes in [('west',x==0),('east',x==w-1),('north',y==0),('south',y==h-1)] if yes]
            for direction in directions:
                edge=next((e for e in a['connections'] if e['direction']==direction),None)
                if edge:
                    b=maps[edge['target_map']]['attributes'];q=project(a,b,edge,v)
                    if 0<=q[0]<b['width']*2 and 0<=q[1]<b['height']*2:
                        continue
                changes[v]='rock'
        # A deliberate two-tile snow entry opens the previously sealed marsh lake.
        if name=='MnR17C00R01':
            for v in [(22,19),(23,19)]:
                assert k[v]=='WALL' and k[v[0],v[1]-1]=='FLOOR' and k[v[0],v[1]+1]=='WATER'
                changes[v]='ramp'
        cells=[]
        for my in range(a['height']):
            for mx in range(a['width']):
                before=m['blocks'][my*a['width']+mx]
                parts=[changes.get((mx*2+q%2,my*2+q//2)) for q in range(4)]
                if not any(parts):continue
                key=(target,before,tuple(parts))
                if key not in lookup:
                    ident=len(pack['runtime_files'][f'data/tilesets/{target}_metatiles.bin'])//16+len(recipes[target])
                    assert ident<256,('metatile capacity',target,ident)
                    lookup[key]=ident
                    recipes[target].append(dict(id=ident,source_block=before,quadrants=parts))
                cells.append(dict(at=[mx,my],before=before,after=lookup[key]))
        totals.update(changes.values())
        rows.append(dict(id=name,old_tileset=a['tileset_name'],tileset=target,
                         expected_blocks_sha256=hashlib.sha256(bytes(m['blocks'])).hexdigest(),changes=cells))
    # Every private tileset needs an explicit solid padding block, even when
    # its placed shoreline contains only mixed rock/ice metatiles.
    for target,entries in recipes.items():
        if not any(r['quadrants']==['rock']*4 for r in entries):
            wall=next(int(k,16) for k,v in pack['data']['tilesets'][target]['collision'].items() if v==['WALL']*4)
            ident=len(pack['runtime_files'][f'data/tilesets/{target}_metatiles.bin'])//16+len(entries)
            assert ident<256
            entries.append(dict(id=ident,source_block=wall,quadrants=['rock']*4))
    design=dict(schema_version=1,base_content_hash=pack['identity']['content_hash'],base_pack=str(BASE),base_pack_sha256=digest,
                rock_source=dict(tileset='ice_path',block=25,palette='indoor',index_map=[1,2,3,3]),
                art=[dict(tileset=n,expected_blocks_sha256=hashlib.sha256(bytes(pack['runtime_files'][f'data/tilesets/{n}_metatiles.bin'])).hexdigest(),recipes=r) for n,r in recipes.items()],maps=rows)
    SOURCE.write_text(json.dumps(design,indent=2)+'\n')
    print(json.dumps(dict(maps=names,quadrants=dict(totals),recipes={n:len(r) for n,r in recipes.items()}),indent=2),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--base',type=Path,required=True);parser.add_argument('--output',type=Path,default=SOURCE);args=parser.parse_args();BASE=args.base;SOURCE=args.output
    assert not SOURCE.exists(), 'source epoch must be fresh'
    author()
