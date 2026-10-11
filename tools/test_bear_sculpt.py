#!/usr/bin/env python3
"""Stored bear anatomy, physical contacts and fitted lunar-marking regressions."""
import json,unittest
from pathlib import Path
from bear_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BearTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,ps,prefix):return next(p for p in ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
 def test_normals_follow_stored_triangles(self):assert_model_shading(self,self.models)
 def test_regeneration_height_budget_and_distinct_anatomy(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(m,sculpture(s));self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),29000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
   for prefix,n in (('Ear / round',2),('Leg /',2),('Arm /',2),('Foot claw /',6),('Hand claw /',10 if s=='ursaring' else 6),('Ruff /',8 if s=='ursaring' else 0)):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n)
  def width(p):return max(p['positions'][::3])-min(p['positions'][::3])
  self.assertGreater(width(self.models['teddiursa']['primitives'][2])/width(self.models['teddiursa']['primitives'][0]),1.2)
  self.assertLess(width(self.models['ursaring']['primitives'][2])/width(self.models['ursaring']['primitives'][0]),.6)
 def has_overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0004:return True
  return False
 def overlap(self,a,b):self.assertTrue(self.has_overlap(a,b),(a['part'],b['part']))
 def test_body_head_ears_limbs_tail_and_grounded_paws(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.overlap(ps[0],ps[1]);self.overlap(ps[1],ps[2]);self.overlap(self.find(ps,'Tail /'),ps[0])
   for side in (-1,1):
    leg=self.find(ps,'Leg / heavy bent bear shin '+str(side));foot=self.find(ps,'Foot / broad grounded bear paw '+str(side));self.overlap(leg,ps[0]);self.overlap(leg,foot);self.assertEqual(min(foot['positions'][1::3]),0.)
    arm=self.find(ps,'Arm / sculpted bear forelimb '+str(side));self.overlap(arm,ps[0]);self.overlap(arm,self.find(ps,'Hand / broad rounded bear palm '+str(side)));self.overlap(self.find(ps,'Ear / round bear pinna '+str(side)),ps[2])
    for j in range(3):self.overlap(self.find(ps,f'Foot claw / ivory forward talon {side} {j}'),foot)
    for j in range(5 if s=='ursaring' else 3):self.overlap(self.find(ps,f'Hand claw / ivory curved talon {side} {j}'),self.find(ps,'Hand / broad rounded bear palm '+str(side)))
 def test_muzzles_noses_and_honey_paw_are_attached(self):
  for s,m in self.models.items():
   ps=m['primitives'];muzzle=self.find(ps,'Muzzle /');self.overlap(muzzle,ps[2]);self.overlap(self.find(ps,'Nose /'),muzzle)
  ps=self.models['teddiursa']['primitives'];self.overlap(self.find(ps,'Hand / broad rounded bear palm -1'),self.find(ps,'Muzzle /'))
 def test_shoulder_ruff_has_connected_body_support(self):
  ps=self.models['ursaring']['primitives'];supported=[ps[0],ps[1],self.find(ps,'Arm / sculpted bear forelimb -1'),self.find(ps,'Arm / sculpted bear forelimb 1')];pending=[p for p in ps if p['part'].startswith('Ruff /')]
  while pending:
   rooted=[p for p in pending if any(self.has_overlap(p,q) for q in supported)]
   self.assertTrue(rooted,[p['part'] for p in pending]);supported.extend(rooted);pending=[p for p in pending if p not in rooted]
 def test_shoulder_fur_front_triangles_clear_chest_and_neck(self):
  ps=self.models['ursaring']['primitives']
  for p in ps:
   if not p['part'].startswith('Ruff /'):continue
   v=p['positions']
   for k in range(0,len(p['indices']),3):
    ids=p['indices'][k:k+3]
    if sum(p['normals'][i*3+2] for i in ids)<=.001:continue
    x,y,z=[sum(v[i*3+j] for i in ids)/3 for j in range(3)]
    hits=ray_intersections(ps[0],x,y)+ray_intersections(ps[1],x,y)
    if hits:self.assertGreater(z-max(hits),.001,(p['part'],x,y,z,max(hits)))
 def test_lunar_markings_have_real_holes_and_fitted_vertices(self):
  for s,m in self.models.items():
   ps=m['primitives'];mark=self.find(ps,'Moon /' if s=='teddiursa' else 'Ring /');support=ps[2] if s=='teddiursa' else ps[0]
   for x,y,z in {tuple(mark['positions'][i:i+3]) for i in range(0,len(mark['positions']),3)}:
    hits=ray_intersections(support,x,y);self.assertTrue(hits);self.assertLess(abs(z-max(hits)),.011)
   x=(min(mark['positions'][::3])+max(mark['positions'][::3]))/2;y=(min(mark['positions'][1::3])+max(mark['positions'][1::3]))/2
   self.assertFalse(ray_intersections(mark,x,y));self.assertTrue(ray_intersections(support,x,y))
 def test_fitted_eyes_and_ear_centers_follow_support(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part'];support=None;side=name.split()[-1]
    if name.startswith('Ear / fitted'):support=self.find(ps,'Ear / round bear pinna '+side)
    elif name.startswith('Eye / large') or name.startswith('Eye / fitted'):support=ps[2]
    elif name.startswith('Pupil /'):support=self.find(ps,'Eye / fitted stern ivory adult eye '+side)
    if support:
     for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
      hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.010)
if __name__=='__main__':unittest.main()
