#!/usr/bin/env python3
"""Stored mime anatomy, supported glove digits, fitted gaze and open smile."""
import copy,json,os,unittest
from pathlib import Path
from mr_mime_sculpt import sculpture,anatomy,HEIGHT
from mr_mime_glb import read_mr_mime,geometry_digest
from sculpt_geometry import translate
from steelix_sculpt import block
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_steelix_sculpt import occupied
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_MR_MIME_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class MimeTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=read_mr_mime(ROOT/'mr__mime.glb')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'mr__mime':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'mr__mime':self.m})
 def test_recipe_parity_floor_height_and_budget(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
 def test_named_anatomy_and_five_digits_per_hand(self):
  for label,n in (('Ear fan /',2),('Palm /',2),('Finger /',10),('Fingertip /',10),('Finger pad /',10),('Shoulder /',2),('Hip /',2),('Knee /',2),('Shoe curl /',2)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_head_neck_jaws_and_tongue_have_actual_roots(self):
  for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Upper lip /','Head /'),('Lower lip /','Head /'),('Mouth back /','Head /'),('Tongue /','Lower lip /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Ear fan /',s),self.find('Head /')),s)
 def test_shoulders_arms_and_palms_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Palm /','Arm /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
 def test_all_ten_digits_and_fingertips_have_actual_roots(self):
  for s in (-1,1):
   for i in range(5):
    finger=self.find('Finger /',f'{s} {i}');self.assertTrue(self.overlap(finger,self.find('Palm /',s)),(s,i));self.assertTrue(self.overlap(self.find('Fingertip /',f'{s} {i}'),finger),(s,i))
 def test_hips_knees_shins_shoes_and_curls_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Hip /','Body /'),('Thigh /','Hip /'),('Knee /','Thigh /'),('Shin /','Knee /'),('Shoe /','Shin /'),('Shoe curl /','Shoe /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
 def test_both_blue_shoes_support_and_gloves_pose_asymmetrically(self):
  for s in (-1,1):
   p=self.find('Shoe /',s);self.assertEqual(min(p['positions'][1::3]),0);self.assertLess(p['base_color'][0],p['base_color'][2])
  left=self.find('Palm /',-1);right=self.find('Palm /',1);self.assertGreater(min(left['positions'][1::3]),max(right['positions'][1::3])+.15);self.assertGreater(min(right['positions'][2::3]),max(left['positions'][2::3]))
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Eye /','Head /'),('Pupil /','Eye /'),('Eye glint /','Pupil /'),('Cheek /','Head /'),('Chest /','Body /'),('Mouth lining /','Head /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Eye /','Pupil /') else None)
   if p['part'].startswith('Finger pad /'):yield p,self.find('Fingertip /',' '.join(p['part'].split()[-2:]))
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def mouth_points(self):
  raw=anatomy();lo=min(v for p in raw for v in p['positions'][1::3]);hi=max(v for p in raw for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  return [(x*scale,(y-lo)*scale,z*scale) for x in (-.024,0,.024) for y in (1.065,1.070) for z in (.19,.205)]
 def test_smile_has_actual_empty_aperture_and_filled_negative(self):
  points=self.mouth_points()
  for q in points:self.assertFalse(any(occupied(p,q) for p in self.ps),q)
  axes=list(zip(*points));fill=block('Filled smile negative',(0,0,0),tuple((min(a)+max(a))/2 for a in axes),tuple((max(a)-min(a))/2+.01 for a in axes));self.assertTrue(all(occupied(fill,q) for q in points))
 def test_curved_limbs_digits_ear_fans_and_toes_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Finger /','Thigh /','Shin /','Ear fan /','Shoe curl /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_gloves_digits_fans_and_shoes_are_rejected(self):
  for child,parent in (('Palm /','Arm /'),('Finger /','Palm /'),('Ear fan /','Head /'),('Shoe /','Shin /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
