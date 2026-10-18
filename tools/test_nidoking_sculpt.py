#!/usr/bin/env python3
"""Stored king armor, rooted anatomy and actually empty toothed jaw checks."""
import copy,json,os,unittest
from pathlib import Path
from nidoking_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_slime_sculpt import ray_intersections
from sculpt_geometry import translate,ellipsoid
import test_nidorina_sculpt as attachment_checks
import test_lugia_sculpt as jaw_checks
ROOT=Path(os.environ.get('CRYSTAL_NIDOKING_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class NidokingTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from nido_royal_glb import read_nido_royal
  cls.m=read_nido_royal(ROOT/'nidoking.glb','nidoking');cls.ps=cls.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.occupied_overlap
 jaw_gap=jaw_checks.LugiaTests.jaw_gap
 test_fitted_layers_follow_actual_stored_parent=attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent
 test_fitted_face_centers_and_edges_clear_parent=attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'nidoking':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'nidoking':self.m})
 def test_parity_floor_size_budget_and_species_anatomy(self):
  from nido_royal_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Ear /',2),('Ear inset /',2),('Eye /',2),('Horn /',1),('Fang /',6),('Cheek barb /',4),('Chest plate /',1),('Belly plate /',4),('Shoulder /',2),('Knee /',2),('Hand claw /',6),('Foot claw /',2),('Back spine /',3),('Tail spine /',3),('Tail collar /',3),('Tail /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
  tail=self.find('Tail /');self.assertGreater(max(tail['positions'][2::3])-min(tail['positions'][2::3]),.95)
  horn=self.find('Horn /');self.assertGreater(max(horn['positions'][2::3])-min(horn['positions'][2::3]),.34)
 def test_head_horn_ears_cheek_barbs_jaws_teeth_and_tongue_are_rooted(self):
  body=self.find('Body /');head=self.find('Head /');upper=self.find('Jaw / hooked');lower=self.find('Jaw / open')
  for a,b in ((head,body),(self.find('Horn /'),head),(upper,head),(lower,head),(self.find('Mouth /'),head),(self.find('Tongue /'),lower)):
   self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):
   self.assertTrue(self.overlap(self.find('Ear /',s),head),'ear '+str(s))
   for j in range(2):
    self.assertTrue(self.overlap(self.find('Cheek barb /',f'{s} {j}'),head),'cheek '+str(s)+' '+str(j))
    self.assertTrue(self.overlap(self.find('Fang / rooted ivory king upper',f'{s} {j}'),upper),'upper fang '+str(s)+' '+str(j))
   self.assertTrue(self.overlap(self.find('Fang / rooted ivory king lower',s),lower),'lower fang '+str(s))
 def test_limb_armor_and_all_claws_have_volume_roots(self):
  for s in (-1,1):
   body=self.find('Body /');shoulder=self.find('Shoulder /',s);arm=self.find('Arm /',s);hand=self.find('Hand /',s);thigh=self.find('Haunch /',s);knee=self.find('Knee /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s)
   for a,b in ((shoulder,body),(arm,shoulder),(hand,arm),(thigh,body),(knee,thigh),(leg,thigh),(foot,leg)):
    self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):self.assertTrue(self.overlap(self.find('Hand claw /',f'{s} {j}'),hand))
   self.assertTrue(self.overlap(self.find('Foot claw /',s),foot))
 def test_tail_spines_and_segmented_collars_have_real_roots(self):
  body=self.find('Body /');tail=self.find('Tail /');self.assertTrue(self.overlap(tail,body))
  for p in self.ps:
   if p['part'].startswith('Back spine /'):self.assertTrue(self.overlap(p,body),p['part'])
   elif p['part'].startswith(('Tail spine /','Tail collar /')):self.assertTrue(self.overlap(p,tail),p['part'])
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Ear inset /','Ear /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Nostril /','Jaw / hooked')):
    if n.startswith(child):parent=self.find(base,s if base not in ('Head /','Jaw / hooked') else None)
   if n.startswith(('Chest plate /','Belly plate /')):parent=self.find('Body /')
   if parent:yield p,parent
 def test_ears_tail_spines_and_limbs_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Ear /','Ear inset /','Tail /','Tail collar /','Tail spine /','Back spine /','Arm /','Leg /','Horn /','Cheek barb /','Foot claw /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_open_jaw_has_empty_space_above_tongue(self):self.assertTrue(self.jaw_gap(self.ps))
 def test_oral_back_is_recessed_and_visible_in_front_of_head(self):
  mouth=self.find('Mouth /');upper=self.find('Jaw / hooked');y=(min(mouth['positions'][1::3])+max(mouth['positions'][1::3]))/2;hits=ray_intersections(mouth,0,y);self.assertTrue(hits)
  self.assertGreater(max(upper['positions'][2::3])-max(hits),.10)
  for prefix in ('Head /','Body /'):
   hh=ray_intersections(self.find(prefix),0,y)
   if hh:self.assertGreater(max(hits)-max(hh),.008)
 def test_filled_jaw_and_detached_tail_ear_and_claws_are_rejected(self):
  u=self.find('Jaw / hooked');l=self.find('Jaw / open');lo=max(min(u['positions'][2::3]),min(l['positions'][2::3]));hi=min(max(u['positions'][2::3]),max(l['positions'][2::3]));z=lo+(hi-lo)*.7;low=min(l['positions'][1::3]);high=max(u['positions'][1::3]);filler=ellipsoid('Invalid jaw filler',(.5,.5,.5),(0,(low+high)/2,z),(.02,(high-low)*.6,.02),24,16);self.assertFalse(self.jaw_gap(self.ps+[filler]))
  for prefix,parent in (('Tail /','Body /'),('Ear /','Head /'),('Hand claw /','Hand /'),('Foot claw /','Foot /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
