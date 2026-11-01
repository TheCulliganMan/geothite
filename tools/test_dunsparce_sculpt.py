#!/usr/bin/env python3
"""Independent stored anatomy, surface fitting and support checks for Dunsparce."""
import json,math,unittest
from pathlib import Path
from dunsparce_sculpt import sculpture,HEIGHT
from dunsparce_glb import read_dunsparce,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class DunsparceTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=read_dunsparce(ROOT/'dunsparce.glb');cls.ps=cls.model['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'dunsparce':self.model})
 def test_normals_match_stored_triangles(self):assert_model_shading(self,{'dunsparce':self.model})
 def test_regeneration_grounding_and_anatomy_budget(self):
  self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),14000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,count in (('Body /',1),('Belly /',1),('Eye / broad',2),('Eye / pale',2),('Eye / fitted',2),('Jaw /',2),('Wing /',2),('Back band /',2),('Tail /',1),('Drill /',1),('Drill groove /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,prefix)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  good=False
  for i in range(1,16):
   for j in range(1,20):
    x=xl+(xh-xl)*i/16;y=yl+(yh-yl)*j/20;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0006:good=True;break
   if good:break
  self.assertTrue(good,(a['part'],b['part']))
 def test_continuous_shared_blue_belly_boundary(self):
  gold,blue=self.ps[:2]
  a={tuple(gold['positions'][i:i+3]) for i in range(0,len(gold['positions']),3)};b={tuple(blue['positions'][i:i+3]) for i in range(0,len(blue['positions']),3)};seam=a&b
  self.assertEqual(len(seam),58)
  self.assertLess(min(v[2] for v in seam),-.5);self.assertGreater(max(v[2] for v in seam),.6)
  self.assertEqual(min(blue['positions'][1::3]),0.)
 def test_wings_jaw_and_drill_have_actual_support(self):
  for p in self.ps:
   if p['part'].startswith(('Wing /','Jaw /')):self.overlap(p,self.ps[0])
  stem=self.find('Tail /');self.overlap(stem,self.ps[0]);self.overlap(self.find('Drill /'),stem)
 def test_eye_rings_and_closed_creases_fit_stored_skin(self):
  for p in self.ps:
   name=p['part'];side=name.split()[-1]
   if name.startswith('Eye / broad'):skin=self.ps[0];limit=.019
   elif name.startswith('Eye / pale'):skin=self.find('Eye / broad fitted blue oval ring '+side);limit=.011
   elif name.startswith('Eye / fitted'):skin=self.find('Eye / pale closed eyelid '+side);limit=.008
   elif name.startswith('Wing crease /'):skin=self.find('Wing / small curved ivory feather '+side);limit=.005
   else:continue
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y);self.assertTrue(hits,name);self.assertLess(abs(z-max(hits)),limit,name)
 def test_back_ribbons_follow_actual_body_roof(self):
  roof=dict(self.ps[0]);roof['positions']=[v for i in range(0,len(roof['positions']),3) for v in (roof['positions'][i],roof['positions'][i+2],roof['positions'][i+1])]
  for p in self.ps:
   if not p['part'].startswith('Back band /'):continue
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(roof,x,z);self.assertTrue(hits);self.assertLess(abs(y-max(hits)),.01)
 def test_spiral_tracks_drill_and_wraps_three_times(self):
  drill=self.find('Drill /');groove=self.find('Drill groove /')
  # Actual stored triangle intersections along rays from the local cone axis.
  points=[tuple(groove['positions'][i:i+3]) for i in range(0,len(groove['positions']),3)]
  ys=drill['positions'][1::3];lo=min(ys);hi=max(ys)
  for x,y,z in points[::29]:
   if not lo<y<hi:continue
   axis_hits=ray_intersections(drill,0,y);self.assertTrue(axis_hits)
   center=(min(axis_hits)+max(axis_hits))/2;a=math.atan2(x,z-center);c=math.cos(a);sn=math.sin(a)
   rotated=dict(drill);rotated['positions']=[v for i in range(0,len(drill['positions']),3) for v in (drill['positions'][i]*c-(drill['positions'][i+2]-center)*sn,drill['positions'][i+1],drill['positions'][i]*sn+(drill['positions'][i+2]-center)*c)]
   hits=ray_intersections(rotated,0,y);self.assertTrue(hits)
   self.assertLess(abs(math.hypot(x,z-center)-max(hits)),.007)
  # Extent crosses both sides repeatedly at several heights, rather than a ring.
  for low,high in ((.43,.54),(.54,.65),(.65,.78)):
   samples=[x for x,y,z in points if low<y<high];self.assertTrue(samples);self.assertLess(min(samples),-.012);self.assertGreater(max(samples),.012)
if __name__=='__main__':unittest.main()
