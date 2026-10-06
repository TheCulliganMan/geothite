"""Validate original sign geometry and live-lettering placement metadata.

python tools/check-outdoor-sign-frames.py [repository-or-staged-files-root]
No external pack is read or exported by this authoring-asset check.
"""
import sys,math
from pathlib import Path
from model_asset_storage import read_model_json
root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent.parent
for name in ['modern_frame','park_frame','kanto_board','forest_timber']:
 d=read_model_json(root/'crates/crystal-voxel-view/models/outdoor_signs'/f'{name}.mesh.json')
 assert d['name']==name and d['coordinate_system']=='right-handed; +Y up; front +Z'
 assert len(d['primitives'])>=4
 lo=d['bounds']['min'];hi=d['bounds']['max'];assert all(hi[a]>lo[a]for a in range(3));assert hi[2]-lo[2]>=2.5 and hi[1]-lo[1]>=14
 face=d['live_face'];assert len(face)==5 and all(math.isfinite(v)and 0<=v<=1 for v in face)
 assert face[0]<face[1] and face[2]<face[3]
 tris=0;vol=0.;points=[];normals=[];triangles=[]
 for p in d['primitives']:
  assert 'texture'not in p and 'image'not in p
  ps=list(zip(*[iter(p['positions'])]*3));ns=list(zip(*[iter(p['normals'])]*3));assert len(ps)==len(ns)
  assert all(len(p[k])%3==0 for k in ['positions','normals','indices'])
  assert all(0<=i<len(ps)for i in p['indices']);assert all(math.isfinite(v)for k in ['positions','normals'] for v in p[k])
  assert all(abs(sum(v*v for v in n)-1)<1e-4 for n in ns)
  for i in range(0,len(p['indices']),3):
   a,b,c=[ps[j]for j in p['indices'][i:i+3]];triangles.append((a,b,c))
   cross=(b[1]*c[2]-b[2]*c[1],b[2]*c[0]-b[0]*c[2],b[0]*c[1]-b[1]*c[0]);vol+=sum(a[k]*cross[k]for k in range(3))/6
  tris+=len(p['indices'])//3;points+=ps;normals+=ns
 assert 300<=tris<=5000 and vol>100
 assert all(abs(min(p[a]for p in points)-lo[a])<1e-5 and abs(max(p[a]for p in points)-hi[a])<1e-5 for a in range(3))
 # Each sign has actual rear, front, upward and underside-facing surfaces.
 for axis,direction in [(0,-1),(0,1),(1,-1),(1,1),(2,-1),(2,1)]:assert any(n[axis]*direction>.98 for n in normals)
 assert len({round(p[0],2)for p in points if p[1]<lo[1]+.02})>=4
 # Project rays onto the complete front inset: posts, rims and fasteners must
 # not hide even the outside letters in the live source crop.
 left=lo[0]+face[0]*(hi[0]-lo[0]);right=lo[0]+face[1]*(hi[0]-lo[0])
 bottom=lo[1]+face[2]*(hi[1]-lo[1]);top=lo[1]+face[3]*(hi[1]-lo[1]);front=lo[2]+face[4]*(hi[2]-lo[2])
 for iy in range(13):
  for ix in range(33):
   x=left+(right-left)*(ix+.01)/32.02;y=bottom+(top-bottom)*(iy+.01)/12.02
   for a,b,c in triangles:
    denom=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
    if abs(denom)<1e-10:continue
    u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/denom
    v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/denom;w=1-u-v
    if min(u,v,w)>=-1e-7:
     z=u*a[2]+v*b[2]+w*c[2]
     assert z<front-.001,(name,'solid obscures live lettering',x,y,z,front)
 print(f'{name}: {tris} triangles, {len(d["primitives"])} materials, positive solid volume {vol:.3f}, verified inset and six face directions')
