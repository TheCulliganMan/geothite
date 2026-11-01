#!/usr/bin/env python3
"""Continuous virtual-duck anatomy, seated belly and fitted eyes/reflections."""
import json,math,unittest
from pathlib import Path
from porygon2_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import ray_intersections
from porygon2_glb import read_porygon2,geometry_digest
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class Porygon2Tests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_porygon2(ROOT/'porygon2.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'porygon2':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'porygon2':self.m})
 def test_recipe_parity_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),50000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Body /',1),('Belly /',1),('Bill /',1),('Tail /',1),('Foot /',2),('Eye socket /',2),('Eye white /',2),('Pupil /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=sorted(ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23));bb=sorted(ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23))
    for al,ah in zip(aa[::2],aa[1::2]):
     for bl,bh in zip(bb[::2],bb[1::2]):
      if min(ah,bh)-max(al,bl)>.00025:return True
  return False
 def fitted_pairs(self):
  for p in self.ps:
   n=p['part'];parent=None;mark=p;sign=n.split()[-1]
   if n.startswith(('Eye socket /','Head highlight /')):parent=self.find('Body /')
   elif n.startswith('Eye white /'):parent=self.find('Eye socket / thin fitted dark oval border '+sign)
   elif n.startswith('Pupil /'):parent=self.find('Eye white / large fitted ivory virtual-duck oval '+sign)
   elif n.startswith('Eye glint /'):parent=self.find('Pupil / fitted dark vertical curious oval '+sign)
   elif n.startswith('Foot highlight /'):parent=self.find('Foot / grounded rounded splayed cyan paddle '+sign)
   if parent:yield mark,parent
 def test_all_eyes_and_reflections_follow_actual_stored_parents(self):
  for p,parent in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.017)
 def test_front_marking_triangles_clear_actual_support(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_belly_bill_tail_and_two_paddles_physically_join_continuous_body(self):
  body=self.find('Body /')
  for p in self.ps:
   if p['part'].startswith(('Belly /','Bill /','Tail /','Foot /')):self.assertTrue(self.overlap(body,p),p['part'])
 def test_both_splayed_paddles_are_grounded_and_supported_by_blue_belly(self):
  belly=self.find('Belly /')
  for p in self.ps:
   if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0.);self.assertTrue(self.overlap(belly,p),p['part'])
 def test_blue_coat_covers_actual_front_and_rear_lower_torso(self):
  body=self.find('Body /');belly=self.find('Belly /');lo=min(belly['positions'][1::3]);hi=max(belly['positions'][1::3])
  for rear in (False,True):
   parent=reverse_front(body) if rear else body;coat=reverse_front(belly) if rear else belly
   for f in (.18,.35,.55,.75):
    y=lo+(hi-lo)*f
    for x in (0,.025,-.025):
     a=ray_intersections(parent,x,y);b=ray_intersections(coat,x,y);self.assertTrue(a and b);self.assertGreater(max(b)-max(a),.001,(rear,x,y,max(b)-max(a)))
if __name__=='__main__':unittest.main()
