#!/usr/bin/env python3
"""Angular virtual-duck cages, cyan chest and stored side-panel eye fit."""
import json,math,unittest
from pathlib import Path
from porygon_sculpt import sculpture,HEIGHT,side_view
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import ray_intersections
from porygon_glb import read_porygon,geometry_digest
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class PorygonTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_porygon(ROOT/'porygon.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'porygon':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'porygon':self.m})
 def test_recipe_parity_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),50000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Body /',1),('Breast /',1),('Neck /',1),('Head /',1),('Bill /',1),('Tail /',1),('Foot /',2),('Eye border /',2),('Eye white /',2)):
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
   if n.startswith('Eye border /'):parent=self.find('Head /')
   elif n.startswith('Eye white /'):parent=self.find('Eye border / fitted dark hexagonal socket '+sign)
   elif n.startswith('Pupil /'):parent=self.find('Eye white / fitted ivory polygonal panel '+sign)
   elif n.startswith('Eye glint /'):parent=self.find('Pupil / fitted small round digital eye '+sign)
   if parent:yield side_view(mark,int(sign)),side_view(parent,int(sign))
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
 def test_hull_neck_head_bill_tail_and_paddles_have_physical_joins(self):
  body=self.find('Body /');breast=self.find('Breast /');neck=self.find('Neck /');head=self.find('Head /');bill=self.find('Bill /')
  self.assertTrue(self.overlap(body,breast));self.assertTrue(self.overlap(body,neck));self.assertTrue(self.overlap(neck,head));self.assertTrue(self.overlap(head,bill));self.assertTrue(self.overlap(body,self.find('Tail /')))
  for p in self.ps:
   if p['part'].startswith('Foot /'):self.assertTrue(self.overlap(breast,p),p['part']);self.assertEqual(min(p['positions'][1::3]),0.)
 def test_cyan_chest_covers_actual_forward_lower_hull(self):
  body=self.find('Body /');chest=self.find('Breast /')
  for f in (.16,.21,.26,.30):
   y=HEIGHT*f
   for x in (0,.06,-.06):
    a=ray_intersections(body,x,y);b=ray_intersections(chest,x,y);self.assertTrue(a and b);self.assertGreater(max(b)-max(a),.015)
 def test_main_cages_preserve_flat_panel_normals(self):
  for p in self.ps:
   if not p['part'].startswith(('Body /','Breast /','Neck /','Head /','Bill /','Tail /','Foot /')):continue
   for i in range(0,len(p['indices']),3):
    normals=[p['normals'][3*j:3*j+3] for j in p['indices'][i:i+3]];self.assertEqual(normals[0],normals[1]);self.assertEqual(normals[1],normals[2])
if __name__=='__main__':unittest.main()
