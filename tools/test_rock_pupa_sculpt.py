#!/usr/bin/env python3
"""Rock larva/pupa silhouettes, real anatomy roots, visible gaze and fitted armor seams."""
import copy,os,unittest
from pathlib import Path
from rock_pupa_sculpt import sculpture,SPECIES,HEIGHTS
from rock_pupa_glb import read_rock_pupa,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
from sculpt_geometry import translate
from bovine_sculpt import reverse_front
import test_nidorina_sculpt as fitted
ROOT=Path(os.environ.get('CRYSTAL_ROCK_PUPA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class RockPupaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_rock_pupa(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_solids(self):assert_closed_models(self,self.models)
 def test_finite_corner_normals_follow_actual_faces(self):assert_model_shading(self,self.models)
 def test_recipe_floor_height_and_standard_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
 def test_distinct_horned_biped_and_legless_segmented_pupa_anatomy(self):
  for s,m in self.models.items():
   counts={'Body /':1,'Head /':1,'Eye socket /':2,'Horn /':1 if s=='larvitar' else 0,'Cheek tear /':2 if s=='larvitar' else 0,'Tail prong /':6 if s=='larvitar' else 0,'Tail hub /':1 if s=='larvitar' else 0,'Back vent /':2 if s=='larvitar' else 0,'Arm /':2 if s=='larvitar' else 0,'Hand /':0,'Finger /':0,'Foot /':2 if s=='larvitar' else 0,'Toe /':6 if s=='larvitar' else 0,'Crown spike /':3 if s=='pupitar' else 0,'Side spike /':6 if s=='pupitar' else 0,'Shell seam /':3 if s=='pupitar' else 0,'Back seam /':3 if s=='pupitar' else 0,'Tail keel /':1 if s=='pupitar' else 0,'Back spike /':0}
   for prefix,n in counts.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),n,(s,prefix))
 def test_skulls_and_all_crown_points_have_occupied_volume_roots(self):
  for s,m in self.models.items():
   head=self.find(s,'Head /');self.assertTrue(occupied_overlap(head,self.find(s,'Body /')),s)
   for p in m['primitives']:
    if p['part'].startswith(('Horn /','Crown spike /')):self.assertTrue(occupied_overlap(p,head),p['part'])
 def test_larva_tapered_forelimbs_and_hind_supports_are_rooted(self):
  if 'larvitar' not in self.models:return
  s='larvitar'
  for side in (-1,1):
   for child,parent in [('Arm /','Body /'),('Thigh /','Body /'),('Foot /','Thigh /')]:self.assertTrue(occupied_overlap(self.find(s,child,side),self.find(s,parent,None if parent=='Body /' else side)),(child,side))
   self.assertAlmostEqual(min(self.find(s,'Foot /',side)['positions'][1::3]),0.,places=7)
   for j in range(3):
    self.assertTrue(occupied_overlap(self.find(s,'Toe /',f'{side} {j}'),self.find(s,'Foot /',side)),(side,j))
 def test_larva_short_tail_hub_and_all_six_radial_points_have_roots(self):
  if 'larvitar' not in self.models:return
  s='larvitar';tail=self.find(s,'Tail /');self.assertTrue(occupied_overlap(tail,self.find(s,'Body /')))
  hub=self.find(s,'Tail hub /');self.assertTrue(occupied_overlap(hub,tail))
  for j in range(6):self.assertTrue(occupied_overlap(self.find(s,'Tail prong /',j),hub),j)
 def test_pupa_all_side_points_and_lower_fold_have_roots(self):
  if 'pupitar' not in self.models:return
  s='pupitar';self.assertTrue(occupied_overlap(self.find(s,'Tail keel /'),self.find(s,'Body /')))
  for side in (-1,1):
   for j in range(3):self.assertTrue(occupied_overlap(self.find(s,'Side spike /',f'{side} {j}'),self.find(s,'Head /' if j==2 else 'Body /')),(side,j))
 def pairs(self,s):
  for p in self.models[s]['primitives']:
   n=p['part'];side=n.split()[-1];parent=None
   if n.startswith(('Back seam /','Back vent /')):yield reverse_front(p),reverse_front(self.find(s,'Body /'));continue
   for prefix,base in [('Belly /','Body /'),('Lip /','Head /'),('Cheek tear /','Head /'),('Nostril /','Head /'),('Body vent /','Body /'),('Shell seam /','Body /'),('Eye socket /','Head /'),('Eye white /','Eye socket /'),('Iris /','Eye white /'),('Pupil /','Iris /'),('Glint /','Pupil /')]:
    if n.startswith(prefix):parent=self.find(s,base,side if base in ('Eye socket /','Eye white /','Iris /','Pupil /') else None)
   if parent:yield p,parent
 def test_fitted_gaze_marks_and_front_back_seams_clear_actual_parents(self):
  for s in self.models:
   helper=fitted.NidorinaTests();helper.pairs=lambda:self.pairs(s);helper.test_fitted_layers_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_parent()
 def test_both_white_gazes_remain_visible_in_front_of_the_skull(self):
  for s in self.models:
   head=self.find(s,'Head /')
   for side in (-1,1):
    eye=self.find(s,'Eye white /',side);visible=0
    for k in range(0,len(eye['indices']),3):
     tri=[eye['positions'][i*3:i*3+3] for i in eye['indices'][k:k+3]];a,b,c=tri
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(v[j] for v in tri)/3 for j in range(3)];hits=rays(head,x,y)
     if hits and z>max(hits)+.001:visible+=1
    self.assertGreaterEqual(visible,3,(s,side))
 def test_larva_red_diamond_and_black_tears_are_supported_and_distinct(self):
  if 'larvitar' not in self.models:return
  s='larvitar';belly=self.find(s,'Belly /');self.assertNotEqual(belly['base_color'],self.find(s,'Body /')['base_color']);self.assertGreater(max(belly['positions'][1::3])-min(belly['positions'][1::3]),.15)
  for side in (-1,1):self.assertGreater(max(self.find(s,'Cheek tear /',side)['positions'][1::3])-min(self.find(s,'Cheek tear /',side)['positions'][1::3]),.04)
 def test_curved_horns_limbs_tail_points_and_armor_are_uncrossed(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    if p['part'].startswith(('Horn /','Arm /','Finger /','Toe /','Tail /','Tail prong /','Crown spike /','Side spike /','Tail keel /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_crown_and_tail_negative_controls_have_no_overlap(self):
  for s in self.models:
   for child,parent in ([('Horn /','Head /'),('Tail prong /','Tail /')] if s=='larvitar' else [('Crown spike /','Head /'),('Tail keel /','Body /')]):self.assertFalse(occupied_overlap(translate(copy.deepcopy(self.find(s,child)),3,0,0),self.find(s,parent)))
if __name__=='__main__':unittest.main()
