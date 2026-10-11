#!/usr/bin/env python3
"""Stored Kingdra anatomy, crown, actual surface fitting and deep muzzle bore."""
import json,os,unittest,math
from pathlib import Path
from kingdra_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bovine_sculpt import reverse_front
import test_magnet_sculpt as joins
ROOT=Path(os.environ.get('KINGDRA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

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

class KingdraTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'kingdra.mesh.json').read_bytes());cls.ps=cls.model['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_anatomy(self):assert_closed_models(self,{'kingdra':self.model})
 def test_actual_triangle_shading(self):assert_model_shading(self,{'kingdra':self.model})
 def test_recipe_ground_height_budget_and_identity(self):
  self.assertEqual(self.model,sculpture());names=[p['part'] for p in self.ps];self.assertEqual(len(names),len(set(names)))
  self.assertEqual(min(y for p in self.ps for y in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(y for p in self.ps for y in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),65000)
  for prefix,count in {'Crown /':2,'Crown branch /':4,'Cheek /':2,'Neck fin /':2,'Dorsal fin /':2,'Dorsal horn /':2,'Dorsal fork /':2,'Belly fold /':5,'Body seam /':3,'Snout /':3,'Iris /':2}.items():self.assertEqual(sum(x.startswith(prefix) for x in names),count,prefix)
 def test_anatomy_has_real_solid_roots(self):
  body=self.find('Body /');head=self.find('Head /');muzzle=self.find('Snout / long');bore=self.find('Snout / recessed');floor=self.find('Snout / deep')
  pairs=[(body,head),(body,self.find('Tail /')),(head,muzzle),(muzzle,bore),(bore,floor)]
  for sign in (-1,1):
   crown=self.find('Crown /',sign);spine=self.find('Dorsal horn /',sign)
   pairs += [(head,crown),(head,self.find('Cheek /',sign)),(head,self.find('Neck fin /',sign)),(body,self.find('Dorsal fin /',sign)),(body,spine),(spine,self.find('Dorsal fin /',sign)),(spine,self.find('Dorsal fork /',sign))]
   pairs += [(crown,self.find('Crown branch /',str(sign)+' '+str(j))) for j in range(2)]
  for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(a['part'],b['part']))
 def markings(self):
  for p in self.ps:
   name=p['part'];last=name.split()[-1];parent=None
   if name.startswith('Belly /'):parent=self.find('Body /')
   elif name.startswith('Belly fold /'):parent=self.find('Belly /')
   elif name.startswith('Body seam /'):yield reverse_front(p),reverse_front(self.find('Body /'));continue
   elif name.startswith('Eye socket /'):parent=self.find('Head /')
   elif name.startswith('Eye white /'):parent=self.find('Eye socket /',last)
   elif name.startswith('Iris /'):parent=self.find('Eye white /',last)
   elif name.startswith('Pupil /'):parent=self.find('Iris /',last)
   elif name.startswith('Glint /'):parent=self.find('Pupil /',last)
   elif name.startswith('Fin ray /'):parent=self.find('Dorsal fin /',name.split()[-2])
   if parent:yield p,parent
 def test_details_follow_actual_stored_parent_triangles(self):
  for p,parent in self.markings():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,(p['part'],x,y));self.assertLess(abs(z-max(h)),.017,p['part'])
 def test_front_faces_clear_actual_stored_parents(self):
  for p,parent in self.markings():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(p['part'],x,y))
 def test_long_muzzle_is_tapered_and_genuinely_open(self):
  outer=self.find('Snout / long');inner=self.find('Snout / recessed');floor=self.find('Snout / deep')
  zmax=max(outer['positions'][2::3]);y=(min(outer['positions'][1::3])+max(outer['positions'][1::3]))/2
  self.assertFalse(ray_intersections(outer,0,y));self.assertFalse(ray_intersections(inner,0,y))
  hits=ray_intersections(floor,0,y);self.assertTrue(hits);self.assertGreater(zmax-max(hits),.20)
  front=[outer['positions'][i] for i in range(0,len(outer['positions']),3) if outer['positions'][i+2]>zmax-.002]
  self.assertLess(max(front)-min(front),.075)
 def test_crown_is_branched_and_tail_contains_an_inward_turn(self):
  head=self.find('Head /');top=max(head['positions'][1::3])
  for sign in (-1,1):
   stem=self.find('Crown /',sign);self.assertGreater(max(stem['positions'][1::3])-top,.15)
   for j in range(2):
    branch=self.find('Crown branch /',str(sign)+' '+str(j));self.assertGreater(max(branch['positions'][::3])-min(branch['positions'][::3]),.12)
  tail=self.find('Tail /');self.assertGreater(len(tail['positions'])//3,1000);self.assertEqual(min(tail['positions'][1::3]),0.)
  self.assertGreater(max(tail['positions'][::3])-min(tail['positions'][::3]),.27)
 def test_tail_surface_does_not_cross_itself(self):
  self.assertIsNone(crossed_faces(self.find('Tail /')))
if __name__=='__main__':unittest.main()
