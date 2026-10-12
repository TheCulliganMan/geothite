#!/usr/bin/env python3
"""Check actual live-coal anatomy, fitted gaze, root volumes and open bill."""
import copy,json,os,unittest
from pathlib import Path
from magby_sculpt import sculpture,HEIGHT
from sculpt_geometry import ellipsoid,translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_MAGBY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class MagbyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'magby.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'magby':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'magby':self.m})
 def test_recipe_bounds_budget_and_distinct_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),95000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,count in (('Body /',1),('Head /',1),('Collar /',1),('Crown lobe /',5),('Belly /',1),('Eye /',2),('Pupil /',2),('Arm /',2),('Hand /',2),('Hand claw /',6),('Leg /',2),('Foot /',2),('Toe claw /',4),('Tail /',1),('Bill /',1),('Mouth back /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,prefix)
  self.assertFalse(any('flame' in p['part'].lower() for p in self.ps))
 def test_head_crown_collar_bill_and_tail_have_actual_volume_roots(self):
  head=self.find('Head /');body=self.find('Body /');collar=self.find('Collar /')
  for a,b in ((head,collar),(collar,body),(self.find('Bill /'),head),(self.find('Mouth back /'),head),(self.find('Tail /'),body)):
   self.assertTrue(self.overlap(a,b),a['part'])
  for j in range(5):self.assertTrue(self.overlap(self.find('Crown lobe /',j),head))
 def test_limbs_claws_and_grounded_feet_are_rooted(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);hand=self.find('Hand /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s)
   for a,b in ((arm,self.find('Body /')),(hand,arm),(leg,self.find('Body /')),(foot,leg)):self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):self.assertTrue(self.overlap(self.find('Hand claw /',f'{s} {j}'),hand))
   for j in range(2):self.assertTrue(self.overlap(self.find('Toe claw /',f'{s} {j}'),foot))
 def pairs(self):
  for p in self.ps:
   name=p['part'];s=name.split()[-1]
   for child,parent in (('Belly /','Body /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /'),('Glint /','Pupil /')):
    if name.startswith(child):yield p,self.find(parent,s if parent not in ('Body /','Head /') else None)
 def test_fitted_vertices_follow_actual_parent(self):attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_parent(self):attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_claws_tail_and_bill_do_not_cross(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Leg /','Hand claw /','Toe claw /','Tail /','Bill /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_bill_aperture_has_unoccupied_front_and_rejects_a_fill(self):
  bill=self.find('Bill /');scale=max(v for p in self.ps for v in p['positions'][1::3])/1.046;y=.705*scale;z=.265*scale
  def occupied(p,x,y,z):
   hits=ray_intersections(p,x,y);return any(a+.0001<z<b-.0001 for a,b in zip(hits[::2],hits[1::2]))
  for dx,dy in ((0,0),(.009,0),(-.009,0),(0,.006),(0,-.006)):
   self.assertFalse(any(occupied(p,dx,y+dy,z) for p in self.ps),'mouth aperture is filled')
  filled=ellipsoid('filled bill negative',(.5,.5,.5),(0,y,z),(.02,.018,.025),32,22);self.assertTrue(occupied(filled,0,y,z))
 def test_detached_crown_tail_and_claws_are_rejected(self):
  for child,parent in (('Crown lobe /','Head /'),('Tail /','Body /'),('Hand claw /','Hand /'),('Toe claw /','Foot /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
