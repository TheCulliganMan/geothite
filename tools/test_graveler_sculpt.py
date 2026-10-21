#!/usr/bin/env python3
"""Stored four-armed boulder topology, actual roots and fitted face surfaces."""
import copy,json,os,unittest
from pathlib import Path
from graveler_sculpt import sculpture,HEIGHT
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_GRAVELER_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class GravelerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=__import__('stone_pair_glb').read_stone_pair(ROOT/'graveler.glb', 'graveler')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'graveler':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'graveler':self.m})
 def test_recipe_parity_height_floor_budget_and_anatomy(self):
  from stone_pair_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Body /',1),('Arm /',4),('Forearm /',2),('Hand /',4),('Finger /',12),('Foot /',2),('Rubble /',6),('Forehead /',1)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
  self.assertFalse(any(p['part'].startswith('Head /') for p in self.ps))
 def test_all_arms_palms_digits_and_rubble_have_actual_roots(self):
  body=self.find('Body /')
  for s in (-1,1):
   for level in ('upper','lower'):
    arm=self.find('Arm / rooted '+level,s);hand=self.find('Hand / broad '+level,s);self.assertTrue(self.overlap(arm,body))
    wrist=arm if level=='upper' else self.find('Forearm /',s)
    if level=='lower':self.assertTrue(self.overlap(wrist,arm))
    self.assertTrue(self.overlap(hand,wrist))
    for j in range(3):self.assertTrue(self.overlap(self.find('Finger /',f'{s} {level} {j}'),hand),(s,level,j))
  for p in self.ps:
   if p['part'].startswith(('Foot /','Rubble /')):self.assertTrue(self.overlap(p,body),p['part'])
 def test_upper_and_lower_arm_pairs_have_distinct_proportions(self):
  for s in (-1,1):
   upper=self.find('Hand / broad upper',s);lower=self.find('Hand / broad lower',s)
   self.assertGreater(min(upper['positions'][1::3]),max(lower['positions'][1::3]))
   self.assertGreater(max(abs(x) for x in upper['positions'][::3]),max(abs(x) for x in lower['positions'][::3])+.12)
   # Lower wrists must read outside the belly, instead of becoming the old
   # pair of tiny clasped fists at the center of the front silhouette.
   self.assertGreater(sum(abs(x) for x in lower['positions'][::3])/(len(lower['positions'])//3),.39*HEIGHT)
 def test_both_short_feet_are_grounded(self):
  for s in (-1,1):self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Forehead /','Body /'),('Socket /','Body /'),('Eye /','Socket /'),('Pupil /','Eye /'),('Mouth /','Body /'),('Mouth inset /','Mouth /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Socket /','Eye /') else None)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_arms_and_fingers_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Forearm /','Finger /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_hands_digits_and_rubble_are_rejected(self):
  for child,parent in (('Hand / broad upper','Arm / rooted upper'),('Finger /','Hand / broad upper'),('Rubble /','Body /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
