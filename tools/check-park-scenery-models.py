#!/usr/bin/env python3
"""Verify compact original park geometry and the live pond aperture.
python tools/check-park-scenery-models.py [repository-or-staged-files-root]
"""
import math,sys
from collections import Counter,defaultdict
from pathlib import Path
from model_asset_storage import read_model_json,validate_storage
root=Path(sys.argv[1])if len(sys.argv)>1 else Path(__file__).resolve().parents[1]
modelroot=root/'crates/crystal-voxel-view/models/park_scenery'
validate_storage(modelroot)
for name in ['litter_bin','pedestal_fountain','pond_basin']:
 d=read_model_json(modelroot/(name+'.mesh.json'));assert d['name']==name;assert d['coordinate_system']=='right-handed; +Y up; front +Z'
 points=[];triangles=[];normals=[];edges=Counter();volume=0;adj=defaultdict(set)
 for p in d['primitives']:
  assert 'image'not in p and 'texture'not in p
  assert all(math.isfinite(v)for k in ['positions','normals','base_color']for v in p[k])
  ps=list(zip(*[iter(p['positions'])]*3));ns=list(zip(*[iter(p['normals'])]*3));assert len(ps)==len(ns);assert len(p['indices'])%3==0
  assert all(abs(sum(v*v for v in n)-1)<1e-4 for n in ns)
  points.extend(ps);normals.extend(ns)
  for i in range(0,len(p['indices']),3):
   a,b,c=[ps[j]for j in p['indices'][i:i+3]];triangles.append((a,b,c));cross=(b[1]*c[2]-b[2]*c[1],b[2]*c[0]-b[0]*c[2],b[0]*c[1]-b[1]*c[0]);volume+=sum(a[k]*cross[k]for k in range(3))/6
   for u,v in [(a,b),(b,c),(c,a)]:edges[tuple(sorted((u,v)))]+=1;adj[u].add(v);adj[v].add(u)
 assert all(n==2 for n in edges.values()),'non-manifold/open mesh'
 assert 100<=len(triangles)<=1800 and volume>0
 lo=d['bounds']['min'];hi=d['bounds']['max'];assert all(abs(min(p[a]for p in points)-lo[a])<1e-5 and abs(max(p[a]for p in points)-hi[a])<1e-5 for a in range(3))
 for axis,direction in [(0,-1),(0,1),(1,-1),(1,1),(2,-1),(2,1)]:assert any(n[axis]*direction>.90 for n in normals)
 # Straight rays through each intended hollow opening must reach its interior.
 def heights(x,z):
  found=[]
  for a,b,c in triangles:
   denom=(b[2]-c[2])*(a[0]-c[0])+(c[0]-b[0])*(a[2]-c[2])
   if abs(denom)<1e-10:continue
   u=((b[2]-c[2])*(x-c[0])+(c[0]-b[0])*(z-c[2]))/denom;v=((c[2]-a[2])*(x-c[0])+(a[0]-c[0])*(z-c[2]))/denom;w=1-u-v
   if min(u,v,w)>=-1e-7:found.append(u*a[1]+v*b[1]+w*c[1])
  return found
 if name=='litter_bin':assert max(heights(.1,.1))<2,'bin opening capped'
 if name=='pedestal_fountain':assert max(heights(3.0,.1))<7.4,'pedestal bowl is not recessed'
 if name=='pond_basin':
  # No authored solid center may replace the live runtime cap. The center and
  # central ellipse stay clear all the way through the below-water footing.
  for k in range(24):
   a=k/24*math.tau;assert not heights(4.8*math.cos(a),3.3*math.sin(a)),'pond live aperture occluded'
  assert lo[1]==0 and hi[1]==6 and abs(hi[0]-7.04)<1e-4 and abs(hi[2]-5.2)<1e-4
 print(f'{name}: {len(triangles)} triangles, {len(d["primitives"])} materials, closed positive volume {volume:.3f}; verified real hollow opening and six face directions')
