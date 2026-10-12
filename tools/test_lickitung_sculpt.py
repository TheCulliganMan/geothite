#!/usr/bin/env python3
"""Physical roots, fitted markings and solid anatomy for the tongue bearer."""
import copy,json,os,unittest
from pathlib import Path
from lickitung_sculpt import sculpture,HEIGHT,PINK
from sculpt_geometry import translate,linear
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_LICKITUNG_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class LickitungTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'lickitung.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'lickitung':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'lickitung':self.m})
 def test_recipe_palette_floor_height_and_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),110000)
  self.assertEqual(self.find('Head /')['base_color'],linear(PINK));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Tongue /',1),('Tongue groove /',1),('Belly band /',4),('Thigh ring /',2),('Thigh center /',2),('Toe /',2),('Hand tip /',2)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_head_neck_tongue_and_tail_have_actual_roots(self):
  for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Tongue /','Head /'),('Tail /','Body /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
 def test_all_limbs_tips_and_toes_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /'),('Hand tip /','Hand /'),('Thigh /','Body /'),('Foot /','Thigh /'),('Toe /','Foot /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
   self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Mouth lining /','Head /'),('Tongue groove /','Tongue /'),('Belly band /','Body /'),('Eye /','Head /'),('Eye glint /','Eye /'),('Nostril /','Head /'),('Thigh ring /','Thigh /'),('Thigh center /','Thigh ring /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Eye /','Thigh /','Thigh ring /') else None)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_faces_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_tongue_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Tongue /','Tail /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_long_broad_tongue_and_heavy_tail(self):
  tongue=self.find('Tongue /');tail=self.find('Tail /');self.assertGreater(max(tongue['positions'][2::3]),max(self.find('Head /')['positions'][2::3])+.4);self.assertGreater(max(tongue['positions'][::3])-min(tongue['positions'][::3]),.20);self.assertGreater(max(tail['positions'][1::3]),.7);self.assertLess(min(tail['positions'][2::3]),-.5)
 def test_detached_tongue_and_appendages_are_rejected(self):
  for child,parent in (('Tongue /','Head /'),('Tail /','Body /'),('Hand tip /','Hand /'),('Toe /','Foot /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
