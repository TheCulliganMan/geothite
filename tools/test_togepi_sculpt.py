#!/usr/bin/env python3
"""Stored shell cavity, empty color rings, star head and rooted cream limbs."""
import json,math,unittest
from pathlib import Path
from togepi_sculpt import sculpture,HEIGHT
from egg_fairy_glb import read_togepi,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class TogepiTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_togepi(ROOT/'togepi.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'togepi':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'togepi':self.m})
 def test_recipe_parity_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Shell ring /',7),('Shell fleck /',2),('Head /',1),('Arm /',2),('Leg /',2),('Foot /',2)):
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
   if n.startswith(('Shell ring /','Shell fleck /')):
    parent=self.find('Shell /')
    if 'rear' in n or n=='Shell fleck / small solid red triangle 1':mark=reverse_front(p);parent=reverse_front(parent)
   elif n.startswith(('Eye /','Eye crease /','Mouth /')):parent=self.find('Head /')
   elif n.startswith('Eye glint /'):parent=self.find('Eye / small joyful dark vertical oval '+sign)
   elif n.startswith('Tongue /'):parent=self.find('Mouth /')
   elif n.startswith('Foot crease /'):parent=self.find('Foot / grounded rounded cream pad '+sign)
   if parent:yield mark,parent
 def test_all_shell_and_face_vertices_follow_actual_stored_parents(self):
  for p,parent in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.017)
 def test_front_marking_triangles_clear_actual_support(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_shell_has_ten_cracked_rim_teeth_and_a_recessed_interior_floor(self):
  egg=self.find('Shell /');top=max(egg['positions'][1::3]);points={tuple(egg['positions'][i:i+3]) for i in range(0,len(egg['positions']),3)}
  angles=sorted(math.atan2(x,z) for x,y,z in points if top-y<1e-6);groups=[]
  for a in angles:
   if not groups or a-groups[-1][-1]>.1:groups.append([a])
   else:groups[-1].append(a)
  if groups[0][0]+math.tau-groups[-1][-1]<.1:groups[0]=groups[-1]+groups[0];groups.pop()
  self.assertEqual(len(groups),10)
  down=dict(egg);down['positions']=[v for i in range(0,len(egg['positions']),3) for v in (egg['positions'][i],egg['positions'][i+2],egg['positions'][i+1])]
  self.assertGreater(top-max(ray_intersections(down,0,0)),.30)
 def test_all_seven_color_rings_have_actual_open_centers(self):
  for p in self.ps:
   if not p['part'].startswith('Shell ring /'):continue
   points={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)};cx=sum(x for x,y,z in points)/len(points);cy=sum(y for x,y,z in points)/len(points)
   for dx,dy in ((0,0),(.002,0),(-.002,0),(0,.002),(0,-.002)):self.assertFalse(ray_intersections(p,cx+dx,cy+dy),p['part'])
   x,y,z=max(points,key=lambda v:v[1]);self.assertTrue(ray_intersections(p,cx+(x-cx)*.8,cy+(y-cy)*.8),p['part'])
 def test_rounded_star_head_and_both_raised_hands_are_rooted_in_shell(self):
  egg=self.find('Shell /');self.assertTrue(self.overlap(egg,self.find('Head /')))
  for p in self.ps:
   if p['part'].startswith('Arm /'):self.assertTrue(self.overlap(egg,p),p['part'])
 def test_short_legs_support_both_grounded_cream_feet(self):
  egg=self.find('Shell /')
  for sign in (-1,1):
   leg=self.find('Leg / short cream shell support '+str(sign));foot=self.find('Foot / grounded rounded cream pad '+str(sign));self.assertTrue(self.overlap(leg,egg));self.assertTrue(self.overlap(leg,foot));self.assertEqual(min(foot['positions'][1::3]),0.)
if __name__=='__main__':unittest.main()
