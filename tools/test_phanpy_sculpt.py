#!/usr/bin/env python3
"""Fitted elephant markings, a rooted upturned trunk and planted/lifted supports."""
import json,math,unittest
from pathlib import Path
from elephant_glb import read_phanpy,geometry_digest
from phanpy_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class PhanpyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_phanpy(ROOT/'phanpy.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'phanpy':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'phanpy':self.m})
 def test_recipe_parity_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),50000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Head /',1),('Trunk /',1),('Ear /',2),('Ear tip /',4),('Leg /',4),('Foot /',4),('Toenail /',4),('Nostril /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=sorted(ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23));bb=sorted(ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23))
    for al,ah in zip(aa[::2],aa[1::2]):
     for bl,bh in zip(bb[::2],bb[1::2]):
      if min(ah,bh)-max(al,bl)>.00025:return True
  return False
 def fitted_pairs(self):
  for p in self.ps:
   n=p['part'];parent=None;mark=p;sign=n.split()[-1]
   if n.startswith(('Forehead /','Eye /','Mouth /')):parent=self.find('Head /')
   elif n.startswith('Eye glint /'):parent=self.find('Eye / fitted vertical dark baby eye '+sign)
   elif n.startswith('Ear tip /'):parent=self.find('Ear / rounded drooping blue leaf '+n.split()[-2])
   elif n.startswith('Nostril /'):parent=self.find('Nose /')
   elif n.startswith('Tongue /'):parent=self.find('Mouth /')
   elif n.startswith('Toenail /'):
    suffix=' '.join(n.split()[-2:]);parent=next(q for q in self.ps if q['part'].startswith('Foot /') and q['part'].endswith(' '+suffix))
   if parent:yield mark,parent
 def test_all_ear_face_and_nail_vertices_follow_actual_stored_parents(self):
  for p,parent in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.017)
 def test_front_marking_triangles_clear_actual_support(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_head_trunk_tip_and_ears_have_physical_joins(self):
  body=self.find('Body /');head=self.find('Head /');trunk=self.find('Trunk /');nose=self.find('Nose /')
  self.assertTrue(self.overlap(body,head));self.assertTrue(self.overlap(head,trunk));self.assertTrue(self.overlap(trunk,nose))
  for p in self.ps:
   if p['part'].startswith('Ear /'):self.assertTrue(self.overlap(head,p),p['part'])
 def test_four_legs_and_feet_are_supported_including_lifted_foreleg(self):
  body=self.find('Body /');feet=[p for p in self.ps if p['part'].startswith('Foot /')]
  self.assertEqual(sum('grounded' in p['part'] for p in feet),3)
  for foot in feet:
   suffix=' '.join(foot['part'].split()[-2:]);leg=next(p for p in self.ps if p['part'].startswith('Leg /') and p['part'].endswith(' '+suffix))
   self.assertTrue(self.overlap(leg,body),leg['part']);self.assertTrue(self.overlap(leg,foot),foot['part'])
   if 'grounded' in foot['part']:self.assertEqual(min(foot['positions'][1::3]),0.)
   else:self.assertGreater(min(foot['positions'][1::3]),.20)
 def test_tail_is_rooted_and_trunk_tip_curls_above_nose_bend(self):
  self.assertTrue(self.overlap(self.find('Body /'),self.find('Tail /')))
  tip=self.find('Nose /');trunk=self.find('Trunk /');self.assertGreater(sum(tip['positions'][1::3])/len(tip['positions'][1::3]),min(trunk['positions'][1::3])+.08)
if __name__=='__main__':unittest.main()
