#!/usr/bin/env python3
"""Stored shared Wigglytuff topology, rooted anatomy and fitted facial surfaces."""
import json,os,unittest
from pathlib import Path
from wigglytuff_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('WIGGLYTUFF_TEST_MODEL',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/wigglytuff.glb'))
class WigglytuffTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from fairy_pair_glb import read_wigglytuff
  cls.m=read_wigglytuff(ROOT) if ROOT.suffix=='.glb' else json.loads(ROOT.read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'wigglytuff':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'wigglytuff':self.m})
 def test_recipe_height_grounding_and_budget(self):
  from fairy_pair_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for p in self.ps:
   if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0)
 def test_distinct_balloon_ears_eyes_and_rolled_forelock(self):
  for prefix,count in (('Belly /',1),('Tongue /',1),('Ear /',2),('Ear lining /',2),('Arm /',2),('Foot /',2),('Tuft /',1),('Tuft crease /',1),('Eye border /',2),('Eye white /',2),('Iris /',2),('Eye glint /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count)
 def pairs(self):
  for p in self.ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith(('Eye border /','Belly /')):parent=self.find('Body /')
   elif n.startswith('Mouth /'):parent=self.find('Belly /')
   elif n.startswith('Tongue /'):parent=self.find('Mouth /')
   elif n.startswith('Eye white /'):parent=self.find('Eye border /',sign)
   elif n.startswith('Iris /'):parent=self.find('Eye white /',sign)
   elif n.startswith(('Eye glint /','Iris sheen /')):parent=self.find('Iris /',sign)
   elif n.startswith('Ear lining /'):parent=self.find('Ear /',sign)
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
   if p['part'].startswith(('Arm /','Foot /','Ear /','Tuft /')):self.assertTrue(joins.DonphanTests.overlap(self,body,p),p['part'])
 def test_forelock_spiral_follows_rounded_tuft(self):
  p=self.find('Tuft crease /');parent=self.find('Tuft /')
  for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
   h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.012)
 def test_white_belly_covers_the_actual_lower_front_torso(self):
  belly=self.find('Belly /');body=self.find('Body /');self.assertTrue(all(v>.9 for v in belly['base_color'][:3]))
  for y in (.12,.22,.32,.42):
   for x in (-.03,0,.03):
    a=ray_intersections(body,x,y*HEIGHT);b=ray_intersections(belly,x,y*HEIGHT);self.assertTrue(a and b);self.assertGreater(max(b)-max(a),.0001)
 def test_long_swept_ears_extend_beyond_the_pear_torso(self):
  body=self.find('Body /');width=max(abs(v) for v in body['positions'][::3]);top=max(body['positions'][1::3])
  for p in self.ps:
   if p['part'].startswith('Ear /'):self.assertGreater(max(abs(v) for v in p['positions'][::3]),width*1.1);self.assertGreater(max(p['positions'][1::3]),top*1.3)
if __name__=='__main__':unittest.main()
