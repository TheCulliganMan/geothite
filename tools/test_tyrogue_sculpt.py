#!/usr/bin/env python3
"""Stored scuffle anatomy, fitted gold gaze, continuous crown and digit roots."""
import copy,json,os,unittest
from pathlib import Path
from tyrogue_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_TYROGUE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class TyrogueTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'tyrogue.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 test_fitted_layers_follow_actual_stored_parent=attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent
 test_fitted_face_centers_and_edges_clear_parent=attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'tyrogue':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'tyrogue':self.m})
 def test_parity_floor_height_budget_and_distinct_small_fighter(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Head /',1),('Ear /',2),('Eye /',2),('Iris /',2),('Body /',1),('Waist band /',1),('Shorts /',1),('Shorts leg /',2),('Arm /',2),('Wrist band /',2),('Palm /',2),('Finger /',6),('Leg /',2),('Shoe /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
  self.assertFalse(any('scalp spike' in p['part'].lower() for p in self.ps));self.assertGreater(max(self.find('Head /')['positions'][1::3])-min(self.find('Head /')['positions'][1::3]),.36)
 def test_head_neck_ears_torso_waist_and_shorts_have_volume_roots(self):
  torso=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');shorts=self.find('Shorts /');band=self.find('Waist band /')
  for a,b in ((neck,torso),(head,neck),(band,torso),(shorts,torso),(band,shorts)):self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Ear /',s),head));self.assertTrue(self.overlap(self.find('Shorts leg /',s),shorts))
 def test_both_bent_arms_wraps_palms_and_all_six_digits_are_rooted(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);band=self.find('Wrist band /',s);palm=self.find('Palm /',s)
   for a,b in ((arm,self.find('Body /')),(band,arm),(palm,band)):self.assertTrue(self.overlap(a,b),a['part'])
   for j in range(3):self.assertTrue(self.overlap(self.find('Finger /',f'{s} {j}'),palm))
 def test_both_legs_brown_shoes_and_shorts_are_supported_and_grounded(self):
  for s in (-1,1):
   leg=self.find('Leg /',s);shoe=self.find('Shoe /',s)
   for a,b in ((leg,self.find('Shorts leg /',s)),(shoe,leg)):self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(shoe['positions'][1::3]),.002)
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye socket /','Head /'),('Eye /','Eye socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Ear contour /','Ear /'),('Shoe seam /','Shoe /')):
    if n.startswith(child):parent=self.find(base,s if base!='Head /' else None)
   if n.startswith('Mouth /'):parent=self.find('Head /')
   if parent:yield p,parent
 def test_crown_shorts_limbs_and_digits_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Head /','Body /','Waist band /','Shorts leg /','Arm /','Leg /','Finger /','Wrist band /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_head_shorts_hand_finger_and_shoe_are_rejected(self):
  for prefix,parent in (('Head /','Neck /'),('Shorts /','Body /'),('Palm /','Wrist band /'),('Finger /','Palm /'),('Shoe /','Leg /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
