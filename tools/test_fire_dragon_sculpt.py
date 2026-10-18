#!/usr/bin/env python3
"""Fire-lizard shapes, real jaw gaps, fitted faces and complete rooted anatomy."""
import copy,json,os,unittest
from pathlib import Path
from fire_dragon_sculpt import sculpture,SPECIES,HEIGHTS
from fire_dragon_glb import read_fire_dragon,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
from test_steelix_sculpt import occupied
from sculpt_geometry import translate,ellipsoid
import test_nidorina_sculpt as fitted
ROOT=Path(os.environ.get('CRYSTAL_FIRE_DRAGON_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class FireDragonTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_fire_dragon(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_outward_connected_anatomy(self):assert_closed_models(self,self.models)
 def test_finite_corner_normals_follow_actual_faces(self):assert_model_shading(self,self.models)
 def test_source_identity_height_floor_and_standard_geometry_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in m['primitives']),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in m['primitives']),40000)
 def test_single_crest_and_twin_horns_wings_and_complete_digits_are_distinct(self):
  for s,m in self.models.items():
   counts={'Horn /':2 if s=='charizard' else 1,'Wing /':2 if s=='charizard' else 0,'Wing rib /':6 if s=='charizard' else 0,'Arm /':2,'Hand /':2,'Finger /':6,'Hand claw /':6,'Foot /':2,'Foot claw /':6,'Tail /':1,'Flame /':5}
   for prefix,count in counts.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),count,(s,prefix))
 def test_skull_mandible_horns_and_tongue_have_actual_volume_roots(self):
  for s,m in self.models.items():
   body=self.find(s,'Body /');head=self.find(s,'Head /');jaw=self.find(s,'Jaw /');self.assertTrue(occupied_overlap(head,body),s);self.assertTrue(occupied_overlap(jaw,head),s);self.assertTrue(occupied_overlap(self.find(s,'Tongue /'),jaw),s)
   for p in m['primitives']:
    if p['part'].startswith(('Horn /','Fang /')):self.assertTrue(occupied_overlap(p,head),(s,p['part']))
 def test_all_arms_palms_digits_haunches_and_feet_are_rooted(self):
  for s,m in self.models.items():
   for side in (-1,1):
    for child,parent in (('Arm /','Body /'),('Hand /','Arm /'),('Haunch /','Body /'),('Foot /','Haunch /')):self.assertTrue(occupied_overlap(self.find(s,child,side),self.find(s,parent,None if parent=='Body /' else side)),(s,child,side))
    for j in range(3):
     self.assertTrue(occupied_overlap(self.find(s,'Finger /',f'{side} {j}'),self.find(s,'Hand /',side)),(s,side,j));self.assertTrue(occupied_overlap(self.find(s,'Hand claw /',f'{side} {j}'),self.find(s,'Finger /',f'{side} {j}')),(s,side,j));self.assertTrue(occupied_overlap(self.find(s,'Foot claw /',f'{side} {j}'),self.find(s,'Foot /',side)),(s,side,j))
 def test_tapered_tail_and_all_folded_fire_layers_have_occupied_roots(self):
  for s,m in self.models.items():
   tail=self.find(s,'Tail /');ember=self.find(s,'Flame / coherent ember');self.assertTrue(occupied_overlap(tail,self.find(s,'Body /')),s);self.assertTrue(occupied_overlap(ember,tail),s)
   for p in m['primitives']:
    if p['part'].startswith('Flame /') and p is not ember:self.assertTrue(occupied_overlap(p,ember),(s,p['part']))
 def pairs(self,s):
  for p in self.models[s]['primitives']:
   n=p['part'];side=n.split()[-1];parent=None
   if n.startswith('Belly /'):parent=self.find(s,'Body /')
   elif n.startswith(('Eye socket /','Nostril /')):parent=self.find(s,'Head /')
   elif n.startswith('Eye white /'):parent=self.find(s,'Eye socket /',side)
   elif n.startswith('Iris /'):parent=self.find(s,'Eye white /',side)
   elif n.startswith('Pupil /'):parent=self.find(s,'Iris /',side)
   elif n.startswith('Glint /'):parent=self.find(s,'Pupil /',side)
   if parent:yield p,parent
 def test_fitted_belly_and_gaze_vertices_edges_and_centres_follow_actual_parents(self):
  for s in SPECIES:
   helper=fitted.NidorinaTests();helper.pairs=lambda:self.pairs(s);helper.test_fitted_layers_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_parent()
 def aperture_points(self,s):
  raw=sculpture(s,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);scale=HEIGHTS[s]/(hi-lo)
  return [(x*scale,(y-lo)*scale,z*scale) for x in (-.025,0.,.025) for y in ((.947,.965) if s=='charizard' else (.800,.820)) for z in ((.23,.30,.37) if s=='charizard' else (.23,.29,.33))]
 def test_open_mouths_have_real_empty_apertures(self):
  for s,m in self.models.items():
   for point in self.aperture_points(s):self.assertFalse(any(occupied(p,point) for p in m['primitives']),(s,point))
 def test_scalloped_adult_wings_leading_arms_ribs_and_claws_are_attached(self):
  s='charizard'
  for side in (-1,1):
   wing=self.find(s,'Wing /',side);arm=self.find(s,'Wing arm /',side);self.assertTrue(occupied_overlap(wing,self.find(s,'Body /')));self.assertTrue(occupied_overlap(arm,self.find(s,'Body /')));self.assertTrue(occupied_overlap(wing,arm));self.assertTrue(occupied_overlap(self.find(s,'Wing claw /',side),arm))
   for j in range(3):
    rib=self.find(s,'Wing rib /',f'{side} {j}');self.assertTrue(occupied_overlap(rib,wing),(side,j));self.assertTrue(occupied_overlap(rib,arm),(side,j))
 def test_grounded_pads_and_uncrossed_continuous_limbs(self):
  for s,m in self.models.items():
   for side in (-1,1):self.assertEqual(min(self.find(s,'Foot /',side)['positions'][1::3]),0.)
   for p in m['primitives']:
    if p['part'].startswith(('Arm /','Tail /','Finger /','Hand claw /','Foot claw /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_roots_and_filled_mouth_negative_controls(self):
  for s,m in self.models.items():
   for child,parent in (('Head /','Body /'),('Hand /','Arm /'),('Horn /','Head /')):self.assertFalse(occupied_overlap(translate(copy.deepcopy(self.find(s,child)),3,0,0),self.find(s,parent)))
   point=self.aperture_points(s)[0];fill=ellipsoid('Filled jaw negative',(.1,.1,.1),point,(.025,.025,.025),16,12);self.assertTrue(occupied(fill,point))

if __name__=='__main__':unittest.main()
