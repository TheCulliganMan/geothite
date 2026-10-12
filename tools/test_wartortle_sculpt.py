#!/usr/bin/env python3
"""Independent stored turtle roots, topology, fitted eyes and shell seam checks."""
import copy,json,os,unittest
from pathlib import Path
from wartortle_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from bovine_sculpt import reverse_front
from sculpt_geometry import translate
ROOT=Path(os.environ.get('CRYSTAL_WARTORTLE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class WartortleTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'wartortle.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'wartortle':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'wartortle':self.m})
 def test_recipe_parity_height_grounding_budget_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Ear /',2),('Ear lobe /',2),('Ear inset /',2),('Tail /',1),('Tail fold /',4),('Arm /',2),('Hand /',2),('Hand claw /',6),('Haunch /',2),('Leg /',2),('Foot /',2),('Toe /',6),('Fang /',2),('Shell seam /',11)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_shell_belly_head_muzzle_and_tail_have_actual_volume_roots(self):
  body=self.find('Body /');shell=self.find('Shell /');self.assertTrue(self.overlap(body,shell));self.assertTrue(self.overlap(self.find('Belly /'),body));self.assertTrue(self.overlap(self.find('Shell rim /'),shell));self.assertTrue(self.overlap(self.find('Head /'),body));self.assertTrue(self.overlap(self.find('Muzzle /'),self.find('Head /')));self.assertTrue(self.overlap(self.find('Tail /'),shell))
 def test_ivory_belly_clears_underlying_blue_body_in_the_visible_breast(self):
  belly=self.find('Belly /');body=self.find('Body /');count=0
  for j in range(1,12):
   y=.26+( .64-.26)*j/12
   for i in range(-7,8):
    x=i*.018;bh=ray_intersections(belly,x,y);mh=ray_intersections(body,x,y)
    if bh and mh:self.assertGreater(max(bh)-max(mh),.002);count+=1
  self.assertGreater(count,100)
 def test_all_ears_lobes_and_fangs_are_rooted(self):
  for s in (-1,1):
   ear=self.find('Ear /',s);self.assertTrue(self.overlap(ear,self.find('Head /')));self.assertTrue(self.overlap(self.find('Ear lobe /',s),ear));self.assertTrue(self.overlap(self.find('Fang /',s),self.find('Muzzle /')))
 def test_arms_hands_and_all_six_claws_have_actual_roots(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);hand=self.find('Hand /',s);self.assertTrue(self.overlap(arm,self.find('Body /')));self.assertTrue(self.overlap(hand,arm))
   for j in range(3):self.assertTrue(self.overlap(hand,self.find('Hand claw /',f'{s} {j}')))
 def test_haunches_ankles_feet_and_toes_are_rooted_and_grounded(self):
  for s in (-1,1):
   thigh=self.find('Haunch /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(thigh,self.find('Body /')));self.assertTrue(self.overlap(leg,thigh));self.assertTrue(self.overlap(foot,leg));self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):self.assertTrue(self.overlap(self.find('Toe /',f'{s} {j}'),foot))
 def test_cloud_tail_and_feather_ears_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Tail /','Tail fold /','Ear /','Ear lobe /','Arm /','Leg /')):self.assertIsNone(crossed_faces(p),p['part'])
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   if n.startswith('Belly seam /'):parent=self.find('Belly /')
   elif n.startswith('Shell seam /'):yield reverse_front(p),reverse_front(self.find('Shell /'));continue
   elif n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',s)
   elif n.startswith('Iris /'):parent=self.find('Eye /',s)
   elif n.startswith('Pupil /'):parent=self.find('Iris /',s)
   elif n.startswith('Glint /'):parent=self.find('Pupil /',s)
   elif n.startswith(('Mouth /','Nostril /')):parent=self.find('Muzzle /')
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',s)
   elif n.startswith('Tail fold /'):
    parent=self.find('Tail /')
    if s=='-1':yield reverse_front(p),reverse_front(parent);continue
   if parent:yield p,parent
 def test_eyes_shell_seams_ear_and_tail_folds_follow_actual_stored_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.015,p['part'])
 def test_fitted_triangle_centers_and_edges_clear_actual_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['indices']),3):
    vv=[p['positions'][3*j:3*j+3] for j in p['indices'][i:i+3]];a,b,c=vv
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
     x,y,d=[sum(vv[k][axis]*weights[k] for k in range(3)) for axis in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(d-max(hits),.0001,p['part'])
 def test_detached_cloud_tail_and_ear_are_rejected(self):
  self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Tail /')),0,0,-1),self.find('Shell /')));self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Ear /',1)),1,0,0),self.find('Head /')))

if __name__=='__main__':unittest.main()
