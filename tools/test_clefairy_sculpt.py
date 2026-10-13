#!/usr/bin/env python3
"""Stored shared Clefairy topology, rooted anatomy and fitted facial surfaces."""
import json,os,unittest
from pathlib import Path
from clefairy_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('CLEFAIRY_TEST_MODEL',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/actor_props/icon_clefairy.mesh.json'))
class ClefairyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads(ROOT.read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'clefairy':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'clefairy':self.m})
 def test_recipe_height_grounding_and_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),25000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for p in self.ps:
   if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0)
 def test_distinct_ears_wings_hands_and_curled_tail(self):
  for prefix,count in (('Ear /',2),('Ear tip /',2),('Wing /',6),('Arm /',2),('Hand claw /',6),('Foot /',2),('Toe /',2),('Tuft /',1),('Tuft crease /',1),('Tail /',1),('Tooth /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count)
 def pairs(self):
  for p in self.ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith(('Eye /','Cheek /','Mouth /')):parent=self.find('Body /')
   elif n.startswith('Eye glint /'):parent=self.find('Eye /',sign)
   elif n.startswith('Ear tip /'):parent=self.find('Ear /',sign)
   elif n.startswith('Toe /'):parent=self.find('Foot /',sign)
   elif n.startswith(('Tooth /','Tongue /')):parent=self.find('Mouth /')
   if parent:yield p,parent
 def test_marking_vertices_follow_actual_stored_support(self):
  for p,parent in self.pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.017)
 def test_front_marking_triangles_clear_their_support(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_appendages_have_physical_roots(self):
  body=self.find('Body /')
  for p in self.ps:
   if p['part'].startswith(('Arm /','Foot /','Ear /','Tuft /','Tail /')):self.assertTrue(joins.DonphanTests.overlap(self,body,p),p['part'])
  for sign in (-1,1):
   arm=self.find('Arm /',sign)
   for j in (-1,0,1):self.assertTrue(joins.DonphanTests.overlap(self,arm,self.find('Hand claw /',str(sign)+' '+str(j))))
   self.assertTrue(joins.DonphanTests.overlap(self,body,self.find('Wing /',str(sign)+' 0')))
   for j in (1,2):self.assertTrue(joins.DonphanTests.overlap(self,self.find('Wing /',str(sign)+' '+str(j-1)),self.find('Wing /',str(sign)+' '+str(j))))
 def test_forelock_spiral_follows_rounded_tuft(self):
  p=self.find('Tuft crease /');parent=self.find('Tuft /')
  for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
   h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.012)
if __name__=='__main__':unittest.main()
