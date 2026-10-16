#!/usr/bin/env python3
"""Stored flame-bird roots, fire surfaces and both-sided fitted glow regression."""
import copy,json,os,unittest
from pathlib import Path
from moltres_sculpt import sculpture,HEIGHT
from moltres_glb import read_moltres,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import indexed_ray_intersections as ray_intersections
from sculpt_test_support import crossed_faces
from bovine_sculpt import reverse_front
from sculpt_geometry import translate
ROOT=Path(os.environ.get('CRYSTAL_MOLTRES_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class MoltresTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_moltres(ROOT/'moltres.glb');cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'moltres':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'moltres':self.m})
 def test_recipe_parity_grounding_height_and_budget(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),80000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Wing /',2),('Wing fire /',2),('Wing glow /',4),('Wing heart /',12),('Crown /',1),('Crown glow /',2),('Crown heart /',2),('Tail /',1),('Tail glow /',2),('Tail heart /',2),('Leg /',2),('Toe / three',6),('Toe / rear',2),('Talon /',8)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_anatomy_and_closed_flame_banks_have_no_nonadjacent_crossings(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Neck /','Crown /','Wing /','Wing fire /','Wing glow /','Wing heart /','Tail /','Tail glow /','Tail heart /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_head_neck_beaks_crown_and_tail_have_actual_volume_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');self.assertTrue(self.overlap(body,neck));self.assertTrue(self.overlap(neck,head))
  for p in self.ps:
   if p['part'].startswith(('Crown /','Bill /','Bill seam /')):self.assertTrue(self.overlap(p,head),p['part'])
  self.assertTrue(self.overlap(self.find('Tail /'),body))
 def test_gold_wings_and_both_flame_banks_have_actual_roots(self):
  for s in (-1,1):
   wing=self.find('Wing /',s);self.assertTrue(self.overlap(wing,self.find('Body /')));self.assertTrue(self.overlap(wing,self.find('Wing fire /',s)))
 def test_feet_digits_and_claws_are_supported_and_grounded(self):
  for s in (-1,1):
   leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(leg,self.find('Body /')));self.assertTrue(self.overlap(leg,foot));self.assertLess(min(foot['positions'][1::3]),.025)
   for j in range(3):
    toe=self.find('Toe / three',f'{s} {j}');self.assertTrue(self.overlap(toe,foot));claw=self.find('Talon / forward',f'{s} {j}');self.assertTrue(self.overlap(toe,claw));self.assertLess(min(claw['positions'][1::3]),.008)
   toe=self.find('Toe / rear',s);self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / rear',s)))
 def pairs(self):
  for p in self.ps:
   n=p['part'];words=n.split();last=words[-1];parent=None;rear=False
   if n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',last)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',last)
   elif n.startswith('Wing glow /'):parent=self.find('Wing fire /',words[-2]);rear=last=='-1'
   elif n.startswith('Wing heart /'):parent=self.find('Wing glow /',f'{words[-3]} {last}');rear=last=='-1'
   elif n.startswith('Crown glow /'):parent=self.find('Crown /');rear=last=='-1'
   elif n.startswith('Crown heart /'):parent=self.find('Crown glow /',last);rear=last=='-1'
   elif n.startswith('Tail glow /'):parent=self.find('Tail /');rear=last=='-1'
   elif n.startswith('Tail heart /'):parent=self.find('Tail glow /',last);rear=last=='-1'
   if parent:yield (reverse_front(p),reverse_front(parent)) if rear else (p,parent)
 def test_eyes_and_every_warm_and_hot_flame_have_actual_stored_support(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.025,p['part'])
 def test_fitted_front_centers_and_edges_clear_their_parent(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
     hits=ray_intersections(parent,q[0],q[1]);self.assertTrue(hits,(p['part'],k//3));self.assertGreater(q[2]-max(hits),.0001,(p['part'],k//3))
 def test_flame_wings_and_crown_extend_beyond_gold_anatomy(self):
  for s in (-1,1):
   flame=self.find('Wing fire /',s);arm=self.find('Wing /',s);self.assertGreater(max(flame['positions'][1::3])-max(arm['positions'][1::3]),.10)
  self.assertGreater(max(self.find('Crown /')['positions'][1::3])-max(self.find('Head /')['positions'][1::3]),.20)
 def test_gold_leading_arms_clear_overlapping_flame_colors(self):
  for s in (-1,1):
   arm=self.find('Wing /',s);colors=[q for q in self.ps if q['part'].startswith(('Wing glow /','Wing heart /')) and q['part'].split()[-1]=='1' and q['part'].split()[-2 if q['part'].startswith('Wing glow /') else -3]==str(s)]
   covered=0
   for i in range(24):
    for j in range(22):
     x=s*(.13+i*.04);y=.60+j*.025;h=ray_intersections(arm,x,y)
     if not h:continue
     for q in colors:
      k=ray_intersections(q,x,y)
      if k:covered+=1;self.assertGreater(max(h)-max(k),.001,(q['part'],x,y))
   self.assertGreater(covered,20)
 def test_detached_flame_wing_and_tail_negative_controls_fail(self):
  self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Wing fire /',1)),0,0,1),self.find('Wing /',1)))
  self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Tail /')),0,0,-1),self.find('Body /')))

if __name__=='__main__':unittest.main()
