#!/usr/bin/env python3
"""Stored painter-dog anatomy, fitted markings and real appendage contacts."""
import json,unittest
from pathlib import Path
from smeargle_sculpt import sculpture,HEIGHT,CREAM,BROWN
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
from smeargle_glb import read_smeargle,geometry_digest
from chikorita_sculpt import linear
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SmeargleTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=read_smeargle(ROOT/'smeargle.glb');cls.ps=cls.model['primitives']
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'smeargle':self.model})
 def test_normals_match_stored_triangles(self):assert_model_shading(self,{'smeargle':self.model})
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  good=False
  for i in range(1,14):
   for j in range(1,16):
    x=xl+(xh-xl)*i/14;y=yl+(yh-yl)*j/16;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0006:good=True;break
   if good:break
  self.assertTrue(good,(a['part'],b['part']))
 def test_regeneration_height_and_anatomy_budget(self):
  self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),22000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,count in (('Ear /',2),('Eye / fitted',2),('Foot /',2),('Hand /',2),('Finger /',6),('Toe groove /',4),('Tail /',1),('Brush /',2),('Beret /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,prefix)
  for p in self.ps:
   if p['part'].startswith(('Foot /','Tail /')):self.assertEqual(p['base_color'],linear(CREAM))
 def test_real_paw_shin_and_body_support(self):
  for side in (-1,1):
   foot=self.find('Foot / broad grounded cream three-toe paw '+str(side));leg=self.find('Leg / slim cream canine shin '+str(side))
   self.assertEqual(min(foot['positions'][1::3]),0.);self.overlap(foot,leg);self.overlap(leg,self.ps[0]);self.overlap(self.find('Cuff / brown ankle marking '+str(side)),leg)
 def test_head_ears_beret_and_tongue_support(self):
  self.overlap(self.ps[1],self.ps[0]);self.overlap(self.ps[2],self.ps[1]);self.overlap(self.find('Muzzle /'),self.ps[2]);self.overlap(self.find('Tongue /'),self.ps[2])
  cap=self.find('Beret / continuous');self.overlap(cap,self.ps[2]);self.overlap(self.find('Beret / bent'),cap)
  for p in self.ps:
   if p['part'].startswith('Ear /'):self.overlap(p,self.ps[2])
 def test_tail_is_attached_held_and_supports_brush(self):
  tail=self.find('Tail /');self.overlap(tail,self.ps[0]);self.overlap(tail,self.find('Hand / cream open palm 1'));self.overlap(tail,self.find('Brush / cream'))
  cream=self.find('Brush / cream');green=self.find('Brush / tapered')
  a={tuple(cream['positions'][i:i+3]) for i in range(0,len(cream['positions']),3)};b={tuple(green['positions'][i:i+3]) for i in range(0,len(green['positions']),3)}
  seam=a&b;self.assertEqual(len(seam),36);self.assertGreater(max(v[1] for v in seam)-min(v[1] for v in seam),.03)
  xs=tail['positions'][::3];self.assertLess(min(xs),-.42);self.assertGreater(max(xs),.40)
 def test_banded_arms_share_rings_and_support_fingers(self):
  for side in (-1,1):
   bands=[p for p in self.ps if p['part'].startswith('Arm /') and p['part'].split()[-3]==str(side)]
   self.assertEqual(len(bands),13);self.assertEqual(sum(p['base_color']==linear(BROWN) for p in bands),4)
   self.overlap(bands[0],self.ps[0]);hand=self.find('Hand / cream open palm '+str(side));self.overlap(hand,bands[-1])
   for a,b in zip(bands,bands[1:]):
    va={tuple(a['positions'][i:i+3]) for i in range(0,len(a['positions']),3)};vb={tuple(b['positions'][i:i+3]) for i in range(0,len(b['positions']),3)};self.assertEqual(len(va&vb),14)
   for p in self.ps:
    if p['part'].startswith('Finger /') and p['part'].split()[-2]==str(side):self.overlap(p,hand)
 def test_eyes_nostrils_and_paw_grooves_follow_actual_skin(self):
  for p in self.ps:
   name=p['part'];side=name.split()[-1]
   if name.startswith('Eye / fitted'):skin=self.ps[2];limit=.018
   elif name.startswith('Eye / large'):skin=self.find('Eye / fitted large brown oval socket '+side);limit=.006
   elif name.startswith('Eye / narrow'):skin=self.find('Eye / large ivory oval '+side);limit=.004
   elif name.startswith('Nostril /'):skin=self.find('Muzzle /');limit=.004
   elif name.startswith('Toe groove /'):skin=self.find('Foot / broad grounded cream three-toe paw '+name.split()[-2]);limit=.005
   else:continue
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y);self.assertTrue(hits,name);self.assertLess(abs(z-max(hits)),limit,name)
if __name__=='__main__':unittest.main()
