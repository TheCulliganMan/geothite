#!/usr/bin/env python3
"""Stored egg containment, open pouch, physical roots and fitted face checks."""
import copy,json,os,unittest
from pathlib import Path
from chansey_sculpt import anatomy,sculpture,HEIGHT,PINK,POUCH,EGG
from sculpt_geometry import translate,linear,ellipsoid
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_slime_sculpt import ray_intersections
from test_steelix_sculpt import occupied
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_CHANSEY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class ChanseyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.model=json.loads((ROOT/'chansey.mesh.json').read_bytes())
  raw=anatomy();cls.floor=min(v for p in raw for v in p['positions'][1::3]);cls.scale=HEIGHT/(max(v for p in raw for v in p['positions'][1::3])-cls.floor)
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def point(self,x,y,z):return (x*self.scale,(y-self.floor)*self.scale,z*self.scale)
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'chansey':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'chansey':self.m})
 def test_recipe_palette_height_floor_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),80000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Ribbon /',6),('Foot /',2),('Hand /',2),('Eye /',2),('Pouch /',1),('Egg /',1),('Tail /',1)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
  for label,color in (('Body /',PINK),('Pouch /',POUCH),('Egg /',EGG)):self.assertEqual(self.find(label)['base_color'],linear(color))
 def test_pouch_egg_tail_and_ribbons_have_actual_roots(self):
  for child,parent in (('Pouch /','Body /'),('Egg /','Pouch /'),('Tail /','Body /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
  for p in self.ps:
   if p['part'].startswith('Ribbon /'):self.assertTrue(self.overlap(p,self.find('Body /')),p['part'])
 def test_limbs_and_both_egg_holding_hands_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /'),('Hand /','Egg /'),('Ankle /','Body /'),('Foot /','Ankle /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent not in ('Body /','Egg /') else None)),(s,child,parent))
   self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   for child,parent in (('Eye /','Body /'),('Eye glint /','Eye /'),('Smile /','Body /'),('Chin /','Body /')):
    if p['part'].startswith(child):yield p,self.find(parent,p['part'].split()[-1] if parent=='Eye /' else None)
 def test_fitted_vertices_follow_actual_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_faces_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_ribbons_arms_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Ribbon /','Arm /','Tail /')):self.assertIsNone(crossed_faces(p),p['part'])
 def contained(self,egg):
  pouch=self.find('Pouch /');rim=self.point(0,.510,0)[1]
  for i in range(0,len(egg['positions']),3):
   x,y,z=egg['positions'][i:i+3]
   if y>=rim:continue
   hits=ray_intersections(pouch,x,y)
   if not hits or z<min(hits)-1e-6 or z>max(hits)+1e-6:return False
  return True
 def test_egg_stays_inside_outer_pouch_below_rim(self):self.assertTrue(self.contained(self.find('Egg /')))
 def opening(self):return [self.point(s*x,y,.355) for s in (-1,1) for x in (.123,.126) for y in (.490,.498)]
 def test_pouch_has_actual_empty_space_beside_egg(self):
  for point in self.opening():self.assertFalse(any(occupied(p,point) for p in self.ps),point)
 def test_protruding_egg_and_filled_pouch_are_rejected(self):
  self.assertFalse(self.contained(translate(copy.deepcopy(self.find('Egg /')),0,-.06*self.scale,0)))
  fill=ellipsoid('Invalid filled pouch',POUCH,self.point(0,.494,.355),tuple(r*self.scale for r in (.14,.025,.11)),32,24)
  for point in self.opening():self.assertTrue(occupied(fill,point),point)
 def test_detached_egg_hands_ribbons_and_tail_are_rejected(self):
  for child,parent in (('Egg /','Pouch /'),('Hand /','Egg /'),('Ribbon /','Body /'),('Tail /','Body /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
