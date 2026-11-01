#!/usr/bin/env python3
"""Render native-source ice/rock recipes; audit written collision and boundaries."""
import argparse
import collections
import copy
import hashlib
import json
import pickle
from pathlib import Path

from PIL import Image
from audit_outdoor_connections import collision, project
from render_reference import SourceArt, load_pack
from render_progress_atlas import FACING, scene

SOURCE=Path('planning/minnesota/regions/ice/frozen-water-compact-v02.json')


def cached(path):
    digest=hashlib.sha256(path.read_bytes()).hexdigest()
    cache=Path('.local')/f'fruit-review-{digest}.pickle'
    if cache.exists():return pickle.loads(cache.read_bytes())
    pack=load_pack(path);cache.write_bytes(pickle.dumps(pack));return pack


class FrozenArt(SourceArt):
    def __init__(self,pack,design):
        super().__init__(pack)
        self.design=design
        self.assembled={}

    def blocks(self,name,group='day',palette_override=None):
        key=(name,group,palette_override)
        if key in self.assembled:return self.assembled[key]
        original=super().blocks(name,group,palette_override)
        rows=next((a['recipes'] for a in self.design['art'] if a['tileset']==name),[])
        if not rows:return original
        images=list(original)
        rock=super().blocks('ice_path','indoor')[0x19].copy()
        # The native winter convention merges the two darkest indexed shades
        # and gives source zero the existing snow white, avoiding lawn halos.
        import re
        files=self.pack['runtime_files'];current='';colors=[]
        for line in bytes(files['gfx/tilesets/bg_tiles.pal']).decode().splitlines():
            line=line.strip()
            if line.startswith(';'):current=line[1:].strip()
            if current=='indoor' and line.startswith('RGB'):
                values=list(map(int,re.findall(r'\d+',line.split(';')[0])))
                colors.extend(tuple((v<<3)|(v>>2) for v in values[i:i+3]) for i in range(0,len(values),3))
        snow_id=0x83 if name=='mn_winter_north_shore' else 0x3c
        snow_glyph=files[f'data/tilesets/{name}_metatiles.bin'][snow_id*16]
        foregrounds=json.loads(bytes(files[f'data/tilesets/{name}_foregrounds.json']))
        white=tuple(foregrounds[str(snow_glyph)][0])
        donor=files['data/tilesets/ice_path_metatiles.bin'][0x19*16:0x1a*16]
        for i,glyph in enumerate(donor):
            palette=colors[(self.pack['data']['tilesets']['ice_path']['palette_map'][glyph]&7)*4:][:4]
            x,y=i%4*8,i//4*8;tile=rock.crop((x,y,x+8,y+8))
            tile.putdata([white if rgb==palette[0] else palette[3] if rgb==palette[2] else rgb for rgb in tile.getdata()])
            rock.paste(tile,(x,y))
        ice=original[0x85 if name=='mn_winter_north_shore' else 0x3e]
        snow=original[0x83 if name=='mn_winter_north_shore' else 0x3c]
        for row in rows:
            assert row['id']==len(images)
            image=original[row['source_block']].copy()
            for q,material in enumerate(row['quadrants']):
                if material:
                    x,y=q%2*16,q//2*16
                    image.paste({'ice':ice,'rock':rock,'snow':snow,'ramp':snow}[material].crop((x,y,x+16,y+16)),(x,y))
            images.append(image)
        self.assembled[key]=images
        return images


from river_content_lineage import validate

def review(output,written=None):
    output.mkdir(parents=True,exist_ok=False)
    design=json.loads(SOURCE.read_bytes());base=cached(Path(design['base_pack']));validate(base)
    assert base['identity']['content_hash']==design['base_content_hash']
    pack=dict(base);pack['data']=dict(base['data']);pack['data']['maps']=dict(base['data']['maps']);pack['data']['tilesets']=copy.deepcopy(base['data']['tilesets'])
    for art in design['art']:
        name=art['tileset'];definition=pack['data']['tilesets'][name]
        for recipe in art['recipes']:
            kinds=list(definition['collision'][f"{recipe['source_block']:02x}"])
            for q,material in enumerate(recipe['quadrants']):
                if material:kinds[q]={'ice':'ICE','rock':'WALL','snow':'FLOOR','ramp':'FLOOR'}[material]
            definition['collision'][f"{recipe['id']:02x}"]=kinds
    for row in design['maps']:
        m=copy.deepcopy(base['data']['maps'][row['id']]);assert hashlib.sha256(bytes(m['blocks'])).hexdigest()==row['expected_blocks_sha256']
        m['attributes']['tileset_name']=row['tileset']
        m['attributes']['border_block']=next(r['id'] for a in design['art'] if a['tileset']==row['tileset'] for r in a['recipes'] if r['quadrants']==['rock']*4)
        for cell in row['changes']:
            x,y=cell['at'];index=y*m['attributes']['width']+x;assert m['blocks'][index]==cell['before'];m['blocks'][index]=cell['after']
        pack['data']['maps'][row['id']]=m
    expected_art=FrozenArt(base,design)
    if written:
        actual=cached(written);validate(actual)
        edited={row['id'] for row in design['maps']}
        for name,m in pack['data']['maps'].items():assert actual['data']['maps'][name]==m,('written map differs',name)
        for key,value in base['data'].items():
            if key not in ['maps','map_blocks','map_attributes','tilesets']:
                assert actual['data'][key]==value,('unrelated catalog changed',key)
        for field in ['compiled_audio','audio_manifest','audio_compression','format_version']:
            assert actual[field]==base[field],field
        allowed={f'{prefix}/{a["tileset"]}{suffix}' for a in design['art'] for prefix,suffix in [('gfx/tilesets','.2bpp'),('gfx/tilesets','.png'),('data/tilesets','_metatiles.bin'),('data/tilesets','_foregrounds.json'),('data/tilesets','.json'),('data/tilesets','_palette_map.json')]}
        assert actual['runtime_files'].keys()==base['runtime_files'].keys()
        for name,data in base['runtime_files'].items():
            if name not in allowed:assert actual['runtime_files'][name]==data,name
        native_art=SourceArt(actual)
        for row in design['art']:
            name=row['tileset']
            for i,(a,b) in enumerate(zip(expected_art.blocks(name),native_art.blocks(name))):
                assert a.tobytes()==b.tobytes(),('native recipe pixels differ',name,i)
        pack=actual;art=native_art
    else:art=expected_art
    grid={}
    old_grid={}
    def old_kinds(name):
        if name not in old_grid:old_grid[name]=collision(base,base['data']['maps'][name])
        return old_grid[name]
    def kinds(name):
        if name not in grid:grid[name]=collision(pack,pack['data']['maps'][name])
        return grid[name]
    def neighbor(name,point,delta):
        x,y=point;dx,dy=delta;q=(x+dx,y+dy);a=pack['data']['maps'][name]['attributes']
        if 0<=q[0]<a['width']*2 and 0<=q[1]<a['height']*2:return name,q
        direction={(-1,0):'west',(1,0):'east',(0,-1):'north',(0,1):'south'}[delta]
        edge=next((e for e in a['connections'] if e['direction']==direction),None)
        if edge:
            b=pack['data']['maps'][edge['target_map']]['attributes'];q=project(a,b,edge,point)
            if 0<=q[0]<b['width']*2 and 0<=q[1]<b['height']*2:return edge['target_map'],q
        return None
    deltas=[(1,0),(-1,0),(0,1),(0,-1)]
    rock_sites=set();counts=collections.Counter();cases=[]
    for row in design['maps']:
        name=row['id'];m=pack['data']['maps'][name];a=m['attributes'];old=collision(base,base['data']['maps'][name]);k=kinds(name)
        assert not any(v in ['WATER','WATERFALL','WATER_21','BUOY'] for v in k.values()),('liquid remains in frozen area',name)
        for p,before in old.items():
            after=k[p]
            if before!=after:counts[f'{before}->{after}']+=1
            if before in ['FLOOR','TALL_GRASS','HEADBUTT_TREE','CUT_TREE','DOOR','LADDER']:
                assert before==after,('original path or gate changed',name,p)
            if after=='ICE':
                x,y=p
                for d,edge in [((-1,0),x==0),((1,0),x==a['width']*2-1),((0,-1),y==0),((0,1),y==a['height']*2-1)]:
                    if edge:assert neighbor(name,p,d),('ice exposes out of bounds',name,p,d)
        recipes={r['id']:r for arow in design['art'] if arow['tileset']==row['tileset'] for r in arow['recipes']}
        for cell in row['changes']:
            x,y=cell['at'];recipe=recipes[cell['after']]
            for q,material in enumerate(recipe['quadrants']):
                if material=='rock':rock_sites.add((name,(x*2+q%2,y*2+q//2)))
        actors=[dict(tile=[o['x'],o['y']],sprite=o['sprite'],facing=FACING.get(o['spritemovedata'].removeprefix('SPRITEMOVEDATA_STANDING_'),'south')) for o in m['objects']]
        image=scene(art,pack,a['tileset_name'],a['width'],a['height'],m['blocks'],actors,native_origin=True)
        image.save(output/f'{name}-whole-original.png')
    # Native input courses start on ordinary Floor and stop against real new rock.
    for prefix in ['MnR12','MnR13','MnR17','MnRainyRiverIcefields']:
        candidates=[]
        for row in design['maps']:
            name=row['id']
            if not name.startswith(prefix):continue
            m=pack['data']['maps'][name];objects=m['objects'];bodies={(o['x'],o['y']) for o in objects}
            for p,kind in kinds(name).items():
                if kind!='FLOOR' or any(abs(p[0]-o['x'])+abs(p[1]-o['y'])<max(4,(o.get('radius') or 0)+3) for o in objects):continue
                for d in deltas:
                    at=(name,p);path=[];seams=0;old_water=0
                    for _ in range(256):
                        nxt=neighbor(*at,d)
                        if not nxt:break
                        nk=kinds(nxt[0])[nxt[1]]
                        if nk!='ICE':
                            if len(path)>=3 and nxt in rock_sites and (old_water>=3 or prefix=='MnRainyRiverIcefields'):
                                candidates.append((seams,len(path),name,p,d,at,nxt,path))
                            break
                        if any((o['x'],o['y'])==nxt[1] for o in pack['data']['maps'][nxt[0]]['objects']):break
                        seams+=nxt[0]!=at[0]
                        old_water+=old_kinds(nxt[0])[nxt[1]]=='WATER'
                        path.append(dict(map=nxt[0],point=list(nxt[1])));at=nxt
        assert candidates,('no native lake course',prefix)
        seams,length,name,p,d,stop,rock,path=max(candidates,key=lambda c:(c[0],c[1]))
        cases.append(dict(id=prefix,map=name,start=list(p),direction=list(d),stop=dict(map=stop[0],point=list(stop[1])),rock=dict(map=rock[0],point=list(rock[1])),path=path,seams=seams))
    edge_cases=[];seen_edges=set()
    for name,rock in sorted(rock_sites):
        a=pack['data']['maps'][name]['attributes'];x,y=rock
        for d,on_edge in [((-1,0),x==0),((1,0),x==a['width']*2-1),((0,-1),y==0),((0,1),y==a['height']*2-1)]:
            p=(x-d[0],y-d[1])
            if on_edge and kinds(name).get(p)=='ICE' and (name,d) not in seen_edges:
                seen_edges.add((name,d));edge_cases.append(dict(map=name,start=list(p),direction=list(d),rock=list(rock)))
    for route,rows,columns in [('R17',3,1)]:
        width=1792 if columns==2 else 1024;image=Image.new('RGB',(width,rows*768),'#101820')
        for row in range(rows):
            for col in range(columns):
                name=f'Mn{route}C{col:02}R{row:02}';m=pack['data']['maps'][name];a=m['attributes'];actors=[dict(tile=[o['x'],o['y']],sprite=o['sprite'],facing='south') for o in m['objects']]
                chunk=scene(art,pack,a['tileset_name'],a['width'],a['height'],m['blocks'],actors,native_origin=True)
                image.paste(chunk,(col*1024,row*768))
        image.save(output/f'{route}-complete-original.png');image.thumbnail((896,1536));image.save(output/f'{route}-preview.png')
    proof=dict(pack=str(written) if written else 'source preview',pack_sha256=hashlib.sha256(written.read_bytes()).hexdigest() if written else None,
               source_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),maps=[r['id'] for r in design['maps']],changes=dict(counts),
               crystal_rock_quadrants=len(rock_sites),border_blocks={r['id']:pack['data']['maps'][r['id']]['attributes']['border_block'] for r in design['maps']},all_ice_edges_have_connections=True,all_original_paths_events_and_activities_preserved=True,cases=cases,edge_cases=edge_cases)
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps({k:v for k,v in proof.items() if k!='cases'},indent=2),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--written',type=Path);parser.add_argument('--source',type=Path,default=SOURCE);args=parser.parse_args();SOURCE=args.source;review(args.output,args.written)
