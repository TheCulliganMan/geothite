"""Verify original structural meshes, source-aperture geometry and handedness.
Usage: python tools/check-structure-extensions.py [model-directory]
"""
import json,math,sys
from pathlib import Path
from model_asset_storage import read_model_json
P=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/world_exteriors'
names=['highland_mesa','diglett_cave','tower_forecourt','lavender_radio_tower','kanto_route_gate']+['ice_shelf_'+s for s in ['northwest','north','northeast','interior','southwest','south','southeast','notch_east','notch_west','stair_right','stair_corner','stair_left']]
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def dot(a,b):return sum(x*y for x,y in zip(a,b))
def triangles(m):
 for p in m['primitives']:
  vs=list(zip(*[iter(p['positions'])]*3));ns=list(zip(*[iter(p['normals'])]*3))
  for a,b,c in zip(*[iter(p['indices'])]*3):yield vs[a],vs[b],vs[c],ns[a]
models={};total=0
for name in names:
 m=read_model_json(P/f'{name}.mesh.json');models[name]=m;vertices=[];count=0
 for p in m['primitives']:
  assert len(p['positions'])==len(p['normals']) and len(p['positions'])%3==0
  assert len(p['indices'])%3==0 and all(0<=i<len(p['positions'])//3 for i in p['indices'])
  assert all(math.isfinite(v) for v in p['positions']+p['normals']+p['base_color'])
  assert all(0<=v<=1 for v in p['base_color'])
  vertices.extend(zip(*[iter(p['positions'])]*3))
 for a,b,c,n in triangles(m):
  normal=cross(sub(b,a),sub(c,a));area=math.sqrt(dot(normal,normal));assert area>1e-9,(name,'degenerate')
  assert dot(normal,n)>area*.995,(name,'winding');assert abs(dot(n,n)-1)<.00001
  count+=1
 assert 24<count<12000,(name,count)
 for ax in range(3):
  assert abs(min(v[ax]for v in vertices)-m['bounds']['min'][ax])<1e-5
  assert abs(max(v[ax]for v in vertices)-m['bounds']['max'][ax])<1e-5
  assert m['bounds']['min'][ax]<m['bounds']['max'][ax]
 if 'door_anchor' in m:assert abs(m['door_anchor'][2]-m['bounds']['max'][2])<1e-6
 total+=count
# Barycentric intersection with a line parallel to one axis.
def line_hits(m,axis,point):
 uv=[i for i in range(3) if i!=axis];hits=[]
 for a,b,c,_ in triangles(m):
  a2=[a[i]for i in uv];b2=[b[i]for i in uv];c2=[c[i]for i in uv];p2=[point[i]for i in uv]
  d=(b2[1]-c2[1])*(a2[0]-c2[0])+(c2[0]-b2[0])*(a2[1]-c2[1])
  if abs(d)<1e-10:continue
  u=((b2[1]-c2[1])*(p2[0]-c2[0])+(c2[0]-b2[0])*(p2[1]-c2[1]))/d
  v=((c2[1]-a2[1])*(p2[0]-c2[0])+(a2[0]-c2[0])*(p2[1]-c2[1]))/d;w=1-u-v
  if min(u,v,w)>=-1e-7:hits.append(u*a[axis]+v*b[axis]+w*c[axis])
 return hits
# The genuine cave aperture has deep air before its rear shadow wall.
c=models['diglett_cave'];hits=line_hits(c,2,[0,.70,0]);assert hits and max(hits)<2.0,('sealed cave mouth',hits)
# Entry remains an unobstructed exact two-cell forecourt opening (x=-2..0).
f=models['tower_forecourt'];assert not line_hits(f,2,[-1,.40,0]),'blocked forecourt entry'
# Preserve signed native source coordinates; no x reflection of stair hands.
for name,x in [('ice_shelf_stair_right',1),('ice_shelf_stair_left',-1),('ice_shelf_stair_corner',-1)]:
 m=models[name];h1=max(line_hits(m,1,[x,0,.20]));h2=max(line_hits(m,1,[x,0,1.80]));assert h1>h2+.8,(name,'wrong rise direction',h1,h2)
print(f'Validated {len(names)} structure models, {total:,} triangles; exact cave and forecourt apertures; all three source stair hands and rises')
