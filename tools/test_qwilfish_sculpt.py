#!/usr/bin/env python3
"""Stored puffer body seam, radial spine roots and fitted spotted tail checks."""
import json,unittest
from pathlib import Path
from qwilfish_sculpt import sculpture,HEIGHT
from qwilfish_glb import read_qwilfish,geometry_digest
from bovine_sculpt import reverse_front
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class QwilfishTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_qwilfish(ROOT/'qwilfish.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_parts(self):assert_closed_models(self,{'qwilfish':self.m})
 def test_finite_unit_normals_follow_stored_winding(self):assert_model_shading(self,{'qwilfish':self.m})
 def test_exact_recipe_height_budget_and_features(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),18000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Body /',2),('Spine / rooted teal',8),('Spine / rooted cream',6),('Eye / fitted',2),('Fin /',2),('Fin crease /',2),('Tail spot /',4)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def has_overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:return True
  return False
 def test_two_colors_share_complete_equator_and_smooth_side_normals(self):
  points=lambda p:{tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}
  shared=points(self.ps[0])&points(self.ps[1]);self.assertEqual(len(shared),48);self.assertEqual(len({v[1] for v in shared}),1)
  normal_sets=[]
  for p in self.ps[:2]:
   ns={}
   for i in range(0,len(p['positions']),3):
    xyz=tuple(p['positions'][i:i+3]);n=tuple(p['normals'][i:i+3])
    if xyz in shared and abs(n[1])<.05:ns.setdefault(xyz,set()).add(n)
   normal_sets.append(ns)
  self.assertEqual(normal_sets[0],normal_sets[1]);self.assertEqual(len(normal_sets[0]),48)
 def test_every_three_dimensional_spine_overlaps_correct_hemisphere(self):
  spines=[p for p in self.ps if p['part'].startswith('Spine /')]
  for p in spines:self.assertTrue(self.has_overlap(p,self.ps[0] if 'cream' in p['part'] else self.ps[1]),p['part'])
  zs=[(min(p['positions'][2::3])+max(p['positions'][2::3]))/2 for p in spines]
  self.assertGreater(max(zs)-min(zs),.5)
 def test_lips_and_tail_peduncle_have_actual_volume_support(self):
  lips=self.find('Lips /');self.assertTrue(self.has_overlap(lips,self.ps[0]));self.assertTrue(self.has_overlap(lips,self.ps[1]))
  tail=self.find('Tail /');self.assertTrue(self.has_overlap(tail,self.ps[0]));self.assertTrue(self.has_overlap(tail,self.find('Tail fin / rounded')))
 def test_expression_fins_and_spots_follow_actual_support(self):
  for p in self.ps:
   name=p['part'];s=name.split()[-1];support=None;bound=.008
   if name.startswith('Eye / fitted'):support=self.ps[1]
   elif name.startswith('Eye / ivory'):support=self.find('Eye / fitted pointed dark socket '+s)
   elif name.startswith('Pupil /'):support=self.find('Eye / ivory triangular sclera '+s)
   elif name.startswith('Fin /'):support=self.ps[0];bound=.016
   elif name.startswith('Fin crease /'):support=self.find('Fin / small raised cream pectoral '+s)
   elif name.startswith('Mouth /'):support=self.find('Lips /')
   elif name.startswith('Tail fin / fitted'):support=self.find('Tail fin / rounded')
   elif name.startswith('Tail spot / fitted rear'):support=self.find('Tail fin / fitted cream reverse')
   elif name.startswith('Tail spot /'):support=self.find('Tail fin / fitted cream paddle')
   if support:
    if name.startswith(('Tail fin / fitted cream reverse','Tail spot / fitted rear')):p=reverse_front(p);support=reverse_front(support)
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),bound)
 def test_front_almond_and_tail_center_triangles_do_not_pierce_support(self):
  for p,support in ((self.find('Eye / fitted pointed dark socket -1'),self.ps[1]),(self.find('Eye / fitted pointed dark socket 1'),self.ps[1]),(self.find('Tail fin / fitted'),self.find('Tail fin / rounded')),(reverse_front(self.find('Tail fin / fitted cream reverse')),reverse_front(self.find('Tail fin / rounded')))):
   for k in range(0,len(p['indices']),3):
    points=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]];a,b,c=points
    nz=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    if nz<1e-10:continue
    x,y,z=[sum(v[axis] for v in points)/3 for axis in range(3)];hits=ray_intersections(support,x,y)
    self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
if __name__=='__main__':unittest.main()
