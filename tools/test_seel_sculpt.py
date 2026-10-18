#!/usr/bin/env python3
"""Stored reclining seal topology, continuous tail, rooted face and fitted details."""
import json,os,unittest
from pathlib import Path
from seel_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('SEEL_TEST_MODEL',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/seel.glb'))
class SeelTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from seal_glb import read_seal
  cls.m=read_seal(ROOT,'seel');cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'seel':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'seel':self.m})
 def test_exact_recipe_height_grounding_and_budget(self):
  from seal_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
 def test_distinct_tail_paddles_muzzle_and_playful_face(self):
  for prefix,count in (('Body /',1),('Head /',1),('Horn /',1),('Tail fluke /',2),('Flipper /',2),('Flipper fold /',4),('Muzzle /',1),('Mouth /',1),('Tongue /',1),('Tusk /',2),('Eye /',2),('Brow /',2),('Nose /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count)
 def pairs(self):
  for p in self.ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith('Eye /'):parent=self.find('Head /')
   elif n.startswith(('Eye sheen /','Eye glint /')):parent=self.find('Eye /',sign)
   elif n.startswith('Nose glint /'):parent=self.find('Nose /')
   if parent:yield p,parent
 def test_marking_vertices_fit_actual_stored_support(self):
  for p,parent in self.pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.017)
 def test_visible_marking_triangles_clear_actual_support(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(p['part'],k//3,z-max(h)))
 def test_appendages_and_face_have_physical_roots(self):
  body=self.find('Body /');head=self.find('Head /');muzzle=self.find('Muzzle /');mouth=self.find('Mouth /')
  pairs=[(body,head),(head,self.find('Horn /')),(head,muzzle),(muzzle,mouth),(mouth,self.find('Tongue /')),(muzzle,self.find('Nose /'))]
  pairs += [(body,p) for p in self.ps if p['part'].startswith(('Flipper /','Tail fluke /'))]
  pairs += [(muzzle,p) for p in self.ps if p['part'].startswith('Tusk /')]
  for a,b in pairs:self.assertTrue(joins.DonphanTests.overlap(self,a,b),(a['part'],b['part']))
 def test_tail_fans_are_raised_and_paddles_are_broad_and_low(self):
  head=self.find('Head /');headtop=max(head['positions'][1::3]);headwidth=max(abs(v) for v in head['positions'][::3])
  for p in self.ps:
   if p['part'].startswith('Tail fluke /'):self.assertGreater(min(p['positions'][1::3]),headtop*1.3);self.assertLess(max(p['positions'][2::3]),min(head['positions'][2::3]))
   if p['part'].startswith('Flipper /'):self.assertGreater(max(abs(v) for v in p['positions'][::3]),headwidth*2);self.assertLess(max(p['positions'][1::3]),headtop*.2);self.assertAlmostEqual(min(p['positions'][1::3]),0,places=6)
 def test_brows_and_tongue_fold_are_supported(self):
  for p in self.ps:
   if p['part'].startswith('Brow /'):parent=self.find('Head /')
   elif p['part'].startswith('Tongue crease /'):parent=self.find('Tongue /')
   elif p['part'].startswith('Flipper fold /'):
    parent=self.find('Flipper /',p['part'].split()[-2])
    def top(q):return dict(q,positions=[v for i in range(0,len(q['positions']),3) for v in (q['positions'][i],-q['positions'][i+2],q['positions'][i+1])])
    parent=top(parent);p=top(p)
   else:continue
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.012)
if __name__=='__main__':unittest.main()
