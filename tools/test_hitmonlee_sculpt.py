#!/usr/bin/env python3
"""Stored headless fighter, continuous spring solids and fitted face checks."""
import copy,json,os,unittest
from pathlib import Path
from hitmonlee_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from bovine_sculpt import reverse_front
from goldfish_sculpt import zy
from sculpt_geometry import translate
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_HITMONLEE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class HitmonleeTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'hitmonlee.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 test_fitted_layers_follow_actual_stored_parent=attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent
 test_fitted_face_centers_and_edges_clear_parent=attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'hitmonlee':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'hitmonlee':self.m})
 def test_parity_floor_size_budget_and_headless_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),80000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertFalse(any(p['part'].startswith(('Head /','Mouth /','Ring /')) for p in self.ps))
  for prefix,n in (('Body /',1),('Eye /',2),('Pupil /',2),('Pupil core /',2),('Hip /',2),('Leg spring /',2),('Foot /',2),('Toe claw /',6),('Foot mark /',2),('Arm /',2),('Wrist spring /',2),('Hand /',2),('Finger /',6)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_hips_spring_legs_feet_and_six_claws_have_volume_roots(self):
  for s in (-1,1):
   hip=self.find('Hip /',s);leg=self.find('Leg spring /',s);foot=self.find('Foot /',s)
   for a,b in ((hip,self.find('Body /')),(leg,hip),(foot,leg)):self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):self.assertTrue(self.overlap(self.find('Toe claw /',f'{s} {j}'),foot))
 def test_arms_spring_wrists_palms_and_all_fingers_have_volume_roots(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);wrist=self.find('Wrist spring /',s);hand=self.find('Hand /',s)
   for a,b in ((arm,self.find('Body /')),(wrist,arm),(hand,wrist)):self.assertTrue(self.overlap(a,b),a['part'])
   for j in range(3):self.assertTrue(self.overlap(self.find('Finger /',f'{s} {j}'),hand))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye /','Body /'),('Pupil /','Eye /'),('Pupil core /','Pupil /')):
    if n.startswith(child):parent=self.find(base,s if base!='Body /' else None)
   if n.startswith('Foot mark /'):
    s=int(s);a,b=zy(p),zy(self.find('Foot /',s))
    if s==1:a,b=reverse_front(a),reverse_front(b)
    yield a,b;continue
   if parent:yield p,parent
 def test_curved_springs_and_digits_have_no_nonadjacent_crossings(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Hip /','Leg spring /','Arm /','Wrist spring /','Finger /','Toe claw /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_spring_finger_and_foot_negative_controls_are_rejected(self):
  for prefix,parent in (('Leg spring /','Hip /'),('Wrist spring /','Arm /'),('Finger /','Hand /'),('Foot /','Leg spring /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
