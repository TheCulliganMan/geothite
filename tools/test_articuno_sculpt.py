#!/usr/bin/env python3
"""Stored ice-bird roots, feather geometry and continuous streamer regression."""
import copy,json,os,unittest
from pathlib import Path
from articuno_sculpt import sculpture,HEIGHT
from articuno_glb import read_articuno,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import indexed_ray_intersections as ray_intersections
from sculpt_test_support import crossed_faces
from sculpt_test_support import surface_distances
from sculpt_geometry import translate
ROOT=Path(os.environ.get('CRYSTAL_ARTICUNO_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class ArticunoTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_articuno(ROOT/'articuno.glb');cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'articuno':self.m})
 def test_finite_unit_normals_follow_actual_faces(self):assert_model_shading(self,{'articuno':self.m})
 def test_recipe_parity_preserved_height_grounding_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),80000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Crown /',3),('Wing /',2),('Primary /',36),('Covert /',28),('Tail /',1),('Tail fold /',3),('Leg /',2),('Toe / three',6),('Toe / rear',2),('Talon /',8)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_body_neck_streamer_and_feathers_have_no_nonadjacent_crossings(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Neck /','Tail /','Crown /','Primary /','Covert /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_neck_head_beaks_crystalline_crown_and_tail_have_actual_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');self.assertTrue(self.overlap(body,neck));self.assertTrue(self.overlap(neck,head))
  for p in self.ps:
   if p['part'].startswith(('Crown /','Bill /')):self.assertTrue(self.overlap(p,head),p['part'])
  self.assertTrue(self.overlap(body,self.find('Tail /')))
 def test_both_sides_of_every_ice_wing_layer_are_rooted(self):
  for sign in (-1,1):
   wing=self.find('Wing /',sign);self.assertTrue(self.overlap(wing,self.find('Body /')),sign)
   for p in self.ps:
    if p['part'].startswith(('Primary /','Covert /')) and p['part'].split()[-3]==str(sign):self.assertTrue(self.overlap(p,wing),p['part'])
 def test_feet_digits_and_talons_are_supported(self):
  for s in (-1,1):
   leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(leg,self.find('Body /')));self.assertTrue(self.overlap(leg,foot))
   for j in range(3):
    toe=self.find('Toe / three',f'{s} {j}');self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / forward',f'{s} {j}')))
   toe=self.find('Toe / rear',s);self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / rear',s)))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   if n.startswith('Breast /'):parent=self.find('Body /')
   elif n.startswith('Throat /'):parent=self.find('Neck /')
   elif n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',s)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',s)
   elif n.startswith('Glint /'):parent=self.find('Pupil /',s)
   if parent:yield p,parent
 def test_fitted_eyes_and_pale_bib_have_actual_stored_support(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.018,p['part'])
 def test_fitted_front_centers_and_edges_clear_their_parent(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
     hits=ray_intersections(parent,q[0],q[1]);self.assertTrue(hits);self.assertGreater(q[2]-max(hits),.0001,(p['part'],k//3))
 def test_streamer_channels_follow_the_actual_curved_ribbon(self):
  parent=self.find('Tail /')
  for p in self.ps:
   if p['part'].startswith('Tail fold /'):
    points=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
    self.assertLess(max(surface_distances(parent,points,.006)),.006,p['part'])
 def test_detached_wing_and_tail_channel_negative_controls_fail(self):
  self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Wing /',1)),0,0,1),self.find('Body /')))
  p=translate(copy.deepcopy(self.find('Tail fold /')),0,1,0);self.assertGreater(max(surface_distances(self.find('Tail /'),[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)],.006)),.006)

if __name__=='__main__':unittest.main()
