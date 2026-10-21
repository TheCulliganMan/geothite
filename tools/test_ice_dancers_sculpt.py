#!/usr/bin/env python3
"""Stored dancer clothing, hair, hands and real recessed kiss-rim checks."""
import copy,json,os,unittest
from pathlib import Path
from smoochum_glb import read_smoochum,geometry_digest
from ice_dancers_sculpt import sculpture,SPECIES,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate
from bovine_sculpt import reverse_front
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_ICE_DANCERS_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class IceDancersTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={}
  for n in SPECIES:
   if n=='smoochum':cls.models[n]=read_smoochum(ROOT/'smoochum.glb')
   elif (ROOT/(n+'.glb')).exists():
    from jynx_glb import read_jynx
    cls.models[n]=read_jynx(ROOT/(n+'.glb'))
   else:cls.models[n]=json.loads((ROOT/(n+'.mesh.json')).read_bytes())
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 def each(self):
  for n,m in self.models.items():self.m=m;self.ps=m['primitives'];yield n
 def test_closed_connected_outward_solids(self):assert_closed_models(self,self.models)
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,self.models)
 def test_parity_floor_height_budget_and_distinct_anatomy(self):
  for n in self.each():
   self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture(n)));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000)
   self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHTS[n],places=6)
   counts=(('Gown /',1),('Sleeve /',2),('Palm /',2),('Finger /',10),('Hair curtain /',2),('Hair part /',2),('Hair tip /',4),('Hair strand /',2),('Back hair strand /',3),('Chest trim /',2)) if n=='jynx' else (('Body /',1),('Arm /',2),('Hand /',2),('Finger /',6),('Hair tuft /',4),('Hair side /',2),('Bangs /',1),('Foot /',2),('Chest /',1),('Chest bib /',1))
   for prefix,num in counts+ (('Head /',1),('Eye /',2),('Lips /',1),('Mouth /',1)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),num,prefix)
 def test_neck_head_hair_crown_and_clothes_have_real_roots(self):
  for n in self.each():
   neck=self.find('Neck /');head=self.find('Head /');back=self.find('Hair back /');cap=self.find('Hair crown /');torso=self.find('Gown /' if n=='jynx' else 'Body /')
   for a,b in ((neck,torso),(head,neck),(back,head),(cap,back)):self.assertTrue(self.overlap(a,b),(n,a['part']))
   if n=='jynx':
    for s in (-1,1):
     for a,b in ((self.find('Hair curtain /',s),cap),(self.find('Chest trim /',s),torso),(self.find('Chest bridge /'),self.find('Chest trim /',s))):self.assertTrue(self.overlap(a,b),a['part'])
     self.assertTrue(self.overlap(self.find('Hair part /',s),cap))
     self.assertTrue(self.overlap(self.find('Hair part /',s),self.find('Hair curtain /',s)))
     for j in range(2):self.assertTrue(self.overlap(self.find('Hair tip /',f'{s} {j}'),self.find('Hair curtain /',s)))
   else:
    self.assertTrue(self.overlap(self.find('Chest /'),torso));self.assertTrue(self.overlap(self.find('Bangs /'),cap))
    for s in (-1,1):self.assertTrue(self.overlap(self.find('Hair side /',s),back))
    for j in range(4):self.assertTrue(self.overlap(self.find('Hair tuft /',j),cap))
 def test_both_arms_hands_and_every_digit_have_real_roots(self):
  for n in self.each():
   for s in (-1,1):
    arm=self.find('Sleeve /' if n=='jynx' else 'Arm /',s);hand=self.find('Palm /' if n=='jynx' else 'Hand /',s);torso=self.find('Gown /' if n=='jynx' else 'Chest /')
    self.assertTrue(self.overlap(arm,torso),arm['part']);self.assertTrue(self.overlap(hand,arm),hand['part'])
    for j in range(5 if n=='jynx' else 3):self.assertTrue(self.overlap(self.find('Finger /',f'{s} {j}'),hand))
 def test_baby_legs_and_toes_are_rooted_and_grounded(self):
  self.m=self.models['smoochum'];self.ps=self.m['primitives']
  for s in (-1,1):
   leg=self.find('Leg /',s);foot=self.find('Foot /',s)
   self.assertTrue(self.overlap(leg,self.find('Body /')));self.assertTrue(self.overlap(foot,leg));self.assertLess(min(foot['positions'][1::3]),.002)
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /'),('Iris /','Pupil /'),('Glint /','Pupil /'),('Bodice /','Chest trim /'),('Hair strand /','Hair curtain /')):
    if n.startswith(child):parent=self.find(base,s if base!='Head /' else None)
   if n.startswith('Mouth /'):parent=self.find('Head /')
   if n.startswith('Chest bib /'):parent=self.find('Body /')
   if n.startswith('Back hair strand /'):
    yield reverse_front(p),reverse_front(self.find('Hair back /'));continue
   if parent:yield p,parent
 def test_fitted_layers_follow_actual_stored_parent(self):
  for n in self.each():attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_parent(self):
  for n in self.each():attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_lips_are_rooted_and_leave_a_real_recessed_open_center(self):
  for n in self.each():
   lip=self.find('Lips /');head=self.find('Head /');mouth=self.find('Mouth /');self.assertTrue(self.overlap(lip,head))
   x=(min(mouth['positions'][::3])+max(mouth['positions'][::3]))/2;y=(min(mouth['positions'][1::3])+max(mouth['positions'][1::3]))/2
   self.assertFalse(ray_intersections(lip,x,y),'lip fills central mouth opening');hits=ray_intersections(mouth,x,y);self.assertTrue(hits);self.assertGreater(max(lip['positions'][2::3])-max(hits),.014)
   filled=copy.deepcopy(lip);filled['positions']=mouth['positions'];filled['indices']=mouth['indices'];self.assertTrue(ray_intersections(filled,x,y),'filled mouth negative must be detected')
 def test_hair_gown_limbs_lips_and_digits_do_not_cross_themselves(self):
  for n in self.each():
   for p in self.ps:
    if p['part'].startswith(('Gown /','Sleeve /','Arm /','Leg /','Finger /','Hair crown /','Hair curtain /','Hair part /','Hair tip /','Hair side /','Hair tuft /','Bangs /','Lips /')):self.assertIsNone(crossed_faces(p),(n,p['part']))
 def test_detached_hair_lips_hands_and_toes_are_rejected(self):
  for n in self.each():
   for child,parent in (('Hair crown /','Hair back /'),('Lips /','Head /'),('Finger /','Palm /' if n=='jynx' else 'Hand /')):
    self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
