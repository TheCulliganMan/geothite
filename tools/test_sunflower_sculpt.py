#!/usr/bin/env python3
"""Stored seed cup, fitted coat, sunflower petal and rooted leaf checks."""
import json,math,unittest
from pathlib import Path
from sunflower_sculpt import sculpture,SPECIES,HEIGHT
from sunflower_glb import read_sunflower,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SunflowerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.ms={s:read_sunflower(s,ROOT/(s+'.glb')) for s in SPECIES}
 def find(self,s,prefix):return next(p for p in self.ms[s]['primitives'] if p['part'].startswith(prefix))
 def test_closed_connected_outward_parts(self):assert_closed_models(self,self.ms)
 def test_finite_unit_normals_follow_real_winding(self):assert_model_shading(self,self.ms)
 def test_recipe_parity_height_budget_and_distinct_botanical_features(self):
  for s,m in self.ms.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),22000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
  for s,counts in (('sunkern',(('Coat /',4),('Leaf /',2),('Eye /',2))),('sunflora',(('Petal /',8),('Bract /',8),('Leaf arm /',2),('Leg /',2),('Foot /',2)))):
   for prefix,n in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ms[s]['primitives']),n)
 def has_overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:return True
  return False
 def support_hits(self,parents,x,y):return [z for p in parents for z in ray_intersections(p,x,y)]
 def fitted_pairs(self,s):
  ps=self.ms[s]['primitives'];body=ps[0];face=self.find(s,'Face /') if s=='sunflora' else None
  for p in ps:
   name=p['part'];sign=name.split()[-1];parents=None;mark=p
   if name.startswith('Coat /'):
    parents=[body]
    if 'rear' in name:mark=reverse_front(p);parents=[reverse_front(body)]
   elif name.startswith('Eye /'):parents=[face] if face else [body]+[q for q in ps if q['part'].startswith('Coat / broad fitted dark front')]
   elif name.startswith('Eye glint /'):parents=[self.find(s,'Eye / large black seed button '+sign)]
   elif name.startswith('Mouth /'):parents=[face] if face else [body]+[q for q in ps if q['part'].startswith('Coat / broad fitted dark front')]
   elif name.startswith('Tongue /'):parents=[self.find(s,'Mouth /')]
   if parents:yield mark,parents
 def test_coats_faces_and_tongues_follow_actual_stored_support(self):
  for s in SPECIES:
   for p,parents in self.fitted_pairs(s):
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=self.support_hits(parents,x,y);self.assertTrue(hits,(s,p['part'],x,y));self.assertLess(abs(z-max(hits)),.014)
 def test_front_marking_triangles_clear_support_including_both_coat_faces(self):
  for s in SPECIES:
   for p,parents in self.fitted_pairs(s):
    for k in range(0,len(p['indices']),3):
     pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts;nz=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
     if nz<1e-10:continue
     x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=self.support_hits(parents,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(s,p['part'],k//3,z-max(hits)))
 def test_leaf_midribs_follow_the_actual_leaf_faces(self):
  for s in SPECIES:
   for p in self.ms[s]['primitives']:
    if not p['part'].startswith('Leaf vein /'):continue
    sign=p['part'].split()[-1];parent=self.find(s,('Leaf / broad curved seed sprout ' if s=='sunkern' else 'Leaf arm / cupped asymmetrical green blade ')+sign)
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part'],x,y));self.assertLess(abs(z-max(hits)),.007)
 def test_seed_crown_has_eight_teeth_and_actual_recessed_cavity_floor(self):
  body=self.ms['sunkern']['primitives'][0];top=max(body['positions'][1::3]);pts={tuple(body['positions'][i:i+3]) for i in range(0,len(body['positions']),3)}
  angles=sorted(math.atan2(x,z) for x,y,z in pts if top-y<1e-6);groups=[]
  for a in angles:
   if not groups or a-groups[-1][-1]>.1:groups.append([a])
   else:groups[-1].append(a)
  if groups[0][0]+math.tau-groups[-1][-1]<.1:groups[0]=groups[-1]+groups[0];groups.pop()
  self.assertEqual(len(groups),8)
  down=dict(body);down['positions']=[v for i in range(0,len(body['positions']),3) for v in (body['positions'][i],body['positions'][i+2],body['positions'][i+1])]
  hits=ray_intersections(down,0,0);self.assertTrue(hits);self.assertGreater(top-max(hits),.12)
  stem=self.find('sunkern','Sprout /');self.assertLess(min(stem['positions'][1::3]),max(hits)-.004);self.assertTrue(self.has_overlap(stem,body))
 def test_seed_leaves_and_sunflower_leaf_arms_have_real_roots(self):
  for s,prefix,parent in (('sunkern','Leaf /',self.find('sunkern','Sprout /')),('sunflora','Leaf arm /',self.ms['sunflora']['primitives'][0])):
   for p in self.ms[s]['primitives']:
    if p['part'].startswith(prefix):self.assertTrue(self.has_overlap(p,parent),(s,p['part']))
 def test_every_ray_petal_and_rear_bract_overlaps_flower_disk(self):
  face=self.find('sunflora','Face /');body=self.ms['sunflora']['primitives'][0];self.assertTrue(self.has_overlap(body,face))
  for p in self.ms['sunflora']['primitives']:
   if p['part'].startswith(('Petal /','Bract /')):self.assertTrue(self.has_overlap(p,face),p['part'])
 def test_green_supporting_roots_join_two_grounded_foot_pads(self):
  for sign in (-1,1):
   leg=self.find('sunflora','Leg / curved supporting green root '+str(sign));foot=self.find('sunflora','Foot / grounded broad green root pad '+str(sign))
   self.assertTrue(self.has_overlap(leg,self.ms['sunflora']['primitives'][0]));self.assertTrue(self.has_overlap(leg,foot));self.assertEqual(min(foot['positions'][1::3]),0.)
if __name__=='__main__':unittest.main()
