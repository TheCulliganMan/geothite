#!/usr/bin/env python3
"""Stored witch-crow topology, hat, folded wing, broom and toe regressions."""
import json,unittest
from pathlib import Path
from murkrow_sculpt import sculpture,HEIGHT
from murkrow_glb import read_murkrow,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MurkrowTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_murkrow(ROOT/'murkrow.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'murkrow':self.m})
 def test_normals_follow_stored_triangles(self):assert_model_shading(self,{'murkrow':self.m})
 def test_regeneration_height_budget_and_distinct_features(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),24000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Crest /',3),('Throat /',4),('Wing /',2),('Wing feather /',6),('Toe / three',6),('Toe / short',2),('Talon /',8),('Tail fan /',1),('Binding /',1)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def has_overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:return True
  return False
 def overlap(self,a,b):self.assertTrue(self.has_overlap(a,b),(a['part'],b['part']))
 def test_body_neck_head_brim_and_crest_have_support(self):
  self.overlap(self.ps[0],self.ps[1]);self.overlap(self.ps[1],self.ps[2]);brim=self.find('Hat /');self.overlap(brim,self.ps[2])
  for p in self.ps:
   if p['part'].startswith('Crest /'):self.overlap(p,brim)
   elif p['part'].startswith('Throat /'):self.assertTrue(self.has_overlap(p,self.ps[0]) or self.has_overlap(p,self.ps[1]),p['part'])
 def test_folded_wing_layers_are_attached(self):
  for s in (-1,1):
   wing=self.find('Wing / broad folded crow wing '+str(s));self.overlap(wing,self.ps[0]);supported=[wing]
   for j in range(3):
    feather=self.find(f'Wing feather / layered folded vane {s} {j}');self.assertTrue(any(self.has_overlap(feather,p) for p in supported),feather['part']);supported.append(feather)
 def test_shins_toes_and_white_talons_meet_grounded_feet(self):
  for s in (-1,1):
   shin=self.find('Leg / slender golden crow shin '+str(s));foot=self.find('Foot / grounded golden toe junction '+str(s));self.overlap(shin,self.ps[0]);self.overlap(shin,foot);self.assertEqual(min(foot['positions'][1::3]),0.)
   for j in range(3):
    toe=self.find(f'Toe / three golden forward toes {s} {j}');self.overlap(toe,foot);self.overlap(toe,self.find(f'Talon / white forward toe tip {s} {j}'))
   toe=self.find('Toe / short golden rear toe '+str(s));self.overlap(toe,foot);self.overlap(toe,self.find('Talon / white rear toe tip '+str(s)))
 def test_broom_stem_band_and_jagged_fan_have_physical_support(self):
  stem=self.find('Tail /');self.overlap(stem,self.ps[0]);self.overlap(stem,self.find('Binding /'));self.overlap(stem,self.find('Tail fan /'))
 def test_upper_lower_beak_and_dark_seam_are_rooted(self):
  for p in self.ps:
   if p['part'].startswith('Bill /'):self.overlap(p,self.ps[2])
  self.assertGreater(max(self.find('Bill / broad')['positions'][2::3]),max(self.ps[2]['positions'][2::3])+.09)
 def test_eye_layers_and_nostrils_follow_real_support(self):
  for p in self.ps:
   name=p['part'];support=None;s=name.split()[-1]
   if name.startswith('Eye / fitted sly'):support=self.ps[2]
   elif name.startswith('Eye / fitted ivory'):support=self.find('Eye / fitted sly dark socket '+s)
   elif name.startswith('Eye / red'):support=self.find('Eye / fitted ivory white '+s)
   elif name.startswith('Pupil /'):support=self.find('Eye / red vertical iris '+s)
   elif name.startswith('Nostril /'):support=self.find('Bill / broad')
   if support:
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.006)
if __name__=='__main__':unittest.main()
