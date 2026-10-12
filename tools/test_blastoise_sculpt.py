#!/usr/bin/env python3
"""Stored armored turtle anatomy and independent real cannon-void checks."""
import copy,json,os,unittest
from pathlib import Path
from blastoise_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from bovine_sculpt import reverse_front
from sculpt_geometry import translate,ellipsoid
from chikorita_sculpt import sub,dot,cross,unit,mul,add
ROOT=Path(os.environ.get('CRYSTAL_BLASTOISE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class BlastoiseTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'blastoise.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'blastoise':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'blastoise':self.m})
 def test_recipe_parity_height_floor_budget_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),110000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Cannon /',2),('Cannon floor /',2),('Cannon lining /',2),('Cannon mount /',2),('Ear /',2),('Ear inset /',2),('Tail /',1),('Arm /',2),('Hand /',2),('Hand claw /',6),('Haunch /',2),('Leg /',2),('Foot /',2),('Toe /',6),('Foot claw /',6),('Shell seam /',11)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_body_shell_belly_rim_head_jaw_and_tail_have_volume_roots(self):
  body=self.find('Body /');shell=self.find('Shell /');head=self.find('Head /');self.assertTrue(self.overlap(body,shell));self.assertTrue(self.overlap(self.find('Belly /'),body));self.assertTrue(self.overlap(self.find('Shell rim /'),shell));self.assertTrue(self.overlap(head,body));self.assertTrue(self.overlap(self.find('Muzzle /'),head));self.assertTrue(self.overlap(self.find('Jaw /'),head));self.assertTrue(self.overlap(self.find('Tail /'),shell))
 def test_cannon_mounts_barrels_and_ears_have_real_roots(self):
  for s in (-1,1):
   mount=self.find('Cannon mount /',s);self.assertTrue(self.overlap(mount,self.find('Shell /')));self.assertTrue(self.overlap(mount,self.find('Cannon /',s)));self.assertTrue(self.overlap(self.find('Cannon lining /',s),self.find('Cannon /',s)));self.assertTrue(self.overlap(self.find('Ear /',s),self.find('Head /')))
 def test_all_arm_hand_and_ivory_claw_roots(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);hand=self.find('Hand /',s);self.assertTrue(self.overlap(arm,self.find('Body /')));self.assertTrue(self.overlap(hand,arm))
   for j in range(3):self.assertTrue(self.overlap(hand,self.find('Hand claw /',f'{s} {j}')))
 def test_feet_toes_and_ivory_hooks_are_supported_and_grounded(self):
  for s in (-1,1):
   thigh=self.find('Haunch /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(thigh,self.find('Body /')));self.assertTrue(self.overlap(leg,thigh));self.assertTrue(self.overlap(foot,leg));self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):
    toe=self.find('Toe /',f'{s} {j}');self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Foot claw /',f'{s} {j}')))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   if n.startswith('Belly seam /'):parent=self.find('Belly /')
   elif n.startswith('Shell seam /'):yield reverse_front(p),reverse_front(self.find('Shell /'));continue
   elif n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',s)
   elif n.startswith('Iris /'):parent=self.find('Eye /',s)
   elif n.startswith('Pupil /'):parent=self.find('Iris /',s)
   elif n.startswith(('Mouth /','Nostril /')):parent=self.find('Muzzle /')
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',s)
   if parent:yield p,parent
 def test_all_fitted_eyes_ears_mouth_and_shell_seams_follow_actual_stored_skin(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.016,p['part'])
 def test_fitted_triangle_centers_and_edges_clear_actual_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['indices']),3):
    vv=[p['positions'][3*j:3*j+3] for j in p['indices'][i:i+3]];a,b,c=vv
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
     x,y,d=[sum(vv[k][axis]*weights[k] for k in range(3)) for axis in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(d-max(hits),.0001,p['part'])
 def test_belly_clears_underlying_blue_mantle(self):
  count=0
  for j in range(1,12):
   for i in range(-7,8):
    x=i*.021;y=.25+(.71-.25)*j/12;bh=ray_intersections(self.find('Belly /'),x,y);mh=ray_intersections(self.find('Body /'),x,y)
    if bh and mh:self.assertGreater(max(bh)-max(mh),.002);count+=1
  self.assertGreater(count,100)
 def test_barrels_ears_tail_and_limbs_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Cannon /','Cannon lining /','Ear /','Tail /','Arm /','Leg /')):self.assertIsNone(crossed_faces(p),p['part'])
 def bore_frame(self,s):
  axis=unit((s*.075,.280,.383));side=unit(cross(axis,(0,1,0)));other=cross(axis,side);barrel=self.find('Cannon /',s);points=set(tuple(barrel['positions'][i:i+3]) for i in range(0,len(barrel['positions']),3));front=max(dot(q,axis) for q in points);ring=[q for q in points if front-dot(q,axis)<1e-6];tip=tuple(sum(q[k] for q in ring)/len(ring) for k in range(3));return tip,axis,side,other
 def axial(self,p,s):
  tip,axis,side,other=self.bore_frame(s);q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in (dot(sub(p['positions'][i:i+3],tip),side),dot(sub(p['positions'][i:i+3],tip),other),dot(sub(p['positions'][i:i+3],tip),axis))];return q
 def clear_bore(self,parts,s):
  for p in parts:
   hits=sorted(ray_intersections(self.axial(p,s),0,0))
   for lo,hi in zip(hits[::2],hits[1::2]):
    if min(hi,-.006)-max(lo,-.125)>.0001:return False
  return True
 def test_both_cannon_mouths_have_real_recessed_empty_space(self):
  for s in (-1,1):
   self.assertTrue(self.clear_bore(self.ps,s));barrel=ray_intersections(self.axial(self.find('Cannon /',s),s),0,0);floor=ray_intersections(self.axial(self.find('Cannon floor /',s),s),0,0);self.assertTrue(barrel);self.assertTrue(floor);self.assertLess(max(barrel),-.13);self.assertLess(max(floor),-.13);self.assertLess(min(floor),max(barrel)-.0005)
 def test_filled_barrel_and_detached_cannon_negative_controls(self):
  tip,axis,_,_=self.bore_frame(1);filler=ellipsoid('Invalid cannon filler',(.5,.5,.5),add(tip,mul(axis,-.060)),(.025,.025,.025),24,16);self.assertFalse(self.clear_bore(self.ps+[filler],1));self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Cannon /',1)),1,0,0),self.find('Cannon mount /',1)))

if __name__=='__main__':unittest.main()
