#!/usr/bin/env python3
"""Lightweight source/closed-geometry contracts; no Cargo, rustc or Blender.

python3 tools/check-ship-fronts.py --pack content-packs/realtime-clock.browser.crystalpack
This checks authored geometry and the external source contract, not native UI,
the Rust adapter, player movement, cutaway animation or performance.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
from model_asset_storage import read_model_json as decode
from model_asset_storage import stored_model_size

ROOT=Path(__file__).resolve().parents[1]
def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result)
    return result
def cross(a,b): return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def sub(a,b): return tuple(x-y for x,y in zip(a,b))
def owns(side,x,y):
    entrance=8 if side=='west' else 12
    return 0<=x<24 and 0<=y<4 and not entrance<=x<entrance+4 and (x,y)not in [(1,1),(22,1)]
def inside(side,x,z):
    eps=1e-5;door=64 if side=='west' else 96
    return -eps<=x<=192+eps and -eps<=z<=32+eps and not door+eps<x<door+32-eps and not (
        8+eps<z<16-eps and (8+eps<x<16-eps or 176+eps<x<184-eps))
def check_models():
    build=module('ship_front_build',ROOT/'tools/build-ship-fronts.py')
    results=[]
    for asset in build.assets():
        side=asset.name.rsplit('_',1)[1]
        path=ROOT/'crates/crystal-voxel-view/models/ship_fronts'/(asset.name+'.mesh.json')
        doc=decode(path);assert doc==json.loads(json.dumps(build.document(asset)))
        assert doc['format']=='geothite-ship-fronts-v1' and doc['dimensions_pixels']==[192,16,32]
        count=0;labels=set();all_normals=[];points=[]
        for part in doc['primitives']:
            assert part['part']not in labels;labels.add(part['part'])
            assert part['base_color'][3]==1 and all(math.isfinite(v)and 0<=v<=1 for v in part['base_color'])
            assert 'source_pixel'not in part
            vertices=list(zip(*[iter(part['positions'])]*3));normals=list(zip(*[iter(part['normals'])]*3))
            assert len(vertices)==len(normals) and all(math.isfinite(v)for p in vertices for v in p)
            assert all(abs(sum(v*v for v in p)-1)<1e-6 for p in normals)
            edges=Counter();winding=Counter();volume=0
            for k in range(0,len(part['indices']),3):
                ids=part['indices'][k:k+3];a,b,c=[vertices[i]for i in ids]
                normal=cross(sub(b,a),sub(c,a));assert sum(v*v for v in normal)>1e-16
                assert all(sum(x*y for x,y in zip(normal,normals[i]))>0 for i in ids)
                volume+=sum(x*y for x,y in zip(a,cross(b,c)))/6
                for u,v in [(a,b),(b,c),(c,a)]:
                    edge=tuple(sorted((u,v)));edges[edge]+=1;winding[edge]+=1 if u<v else -1
                # Vertices alone miss triangles that bridge a doorway or hole.
                for weights in [(1,0,0),(0,1,0),(0,0,1),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1/3,1/3,1/3)]:
                    sample=tuple(sum(p[i]*v for p,v in zip([a,b,c],weights)) for i in range(3))
                    assert inside(side,sample[0]*192,sample[2]*32),(part['part'],sample)
            assert all(v==2 for v in edges.values()),(part['part'],Counter(edges.values()))
            assert all(v==0 for v in winding.values()),part['part']
            assert volume>0,(part['part'],volume)
            count+=len(part['indices'])//3;all_normals+=normals;points+=vertices
        # Actual external surfaces in every orientation. No back/side removal.
        assert all(any(n[a]>.99 for n in all_normals) and any(n[a]<-.99 for n in all_normals)for a in range(3))
        assert all(min(p[a]for p in points)==0 and max(p[a]for p in points)==1 for a in range(3))
        assert count<3000
        results.append({'asset':asset.name,'closed_components':len(labels),'triangles':count,'bytes':stored_model_size(path)})
    return results
def check_pack(pack_path):
    helper=ROOT/'tools/check-ship-floor-source.py'
    if not helper.exists(): helper=Path(os.environ.get('GEOTHITE_ART_TOOLS','tools'))/helper.name
    read=module('ship_front_pack_reader',helper)
    before=hashlib.sha256(pack_path.read_bytes()).hexdigest();pack=read.load(pack_path)
    source=(ROOT/'crates/crystal-voxel-view/src/mesh/ship_front_source.rs').read_text()
    m=pack['data']['maps']['FastShipB1F'];assert m['attributes']['tileset_name']=='lighthouse'
    assert not m['script_block_changes'];assert m['attributes']['width']==16
    layout=pack['runtime_files']['data/tilesets/lighthouse_metatiles.bin']
    collision=pack['data']['tilesets']['lighthouse']['collision']
    selected=set();void=[];mouth=[]
    for side,left,digest in [('west',12,0x0b6467b629e9e383),('east',36,0x412b19233c4b0cd3)]:
        assert f'0x{digest:016x}'in source
        result=0xcbf29ce484222325
        for y in range(4):
            for x in range(24):
                xx,yy=left+x,12+y;b=m['blocks'][yy//4*16+xx//4];tile=layout[b*16+(yy%4)*4+xx%4]
                for v in [b&255,b>>8,xx%4,yy%4,tile&255,tile>>8]:result=((result^v)*0x100000001b3)&((1<<64)-1)
                permission=collision[f'{b:02x}'][(yy%4//2)*2+xx%4//2]
                if owns(side,x,y):
                    assert permission=='WALL' and tile!=0x01
                    selected.add((xx,yy))
                elif (x,y)in [(1,1),(22,1)]:
                    assert tile==0x01;void.append((xx,yy))
                else:
                    assert b==0x0b and permission=='FLOOR' and tile in [0x0d,0x1d]
                    mouth.append((xx,yy))
        assert result==digest,(side,hex(result))
    assert len(selected)==156 and len(void)==4 and len(mouth)==32
    assert set(void)=={(13,13),(34,13),(37,13),(58,13)}
    def clear_event(event):
        return not any((event['x']*2+dx,event['y']*2+dy)in selected for dx in range(2)for dy in range(2))
    assert all(clear_event(e)for events in m['events'].values()for e in events)
    assert all(clear_event(o)for o in m['objects'])
    assert {(e['x'],e['y'])for e in m['events']['warps']}=={(5,11),(31,13)}
    assert {(e['x'],e['y'])for e in m['events']['coord_events']}=={(30,7),(31,7)}
    assert hashlib.sha256(pack_path.read_bytes()).hexdigest()==before
    return {'guarded_assemblies':2,'architecture_claims':156,'previously_partial_claims_replaced':36,
            'residual_architecture_finished':120,'preserved_void_cells':len(void),'preserved_entrance_cells':len(mouth),
            'event_and_actor_footprints_clear':True,'external_pack_unchanged':True}
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path)
    args=parser.parse_args();result={'models':check_models()}
    if args.pack:result['source']=check_pack(args.pack)
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
