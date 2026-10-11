#!/usr/bin/env python3
"""Stored Aipom anatomy, surface fit, grounding and tail-hand regressions."""
import json,unittest
from pathlib import Path
from aipom_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class AipomTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'aipom.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'aipom':self.m})
 def test_normals_follow_stored_triangles(self):assert_model_shading(self,{'aipom':self.m})
 def test_regeneration_height_budget_and_distinct_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),35000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Tail finger /',3),('Tail fingertip /',3),('Ear / large',2),('Tuft /',3),('Grin / fitted tooth division',4)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0004:return
  self.fail((a['part'],b['part']))
 def test_body_head_and_grounded_limb_contacts(self):
  self.overlap(self.ps[0],self.ps[1]);self.overlap(self.ps[1],self.ps[2])
  for s in (-1,1):
   leg=self.find('Leg / bent supporting monkey shin '+str(s));foot=self.find('Foot / broad grounded cream pad '+str(s));self.overlap(leg,self.ps[0]);self.overlap(leg,foot);self.assertEqual(min(foot['positions'][1::3]),0.)
   arm=self.find('Arm / curved playful reaching arm '+str(s));self.overlap(arm,self.ps[0]);self.overlap(arm,self.find('Hand / rounded purple mitten '+str(s)));self.overlap(self.find('Ear / large upright rounded pinna '+str(s)),self.ps[2])
  for p in self.ps:
   if p['part'].startswith('Tuft /'):self.overlap(p,self.ps[2])
 def test_tail_palm_and_three_radial_fingers_have_real_support(self):
  tail=self.find('Tail /');palm=self.find('Tail hand /');self.overlap(tail,self.ps[0]);self.overlap(tail,palm)
  tips=[]
  for j in range(3):
   digit=self.find('Tail finger / short radiating cream digit '+str(j));tip=self.find('Tail fingertip / soft bulbous cream pad '+str(j));self.overlap(palm,digit);self.overlap(digit,tip);tips.append(((min(tip['positions'][::3])+max(tip['positions'][::3]))/2,(min(tip['positions'][1::3])+max(tip['positions'][1::3]))/2))
  self.assertGreater(max(x for x,y in tips)-min(x for x,y in tips),.20);self.assertGreater(max(y for x,y in tips)-min(y for x,y in tips),.20)
 def test_fitted_masks_eyes_and_ear_centers_follow_real_support(self):
  for p in self.ps:
   name=p['part'];support=None;s=name.split()[-1]
   if name.startswith('Face /'):support=self.ps[2]
   elif name.startswith('Belly /'):support=self.ps[0]
   elif name.startswith('Ear / fitted'):support=self.find('Ear / large upright rounded pinna '+s)
   elif name.startswith('Eye / dark'):support=self.find('Face /')
   elif name.startswith('Eye / large'):support=self.find('Eye / dark fitted round socket '+s)
   elif name.startswith('Eye / blue'):support=self.find('Eye / large ivory white '+s)
   elif name.startswith('Pupil /'):support=self.find('Eye / blue grey iris '+s)
   elif name.startswith('Grin / fitted dark'):support=self.find('Face /')
   elif name.startswith('Grin / single'):support=self.find('Grin / fitted dark')
   if support:
    # Check distinct positions; flat-normal corners may repeat the same point.
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.015,(name,z,max(hits)))
 def test_face_and_grin_front_triangles_clear_underlying_skin(self):
  for patch,support in ((self.find('Face /'),self.ps[2]),(self.find('Grin / single'),self.find('Grin / fitted dark'))):
   v=patch['positions']
   for k in range(0,len(patch['indices']),3):
    ids=patch['indices'][k:k+3]
    if sum(patch['normals'][i*3+2] for i in ids)<=.001:continue
    x,y,z=[sum(v[i*3+j] for i in ids)/3 for j in range(3)];hits=ray_intersections(support,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0003,(patch['part'],x,y,z,max(hits)))
if __name__=='__main__':unittest.main()
