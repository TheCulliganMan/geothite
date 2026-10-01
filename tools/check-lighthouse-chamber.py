"""Validate each named component's closed topology, winding and source bounds."""
from pathlib import Path
from model_asset_storage import read_model_json as decode
from model_asset_storage import stored_model_size
from collections import Counter
import json,math,sys
NAMES=['tea_table','keeper_cot','red_stool']
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def check(path):
 d=decode(path);assert d['format']=='geothite-lighthouse-chamber-v1';parts=[];labels=set();total=0;materials=set()
 for p in d['primitives']:
  assert p['part'] not in labels;labels.add(p['part']);materials.add(p['material']);assert p['base_color'][3]==1 and all(math.isfinite(v) and 0<=v<=1 for v in p['base_color']);assert 'source_pixel' not in p
  pos=list(zip(*[iter(p['positions'])]*3));norm=list(zip(*[iter(p['normals'])]*3));assert len(pos)==len(norm);assert all(math.isfinite(v) and 0<=v<=1 for xyz in pos for v in xyz)
  assert all(abs(sum(v*v for v in n)-1)<1e-6 for n in norm);edges=Counter();directions=Counter();volume=0.;count=0
  for i in range(0,len(p['indices']),3):
   ids=p['indices'][i:i+3];assert all(j<len(pos) for j in ids);a,b,c=[pos[j] for j in ids];n=cross(sub(b,a),sub(c,a));assert sum(v*v for v in n)>1e-16,p['part'];assert all(sum(v*w for v,w in zip(n,norm[j]))>1e-12 for j in ids)
   volume+=sum(v*w for v,w in zip(a,cross(b,c)))/6;count+=1
   for u,v in [(a,b),(b,c),(c,a)]:
    e=tuple(sorted((u,v)));edges[e]+=1;directions[e]+=1 if u<v else -1
  assert all(v==2 for v in edges.values()),(p['part'],Counter(edges.values()));assert all(v==0 for v in directions.values()),p['part'];assert volume>1e-10,(p['part'],volume)
  parts.append({'part':p['part'],'triangles':count,'signed_volume':volume});total+=count
 assert total<2000
 if d['name']=='tea_table':assert {'Teapot rising pouring spout','Teapot open handle','Cup loop handle','Linen tea runner'}<=labels
 if d['name']=='keeper_cot':assert {'Cream pillow broad facet','Folded sage blanket','Ivory mattress welt'}<=labels
 if d['name']=='red_stool':assert d['dimensions_pixels'][1]==6
 return {'asset':d['name'],'triangles':total,'closed_parts':len(parts),'runtime_bytes':stored_model_size(path),'materials':sorted(materials),'parts':parts}
if __name__=='__main__':
 root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/lighthouse_chamber'
 print(json.dumps([check(root/(name+'.mesh.json')) for name in NAMES],indent=2))
