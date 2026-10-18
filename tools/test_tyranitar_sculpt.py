#!/usr/bin/env python3
"""Independent Tyranitar armor, roots, real gape, visible gaze and complete claws."""
import copy,os,unittest
from pathlib import Path
from tyranitar_sculpt import sculpture,HEIGHT,body_geometry
from tyranitar_glb import read_tyranitar,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
from test_steelix_sculpt import occupied
from sculpt_geometry import ellipsoid,translate
import test_nidorina_sculpt as fitted
ROOT=Path(os.environ.get('CRYSTAL_TYRANITAR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class TyranitarTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=read_tyranitar(ROOT/'tyranitar.glb')
 def find(self,prefix,suffix=None):return next(p for p in self.model['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'tyranitar':self.model})
 def test_actual_corner_normals_remain_finite_and_outward(self):assert_model_shading(self,{'tyranitar':self.model})
 def test_recipe_floor_height_and_standard_budgets(self):
  self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()));ps=self.model['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
 def test_complete_crown_armor_and_all_digits_remain_distinct(self):
  for prefix,n in [('Body /',1),('Head /',1),('Jaw /',1),('Belly /',1),('Belly armor /',2),('Torso vent /',2),('Thigh vent /',2),('Crown spike /',5),('Cheek plate /',4),('Back spike /',4),('Tail spike /',3),('Arm /',2),('Forearm /',2),('Hand /',2),('Hand claw /',6),('Foot /',2),('Foot claw /',6),('Fang /',2),('Lower tooth /',3)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.model['primitives']),n,prefix)
 def test_skull_jaw_crown_cheeks_teeth_and_throat_have_occupied_roots(self):
  self.assertTrue(occupied_overlap(self.find('Head /'),self.find('Body /')));self.assertTrue(occupied_overlap(self.find('Jaw /'),self.find('Head /')));self.assertTrue(occupied_overlap(self.find('Mouth back /'),self.find('Head /')));self.assertTrue(occupied_overlap(self.find('Tongue /'),self.find('Jaw /')))
  for p in self.model['primitives']:
   if p['part'].startswith(('Crown spike /','Cheek plate /','Fang /')):self.assertTrue(occupied_overlap(p,self.find('Head /')),p['part'])
   elif p['part'].startswith('Lower tooth /'):self.assertTrue(occupied_overlap(p,self.find('Jaw /')),p['part'])
 def test_complete_arms_palms_haunches_ankles_and_claws_have_roots(self):
  for side in (-1,1):
   for child,parent in [('Arm /','Body /'),('Forearm /','Arm /'),('Hand /','Forearm /'),('Thigh /','Body /'),('Shin /','Thigh /'),('Foot /','Shin /')]:self.assertTrue(occupied_overlap(self.find(child,side),self.find(parent,None if parent=='Body /' else side)),(child,side))
   self.assertAlmostEqual(min(self.find('Foot /',side)['positions'][1::3]),0.,places=7)
   for j in range(3):
    for claw,parent in [('Hand claw /','Hand /'),('Foot claw /','Foot /')]:self.assertTrue(occupied_overlap(self.find(claw,f'{side} {j}'),self.find(parent,side)),(claw,side,j))
 def test_all_dorsal_spines_and_heavy_curved_tail_have_occupied_roots(self):
  self.assertTrue(occupied_overlap(self.find('Tail /'),self.find('Body /')))
  for p in self.model['primitives']:
   if p['part'].startswith(('Back spike /','Tail spike /')):self.assertTrue(occupied_overlap(p,self.find('Tail /' if p['part'].startswith('Tail spike /') else 'Body /')),p['part'])
 def pairs(self):
  for p in self.model['primitives']:
   n=p['part'];parent=None;side=n.split()[-1]
   for prefix,base in [('Belly /','Body /'),('Belly armor /','Belly /'),('Torso vent /','Body /'),('Eye socket /','Head /'),('Eye white /','Eye socket /'),('Iris /','Eye white /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Nostril /','Head /'),('Thigh vent /','Thigh /')]:
    if n.startswith(prefix):parent=self.find(base,side if base in ('Eye socket /','Eye white /','Iris /','Pupil /','Thigh /') else None)
   if parent:yield p,parent
 def test_fitted_armor_and_gaze_follow_actual_stored_vertices_edges_and_centres(self):
  helper=fitted.NidorinaTests();helper.pairs=self.pairs;helper.test_fitted_layers_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_parent()
 def test_both_white_gazes_are_visible_in_front_of_the_complete_skull(self):
  head=self.find('Head /')
  for side in (-1,1):
   eye=self.find('Eye white /',side);visible=0
   for k in range(0,len(eye['indices']),3):
    tri=[eye['positions'][i*3:i*3+3] for i in eye['indices'][k:k+3]];a,b,c=tri
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    x,y,z=[sum(v[j] for v in tri)/3 for j in range(3)];hits=rays(head,x,y)
    if hits and z>max(hits)+.001:visible+=1
   self.assertGreaterEqual(visible,3,(side,'gaze hidden behind skull'))
 def aperture_points(self):
  raw=sculpture(False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  return [(x*scale,(y-lo)*scale,z*scale) for x in (-.025,0.,.025) for y in (1.05,1.065) for z in (.24,.29,.34)]
 def test_mouth_has_real_empty_space(self):
  for point in self.aperture_points():self.assertFalse(any(occupied(p,point) for p in self.model['primitives']),point)
 def test_actual_torso_vents_change_the_green_surface_depth(self):
  original=body_geometry(False);dented=body_geometry();self.assertEqual(len(original['positions']),len(dented['positions']));delta=[a-b for a,b in zip(original['positions'][2::3],dented['positions'][2::3])];self.assertGreater(max(delta),.01);self.assertTrue(all(v>=-1e-7 for v in delta))
 def test_curved_limbs_tail_spines_and_claws_have_uncrossed_faces(self):
  for p in self.model['primitives']:
   if p['part'].startswith(('Arm /','Forearm /','Tail /','Hand claw /','Foot claw /','Crown spike /','Back spike /','Tail spike /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_spines_claws_and_filled_mouth_negative_controls(self):
  for child,parent in [('Crown spike /','Head /'),('Back spike /','Body /'),('Tail spike /','Tail /'),('Hand claw /','Hand /')]:self.assertFalse(occupied_overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
  point=self.aperture_points()[0];self.assertTrue(occupied(ellipsoid('filled mouth negative',(.1,.1,.1),point,(.02,.02,.02),16,12),point))
if __name__=='__main__':unittest.main()
