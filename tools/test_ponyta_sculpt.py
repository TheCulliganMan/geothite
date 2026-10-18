#!/usr/bin/env python3
"""Stored pony anatomy, supported flowing fire, fitted skin and blue hooves."""
import copy,json,os,unittest
from pathlib import Path
from fire_horse_glb import read_ponyta,geometry_digest
from ponyta_sculpt import sculpture,HEIGHT
from sculpt_geometry import translate
from bovine_sculpt import reverse_front
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_PONYTA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def unzy(p):
 p=dict(p)
 for field in ('positions','normals'):p[field]=[v for i in range(0,len(p[field]),3) for v in (-p[field][i+2],p[field][i+1],p[field][i])]
 return p
class PonytaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=read_ponyta(ROOT/'ponyta.glb')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'ponyta':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'ponyta':self.m})
 def test_recipe_parity_floor_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Eye /',2),('Ear /',2),('Leg /',4),('Hoof /',4),('Flame /',7),('Flame glow /',14)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
  self.assertFalse(any('horn' in p['part'].lower() for p in self.ps))
 def test_head_muzzle_ears_neck_and_tail_have_actual_roots(self):
  for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Muzzle /','Head /'),('Tail stem /','Body /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Ear /',s),self.find('Head /')),s)
 def test_all_legs_and_hooves_have_actual_roots(self):
  for s in (-1,1):
   for row in (0,1):
    leg=self.find('Leg /',f'{s} {row}');self.assertTrue(self.overlap(leg,self.find('Body /')));self.assertTrue(self.overlap(self.find('Hoof /',f'{s} {row}'),leg))
 def test_front_shoulders_root_into_torso_and_legs(self):
  for s in (-1,1):
   shoulder=self.find('Shoulder /',s);self.assertTrue(self.overlap(shoulder,self.find('Body /')));self.assertTrue(self.overlap(shoulder,self.find('Leg /',f'{s} 0')))
 def test_four_dark_hooves_support_the_model(self):
  for p in self.ps:
   if p['part'].startswith('Hoof /'):self.assertEqual(min(p['positions'][1::3]),0);self.assertLess(p['base_color'][0],p['base_color'][2])
 def test_all_seven_fire_banks_have_actual_roots(self):
  for location,parent in (('forelock','Head /'),('mane','Neck /'),('tail','Tail stem /')):self.assertTrue(self.overlap(self.find('Flame / '+location),self.find(parent)),location)
  self.assertTrue(self.overlap(self.find('Flame / mane'),self.find('Body /')))
  for s in (-1,1):
   for row in (0,1):self.assertTrue(self.overlap(self.find('Flame / ankle',f'{s} {row}'),self.find('Leg /',f'{s} {row}')),(s,row))
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Ear inset /','Ear /'),('Eye /','Head /'),('Eye glint /','Eye /'),('Nostril /','Muzzle /'),('Mouth seam /','Muzzle /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Ear /','Eye /') else None)
   if p['part'].startswith('Flame glow /'):
    location=p['part'][len('Flame glow / '):].rsplit(' ',1)[0];parent=next(q for q in self.ps if q['part']=='Flame / '+location);a,b=unzy(p),unzy(parent)
    yield (a,b) if s=='1' else (reverse_front(a),reverse_front(b))
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_legs_ears_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Leg /','Ear /','Tail stem /','Flame /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_fire_and_hooves_are_rejected(self):
  for child,parent in (('Flame / forelock','Head /'),('Flame / ankle','Leg /'),('Hoof /','Leg /'),('Tail stem /','Body /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
