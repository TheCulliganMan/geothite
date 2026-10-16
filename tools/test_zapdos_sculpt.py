#!/usr/bin/env python3
"""Stored thunder-bird anatomy, roots and independent surface-fit regression."""
import copy,json,os,unittest
from pathlib import Path
from zapdos_sculpt import sculpture,HEIGHT
from zapdos_glb import read_zapdos,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import indexed_ray_intersections as ray_intersections
from sculpt_test_support import crossed_faces
from sculpt_geometry import translate
ROOT=Path(os.environ.get('CRYSTAL_ZAPDOS_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class ZapdosTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_zapdos(ROOT/'zapdos.glb');cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'zapdos':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'zapdos':self.m})
 def test_recipe_parity_height_grounding_budget_and_feature_counts(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),80000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Crown /',3),('Wing /',2),('Underfeather /',6),('Covert /',2),('Cheek /',2),('Breast feather /',2),('Tail /',2),('Thigh /',2),('Leg /',2),('Toe / three',6),('Toe / rear',2),('Talon /',8)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_body_neck_crown_wings_and_tail_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Neck /','Crown /','Wing /','Underfeather /','Tail /','Cheek /','Breast feather /','Thigh /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_head_crown_cheeks_bill_and_tail_have_actual_volume_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');self.assertTrue(self.overlap(body,neck));self.assertTrue(self.overlap(neck,head))
  for p in self.ps:
   if p['part'].startswith(('Crown /','Cheek /','Bill /','Bill seam /')):self.assertTrue(self.overlap(p,head),p['part'])
   elif p['part'].startswith(('Tail /','Breast feather /')):self.assertTrue(self.overlap(p,body),p['part'])
 def test_all_six_black_wing_blades_have_actual_gold_wing_roots(self):
  for s in (-1,1):
   wing=self.find('Wing /',s);self.assertTrue(self.overlap(wing,self.find('Body /')),s)
   for j in range(3):self.assertTrue(self.overlap(self.find('Underfeather /',f'{s} {j}'),wing),(s,j))
 def test_feet_digits_and_claws_are_supported(self):
  for s in (-1,1):
   leg=self.find('Leg /',s);foot=self.find('Foot /',s);thigh=self.find('Thigh /',s);self.assertTrue(self.overlap(thigh,self.find('Body /')));self.assertTrue(self.overlap(leg,thigh));self.assertTrue(self.overlap(leg,foot))
   for j in range(3):
    toe=self.find('Toe / three',f'{s} {j}');self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / forward',f'{s} {j}')))
   toe=self.find('Toe / rear',s);self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / rear',s)))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   if n.startswith('Throat /'):parent=self.find('Neck /')
   elif n.startswith('Covert /'):parent=self.find('Wing /',s)
   elif n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',s)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',s)
   if parent:yield p,parent
 def test_fitted_eyes_throat_and_shoulders_have_stored_skin_support(self):
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
 def test_broad_gold_and_black_lightning_silhouettes_remain_distinct(self):
  for s in (-1,1):
   wing=self.find('Wing /',s);root=self.find('Body /');self.assertGreater(max(abs(x) for x in wing['positions'][::3]),max(abs(x) for x in root['positions'][::3])*6)
   black=self.find('Underfeather /',f'{s} 0');self.assertGreater(max(black['positions'][1::3]),1.0)
  bill=self.find('Bill / long tapered');self.assertGreater(max(bill['positions'][2::3])-max(self.find('Head /')['positions'][2::3]),.30)
 def test_detached_wing_and_tail_negative_controls_fail(self):
  self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Wing /',1)),0,0,1),self.find('Body /')))
  self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Tail / gold')),0,0,-1),self.find('Body /')))

if __name__=='__main__':unittest.main()
