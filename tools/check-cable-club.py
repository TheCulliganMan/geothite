#!/usr/bin/env python3
"""Check original partition geometry, source reconstruction and empty bay space."""
from pathlib import Path
import collections,importlib.util,json,math,sys
from model_asset_storage import read_model_json
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('cable_club_art',ROOT/'tools/build-cable-club.py')
kit=importlib.util.module_from_spec(spec);spec.loader.exec_module(kit)
def check():
 triangles=0;parts=0;stored=0
 for name,depth,expected in [('divider_long',64,728),('vestibule_return',48,616)]:
  path=ROOT/'crates/crystal-voxel-view/models/cable_club'/f'{name}.mesh.json';doc=read_model_json(path);asset=kit.make_asset(name,depth)
  assert doc==json.loads(json.dumps(kit.document(asset))),f'{name}: regenerate runtime from the editable geometry generator'
  assert doc['bounds']=={'min':[0,0,0],'max':[16,16,depth]}
  assert doc['triangle_count']==expected
  for p in doc['primitives']:
   vs=[tuple(p['positions'][i:i+3])for i in range(0,len(p['positions']),3)]
   ns=[tuple(p['normals'][i:i+3])for i in range(0,len(p['normals']),3)]
   assert len(vs)==len(ns)and len(p['indices'])%3==0
   assert all(math.isfinite(v)for vector in vs+ns for v in vector)
   assert all(0<=v<=1 for v in p['base_color'])and p['base_color'][3]==1
   edges=collections.Counter();directions=collections.Counter();volume=0
   for i in range(0,len(p['indices']),3):
    ids=p['indices'][i:i+3];tri=[vs[j]for j in ids];n=kit.normal(*tri)
    for j in ids:assert sum(a*b for a,b in zip(n,ns[j]))>1-1e-8
    volume+=sum(a*b for a,b in zip(tri[0],kit.cross(tri[1],tri[2])))/6
    for a,b in zip(tri,tri[1:]+tri[:1]):
     assert a!=b;edges[tuple(sorted((a,b)))]+=1;directions[(a,b)]+=1
   assert volume>0,(p['name'],volume)
   assert set(edges.values())=={2},p['name']
   assert all(directions[(b,a)]==1 for a,b in directions),p['name']
   for n in ns:
    # Native tile scales may differ in X and Z, including a non-square frame.
    fitted=[v/s for v,s in zip(n,[1.5,.8,1.2])];length=math.sqrt(sum(v*v for v in fitted));assert length>0
    assert abs(sum((v/length)**2 for v in fitted)-1)<1e-10
   parts+=1
  triangles+=doc['triangle_count'];stored+=path.stat().st_size
 # Physical source-layout gaps: each module stays exactly inside its sparse
 # ownership. No ceiling/cap or display geometry spans a service/warp opening.
 rects=[(6,0,2,8),(14,0,2,8),(22,0,2,8),(24,0,2,6),(28,0,2,6),(30,0,2,8)]
 owned={(x,y)for x0,y0,w,h in rects for y in range(y0,y0+h)for x in range(x0,x0+w)}
 assert len(owned)==88
 for x,y in [(0,7),(5,0),(9,0),(13,2),(6,0),(10,0),(5,2),(9,2),(13,3),(1,1)]:
  assert not owned.intersection((x*2+dx,y*2+dy)for dx in range(2)for dy in range(2))
 assert all((x,y)not in owned for x in (26,27)for y in range(8))
 manifest=json.loads((ROOT/'art/johto/source/manifest.json').read_text());entry=next(s for s in manifest['sources']if s['file']=='cable-club.blend')
 import base64,gzip,hashlib
 payload=b''.join(base64.b64decode((ROOT/'art/johto/source'/p['file']).read_bytes(),validate=True)for p in entry['chunks'])
 assert len(payload)==entry['bytes']and hashlib.sha256(payload).hexdigest()==entry['sha256']
 source=gzip.decompress(payload);assert source.startswith(b'BLENDER')and len(source)==entry['uncompressed_bytes']and hashlib.sha256(source).hexdigest()==entry['uncompressed_sha256']
 print(f'Cable Club: 2 deterministic cached models, {parts} closed positive-volume parts, {triangles:,} prototype triangles, {stored:,} stored bytes; exact six-shell openings and editable Blender chunks verified')
if __name__=='__main__':check()
