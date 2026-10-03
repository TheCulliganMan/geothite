#!/usr/bin/env python3
"""Validate Gym mesh geometry, per-part closedness, and connected wall datums."""
import sys,json,math,collections,argparse,struct
from pathlib import Path
from model_asset_storage import read_model_json
ROOT=Path(__file__).resolve().parents[1]
EXPECTED=['planter_leafy','planter_round','azalea_broad_tree','celadon_round_hedge']+[f'maze_wall_{i:02x}'for i in range(16)]
def validate(root,generated=None,unindexed=None):
 total=0;parts_total=0
 for name in EXPECTED:
  d=read_model_json(root/(name+'.mesh.json'));assert d['name']==name;
  if generated is not None:assert d==read_model_json(generated/(name+'.mesh.json')),(name,'generator output differs')
  if unindexed is not None:
   raw=read_model_json(unindexed/(name+'.mesh.json'))
   assert len(d['primitives'])==len(raw['primitives'])
   for indexed,plain in zip(d['primitives'],raw['primitives']):
    fields=lambda p:{k:v for k,v in p.items() if k not in ('positions','normals','indices')}
    assert fields(indexed)==fields(plain),(name,'material or component identity changed')
    def expanded(p):
     return b''.join(struct.pack('!12f',*(p['positions'][i*3:i*3+3]+p['normals'][i*3:i*3+3]+p['base_color']+[0.,0.])) for i in p['indices'])
    assert expanded(indexed)==expanded(plain),(name,indexed['part'],'indexing changed expanded attributes or triangle order')
  parts={};volumes=collections.defaultdict(float);vertices=[];ntri=0;colors=set()
  for p in d['primitives']:
   assert len(p['positions'])%3==0 and len(p['positions'])==len(p['normals'])
   xyz=list(zip(*[iter(p['positions'])]*3));normal=list(zip(*[iter(p['normals'])]*3));edges=parts.setdefault(p['part'],collections.Counter());colors.add(tuple(p['base_color']))
   assert all(math.isfinite(v)for x in xyz+normal for v in x)
   assert p['base_color'][3]==1 and all(0<=v<=1 for v in p['base_color'])
   assert all(abs(sum(v*v for v in n)-1)<1e-4 for n in normal)
   assert len(p['indices'])%3==0 and all(0<=i<len(xyz)for i in p['indices'])
   for start in range(0,len(p['indices']),3):
    ids=p['indices'][start:start+3];a,b,c=[xyz[i]for i in ids];u=[b[i]-a[i]for i in range(3)];v=[c[i]-a[i]for i in range(3)];cross=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);assert sum(x*x for x in cross)>1e-15,(name,p['part'],'degenerate')
    assert sum(cross[i]*normal[ids[0]][i]for i in range(3))>0,(name,p['part'],'winding')
    for a,b in [(a,b),(b,c),(c,a)]:edges[tuple(sorted((a,b)))]+=1
   for start in range(0,len(p['indices']),3):
    a,b,c=[xyz[i]for i in p['indices'][start:start+3]]
    volumes[p['part']]+=sum(a[i]*(b[(i+1)%3]*c[(i+2)%3]-b[(i+2)%3]*c[(i+1)%3])for i in range(3))/6
   ntri+=len(p['indices'])//3;vertices+=xyz
  for part,edges in parts.items():
   assert all(n==2 for n in edges.values()),(name,part,'open or non-manifold part',collections.Counter(edges.values()))
   assert volumes[part]>1e-9,(name,part,'nonpositive enclosed volume',volumes[part])
  assert ntri==d['triangle_count'] and 24<=ntri<1000
  assert len(colors)>=3
  for axis in range(3):
   lo=min(v[axis]for v in vertices);hi=max(v[axis]for v in vertices)
   assert hi-lo>.1 and abs(lo-d['bounds']['min'][axis])<1e-5 and abs(hi-d['bounds']['max'][axis])<1e-5
  if name.startswith('maze_wall_'):
   assert d['bounds']=={'min':[-.5,0.,-.5],'max':[.5,1.,.5]},(name,'join datum changed',d['bounds'])
  total+=ntri;parts_total+=len(parts);print(f'{name}: {ntri} triangles, {len(parts)} closed parts')
 print(f'PASS: {len(EXPECTED)} meshes, {parts_total} closed original parts, {total} triangles; all 16 wall joins retain identical datums')
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('models',nargs='?',type=Path,default=ROOT/'crates/crystal-voxel-view/models/gym_scenery')
 parser.add_argument('--generated',type=Path,help='Compare exact decoded documents with fresh Blender exports')
 parser.add_argument('--unindexed-reference',type=Path,help='Prove expanded f32 attributes and triangle order against fresh unindexed exports')
 args=parser.parse_args();validate(args.models,args.generated,args.unindexed_reference)
