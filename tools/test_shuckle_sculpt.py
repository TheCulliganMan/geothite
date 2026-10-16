#!/usr/bin/env python3
"""Stored shell perforations, soft-body roots, collars and surface markings."""
import json,math,unittest
from pathlib import Path
from shuckle_sculpt import sculpture,HEIGHT,HOLES,project,FACES,CENTER
from shuckle_glb import read_shuckle,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ShuckleTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=read_shuckle(ROOT/'shuckle.glb');cls.ps=cls.model['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'shuckle':self.model})
 def test_normals_match_stored_triangles(self):assert_model_shading(self,{'shuckle':self.model})
 def test_regeneration_height_and_anatomy(self):
  self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),21000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Shell /',1),('Rim /',6),('Limb /',4),('Foot /',4),('Neck /',1),('Head /',1),('Eye /',2),('Spot /',6)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]));good=False
  for i in range(1,16):
   for j in range(1,20):
    x=xl+(xh-xl)*i/16;y=yl+(yh-yl)*j/20;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0005:good=True;break
   if good:break
  self.assertTrue(good,(a['part'],b['part']))
 def test_real_limb_roots_and_grounded_soft_feet(self):
  core=self.ps[1]
  for p in self.ps:
   if p['part'].startswith('Limb /'):
    self.overlap(p,core);label=p['part'].removeprefix('Limb / smooth flexible yellow ');foot=self.find('Foot / rounded grounded yellow '+label);self.overlap(p,foot)
    self.assertAlmostEqual(min(foot['positions'][1::3]),0.,places=6)
  neck=self.find('Neck /');self.overlap(neck,core);self.overlap(neck,self.find('Head /'))
 def frame(self,face,sx,sy):
  pt=project(face,sx*.5,sy*.5);d=tuple(pt[i]-CENTER[i] for i in range(3));length=math.sqrt(sum(v*v for v in d));d=tuple(v/length for v in d)
  u=FACES[face][1];du=sum(a*b for a,b in zip(d,u));x=tuple(u[i]-d[i]*du for i in range(3));length=math.sqrt(sum(v*v for v in x));x=tuple(v/length for v in x)
  y=(d[1]*x[2]-d[2]*x[1],d[2]*x[0]-d[0]*x[2],d[0]*x[1]-d[1]*x[0]);return x,y,d
 def rotated(self,p,frame):
  core=self.ps[1];center=(0,(max(core['positions'][1::3])+min(core['positions'][1::3]))/2,0)
  q=dict(p);q['positions']=[sum((p['positions'][i+k]-center[k])*axis[k] for k in range(3)) for i in range(0,len(p['positions']),3) for axis in frame];return q
 def test_all_six_shell_apertures_and_annular_rims_are_unfilled(self):
  for (face,sx,sy),label in HOLES.items():
   frame=self.frame(face,sx,sy);q=self.rotated(self.ps[0],frame);hits=ray_intersections(q,0,0)
   self.assertTrue(hits,label);self.assertLess(max(hits),0.,label)
   rim=self.find('Rim / curved cream annular collar '+label);rr=self.rotated(rim,frame);self.assertFalse(ray_intersections(rr,0,0),label)
 def test_cream_collars_are_supported_by_actual_shell(self):
  for p in self.ps:
   if p['part'].startswith('Rim /'):self.overlap(p,self.ps[0])
 def test_front_and_rear_spots_follow_stored_outer_skin(self):
  for p in self.ps:
   if not p['part'].startswith('Spot /'):continue
   rear='rear' in p['part'];skin=dict(self.ps[0])
   if rear:skin['positions']=[-v if i%3!=1 else v for i,v in enumerate(skin['positions'])]
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3]
    if rear:x=-x;z=-z
    hits=ray_intersections(skin,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.006)
 def test_eyes_and_mouth_fit_the_head(self):
  head=self.find('Head /')
  for p in self.ps:
   if not p['part'].startswith(('Eye /','Mouth /')):continue
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(head,x,y);self.assertTrue(hits);self.assertLess(abs(z-max(hits)),.005)
if __name__=='__main__':unittest.main()
