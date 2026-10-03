#!/usr/bin/env python3
"""Validate gate geometry, shared module bounds, and exact sparse topology."""
import json,math,re,sys
from pathlib import Path
from model_asset_storage import read_model_json
ROOT=Path(__file__).resolve().parent.parent
MODELS=ROOT/'crates/crystal-voxel-view/models/gate_counters'

def check():
 paths=sorted(MODELS.glob('*.mesh.json'));assert len(paths)==17
 bounds=[];triangles=0
 for path in paths:
  d=read_model_json(path);assert d['coordinate_system']=='right-handed; +Y up; front +Z'
  ps=d['primitives'];lo=[math.inf]*3;hi=[-math.inf]*3;count=0
  assert len(ps)>=3
  for p in ps:
   n=len(p['positions'])//3;assert n and len(p['positions'])%3==0 and len(p['normals'])==n*3
   assert p['indices'] and len(p['indices'])%3==0 and all(0<=i<n for i in p['indices'])
   assert all(math.isfinite(v) for v in p['positions']+p['normals'])
   assert all(0<=v<=1 for v in p['base_color'])
   for i in range(n):
    normal=p['normals'][i*3:i*3+3];assert abs(sum(v*v for v in normal)-1)<.0001
    for a in range(3):lo[a]=min(lo[a],p['positions'][i*3+a]);hi[a]=max(hi[a],p['positions'][i*3+a])
   # Nonuniform native fitting must still produce finite normalized normals.
   for i in range(0,len(p['normals']),3):
    normal=[v/s for v,s in zip(p['normals'][i:i+3],[8,12,8])];length=math.sqrt(sum(v*v for v in normal));assert length>0
    unit=[v/length for v in normal];assert abs(sum(v*v for v in unit)-1)<1e-9
   count+=len(p['indices'])//3
  assert lo==d['bounds']['min'] and hi==d['bounds']['max'] and count==d['triangle_count']
  assert all(hi[a]>lo[a] for a in range(3));triangles+=count
  if path.name.startswith('counter_'):bounds.append((lo,hi));assert lo[0]==lo[2]==-.5 and hi[0]==hi[2]==.5
  else:
   mats={p['name'] for p in ps};assert {'rubber','key','enamel','teal','brass'}<=mats
   assert count>1000
 assert all(b==bounds[0] for b in bounds),'Mismatched tile-module extents would crack joins'
 src=(ROOT/'crates/crystal-voxel-view/src/gate_counter_models.rs').read_text()
 assert all(f'/gate_counters/{p.name}' in src for p in paths)
 binding=(ROOT/'crates/crystal-voxel-view/src/mesh/gate_counter_bindings.rs').read_text()
 assert len(re.findall(r'fingerprint:',binding))==14
 print(f'Validated 17 original full-volume gate assets, {triangles:,} triangles; all 16 edge masks share exact flush bounds')
if __name__=='__main__':check()
