#!/usr/bin/env python3
"""Stored vine roots, uncrossed strands, visible pupils and rounded shoe checks."""
import copy,json,os,unittest
from pathlib import Path
from tangela_sculpt import sculpture,HEIGHT,RED,WHITE
from sculpt_geometry import translate,linear
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_slime_sculpt import ray_intersections
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_TANGELA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class TangelaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'tangela.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def vines(self):return [p for p in self.ps if p['part'].startswith(('Front vine /','Front weave /','Rear vine /','Rear weave /'))]
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'tangela':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'tangela':self.m})
 def test_recipe_floor_height_palette_and_round_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),110000)
  self.assertEqual(len(self.vines()),41);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for s in (-1,1):
   shoe=self.find('Shoe /',s);self.assertEqual(shoe['base_color'],linear(RED));self.assertEqual(self.find('Eye /',s)['base_color'],linear(WHITE));self.assertEqual(min(shoe['positions'][1::3]),0)
   self.assertGreater(max(shoe['positions'][2::3])-min(shoe['positions'][2::3]),max(shoe['positions'][1::3])-min(shoe['positions'][1::3]))
 def test_every_vine_shoe_and_eye_has_an_actual_body_root(self):
  for p in self.vines()+[p for p in self.ps if p['part'].startswith(('Eye /','Shoe /'))]:self.assertTrue(self.overlap(p,self.find('Body /')),p['part'])
 def pairs(self):
  yield self.find('Face /'),self.find('Body /')
  for s in (-1,1):yield self.find('Pupil /',s),self.find('Eye /',s)
 def test_fitted_face_and_pupil_vertices_follow_actual_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_and_pupil_triangles_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_all_individual_vines_are_uncrossed(self):
  for p in self.vines():self.assertIsNone(crossed_faces(p),p['part'])
 def test_vines_do_not_occlude_the_stored_pupils(self):
  for s in (-1,1):
   pupil=self.find('Pupil /',s)
   for i in range(0,len(pupil['positions']),3):
    x,y,z=pupil['positions'][i:i+3]
    for p in self.vines():
     hits=ray_intersections(p,x,y)
     if hits:self.assertLess(max(hits),z,p['part'])
 def test_detached_strands_shoes_and_eyes_are_rejected(self):
  for child in ('Front vine /','Rear vine /','Shoe /','Eye /'):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find('Body /')))
if __name__=='__main__':unittest.main()
