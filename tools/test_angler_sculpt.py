#!/usr/bin/env python3
"""Stored angler surface, shading, anatomy and attachment regressions."""
import json,unittest
from pathlib import Path
from angler_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class AnglerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,ps,prefix):return next(p for p in ps if p['part'].startswith(prefix))
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
 def test_normals_follow_stored_triangles(self):assert_model_shading(self,self.models)
 def test_regeneration_height_budget_and_distinct_faces(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(m,sculpture(s));self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),25000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
   for prefix,n in (('Fin / broad',2),('Fin / soft',2),('Lure / golden' if s=='lanturn' else 'Lure / hanging',2),('Pupil / black cross' if s=='chinchou' else 'Eye / deep red',4 if s=='chinchou' else 2)):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0003:return
  self.fail((a['part'],b['part']))
 def test_broad_mask_triangle_centers_stay_above_cheek(self):
  ps=self.models['lanturn']['primitives'];mask=self.find(ps,'Mask /');v=mask['positions']
  for k in range(0,len(mask['indices']),3):
   ids=mask['indices'][k:k+3]
   if sum(mask['normals'][i*3+2] for i in ids)<=.001:continue
   x,y,z=[sum(v[i*3+j] for i in ids)/3 for j in range(3)];hits=ray_intersections(ps[0],x,y)
   self.assertTrue(hits);self.assertGreater(z-max(hits),.001,(x,y,z,max(hits)))
 def test_fins_are_rooted_in_body(self):
  for m in self.models.values():
   for p in m['primitives']:
    if p['part'].startswith('Fin /'):self.overlap(p,m['primitives'][0])
 def test_light_stalks_root_and_bulbs_attach(self):
  ps=self.models['chinchou']['primitives']
  for s in (-1,1):
   stalk=self.find(ps,'Stalk / arched flexible antenna '+str(s));self.overlap(stalk,ps[0]);self.overlap(stalk,self.find(ps,'Lure / hanging golden teardrop '+str(s)))
  ps=self.models['lanturn']['primitives'];main=self.find(ps,'Stalk / single');branch=self.find(ps,'Stalk / short');self.overlap(main,ps[0]);self.overlap(main,branch);self.overlap(main,self.find(ps,'Lure / golden large'));self.overlap(branch,self.find(ps,'Lure / golden small'))
 def test_lanturn_tail_lobes_root_in_real_peduncle(self):
  ps=self.models['lanturn']['primitives'];stem=self.find(ps,'Tail /');self.overlap(stem,ps[0])
  for p in ps:
   if p['part'].startswith('Tail fin /'):self.overlap(p,stem)
 def test_fitted_eyes_pupils_and_lights_follow_support(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part'];support=None;side=name.split()[-1]
    if name.startswith('Eye / large'):support=ps[0]
    elif name.startswith('Pupil / black cross'):support=self.find(ps,'Eye / large golden oval '+side)
    elif name.startswith('Mask /'):support=ps[0]
    elif name.startswith('Eye / dark'):support=self.find(ps,'Mask /')
    elif name.startswith('Eye / ivory'):support=self.find(ps,'Eye / dark fitted almond socket '+side)
    elif name.startswith('Eye / deep'):support=self.find(ps,'Eye / ivory almond white '+side)
    elif name.startswith('Pupil / tall'):support=self.find(ps,'Eye / deep red iris '+side)
    elif name.startswith('Lure / pale'):
     support=self.find(ps,'Lure / hanging golden teardrop '+side) if s=='chinchou' else self.find(ps,'Lure / golden '+name.split()[3])
    if support:
     for i in range(0,len(p['positions']),3):
      x,y,z=p['positions'][i:i+3];hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.023,(name,z,max(hits)))
if __name__=='__main__':unittest.main()
