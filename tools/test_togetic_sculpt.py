#!/usr/bin/env python3
"""Open torso markings, fitted facial details and physically rooted swept wings."""
import json,math,unittest
from pathlib import Path
from togetic_sculpt import sculpture,HEIGHT
from egg_fairy_glb import read_togetic,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class TogeticTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_togetic(ROOT/'togetic.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'togetic':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'togetic':self.m})
 def test_recipe_parity_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Body ring /',6),('Body fleck /',1),('Head /',1),('Wing feather /',6),('Arm /',2),('Leg /',2),('Foot /',2)):
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
   if n.startswith(('Body ring /','Body fleck /')):
    parent=self.find('Body /')
    if 'rear' in n:mark=reverse_front(p);parent=reverse_front(parent)
   elif n.startswith(('Eye /','Brow /','Mouth /')):parent=self.find('Head /')
   elif n.startswith('Eye glint /'):parent=self.find('Eye / small dark vertical happiness oval '+sign)
   elif n.startswith('Tongue /'):parent=self.find('Mouth /')
   elif n.startswith('Toe crease /'):parent=self.find('Foot / rounded grounded ivory leaf pad '+sign)
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
 def test_all_six_color_rings_have_actual_open_centers(self):
  for p in self.ps:
   if not p['part'].startswith('Body ring /'):continue
   points={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)};cx=sum(x for x,y,z in points)/len(points);cy=sum(y for x,y,z in points)/len(points)
   for dx,dy in ((0,0),(.002,0),(-.002,0),(0,.002),(0,-.002)):self.assertFalse(ray_intersections(p,cx+dx,cy+dy),p['part'])
   x,y,z=max(points,key=lambda v:v[1]);self.assertTrue(ray_intersections(p,cx+(x-cx)*.8,cy+(y-cy)*.8),p['part'])
 def test_head_arms_and_wing_feathers_join_actual_torso(self):
  body=self.find('Body /');self.assertTrue(self.overlap(body,self.find('Head /')))
  for p in self.ps:
   if p['part'].startswith(('Arm /','Wing feather /')):self.assertTrue(self.overlap(body,p),p['part'])
 def test_legs_and_pointed_feet_have_physical_support(self):
  body=self.find('Body /')
  for sign in (-1,1):
   leg=self.find('Leg / short supporting ivory root '+str(sign));foot=self.find('Foot / rounded grounded ivory leaf pad '+str(sign));toe=self.find('Foot point / long tapered ivory leaf toe '+str(sign))
   self.assertTrue(self.overlap(leg,body));self.assertTrue(self.overlap(leg,foot));self.assertTrue(self.overlap(foot,toe));self.assertEqual(min(foot['positions'][1::3]),0.)
 def test_wing_seams_follow_actual_feather_surface(self):
  for p in self.ps:
   if not p['part'].startswith('Feather crease /'):continue
   suffix=' '.join(p['part'].split()[-2:]);parent=self.find('Wing feather / swept flattened ivory plume '+suffix)
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.012)
if __name__=='__main__':unittest.main()
