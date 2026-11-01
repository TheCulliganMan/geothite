#!/usr/bin/env python3
"""Stored two-color body, rooted curls/frills and supported egg geometry."""
import copy,json,os,unittest
from pathlib import Path
from egg_nurse_glb import read_blissey,geometry_digest
from blissey_sculpt import anatomy,sculpture,HEIGHT,PINK,WHITE,POUCH,EGG
from sculpt_geometry import translate,linear,ellipsoid
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_steelix_sculpt import occupied
import test_nidorina_sculpt as attachments
import test_chansey_sculpt as egg_checks
ROOT=Path(os.environ.get('CRYSTAL_BLISSEY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class BlisseyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.model=read_blissey(ROOT/'blissey.glb');raw=anatomy();cls.floor=min(v for p in raw for v in p['positions'][1::3]);cls.scale=HEIGHT/(max(v for p in raw for v in p['positions'][1::3])-cls.floor)
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 point=egg_checks.ChanseyTests.point
 contained=egg_checks.ChanseyTests.contained
 opening=egg_checks.ChanseyTests.opening
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'blissey':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'blissey':self.m})
 def test_recipe_palette_floor_height_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000)
  for label,color in (('Lower body /',WHITE),('Upper body /',PINK),('Pouch /',POUCH),('Egg /',EGG)):self.assertEqual(self.find(label)['base_color'],linear(color))
  for label,n in (('Crown /',2),('Curl /',2),('Arm frill /',6),('Skirt /',10),('Leg /',2),('Foot /',2)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_body_halves_pouch_egg_and_crown_have_actual_roots(self):
  for child,parent in (('Upper body /','Lower body /'),('Pouch /','Lower body /'),('Egg /','Pouch /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
  for s in (-1,1):
   self.assertTrue(self.overlap(self.find('Crown /',s),self.find('Upper body /')));self.assertTrue(self.overlap(self.find('Curl /',s),self.find('Crown /',s)))
 def test_limbs_frills_skirt_and_feet_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Upper body /'),('Arm /','Shoulder /'),('Leg /','Lower body /'),('Foot /','Leg /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent not in ('Upper body /','Lower body /') else None)),(s,child))
   self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
   for j in range(3):self.assertTrue(self.overlap(self.find('Arm frill /',f'{s} {j}'),self.find('Shoulder /',s)),(s,j))
  for p in self.ps:
   if p['part'].startswith('Skirt /'):self.assertTrue(self.overlap(p,self.find('Lower body /')),p['part'])
 def pairs(self):
  for p in self.ps:
   for child,parent in (('Eye /','Upper body /'),('Eye glint /','Eye /'),('Mouth lining /','Upper body /'),('Tongue /','Mouth lining /')):
    if p['part'].startswith(child):yield p,self.find(parent,p['part'].split()[-1] if parent=='Eye /' else None)
 def test_fitted_vertices_follow_actual_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_faces_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curls_frills_skirt_and_arms_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Curl /','Arm frill /','Skirt /','Arm /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_egg_is_contained_and_pouch_opening_is_empty(self):
  self.assertTrue(self.contained(self.find('Egg /')))
  for point in self.opening():self.assertFalse(any(occupied(p,point) for p in self.ps),point)
 def mouth_space(self):return [self.point(x,y,z) for x in (-.013,0,.013) for y in (.979,.991) for z in (.150,.158)]
 def test_happy_mouth_has_actual_recess_space(self):
  for point in self.mouth_space():self.assertFalse(any(occupied(p,point) for p in self.ps),point)
 def test_filled_mouth_and_pouch_are_rejected(self):
  fill=ellipsoid('Invalid filled mouth',PINK,self.point(0,.985,.153),tuple(r*self.scale for r in (.040,.030,.025)),32,24)
  for point in self.mouth_space():self.assertTrue(occupied(fill,point),point)
  fill=ellipsoid('Invalid filled pouch',POUCH,self.point(0,.494,.355),tuple(r*self.scale for r in (.14,.025,.11)),32,24)
  for point in self.opening():self.assertTrue(occupied(fill,point),point)
 def test_detached_appendages_and_protruding_egg_are_rejected(self):
  for child,parent in (('Curl /','Crown /'),('Arm frill /','Shoulder /'),('Skirt /','Lower body /'),('Egg /','Pouch /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
  self.assertFalse(self.contained(translate(copy.deepcopy(self.find('Egg /')),0,-.06*self.scale,0)))
if __name__=='__main__':unittest.main(defaultTest='BlisseyTests')
