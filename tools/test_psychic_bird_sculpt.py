#!/usr/bin/env python3
"""Actual fitted symbolism, wing coatings and psychic-bird anatomy checks."""
import json,unittest
from pathlib import Path
from psychic_bird_sculpt import sculpture,SPECIES,HEIGHT,transform
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class PsychicBirdTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.ms={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,s,prefix):return next(p for p in self.ms[s]['primitives'] if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,self.ms)
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,self.ms)
 def test_recipe_parity_height_budget_and_distinct_anatomy(self):
  for s,m in self.ms.items():
   self.assertEqual(m,sculpture(s));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),60000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
  for s,counts in (('natu',(('Crest /',1),('Wing gold /',4),('Wing red /',12),('Tail /',2))),('xatu',(('Streamer /',2),('Wing hem /',4),('Wing hem red /',4),('Wing badge /',2),('Chest symbol /',2),('Back stripe /',3)))):
   for prefix,n in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ms[s]['primitives']),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=sorted(ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23));bb=sorted(ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23))
    for al,ah in zip(aa[::2],aa[1::2]):
     for bl,bh in zip(bb[::2],bb[1::2]):
      if min(ah,bh)-max(al,bl)>.00025:return True
  return False
 def fitted_pairs(self,s):
  body=self.find(s,'Body /');head=body if s=='natu' else self.find(s,'Head /')
  for p in self.ms[s]['primitives']:
   n=p['part'];parent=None;mark=p
   if n.startswith('Eye /'):parent=head
   elif n.startswith('Pupil /'):parent=self.find(s,'Eye / calm ivory almond '+n.split()[-1])
   elif n.startswith('Eye glint /'):parent=self.find(s,'Pupil / broad dark watchful oval '+n.split()[-1])
   elif n.startswith('Bill seam /'):parent=self.find(s,'Beak /')
   elif n.startswith(('Collar /','Chest symbol /')):parent=body
   elif n.startswith('Chest pupil /'):parent=self.find(s,'Chest symbol / red watchful almond '+n.split()[-1])
   elif n.startswith(('Wing gold /','Wing red /')):
    core=n.removesuffix(' rear');sign=int(core.split()[-1] if n.startswith('Wing gold /') else core.split()[-2]);mark=transform(p,sign,True);parent=transform(self.find(s,'Wing / dark outlined rounded Natu shield '+str(sign)),sign,True)
    if n.endswith(' rear'):mark=reverse_front(mark);parent=reverse_front(parent)
   elif n.startswith('Wing badge /'):parent=self.find(s,'Wing / long folded ivory ceremonial sleeve '+n.split()[-1])
   elif n.startswith('Wing badge gold /'):parent=self.find(s,'Wing badge / fitted red diagonal sleeve panel '+n.split()[-1])
   elif n.startswith(('Wing hem /','Wing hem red /')):
    sign=n.split()[-2];parent=self.find(s,'Wing / long folded ivory ceremonial sleeve '+sign)
    if n.split()[-1]=='1':mark=reverse_front(mark);parent=reverse_front(parent)
   elif n.startswith('Back stripe /'):mark=reverse_front(p);parent=reverse_front(body)
   if parent:yield mark,parent
 def test_all_faces_symbols_and_both_wing_faces_fit_actual_stored_vertices(self):
  for s in SPECIES:
   for p,parent in self.fitted_pairs(s):
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part'],x,y));self.assertLess(abs(z-max(hits)),.025)
 def test_all_front_marking_triangles_clear_parent_surfaces(self):
  for s in SPECIES:
   for p,parent in self.fitted_pairs(s):
    for k in range(0,len(p['indices']),3):
     pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
     x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(s,p['part'],k//3,z-max(hits)))
 def test_actual_body_head_bill_crest_streamer_tail_and_wing_roots(self):
  for s in SPECIES:
   body=self.find(s,'Body /');head=body if s=='natu' else self.find(s,'Head /')
   if s=='xatu':self.assertTrue(self.overlap(body,head))
   self.assertTrue(self.overlap(self.find(s,'Beak /'),head))
   for p in self.ms[s]['primitives']:
    n=p['part']
    if n.startswith('Wing /'):self.assertTrue(self.overlap(p,body),(s,n))
    if n.startswith(('Crest /','Streamer /')):self.assertTrue(self.overlap(p,head),(s,n))
    if n.startswith('Tail /'):self.assertTrue(self.overlap(p,body),(s,n))
 def test_all_supporting_ankles_splayed_toes_and_ivory_talons_are_joined(self):
  for s in SPECIES:
   body=self.find(s,'Body /')
   for sign in (-1,1):
    leg=self.find(s,'Leg / short supporting red bird ankle '+str(sign));self.assertTrue(self.overlap(leg,body),(s,sign))
    self.assertTrue(self.overlap(leg,self.find(s,'Toe / curved red rear bird digit '+str(sign))))
    for j in range(3):
     toe=self.find(s,'Toe / long splayed red bird digit '+str(sign)+' '+str(j));claw=self.find(s,'Claw / pointed ivory bird talon '+str(sign)+' '+str(j));self.assertTrue(self.overlap(leg,toe));self.assertTrue(self.overlap(toe,claw),(s,claw['part']))
if __name__=='__main__':unittest.main()
