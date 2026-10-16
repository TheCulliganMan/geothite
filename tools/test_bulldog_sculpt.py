#!/usr/bin/env python3
"""Stored bulldog contact, winding, grounding and fitted-feature regressions."""
import json,unittest
from pathlib import Path
from bulldog_sculpt import sculpture,SPECIES,HEIGHT
from bulldog_glb import read_bulldog,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BulldogTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_bulldog(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,ps,prefix):return next(p for p in ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
 def test_normals_follow_stored_triangles(self):assert_model_shading(self,self.models)
 def test_regeneration_height_budget_and_distinct_features(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),26000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
   for prefix,n in (('Ear / broad',2),('Leg /',2),('Arm /',2),('Foot /',2),('Pupil /',2)):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n)
   for prefix,n in ((('Spot /',4),('Ear marking /',2),('Jowl /',2)) if s=='snubbull' else (('Tusk /',2),('Wrist /',2))):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0004:return
  self.fail((a['part'],b['part']))
 def test_body_head_legs_arms_tail_and_grounded_paws(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.overlap(ps[0],ps[1]);self.overlap(ps[1],ps[2]);self.overlap(self.find(ps,'Tail /'),ps[0])
   for side in (-1,1):
    leg=self.find(ps,'Leg / bent muscular supporting shin '+str(side));foot=self.find(ps,'Foot / broad grounded bulldog paw '+str(side));self.overlap(leg,ps[0]);self.overlap(leg,foot);self.assertEqual(min(foot['positions'][1::3]),0.)
    arm=self.find(ps,'Arm / curved reaching bulldog forelimb '+str(side));self.overlap(arm,ps[0]);self.overlap(arm,self.find(ps,'Hand / broad rounded bulldog fist '+str(side)));self.overlap(self.find(ps,'Ear / broad folded bulldog flap '+str(side)),ps[2])
    if s=='granbull':self.overlap(arm,self.find(ps,'Wrist / dark snug cuff '+str(side)))
 def test_jaws_muzzles_noses_and_canines_have_real_support(self):
  for s,m in self.models.items():
   ps=m['primitives'];head=ps[2];jaw=self.find(ps,'Jaw /');self.overlap(jaw,head)
   if s=='snubbull':
    for side in (-1,1):self.overlap(self.find(ps,'Jowl / broad rounded pink cheek '+str(side)),head);self.overlap(self.find(ps,'Fang / small downward ivory canine '+str(side)),self.find(ps,'Jowl / broad rounded pink cheek '+str(side)))
    self.overlap(self.find(ps,'Nose /'),head)
   else:
    muzzle=self.find(ps,'Muzzle /');self.overlap(muzzle,head);self.overlap(self.find(ps,'Nose /'),muzzle)
    for side in (-1,1):self.overlap(self.find(ps,'Tusk / enormous upward ivory lower canine '+str(side)),jaw);self.overlap(self.find(ps,'Fang / small downward upper canine '+str(side)),muzzle)
 def test_fitted_spots_eyes_and_ear_markings_follow_support(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part'];support=None;side=name.split()[-1]
    if name.startswith('Eye / fitted'):support=ps[2]
    elif name.startswith('Eye / tilted'):support=self.find(ps,'Eye / fitted stern dark socket '+side)
    elif name.startswith('Pupil /'):support=self.find(ps,'Eye / tilted ivory white '+side)
    elif name.startswith('Ear marking /'):support=self.find(ps,'Ear / broad folded bulldog flap '+side)
    elif name.startswith('Spot /') or name.startswith('Collar / fitted'):support=ps[0]
    elif name.startswith('Mouth / broad') or name.startswith('Mouth / fitted dark'):support=ps[2]
    elif name.startswith('Mouth / fitted rosy'):support=self.find(ps,'Mouth / broad')
    if support:
     for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
      hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.014,(name,z,max(hits)))
 def test_granbull_tusks_curve_beyond_upper_muzzle(self):
  ps=self.models['granbull']['primitives'];muzzle=self.find(ps,'Muzzle /');nose=self.find(ps,'Nose /')
  for side in (-1,1):
   tusk=self.find(ps,'Tusk / enormous upward ivory lower canine '+str(side));self.assertGreater(max(tusk['positions'][1::3]),max(muzzle['positions'][1::3])+.04);self.assertGreater(max(tusk['positions'][2::3]),min(muzzle['positions'][2::3])+.05)
if __name__=='__main__':unittest.main()
