#!/usr/bin/env python3
"""Verify authored train geometry, both absolute boarding holes and source files."""
from pathlib import Path
from johto_art_sources import read_source
import argparse,collections,importlib.util,json,math,sys
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from model_asset_storage import read_model_json
spec=importlib.util.spec_from_file_location('train_kit',ROOT/'tools/build-train-station.py');kit=importlib.util.module_from_spec(spec);spec.loader.exec_module(kit)
def check(blender_required=True,source_manifest=None):
 a=kit.make_train();doc=read_model_json(ROOT/'crates/crystal-voxel-view/models/train_station/magnet_train_shell.mesh.json')
 assert doc==json.loads(json.dumps(kit.common.document(a))),'runtime differs from original editable geometry'
 assert doc['bounds']=={'min':[0,0,0],'max':[160,26.2,32]}
 assert doc['triangle_count']==2232 and len(doc['primitives'])==82
 for p in doc['primitives']:
  vs=[tuple(p['positions'][i:i+3])for i in range(0,len(p['positions']),3)];ns=[tuple(p['normals'][i:i+3])for i in range(0,len(p['normals']),3)]
  assert len(vs)==len(ns)and all(math.isfinite(v)for pt in vs+ns for v in pt)
  assert all(0<=c<=1 for c in p['base_color'])and p['base_color'][3]==1
  edges=collections.Counter();directions=collections.Counter();volume=0
  for i in range(0,len(p['indices']),3):
   ids=p['indices'][i:i+3];tri=[vs[j]for j in ids];normal=kit.common.normal(*tri)
   assert all(sum(v*w for v,w in zip(normal,ns[j]))>1-1e-8 for j in ids)
   for left,right in [(32,48),(112,128)]:
    assert max(v[0]for v in tri)<=left+1e-8 or min(v[0]for v in tri)>=right-1e-8 or max(v[2]for v in tri)<=16+1e-8,(p['name'],'closed boarding aperture')
   volume+=sum(x*y for x,y in zip(tri[0],kit.common.cross(tri[1],tri[2])))/6
   for x,y in zip(tri,tri[1:]+tri[:1]):edges[tuple(sorted((x,y)))]+=1;directions[(x,y)]+=1
  assert volume>0,(p['name'],'inward solid',volume)
  assert set(edges.values())=={2},(p['name'],'unsealed geometry')
  assert all(directions[(y,x)]==1 for x,y in directions),(p['name'],'inconsistent winding')
 if blender_required:
  manifest_path=source_manifest or ROOT/'art/johto/source/manifest.json'
  manifest=json.loads(manifest_path.read_text());entry=next(s for s in manifest.get('sources',[manifest])if s['file']=='train-station.blend')
  read_source(manifest_path.parent,entry)
 print('PASS train: 82 positive-volume watertight parts, 2232 triangles, both 16x16 boarding apertures empty at every height; generator/runtime exact'+('; editable Blender source verified'if blender_required else''))
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--runtime-only',action='store_true');parser.add_argument('--source-manifest',type=Path);args=parser.parse_args();check(not args.runtime_only,args.source_manifest)
