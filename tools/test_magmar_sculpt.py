#!/usr/bin/env python3
"""Stored spitfire anatomy, fitted fire layers and a clear open bill cavity."""
import copy,json,os,unittest
from pathlib import Path
from magmar_sculpt import sculpture,HEIGHT
from sculpt_geometry import ellipsoid,translate
from bovine_sculpt import reverse_front
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_MAGMAR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class MagmarTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from magmar_glb import read_magmar
  cls.model=read_magmar(ROOT/'magmar.glb')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'magmar':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'magmar':self.m})
 def test_recipe_bounds_budget_and_distinct_anatomy(self):
  from magmar_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),115000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,count in (('Body /',1),('Collar /',1),('Head /',1),('Forehead lobe /',2),('Eye /',2),('Belly /',1),('Bill /',2),('Tongue /',1),('Shoulder /',2),('Arm /',2),('Cuff /',4),('Hand /',2),('Hand claw /',6),('Thigh /',2),('Shin band /',2),('Foot /',2),('Toe claw /',4),('Dorsal point /',3),('Tail /',1),('Crown flame /',2),('Crown glow /',4),('Tail flame /',1),('Tail glow /',2)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,prefix)
 def test_head_lobes_collar_bill_tongue_and_spines_are_rooted(self):
  head=self.find('Head /');body=self.find('Body /');collar=self.find('Collar /');upper=self.find('Bill / rooted cream');lower=self.find('Bill / rooted open')
  for a,b in ((head,collar),(collar,body),(upper,head),(lower,head),(self.find('Mouth back /'),head),(self.find('Tongue /'),lower)):
   self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Forehead lobe /',s),head))
  for j in range(3):self.assertTrue(self.overlap(self.find('Dorsal point /',j),head if j==2 else body))
 def test_arms_cuffs_hands_and_claws_are_rooted(self):
  for s in (-1,1):
   shoulder=self.find('Shoulder /',s);arm=self.find('Arm /',s);hand=self.find('Hand /',s)
   for a,b in ((shoulder,self.find('Body /')),(arm,shoulder),(hand,arm)):self.assertTrue(self.overlap(a,b),a['part'])
   for j in range(2):self.assertTrue(self.overlap(self.find('Cuff /',f'{s} {j}'),arm))
   for j in range(3):self.assertTrue(self.overlap(self.find('Hand claw /',f'{s} {j}'),hand))
 def test_thighs_shins_feet_and_toe_claws_are_rooted(self):
  for s in (-1,1):
   thigh=self.find('Thigh /',s);shin=self.find('Shin band /',s);foot=self.find('Foot /',s)
   for a,b in ((thigh,self.find('Body /')),(shin,thigh),(foot,shin)):self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(2):self.assertTrue(self.overlap(self.find('Toe claw /',f'{s} {j}'),foot))
 def test_tail_and_all_flame_roots_are_attached(self):
  tail=self.find('Tail /');self.assertTrue(self.overlap(tail,self.find('Body /')));self.assertTrue(self.overlap(self.find('Tail flame /'),tail))
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Crown flame /',s),self.find('Forehead lobe /',s)))
 def pairs(self):
  for p in self.ps:
   name=p['part'];s=name.split()[-1];parent=None
   for child,base in (('Belly /','Body /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /')):
    if name.startswith(child):parent=self.find(base,s if base not in ('Body /','Head /') else None)
   if name.startswith('Crown glow /'):parent=self.find('Crown flame /',name.split()[-2])
   if name.startswith('Tail glow /'):parent=self.find('Tail flame /')
   if parent:
    if name.startswith(('Crown glow /','Tail glow /')) and s=='-1':yield reverse_front(p),reverse_front(parent)
    else:yield p,parent
 def test_fitted_vertices_follow_actual_parent(self):attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_parent(self):attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_claws_tail_cuffs_and_flames_do_not_cross(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Cuff /','Hand claw /','Shin band /','Toe claw /','Dorsal point /','Tail /','Crown flame /','Tail flame /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_bill_gap_is_clear_and_rejects_a_fill(self):
  upper=self.find('Bill / rooted cream');lower=self.find('Bill / rooted open');y=(min(upper['positions'][1::3])+max(lower['positions'][1::3]))/2;z=.92*max(lower['positions'][2::3])
  def occupied(p,x,y,z):
   hits=ray_intersections(p,x,y);return any(a+.0001<z<b-.0001 for a,b in zip(hits[::2],hits[1::2]))
  for dx,dy in ((0,0),(.015,0),(-.015,0),(0,-.006)):
   self.assertFalse(any(occupied(p,dx,y+dy,z) for p in self.ps),'bill gap is filled')
  filled=ellipsoid('filled bill negative',(.5,.5,.5),(0,y,z),(.025,.015,.018),32,22);self.assertTrue(occupied(filled,0,y,z))
 def test_detached_flames_cuff_and_claws_are_rejected(self):
  for child,parent in (('Crown flame /','Forehead lobe /'),('Tail flame /','Tail /'),('Cuff /','Arm /'),('Hand claw /','Hand /'),('Toe claw /','Foot /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
