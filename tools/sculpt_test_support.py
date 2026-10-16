"""Independent stored-mesh topology and shading checks shared by sculpture tests."""
import math
from collections import Counter,defaultdict
from chikorita_sculpt import cross,sub,dot

def assert_closed_models(self,models):
    for name,m in models.items():
        for p in m['primitives']:
            with self.subTest(species=name,part=p['part']):
                points=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
                edges=Counter();directions=defaultdict(int);adj=defaultdict(set);volume=0
                for i in range(0,len(p['indices']),3):
                    a,b,c=[points[j] for j in p['indices'][i:i+3]];n=cross(sub(b,a),sub(c,a))
                    self.assertGreater(dot(n,n),1e-22);volume+=dot(a,cross(b,c))
                    for x,y in ((a,b),(b,c),(c,a)):
                        edge=tuple(sorted((x,y)));edges[edge]+=1;directions[edge]+=1 if x<y else -1
                        adj[x].add(y);adj[y].add(x)
                self.assertTrue(all(n==2 for n in edges.values()))
                self.assertTrue(all(n==0 for n in directions.values()))
                self.assertGreater(volume,1e-11)
                seen=set();queue=[next(iter(adj))]
                while queue:
                    a=queue.pop()
                    if a in seen:continue
                    seen.add(a);queue.extend(adj[a]-seen)
                self.assertEqual(len(seen),len(adj))


def assert_model_shading(self,models):
    for name,m in models.items():
        for p in m['primitives']:
            with self.subTest(species=name,part=p['part']):
                ps=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
                ns=[tuple(p['normals'][i:i+3]) for i in range(0,len(p['normals']),3)]
                self.assertTrue(all(math.isfinite(v) for q in ps+ns for v in q))
                for n in ns:self.assertAlmostEqual(dot(n,n),1,places=5)
                for i in range(0,len(p['indices']),3):
                    ids=p['indices'][i:i+3];a,b,c=[ps[j] for j in ids];n=cross(sub(b,a),sub(c,a))
                    avg=tuple(sum(ns[j][k] for j in ids) for k in range(3))
                    self.assertGreaterEqual(dot(n,avg),-1e-8,(name,p['part'],i//3))




def ray_intersections(part, x, y):
    """Independent +Z ray hits on stored triangles, without authoring surface math."""
    positions = [part['positions'][i:i+3] for i in range(0, len(part['positions']), 3)]
    hits = []
    for i in range(0, len(part['indices']), 3):
        a, b, c = [positions[j] for j in part['indices'][i:i+3]]
        denominator = (b[1]-c[1])*(a[0]-c[0]) + (c[0]-b[0])*(a[1]-c[1])
        if abs(denominator) < 1e-12:
            continue
        u = ((b[1]-c[1])*(x-c[0]) + (c[0]-b[0])*(y-c[1])) / denominator
        v = ((c[1]-a[1])*(x-c[0]) + (a[0]-c[0])*(y-c[1])) / denominator
        if min(u, v, 1-u-v) >= -1e-7:
            hits.append(u*a[2] + v*b[2] + (1-u-v)*c[2])
    return hits


def parts_overlap(a, b):
    """Require a shared occupied interval in actual closed triangle volumes."""
    xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]))
    yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
    if xh<=xl or yh<=yl:return False
    for i in range(1,19):
        for j in range(1,23):
            x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23
            aa=sorted(ray_intersections(a,x,y));bb=sorted(ray_intersections(b,x,y))
            for al,ah in zip(aa[::2],aa[1::2]):
                for bl,bh in zip(bb[::2],bb[1::2]):
                    if min(ah,bh)-max(al,bl)>.00025:return True
    return False


_RAY_INDEX={}
def indexed_ray_intersections(parent,x,y):
 # Independent projection index over stored triangles. Every candidate keeps
 # the same barycentric test as the original unindexed ray oracle.
 key=id(parent)
 if key not in _RAY_INDEX:
  vs=[parent['positions'][i:i+3] for i in range(0,len(parent['positions']),3)]
  xmin=min(v[0] for v in vs)-1e-6;xmax=max(v[0] for v in vs)+1e-6;ymin=min(v[1] for v in vs)-1e-6;ymax=max(v[1] for v in vs)+1e-6
  nx=12;ny=20;bins=[[] for _ in range(nx*ny)]
  for k in range(0,len(parent['indices']),3):
   a,b,c=[vs[i] for i in parent['indices'][k:k+3]];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
   if abs(den)<1e-12:continue
   ix0=max(0,min(nx-1,int((min(a[0],b[0],c[0])-1e-6-xmin)/(xmax-xmin)*nx)));ix1=max(0,min(nx-1,int((max(a[0],b[0],c[0])+1e-6-xmin)/(xmax-xmin)*nx)))
   iy0=max(0,min(ny-1,int((min(a[1],b[1],c[1])-1e-6-ymin)/(ymax-ymin)*ny)));iy1=max(0,min(ny-1,int((max(a[1],b[1],c[1])+1e-6-ymin)/(ymax-ymin)*ny)))
   for ix in range(ix0,ix1+1):
    for iy in range(iy0,iy1+1):bins[ix*ny+iy].append((a,b,c,den))
  _RAY_INDEX[key]=(parent,xmin,xmax,ymin,ymax,nx,ny,bins)
 _,xmin,xmax,ymin,ymax,nx,ny,bins=_RAY_INDEX[key]
 if not xmin<=x<=xmax or not ymin<=y<=ymax:return []
 ix=min(nx-1,int((x-xmin)/(xmax-xmin)*nx));iy=min(ny-1,int((y-ymin)/(ymax-ymin)*ny));hits=[]
 for a,b,c,den in bins[ix*ny+iy]:
  u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
  if min(u,v,1-u-v)>=-1e-7:hits.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
 return hits


def crossed_faces(p):
 """Independent spatially culled proper segment/triangle intersection test."""
 def sub(a,b):return tuple(x-y for x,y in zip(a,b))
 def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
 def dot(a,b):return sum(x*y for x,y in zip(a,b))
 def hits(a,b,tri):
  v,e,f=tri[0],sub(tri[1],tri[0]),sub(tri[2],tri[0]);d=sub(b,a);h=cross(d,f);det=dot(e,h)
  if abs(det)<1e-12:return False
  s=sub(a,v);u=dot(s,h)/det
  if u<1e-8 or u>1-1e-8:return False
  q=cross(s,e);w=dot(d,q)/det;t=dot(f,q)/det
  return w>1e-8 and u+w<1-1e-8 and 1e-8<t<1-1e-8
 xyz=[tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)]
 tris=[tuple(xyz[i] for i in p['indices'][j:j+3]) for j in range(0,len(p['indices']),3)]
 lo=[min(v[k] for v in xyz) for k in range(3)];span=[max(v[k] for v in xyz)-lo[k] for k in range(3)];bins={};bounds=[]
 for j,tri in enumerate(tris):
  low=[min(v[k] for v in tri) for k in range(3)];high=[max(v[k] for v in tri) for k in range(3)];bounds.append((low,high))
  ranges=[range(max(0,min(11,math.floor((low[k]-lo[k])/span[k]*12))),min(11,math.floor((high[k]-lo[k])/span[k]*12))+1) for k in range(3)]
  for x in ranges[0]:
   for y in ranges[1]:
    for z in ranges[2]:bins.setdefault((x,y,z),[]).append(j)
 seen=set()
 for ids in bins.values():
  for i,a in enumerate(ids):
   for b in ids[i+1:]:
    key=(a,b)
    if key in seen:continue
    seen.add(key);ta,tb=tris[a],tris[b]
    if set(ta)&set(tb):continue
    la,ha=bounds[a];lb,hb=bounds[b]
    if any(ha[k]<lb[k] or hb[k]<la[k] for k in range(3)):continue
    if any(hits(t[k],t[(k+1)%3],other) for t,other in ((ta,tb),(tb,ta)) for k in range(3)):return a,b
 return None


def surface_distances(parent,points,limit):
 # Independent point/triangle distance over actual stored geometry. Axis rays
 # can miss a nearby tilted end cap; a vertex was 2.1 mm from its parent while
 # the closest axis ray reported 21 mm. No recipe frame or radius is used.
 from chikorita_sculpt import sub,dot,cross
 vs=[tuple(parent['positions'][i:i+3]) for i in range(0,len(parent['positions']),3)];bins={};cell=limit*2
 def key(p):return tuple(int(v//cell) for v in p)
 for i in range(0,len(parent['indices']),3):
  tri=tuple(vs[j] for j in parent['indices'][i:i+3]);lo=key(tuple(min(v[k] for v in tri)-limit for k in range(3)));hi=key(tuple(max(v[k] for v in tri)+limit for k in range(3)))
  for x in range(lo[0],hi[0]+1):
   for y in range(lo[1],hi[1]+1):
    for z in range(lo[2],hi[2]+1):bins.setdefault((x,y,z),[]).append(tri)
 def distance2(p,tri):
  a,b,c=tri;e=sub(b,a);f=sub(c,a);v=sub(p,a);n=cross(e,f);nn=dot(n,n);ee=dot(e,e);ff=dot(f,f);ef=dot(e,f);ve=dot(v,e);vf=dot(v,f);den=ee*ff-ef*ef
  if den>1e-24:
   u=(ve*ff-vf*ef)/den;w=(vf*ee-ve*ef)/den
   if u>=0 and w>=0 and u+w<=1:return dot(v,n)**2/nn
  best=float('inf')
  for a,b in ((a,b),(b,c),(c,a)):
   d=sub(b,a);v=sub(p,a);t=max(0,min(1,dot(v,d)/dot(d,d)));best=min(best,sum((v[k]-t*d[k])**2 for k in range(3)))
  return best
 for p in points:yield min((distance2(p,t)**.5 for t in bins.get(key(p),[])),default=float('inf'))
