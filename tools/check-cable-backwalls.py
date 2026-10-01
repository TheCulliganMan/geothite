#!/usr/bin/env python3
"""Validate the Cable Club terminal and the reused open portal's aperture."""
from pathlib import Path
import base64,collections,gzip,hashlib,importlib.util,json,math
from model_asset_storage import read_model_json
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('cable_backwalls',ROOT/'tools/build-cable-backwalls.py');kit=importlib.util.module_from_spec(spec);spec.loader.exec_module(kit)
def check():
 doc=read_model_json(ROOT/'crates/crystal-voxel-view/models/cable_club/link_console.mesh.json');a=kit.make_console()
 assert doc==json.loads(json.dumps(kit.common.document(a))), 'Runtime differs from original editable geometry'
 assert doc['bounds']=={'min':[0,0,0],'max':[16,20,32]};assert doc['triangle_count']==812
 for p in doc['primitives']:
  vs=[tuple(p['positions'][i:i+3])for i in range(0,len(p['positions']),3)];ns=[tuple(p['normals'][i:i+3])for i in range(0,len(p['normals']),3)]
  assert len(vs)==len(ns);assert all(math.isfinite(x)for v in vs+ns for x in v)
  assert min(v[0]for v in vs)>=8 or min(v[2]for v in vs)>=8, 'Geometry crosses source negative space'
  assert all(0<=c<=1 for c in p['base_color'])and p['base_color'][3]==1
  edges=collections.Counter();direction=collections.Counter();volume=0
  for i in range(0,len(p['indices']),3):
   ids=p['indices'][i:i+3];tri=[vs[j]for j in ids];n=kit.common.normal(*tri)
   assert all(sum(v*w for v,w in zip(n,ns[j]))>1-1e-8 for j in ids)
   volume+=sum(x*y for x,y in zip(tri[0],kit.common.cross(tri[1],tri[2])))/6
   for x,y in zip(tri,tri[1:]+tri[:1]):edges[tuple(sorted((x,y)))]+=1;direction[(x,y)]+=1
  assert volume>0 and set(edges.values())=={2},p['name'];assert all(direction[(y,x)]==1 for x,y in direction)
 # Reuse the specific existing open sliding-door asset, and prove that its
 # central 8-source-pixel-wide x 14-source-pixel-high aperture stays empty.
 door=read_model_json(ROOT/'crates/crystal-voxel-view/models/interiors/gate_door_frame.mesh.json');lo=door['bounds']['min'];hi=door['bounds']['max']
 for p in door['primitives']:
  vs=[[ (p['positions'][i+j]-lo[j])/(hi[j]-lo[j])*[16,18,3][j] for j in range(3)]for i in range(0,len(p['positions']),3)]
  for i in range(0,len(p['indices']),3):
   t=[vs[j]for j in p['indices'][i:i+3]]
   assert max(v[0]for v in t)<=4 or min(v[0]for v in t)>=12 or max(v[1]for v in t)<=1 or min(v[1]for v in t)>=15, 'Door frame fills central aperture'
 manifest=json.loads((ROOT/'art/johto/source/manifest.json').read_text());entry=next(s for s in manifest['sources']if s['file']=='cable-backwalls.blend')
 payload=b''.join(base64.b64decode((ROOT/'art/johto/source'/p['file']).read_bytes(),validate=True)for p in entry['chunks'])
 assert len(payload)==entry['bytes']and hashlib.sha256(payload).hexdigest()==entry['sha256']
 raw=gzip.decompress(payload);assert raw.startswith(b'BLENDER')and len(raw)==entry['uncompressed_bytes']and hashlib.sha256(raw).hexdigest()==entry['uncompressed_sha256']
 print(f'Cable rear kit: {len(doc["primitives"])} closed positive-volume named parts, 812 triangles; exact negative-space corner, open doorway aperture and editable Blender source verified')
if __name__=='__main__':check()
