"""Validate each named component's closed topology, winding and source bounds."""
from pathlib import Path
from collections import Counter
import base64,gzip,hashlib,json,math,sys
NAMES=['tea_table_short','tea_table_long','tea_table_mess','captains_desk','lower_bulkhead_u','captains_chair']
def decode(path):
 e=json.loads(path.read_text());assert e['storage']=='geothite-model-gzip-v1';raw=gzip.decompress(base64.b64decode(e['data'],validate=True));assert len(raw)==e['bytes'];assert hashlib.sha256(raw).hexdigest()==e['sha256'];return json.loads(raw)
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def check(path):
 d=decode(path);assert d['format']=='geothite-ship-rooms-v1';parts=[];labels=set();total=0;materials=set()
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
 assert total<3000
 if d['name'].startswith('tea_table_'):
  assert 'Linen tea runner' in labels
  assert any(x.startswith('Cup loop handle') for x in labels)
  assert (any(x.startswith('Teapot rising pouring spout') for x in labels)) == (d['name']!='tea_table_short')
 if d['name']=='captains_desk':assert {'Open logbook leather cover','Book central fabric binding','Port open cream page','Starboard open cream page'}<=labels
 if d['name']=='captains_chair':assert {'Chair navy cushion broad facets','Chair faceted navy back cushion','Chair back upper crest'}<=labels
 if d['name']=='lower_bulkhead_u':
  assert {'Continuous joined recessed steel room shell','Continuous deck skirting','Continuous ivory top rail'}<=labels
  def inside(v):
   x,y,z=[n*scale for n,scale in zip(v,d['dimensions_pixels'])]
   return -1e-5<=x<=192.00001 and -1e-5<=z<=128.00001 and (z>=124.99999 or x<=8.00001 or x>=183.99999 or (z<=112.00001 and (x<=16.00001 or x>=175.99999)))
  for p in d['primitives']:
   vv=list(zip(*[iter(p['positions'])]*3))
   for k in range(0,len(p['indices']),3):
    a,b,c=[vv[j] for j in p['indices'][k:k+3]]
    for sample in [a,b,c,tuple((a[i]+b[i]+c[i])/3 for i in range(3))]+[tuple((u[i]+v[i])/2 for i in range(3)) for u,v in [(a,b),(b,c),(c,a)]]:assert inside(sample),(p['part'],sample)
 return {'asset':d['name'],'triangles':total,'closed_parts':len(parts),'runtime_bytes':path.stat().st_size,'materials':sorted(materials),'parts':parts}
if __name__=='__main__':
 root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/ship_rooms'
 print(json.dumps([check(root/(name+'.mesh.json')) for name in NAMES],indent=2))
