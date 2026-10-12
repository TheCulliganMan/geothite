#!/usr/bin/env python3
"""Adult skull gaze, supported raised bone, strong tail and posed anatomy."""
import copy,json,os,unittest
from pathlib import Path
from marowak_sculpt import sculpture,anatomy,HEIGHT,INK
from sculpt_geometry import translate
from steelix_sculpt import block
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_steelix_sculpt import occupied
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_MAROWAK_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class MarowakTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'marowak.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'marowak':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'marowak':self.m})
 def test_recipe_parity_floor_height_and_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Orbit /',2),('Horn /',2),('Cheek /',2),('Finger /',6),('Thumb /',1),('Bone lobe /',4),('Toe claw /',2),('Hand claw /',2),('Belly seam /',2),('Tail spine /',1)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_skull_neck_snout_tail_and_spine_have_actual_roots(self):
  for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Skull /','Head /'),('Snout /','Skull /'),('Tail /','Body /'),('Tail spine /','Tail /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
  for s in (-1,1):
   for child in ('Horn /','Cheek /'):self.assertTrue(self.overlap(self.find(child,s),self.find('Skull /')),(child,s))
 def test_shoulders_arms_hands_digits_and_claws_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /'),('Hand claw /','Hand /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
   for j in range(3):self.assertTrue(self.overlap(self.find('Finger /',f'{s} {j}'),self.find('Hand /',s)),(s,j))
  self.assertTrue(self.overlap(self.find('Thumb /'),self.find('Hand /',-1)))
 def test_bone_is_actually_held_and_all_end_lobes_root(self):
  bone=self.find('Bone shaft /');self.assertTrue(self.overlap(bone,self.find('Hand /',-1)));self.assertTrue(self.overlap(bone,self.find('Thumb /')))
  for p in self.ps:
   if p['part'].startswith('Bone lobe /'):self.assertTrue(self.overlap(p,bone),p['part'])
 def test_raised_bone_and_extended_opposite_arm(self):
  left=self.find('Hand /',-1);right=self.find('Hand /',1);self.assertGreater(min(left['positions'][1::3]),max(right['positions'][1::3])+.10)
  self.assertGreater(min(self.find('Bone shaft /')['positions'][1::3]),.75);self.assertGreater(max(right['positions'][::3]),max(self.find('Body /')['positions'][::3])+.20)
 def test_haunches_feet_and_claws_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Haunch /','Body /'),('Foot /','Haunch /'),('Toe claw /','Foot /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
   self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Belly /','Body /'),('Belly seam /','Belly /'),('Orbit /','Skull /'),('Eye /','Orbit /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Eye glint /','Pupil /'),('Nostril /','Snout /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Orbit /','Eye /','Iris /','Pupil /') else None)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_horns_limbs_fingers_club_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Horn /','Cheek /','Arm /','Finger /','Thumb /','Hand claw /','Toe claw /','Bone shaft /','Tail /','Tail spine /')):self.assertIsNone(crossed_faces(p),p['part'])
 def recess_points(self):
  raw=anatomy();lo=min(v for p in raw for v in p['positions'][1::3]);hi=max(v for p in raw for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  return [(s*.170*scale,(y-lo)*scale,z*scale) for s in (-1,1) for y in (1.322,1.330) for z in (.255,.265)]
 def test_orbits_have_actual_recess_space(self):
  for q in self.recess_points():self.assertFalse(any(occupied(p,q) for p in self.ps),q)
 def test_filled_recesses_and_detached_appendages_are_rejected(self):
  points=self.recess_points();axes=list(zip(*points));fill=block('Filled recess negative',INK,tuple((min(a)+max(a))/2 for a in axes),tuple((max(a)-min(a))/2+.01 for a in axes));self.assertTrue(all(occupied(fill,q) for q in points))
  for child,parent in (('Bone shaft /','Hand /'),('Finger /','Hand /'),('Horn /','Skull /'),('Foot /','Haunch /'),('Tail spine /','Tail /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
