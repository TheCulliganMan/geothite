#!/usr/bin/env python3
"""Check the original actor kit's public source registry and mesh invariants.

Reads only editable generators and our canonical runtime JSON/GLB models. No packs or
external images are needed; this does not claim coverage of unknown species.
"""
import argparse, ast, hashlib, json, math, re
from pathlib import Path
from cyndaquil_glb import read_cyndaquil
from totodile_glb import read_totodile
from model_asset_storage import read_model_bytes, read_model_json, read_model_text, validate_storage

ROOT = Path(__file__).resolve().parents[1]

def verify(directory):
 source=(ROOT/'crates/crystal-voxel-view/src/johto_actor_props.rs').read_text()
 registry=re.findall(r'^\s*\w+ => "([a-z0-9_]+)",',source,re.M)
 assert len(registry)==73 and len(set(registry))==73, f'Expected 73 unique original model keys, found {len(registry)}'
 tree=ast.parse((ROOT/'tools/build-actor-props.py').read_text())
 authored=[k.value for n in tree.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='BUILDERS' for t in n.targets) for k in n.value.keys]
 assert set(authored)==set(registry), 'Generator and Rust registry differ'
 total_tri=0; total_vertices=0; digests=set(); biggest=(0,'')
 for name in registry:
  if name=='battle_cyndaquil':data=read_cyndaquil(directory/(name+'.glb'))
  elif name=='battle_totodile':data=read_totodile(directory/(name+'.glb'))
  else:data=read_model_json(directory/(name+'.mesh.json'))
  assert data['name']==name and data['version']==1
  digest=hashlib.sha256(json.dumps(data['primitives'],sort_keys=True,separators=(',',':')).encode()).hexdigest();assert digest not in digests, f'Duplicate model bytes: {name}';digests.add(digest)
  assert data['coordinate_system']=='+Y up; front +Z; floor-centered root'
  points=[];triangles=0
  for pi,p in enumerate(data['primitives']):
   tag=f'{name} primitive {pi}';pos=p['positions'];nor=p['normals'];ind=p['indices'];col=p['base_color']
   assert len(pos)>=9 and len(pos)%3==0 and len(nor)==len(pos), tag
   assert ind and len(ind)%3==0 and all(type(i)==int and 0<=i<len(pos)//3 for i in ind),tag
   assert all(math.isfinite(v) for v in pos+nor) and all(0<=v<=1 for v in col),tag
   ps=list(zip(*[iter(pos)]*3));ns=list(zip(*[iter(nor)]*3));points+=ps;volume=0
   for n in ns:assert abs(sum(x*x for x in n)-1)<.00002,(tag,'non-unit normal')
   for i in range(0,len(ind),3):
    a,b,c=(ps[j] for j in ind[i:i+3]);u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)]
    cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
    # Smooth normal dot must agree with outward triangle winding; meshes may
    # retain a tiny coincident pole after authoring quantization.
    average=[sum(ns[j][k] for j in ind[i:i+3]) for k in range(3)]
    assert sum(cross[k]*average[k] for k in range(3))>=-1e-7,(tag,'opposed normal and triangle')
    volume+=a[0]*(b[1]*c[2]-b[2]*c[1])+a[1]*(b[2]*c[0]-b[0]*c[2])+a[2]*(b[0]*c[1]-b[1]*c[0])
   # All modeled surfaces have a positive enclosed volume, except the two
   # open Pokeball hemispheres (with a legitimate cut plane), and original
   # rider subparts whose exported materials partition a complete surface.
   if name not in ('chris_bike','kris_bike'):
    assert volume>=-1e-5,(tag,'inverted closed volume',volume)
   triangles+=len(ind)//3;total_vertices+=len(pos)//3
  lo=[min(p[a] for p in points) for a in range(3)];hi=[max(p[a] for p in points) for a in range(3)]
  assert abs(lo[1])<.0001,(name,'not floor rooted',lo)
  assert all(hi[a]-lo[a]>.005 for a in range(3)),(name,'flat geometry')
  assert max(abs(v) for v in lo+hi)<3,(name,'unexpected unit scale')
  assert triangles>=20 and triangles<25000,(name,'triangle budget',triangles)
  total_tri+=triangles;biggest=max(biggest,(triangles,name))
 source_families=[n for n in registry if not n.startswith('battle_')]
 assert len(source_families)==66 and sum(n.startswith('icon_') for n in source_families)==38
 print(f'PASS: {len(registry)} original volumetric models; 66 source families (28 sprites, 38 icons), 7 dedicated battle species')
 print(f'PASS: {total_tri:,} triangles, {total_vertices:,} vertices; max {biggest[0]:,} triangles ({biggest[1]}); registry/export parity, finite attributes, winding/normals, ground anchors')

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--models',type=Path,default=ROOT/'crates/crystal-voxel-view/models/actor_props');a=p.parse_args();verify(a.models)
