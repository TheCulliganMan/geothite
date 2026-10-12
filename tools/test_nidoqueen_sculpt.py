#!/usr/bin/env python3
"""Stored queen anatomy, fitted armor, solid roots and shading regression checks."""
import copy,json,os,unittest
from pathlib import Path
from nidoqueen_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_NIDOQUEEN_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class NidoqueenTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'nidoqueen.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 test_fitted_layers_follow_actual_stored_parent=attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent
 test_fitted_face_centers_and_edges_clear_parent=attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'nidoqueen':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'nidoqueen':self.m})
 def test_parity_floor_size_budget_and_species_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),110000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Ear /',2),('Ear inset /',2),('Eye /',2),('Horn /',1),('Fang /',2),('Chest plate /',2),('Belly plate /',4),('Shoulder /',2),('Knee /',2),('Hand claw /',6),('Foot claw /',4),('Back spine /',3),('Tail spine /',3),('Tail /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
  tail=self.find('Tail /');self.assertGreater(max(tail['positions'][2::3])-min(tail['positions'][2::3]),.90)
  head=self.find('Head /');self.assertGreater(max(head['positions'][::3])-min(head['positions'][::3]),max(head['positions'][1::3])-min(head['positions'][1::3]))
 def test_head_muzzle_horn_ears_fangs_and_dorsal_roots(self):
  body=self.find('Body /');head=self.find('Head /');muzzle=self.find('Muzzle /');tail=self.find('Tail /')
  for a,b in ((head,body),(muzzle,head),(self.find('Horn /'),head),(tail,body)):
   self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):
   self.assertTrue(self.overlap(self.find('Ear /',s),head),'ear '+str(s));self.assertTrue(self.overlap(self.find('Fang /',s),muzzle),'fang '+str(s))
  for p in self.ps:
   if p['part'].startswith('Back spine /'):self.assertTrue(self.overlap(p,body),p['part'])
   if p['part'].startswith('Tail spine /'):self.assertTrue(self.overlap(p,tail),p['part'])
 def test_limb_armor_and_all_claws_have_volume_roots(self):
  for s in (-1,1):
   body=self.find('Body /');shoulder=self.find('Shoulder /',s);arm=self.find('Arm /',s);hand=self.find('Hand /',s);thigh=self.find('Haunch /',s);knee=self.find('Knee /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s)
   for a,b in ((shoulder,body),(arm,shoulder),(hand,arm),(thigh,body),(knee,thigh),(leg,thigh),(foot,leg)):
    self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):self.assertTrue(self.overlap(self.find('Hand claw /',f'{s} {j}'),hand))
   for j in range(2):self.assertTrue(self.overlap(self.find('Foot claw /',f'{s} {j}'),foot))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Ear inset /','Ear /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Nostril /','Muzzle /')):
    if n.startswith(child):parent=self.find(base,s if base not in ('Head /','Muzzle /') else None)
   if n.startswith(('Chest plate /','Belly plate /')):parent=self.find('Body /')
   if n.startswith(('Chin /','Mouth /')):parent=self.find('Muzzle /')
   if parent:yield p,parent
 def test_ears_tail_spines_and_limbs_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Ear /','Ear inset /','Tail /','Tail spine /','Back spine /','Arm /','Leg /','Horn /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_ear_tail_horn_and_claws_are_rejected(self):
  for prefix,parent in (('Tail /','Body /'),('Ear /','Head /'),('Horn /','Head /'),('Hand claw /','Hand /'),('Foot claw /','Foot /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
