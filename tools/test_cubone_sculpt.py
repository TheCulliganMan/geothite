#!/usr/bin/env python3
"""Stored skull anatomy, rooted gripping fingers, fitted gaze and bone club."""
import copy,json,os,unittest
from pathlib import Path
from cubone_sculpt import sculpture,anatomy,HEIGHT,INK
from sculpt_geometry import translate
from steelix_sculpt import block
from test_steelix_sculpt import occupied
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_CUBONE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class CuboneTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'cubone.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'cubone':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'cubone':self.m})
 def test_recipe_parity_floor_height_and_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Orbit /',2),('Horn /',2),('Cheek /',2),('Finger /',6),('Thumb /',1),('Bone lobe /',4),('Toe claw /',2)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_skull_neck_snout_and_tail_have_actual_roots(self):
  for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Skull /','Head /'),('Snout /','Skull /'),('Tail /','Body /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
  for s in (-1,1):
   for child in ('Horn /','Cheek /'):self.assertTrue(self.overlap(self.find(child,s),self.find('Skull /')),(child,s))
 def test_shoulders_arms_hands_and_digits_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
   for j in range(3):self.assertTrue(self.overlap(self.find('Finger /',f'{s} {j}'),self.find('Hand /',s)),(s,j))
  self.assertTrue(self.overlap(self.find('Thumb /'),self.find('Hand /',-1)))
 def test_bone_is_actually_held_and_all_end_lobes_root(self):
  bone=self.find('Bone shaft /');self.assertTrue(self.overlap(bone,self.find('Hand /',-1)));self.assertTrue(self.overlap(bone,self.find('Thumb /')))
  for p in self.ps:
   if p['part'].startswith('Bone lobe /'):self.assertTrue(self.overlap(p,bone),p['part'])
 def test_haunches_feet_and_claws_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Haunch /','Body /'),('Foot /','Haunch /'),('Toe claw /','Foot /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
   self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Belly /','Body /'),('Orbit /','Skull /'),('Eye /','Orbit /'),('Pupil /','Eye /'),('Eye glint /','Pupil /'),('Nostril /','Snout /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Orbit /','Eye /','Pupil /') else None)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_horns_limbs_fingers_club_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Horn /','Cheek /','Arm /','Finger /','Thumb /','Bone shaft /','Tail /')):self.assertIsNone(crossed_faces(p),p['part'])
 def recess_points(self):
  raw=anatomy();lo=min(v for p in raw for v in p['positions'][1::3]);hi=max(v for p in raw for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  orbit=[(s*.180,y,z) for s in (-1,1) for y in (1.304,1.312) for z in (.27,.28)]
  nostril=[(s*.077,y,z) for s in (-1,1) for y in (1.083,1.090) for z in (.46,.47)]
  return [[(x*scale,(y-.20-lo)*scale,z*scale) for x,y,z in points] for points in (orbit,nostril)]
 def test_orbits_and_nostrils_have_actual_recess_space(self):
  for points in self.recess_points():
   for q in points:self.assertFalse(any(occupied(p,q) for p in self.ps),q)
 def test_filled_recesses_are_rejected(self):
  for points in self.recess_points():
   axes=list(zip(*points));fill=block('Filled recess negative',INK,tuple((min(a)+max(a))/2 for a in axes),tuple((max(a)-min(a))/2+.01 for a in axes));self.assertTrue(all(occupied(fill,q) for q in points))
 def test_detached_club_and_appendages_are_rejected(self):
  for child,parent in (('Bone shaft /','Hand /'),('Finger /','Hand /'),('Horn /','Skull /'),('Foot /','Haunch /'),('Tail /','Body /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
