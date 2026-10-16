#!/usr/bin/env python3
"""Stored psychic-hoofed anatomy, contact, loop and fitted-marking regressions."""
import unittest
from pathlib import Path
from girafarig_glb import read_girafarig
from stantler_glb import read_stantler,geometry_digest
from hoofed_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class HoofedTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_stantler(ROOT/'stantler.glb') if s=='stantler' else read_girafarig(ROOT/'girafarig.glb') for s in SPECIES}
 def find(self,ps,prefix):return next(p for p in ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
 def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
 def test_regeneration_height_and_distinct_anatomy(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),23000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
   for prefix,n in (('Leg /',4),('Hoof /',4),('Ear / broad',2),('Eye / fitted dark',2)):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,prefix))
   counts=(('Horn /',2),('Horn tip /',2),('Mane /',7),('Tail head /',1),('Tail eye / gold',2)) if s=='girafarig' else (('Antler /',2),('Antler tine /',6),('Antler orb /',2),('Tail / large',1))
   for prefix,n in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,prefix))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]));good=False
  for i in range(1,18):
   for j in range(1,22):
    x=xl+(xh-xl)*i/18;y=yl+(yh-yl)*j/22;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0005:good=True;break
   if good:break
  self.assertTrue(good,(a['part'],b['part']))
 def test_grounded_hooves_and_long_leg_body_contacts(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for side in (-1,1):
    for row in range(2):
     leg=self.find(ps,f'Leg / sculpted supporting shin {side} {row}');hoof=self.find(ps,f'Hoof / grounded dark cloven pad {side} {row}')
     self.overlap(leg,hoof);self.overlap(leg,ps[row] if s=='girafarig' else ps[0]);self.assertEqual(min(hoof['positions'][1::3]),0.)
   neck=self.find(ps,'Neck /');self.overlap(neck,ps[0]);self.overlap(neck,self.find(ps,'Head /'));self.overlap(self.find(ps,'Muzzle /'),self.find(ps,'Head /'))
 def test_girafarig_body_color_seam_mane_and_second_head(self):
  ps=self.models['girafarig']['primitives'];front,rear=ps[:2]
  a={tuple(front['positions'][i:i+3]) for i in range(0,len(front['positions']),3)};b={tuple(rear['positions'][i:i+3]) for i in range(0,len(rear['positions']),3)};self.assertEqual(len(a&b),32)
  for p in ps:
   if p['part'].startswith('Mane / pink neck'):self.overlap(p,self.find(ps,'Neck /'))
   elif p['part'].startswith('Mane / pink back'):
    center=(min(p['positions'][2::3])+max(p['positions'][2::3]))/2;self.overlap(p,front if center>0 else rear)
  stem=self.find(ps,'Tail /');self.overlap(stem,rear);self.overlap(stem,self.find(ps,'Tail head /'))
 def test_head_ears_horns_and_antler_branches_have_real_support(self):
  for s,m in self.models.items():
   ps=m['primitives'];head=self.find(ps,'Head /')
   for p in ps:
    name=p['part'];side=name.split()[-1]
    if name.startswith(('Ear / broad','Horn /','Antler stem /')):self.overlap(p,head)
    elif name.startswith('Horn tip /'):self.overlap(p,self.find(ps,'Horn / short ivory ossicone '+side))
    elif name.startswith('Antler /'):self.overlap(p,self.find(ps,'Antler stem / pale rooted brow stalk '+side))
    elif name.startswith('Antler tine /'):self.overlap(p,self.find(ps,'Antler / open branching oval loop '+name.split()[-2]))
    elif name.startswith('Antler orb /'):self.overlap(p,self.find(ps,'Antler / open branching oval loop '+side))
 def test_stantler_loops_are_hollow_and_tail_is_attached(self):
  ps=self.models['stantler']['primitives']
  for p in ps:
   if p['part'].startswith('Antler /'):
    x=(min(p['positions'][::3])+max(p['positions'][::3]))/2;y=(min(p['positions'][1::3])+max(p['positions'][1::3]))/2
    self.assertFalse(ray_intersections(p,x,y));self.assertTrue(ray_intersections(p,x,max(p['positions'][1::3])-.010))
  base=self.find(ps,'Tail / short');self.overlap(base,ps[0]);self.overlap(base,self.find(ps,'Tail / large'))
 def test_flank_and_neck_markings_follow_stored_side_skin(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part']
    if not name.startswith(('Spot /','Spot center /')):continue
    side=int(name.split()[-2]);j=int(name.split()[-1])
    if name.startswith('Spot center /'):skin=self.find(ps,f'Spot / fitted brown {"neck" if "neck" in name else "shoulder"} marking {side} {j}')
    elif 'hindquarter' in name:skin=ps[1]
    elif 'neck' in name or ('shoulder' in name and j==1):skin=self.find(ps,'Neck /')
    else:skin=ps[0]
    q=dict(skin);q['positions']=[v for i in range(0,len(skin['positions']),3) for v in (-side*skin['positions'][i+2],skin['positions'][i+1],side*skin['positions'][i])]
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3];hits=ray_intersections(q,-side*z,y);self.assertTrue(hits,name);self.assertLess(abs(side*x-max(hits)),.006,name)
 def test_fitted_faces_and_rear_facing_tail_features(self):
  for s,m in self.models.items():
   ps=m['primitives'];head=self.find(ps,'Head /')
   for p in ps:
    name=p['part'];side=name.split()[-1];rear=False
    if name.startswith(('Eye / fitted dark','Brow spot /')):skin=head;limit=.013
    elif name.startswith('Eye / fitted ivory'):skin=self.find(ps,'Eye / fitted dark socket '+side);limit=.006
    elif name.startswith('Eye / dark'):skin=self.find(ps,'Eye / fitted ivory white '+side);limit=.004
    elif name.startswith('Eye / small'):skin=self.find(ps,'Eye / dark vertical pupil '+side);limit=.003
    elif name.startswith('Tail eye / gold'):skin=self.find(ps,'Tail head /');limit=.009;rear=True
    elif name.startswith('Tail eye / dark'):skin=self.find(ps,'Tail eye / gold oval '+side);limit=.004;rear=True
    elif name.startswith('Tail mouth /'):skin=self.find(ps,'Tail head /');limit=.008;rear=True
    elif name.startswith('Tail tooth /'):skin=self.find(ps,'Tail mouth /');limit=.003;rear=True
    else:continue
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y);self.assertTrue(hits,(s,name));self.assertLess(abs(z-(min(hits) if rear else max(hits))),limit,(s,name))
if __name__=='__main__':unittest.main()
