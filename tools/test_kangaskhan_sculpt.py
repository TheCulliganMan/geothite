#!/usr/bin/env python3
"""Stored parent/baby attachments, open pouch, mouth clearance and surface guards."""
import copy,json,os,unittest
from pathlib import Path
from kangaskhan_sculpt import sculpture,HEIGHT
from sculpt_geometry import translate,ellipsoid
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_steelix_sculpt import occupied
from test_slime_sculpt import ray_intersections
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_KANGASKHAN_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class KangaskhanTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'kangaskhan.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'kangaskhan':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'kangaskhan':self.m})
 def test_recipe_bounds_unique_parts_and_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),70);self.assertEqual(len({p['part'] for p in self.ps}),70)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),120000)
 def roots(self):
  for child,parent in (('Head /','Body /'),('Pouch /','Body /'),('Skull plate /','Head /'),('Snout /','Head /'),('Jaw /','Head /'),('Mouth back /','Head /'),('Tongue /','Jaw /'),('Tail /','Body /'),('Baby body /','Pouch /'),('Baby neck /','Baby body /'),('Baby head /','Baby neck /')):yield self.find(child),self.find(parent)
  for s in (-1,1):
   for child,parent in (('Fang /','Snout /'),('Ear /','Head /'),('Shoulder /','Body /'),('Arm /','Shoulder /'),('Fist /','Arm /'),('Shoulder plate /','Shoulder /'),('Haunch /','Body /'),('Ankle /','Haunch /'),('Foot /','Ankle /'),('Baby ear /','Baby head /'),('Baby arm /','Baby head /'),('Baby hand /','Baby arm /')):yield self.find(child,s),self.find(parent,s if parent in ('Shoulder /','Arm /','Haunch /','Ankle /','Baby arm /') else None)
   yield self.find('Baby hand /',s),self.find('Pouch /')
   for j in range(3):
    for prefix,parent in (('Hand claw /','Fist /'),('Foot claw /','Foot /')):yield next(p for p in self.ps if p['part'].startswith(prefix) and p['part'].endswith(f' {s} {j}')),self.find(parent,s)
  for child in [p for p in self.ps if p['part'].startswith('Back plate /')]:yield child,self.find('Body /')
 def test_every_limb_claw_plate_pouch_and_baby_has_actual_root(self):
  for child,parent in self.roots():self.assertTrue(self.overlap(child,parent),(child['part'],parent['part']))
 def pairs(self):
  yield self.find('Belly /'),self.find('Body /')
  yield self.find('Baby mouth /'),self.find('Baby head /')
  for s in (-1,1):
   for child,parent in (('Eye /','Head /'),('Pupil /','Eye /'),('Eye slit /','Pupil /'),('Nostril /','Snout /'),('Haunch armor outline /','Haunch /'),('Haunch plate /','Haunch armor outline /'),('Baby eye /','Baby head /'),('Baby glint /','Baby eye /')):yield self.find(child,s),self.find(parent,s if parent in ('Eye /','Pupil /','Haunch /','Haunch armor outline /','Baby eye /') else None)
 def test_fitted_vertices_follow_actual_stored_parents(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parents(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_claws_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Hand claw /','Foot claw /','Tail /','Baby arm /','Shoulder plate /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_mouth_gap_is_actual_empty_space(self):
  scale=max(self.find('Pouch /')['positions'][1::3])/.62
  for x in (-.035,0,.035):
   for y in (.902,.914):
    for z in (.390,.412):
     q=(x*scale,y*scale,z*scale)
     for p in self.ps:self.assertFalse(occupied(p,q),p['part'])
 def test_pouch_has_empty_interior_beyond_baby(self):
  scale=max(self.find('Pouch /')['positions'][1::3])/.62;q=(0,.610*scale,.425*scale)
  self.assertFalse(occupied(self.find('Pouch /'),q));self.assertFalse(occupied(self.find('Baby body /'),q))
  self.assertTrue(occupied(ellipsoid('filled negative',(.9,.8,.6),(0,.48*scale,.34*scale),(.25*scale,.20*scale,.16*scale),32,24),q))
 def contained_baby(self,baby,pouch):
  rim=max(pouch['positions'][1::3])
  for x,y,z in zip(baby['positions'][::3],baby['positions'][1::3],baby['positions'][2::3]):
   if y>=rim:continue
   hits=ray_intersections(pouch,x,y)
   if not hits or z<min(hits)-1e-6 or z>max(hits)+1e-6:return False
  return True
 def test_stored_baby_body_stays_within_actual_pouch_envelope(self):
  baby=self.find('Baby body /');pouch=self.find('Pouch /')
  self.assertTrue(self.contained_baby(baby,pouch))
  self.assertFalse(self.contained_baby(translate(copy.deepcopy(baby),0,0,.25),pouch))
 def test_detached_limb_baby_and_pouch_are_rejected(self):
  for child,parent in (('Arm /','Shoulder /'),('Baby body /','Pouch /'),('Pouch /','Body /'),('Baby hand /','Baby arm /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
