#!/usr/bin/env python3
"""Independent checks of stored Wobbuffet support, fitted expression and tail."""
import json,math,unittest
from pathlib import Path
from wobbuffet_sculpt import sculpture,HEIGHT
from wobbuffet_glb import read_wobbuffet,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class WobbuffetTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_wobbuffet(ROOT/'wobbuffet.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_oriented_connected_geometry(self):assert_closed_models(self,{'wobbuffet':self.m})
 def test_finite_unit_normals_follow_faces(self):assert_model_shading(self,{'wobbuffet':self.m})
 def test_exact_recipe_height_and_budget(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),25000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Arm /',2),('Hand /',2),('Foot /',4),('Eye /',2),('Tail eye /',2),('Tail pupil /',2),('Mouth /',1),('Tongue /',1)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]));found=False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:found=True;break
   if found:break
  self.assertTrue(found,(a['part'],b['part']))
 def test_elastic_arms_and_hands_have_volume_support(self):
  for arm,hand in (('Arm / bent','Hand / rounded salute'),('Arm / raised','Hand / rounded upright')):
   a=self.find(arm);self.overlap(a,self.ps[0]);self.overlap(a,self.find(hand));self.assertGreater(max(a['positions'][1::3]),max(self.ps[0]['positions'][1::3]))
 def test_four_basal_lobes_attach_to_body_and_front_feet_ground(self):
  for p in self.ps:
   if p['part'].startswith('Foot /'):
    self.overlap(p,self.ps[0])
    if p['part'].startswith('Foot / broad'):self.assertAlmostEqual(min(p['positions'][1::3]),0.,places=6)
 def test_black_tail_has_actual_body_and_paddle_overlap(self):
  stem=self.find('Tail /');self.overlap(stem,self.ps[0]);self.overlap(stem,self.find('Tail paddle /'))
 def test_mouth_tongue_and_tail_eye_layers_follow_stored_support(self):
  for prefix,parent,bound in (('Eye /','Body /',.004),('Mouth /','Body /',.006),('Tongue /','Mouth /',.003),('Tail eye /','Tail paddle /',.004),('Tail pupil /','Tail eye /',.002)):
   for p in self.ps:
    if not p['part'].startswith(prefix):continue
    support=self.find(parent)
    if prefix=='Tail pupil /':support=self.find('Tail eye / fitted ivory oval '+p['part'].split()[-1])
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(support,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),bound+.00001)
 def test_both_tail_eyes_clear_body_at_oblique_front_angle(self):
  yaw=-1.0;pitch=.12;c=math.cos(yaw);sn=math.sin(yaw);cp=math.cos(pitch);sp=math.sin(pitch)
  def projected(p):
   q=dict(p);q['positions']=[]
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];q['positions'].extend((c*x-sn*z,-sn*sp*x+cp*y-c*sp*z,sn*cp*x+sp*y+c*cp*z))
   return q
  body=projected(self.ps[0])
  for p in self.ps:
   if not p['part'].startswith('Tail eye /'):continue
   eye=projected(p);points=[tuple(eye['positions'][i:i+3]) for i in range(0,len(eye['positions']),3)]
   x,y=[sum(q[a] for q in points)/len(points) for a in range(2)];eh=ray_intersections(eye,x,y);bh=ray_intersections(body,x,y)
   self.assertTrue(eh,p['part'])
   if bh:self.assertGreater(max(eh)-max(bh),.001,p['part'])
 def test_front_mouth_and_tongue_triangles_do_not_pierce_face(self):
  for p,support in ((self.find('Mouth /'),self.ps[0]),(self.find('Tongue /'),self.find('Mouth /'))):
   for k in range(0,len(p['indices']),3):
    ids=p['indices'][k:k+3]
    if sum(p['normals'][3*i+2] for i in ids)<.001:continue
    x,y,z=[sum(p['positions'][3*i+a] for i in ids)/3 for a in range(3)];hits=ray_intersections(support,x,y)
    self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
if __name__=='__main__':unittest.main()
