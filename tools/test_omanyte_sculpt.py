#!/usr/bin/env python3
"""Stored nautilus-shell seams, mantle attachments, clear eyes and grounded fan."""
import copy,json,os,unittest
from pathlib import Path
from skin_glb import packed
from omanyte_sculpt import sculpture,HEIGHT,CREAM,BLUE,WHITE
from sculpt_geometry import linear,translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from sculpt_test_support import indexed_ray_intersections as ray_intersections
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_OMANYTE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class OmanyteTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=__import__('nautilus_glb').read_nautilus(ROOT/'omanyte.glb', 'omanyte')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'omanyte':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'omanyte':self.m})
 def test_recipe_height_floor_palette_and_budget(self):
  from nautilus_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),32);self.assertEqual(len({p['part'] for p in self.ps}),32)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),22000)
  self.assertEqual(packed(self.find('Shell /')['base_color']),packed(linear(CREAM)));self.assertEqual(packed(self.find('Body /')['base_color']),packed(linear(BLUE)))
  for s in (-1,1):self.assertEqual(packed(self.find('Eye /',s)['base_color']),packed(linear(WHITE)))
 def test_mantle_eyes_and_all_tentacles_have_actual_volume_roots(self):
  body=self.find('Body /');self.assertTrue(self.overlap(self.find('Shell /'),body))
  for p in self.ps:
   if p['part'].startswith(('Eye /','Tentacle /')):self.assertTrue(self.overlap(p,body),p['part'])
 def test_all_shell_seams_and_spirals_touch_actual_shell(self):
  for p in self.ps:
   if p['part'].startswith(('Shell seam /','Shell spiral /')):self.assertTrue(self.overlap(p,self.find('Shell /')),p['part'])
 def pairs(self):
  for s in (-1,1):
   yield self.find('Iris /',s),self.find('Eye /',s)
   yield self.find('Eye gleam /',s),self.find('Iris /',s)
 def test_fitted_eyes_follow_actual_stored_parents(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_eye_centers_and_edges_clear_actual_parents(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_all_tentacles_seams_and_spirals_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Tentacle /','Shell seam /','Shell spiral /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_mantle_shell_and_tentacles_leave_every_stored_iris_vertex_clear(self):
  for s in (-1,1):
   iris=self.find('Iris /',s)
   for x,y,z in zip(iris['positions'][::3],iris['positions'][1::3],iris['positions'][2::3]):
    for p in self.ps:
     if not p['part'].startswith(('Body /','Shell /','Tentacle /')):continue
     hits=ray_intersections(p,x,y)
     if hits:self.assertLess(max(hits),z,p['part'])
 def test_eight_lower_fan_tentacles_are_grounded(self):
  fan=[p for p in self.ps if 'grounded blue Omanyte fan tentacle' in p['part']];self.assertEqual(len(fan),8)
  for p in fan:self.assertEqual(min(p['positions'][1::3]),0,p['part'])
 def test_detached_tentacles_eyes_and_shell_seams_are_rejected(self):
  for child,parent in (('Tentacle /','Body /'),('Eye /','Body /'),('Shell seam /','Shell /'),('Shell spiral /','Shell /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
