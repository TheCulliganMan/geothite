#!/usr/bin/env python3
"""Stored handstand placement, blue balance ends and actual anatomical roots."""
import copy,json,os,unittest
from pathlib import Path
from hitmontop_glb import read_hitmontop,geometry_digest
from hitmontop_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_HITMONTOP_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class HitmontopTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_hitmontop(ROOT/'hitmontop.glb');cls.ps=cls.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 test_fitted_layers_follow_actual_stored_parent=attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent
 test_fitted_face_centers_and_edges_clear_parent=attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'hitmontop':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'hitmontop':self.m})
 def test_parity_height_budget_and_distinct_balance_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Head /',1),('Helmet flange /',2),('Spin point /',1),('Body /',1),('Eye /',2),('Pupil /',2),('Helmet crease /',6),('Arm /',2),('Wrist wrap /',2),('Hand /',2),('Leg /',2),('Ankle wrap /',2),('Foot /',2),('Foot spike /',6),('Foot sole /',2),('Tail /',1),('Tail ball /',1),('Tail spike /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_headstand_has_a_grounded_spin_point_and_raised_blue_feet(self):
  spin=self.find('Spin point /');head=self.find('Head /');body=self.find('Body /')
  self.assertEqual(min(spin['positions'][1::3]),0);self.assertGreater(min(head['positions'][1::3]),.06)
  self.assertLess(max(head['positions'][1::3]),max(body['positions'][1::3]))
  for s in (-1,1):self.assertGreater(min(self.find('Foot /',s)['positions'][1::3]),max(body['positions'][1::3]))
 def test_head_neck_body_spin_point_and_tail_have_volume_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');tail=self.find('Tail /');ball=self.find('Tail ball /')
  for a,b in ((neck,body),(head,neck),(self.find('Spin point /'),head),(tail,body),(ball,tail),(self.find('Tail spike /'),ball)):self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Helmet flange /',s),head))
 def test_both_long_legs_wrapped_ankles_and_all_six_foot_spikes_are_rooted(self):
  for s in (-1,1):
   leg=self.find('Leg /',s);wrap=self.find('Ankle wrap /',s);foot=self.find('Foot /',s)
   for a,b in ((leg,self.find('Body /')),(wrap,leg),(foot,leg),(foot,wrap)):self.assertTrue(self.overlap(a,b),a['part'])
   for j in range(3):self.assertTrue(self.overlap(self.find('Foot spike /',f'{s} {j}'),foot))
 def test_both_arms_wrists_and_fists_have_volume_roots(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);wrap=self.find('Wrist wrap /',s);hand=self.find('Hand /',s)
   for a,b in ((arm,self.find('Body /')),(wrap,arm),(hand,wrap)):self.assertTrue(self.overlap(a,b),a['part'])
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye /','Head /'),('Pupil /','Eye /'),('Foot sole /','Foot /')):
    if n.startswith(child):parent=self.find(base,s if base not in ('Head /',) else None)
   if n.startswith('Helmet crease /'):parent=self.find('Helmet flange /',n.split()[-2])
   if n.startswith('Belly /'):parent=self.find('Body /')
   if n.startswith('Mouth /'):parent=self.find('Head /')
   if parent:yield p,parent
 def test_curved_head_limbs_tail_and_spikes_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Head /','Helmet flange /','Spin point /','Arm /','Wrist wrap /','Leg /','Ankle wrap /','Foot spike /','Tail /','Tail spike /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_spin_point_tail_ball_and_foot_spike_are_rejected(self):
  for prefix,parent in (('Spin point /','Head /'),('Tail ball /','Tail /'),('Foot spike /','Foot /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
