#!/usr/bin/env python3
"""Royal-biped stored roots, shell surfaces, flared collar and fitted gaze."""
import copy,json,os,unittest
from pathlib import Path
from slowking_sculpt import sculpture,HEIGHT
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_steelix_sculpt import occupied
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_SLOWKING_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class SlowkingTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from slow_pair_glb import read_slowking
  cls.model=read_slowking(ROOT/'slowking.glb')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'slowking':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'slowking':self.m})
 def test_recipe_parity_floor_height_budget_and_anatomy(self):
  from slow_pair_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Collar panel /',10),('Crown horn /',2),('Crown crest /',5),('Crown turn /',2),('Eye /',2),('Hand claw /',6),('Foot claw /',4),('Tooth /',2),('Belly band /',3),('Jewel /',1)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_face_teeth_tail_and_collar_have_actual_roots(self):
  body=self.find('Body /');head=self.find('Head /');muzzle=self.find('Muzzle /');self.assertTrue(self.overlap(head,body));self.assertTrue(self.overlap(muzzle,head));self.assertTrue(self.overlap(self.find('Tail /'),body))
  for p in self.ps:
   if p['part'].startswith('Collar panel /'):self.assertTrue(self.overlap(p,body),p['part'])
   if p['part'].startswith('Tooth /'):self.assertTrue(self.overlap(p,muzzle),p['part'])
 def test_crown_brim_horns_crest_and_turns_have_actual_roots(self):
  crown=self.find('Crown body /');brim=self.find('Crown brim /');head=self.find('Head /')
  self.assertTrue(self.overlap(crown,head));self.assertTrue(self.overlap(brim,crown));self.assertTrue(self.overlap(brim,head))
  for p in self.ps:
   if p['part'].startswith(('Crown horn /','Crown crest /','Crown turn /')):self.assertTrue(self.overlap(p,crown),p['part'])
 def test_limbs_palms_and_claws_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Thigh /','Body /'),('Leg /','Thigh /'),('Foot /','Leg /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,None if parent=='Body /' else s)),(child,s))
   for label,parent,n in (('Hand claw /','Arm /',3),('Foot claw /','Foot /',2)):
    for j in range(n):self.assertTrue(self.overlap(self.find(label,f'{s} {j}'),self.find(parent,s)),(label,s,j))
 def test_both_feet_are_grounded(self):
  for s in (-1,1):self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Belly band /','Body /'),('Eye /','Head /'),('Pupil /','Eye /'),('Nostril /','Muzzle /'),('Mouth seam /','Muzzle /'),('Jewel rim /','Crown body /'),('Jewel /','Jewel rim /'),('Jewel glint /','Jewel /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent=='Eye /' else None)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_crown_turn_faces_wrap_actual_shell(self):
  shell=self.find('Crown body /')
  for p in self.ps:
   if not p['part'].startswith('Crown turn /'):continue
   count=len(p['positions'])//(3*2*40);vertices=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
   for offset in range(0,(count-1)*40*12,12):
    for layer,inside in ((0,False),(6,True)):
     for t in (0,3):
      tri=[vertices[k] for k in p['indices'][offset+layer+t:offset+layer+t+3]];point=tuple(sum(v[k] for v in tri)/3 for k in range(3));self.assertEqual(occupied(shell,point),inside,(p['part'],offset,layer,t))
 def test_curved_limbs_tail_horns_and_crest_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Leg /','Hand claw /','Foot claw /','Tail /','Crown horn /','Crown crest /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_collar_horn_tail_and_claw_negatives(self):
  for child,parent in (('Collar panel /','Body /'),('Crown horn /','Crown body /'),('Tail /','Body /'),('Hand claw /','Arm /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
