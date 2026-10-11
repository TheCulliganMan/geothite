#!/usr/bin/env python3
"""Stored-mesh regressions for attached rhino armor, feet and fitted facial art."""
import json,os,unittest
from pathlib import Path
from rhyhorn_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
import test_donphan_sculpt as joins
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class RhyhornTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads(Path(os.environ.get('CRYSTAL_RHYHORN_TEST_MESH',str(ROOT/'rhyhorn.mesh.json'))).read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 overlap=joins.DonphanTests.overlap
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'rhyhorn':self.m})
 def test_finite_unit_normals_follow_faces(self):assert_model_shading(self,{'rhyhorn':self.m})
 def test_recipe_parity_grounding_budget_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),65000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Body /',1),('Head /',1),('Back armor /',4),('Brow armor /',1),('Horn /',1),('Ear /',2),('Eye socket /',2),('Nostril /',2),('Leg /',4),('Leg armor /',4),('Foot /',4),('Claw /',12)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def test_dorsal_plates_intersect_stored_barrel(self):
  body=self.find('Body /')
  for p in self.ps:
   if p['part'].startswith('Back armor /'):self.assertTrue(self.overlap(body,p),p['part'])
 def test_seated_armor_still_has_a_visible_dorsal_silhouette(self):
  # The first attached candidate passed join tests but disappeared into the
  # barrel in the actual front/oblique/rear previews.
  body_top=max(self.find('Body /')['positions'][1::3])
  for p in self.ps:
   if p['part'].startswith('Back armor /'):
    self.assertGreater(max(p['positions'][1::3])-body_top,HEIGHT*.075,p['part'])
 def test_detached_old_plate_is_rejected(self):
  plate=dict(self.find('Back armor /'));plate['positions']=[v+HEIGHT if i%3==1 else v for i,v in enumerate(plate['positions'])]
  self.assertFalse(self.overlap(self.find('Body /'),plate))
 def test_head_jaw_horn_ears_and_forehead_are_rooted(self):
  head=self.find('Head /');self.assertTrue(self.overlap(head,self.find('Body /')))
  for p in self.ps:
   if p['part'].startswith(('Jaw /','Horn /','Ear /','Brow armor /')):self.assertTrue(self.overlap(head,p),p['part'])
 def test_four_legs_cuffs_and_grounded_feet_have_physical_joins(self):
  for foot in [p for p in self.ps if p['part'].startswith('Foot /')]:
   suffix=' '.join(foot['part'].split()[-2:]);leg=self.find('Leg / stocky rooted armored pillar '+suffix);cuff=self.find('Leg armor / rooted tapered knee cuff '+suffix)
   for a,b in ((leg,self.find('Body /')),(leg,foot),(leg,cuff)):self.assertTrue(self.overlap(a,b),(a['part'],b['part']))
   self.assertEqual(min(foot['positions'][1::3]),0.)
  self.assertTrue(self.overlap(self.find('Body /'),self.find('Tail /')))
 def fitted_pairs(self):
  for p in self.ps:
   name=p['part'];sign=name.split()[-1];parent=None
   if name.startswith(('Eye socket /','Nostril /')):parent=self.find('Head /')
   elif name.startswith('Eye white /'):parent=self.find('Eye socket / fitted angular black recess '+sign)
   elif name.startswith('Pupil /'):parent=self.find('Eye white / narrow watchful ivory '+sign)
   elif name.startswith('Claw /'):parent=self.find('Foot / broad grounded gray rhino pad '+' '.join(name.split()[-3:-1]))
   if parent:yield p,parent
 def test_face_and_toes_follow_actual_stored_support(self):
  for p,parent in self.fitted_pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.020,p['part'])
 def test_front_marking_triangles_clear_actual_support(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(z-max(hits),.0001,(p['part'],k//3))
if __name__=='__main__':unittest.main()
