#!/usr/bin/env python3
"""Stored dragonfly wing coatings, six forked feet and continuous abdomen checks."""
import json,unittest
from pathlib import Path
from yanma_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
def vertical(p,lower=False):
 q=dict(p);v=p['positions'];q['positions']=[a for i in range(0,len(v),3) for a in (v[i],-v[i+2],-v[i+1] if lower else v[i+1])];return q
class YanmaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'yanma.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'yanma':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'yanma':self.m})
 def test_recipe_parity_height_budget_and_dragonfly_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Head /',2),('Wing /',4),('Wing band /',8),('Wing vein /',12),('Leg /',6),('Foot /',12),('Abdomen collar /',3),('Tail fin /',2),('Tail tip /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
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
 def fitted_pairs(self):
  for p in self.ps:
   n=p['part'];parents=[];mark=p;sign=n.split()[-1]
   if n.startswith('Eye /'):parents=[self.find('Head / huge green compound eye housing '+sign)]
   elif n.startswith('Eye glint /'):parents=[self.find('Eye / glossy dark dragonfly lens '+sign)]
   elif n.startswith('Forehead /'):parents=[q for q in self.ps if q['part'].startswith('Head /')]
   elif n.startswith('Mouth /'):parents=[q for q in self.ps if q['part'].startswith('Mandible /')]
   elif n.startswith('Wing band /'):
    lower='lower' in n;mark=vertical(p,lower);parents=[vertical(self.find('Wing / curved ivory dragonfly sail '+' '.join(n.split()[-2:])),lower)]
   if parents:yield mark,parents
 def test_eye_forehead_mouth_and_both_wing_faces_fit_actual_vertices(self):
  for p,parents in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=[z for q in parents for z in ray_intersections(q,x,y)];self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.018)
 def test_front_fitted_triangles_clear_parent_surfaces(self):
  for p,parents in self.fitted_pairs():
   # Reflecting the lower wing reverses its triangle winding.
   lower=p['part'].startswith('Wing band / fitted orange lower')
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts;nz=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    if (-nz if lower else nz)<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=[z for q in parents for z in ray_intersections(q,x,y)];self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_all_twelve_gray_veins_follow_actual_ivory_wing_faces(self):
  for p in self.ps:
   if not p['part'].startswith('Wing vein /'):continue
   wing=vertical(self.find('Wing / curved ivory dragonfly sail '+' '.join(p['part'].split()[-3:-1])));mark=vertical(p)
   for x,y,z in {tuple(mark['positions'][i:i+3]) for i in range(0,len(mark['positions']),3)}:
    hits=ray_intersections(wing,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.006)
 def test_all_wings_heads_and_red_crown_points_are_rooted(self):
  thorax=self.find('Thorax /')
  for sign in (-1,1):
   head=self.find('Head / huge green compound eye housing '+str(sign));self.assertTrue(self.overlap(head,thorax));self.assertTrue(self.overlap(head,self.find('Crown / swept red forehead antenna '+str(sign))))
   self.assertTrue(self.overlap(head,self.find('Mandible / rounded red mouth lobe '+str(sign))))
   for rear in (0,1):self.assertTrue(self.overlap(thorax,self.find('Wing / curved ivory dragonfly sail '+str(sign)+' '+str(rear))))
 def test_six_jointed_legs_and_all_twelve_forked_toes_are_attached(self):
  thorax=self.find('Thorax /')
  for sign in (-1,1):
   for j in range(3):
    leg=self.find('Leg / angular jointed dragonfly strut '+str(sign)+' '+str(j));self.assertTrue(self.overlap(leg,thorax),leg['part'])
    for fork in (-1,1):self.assertTrue(self.overlap(leg,self.find('Foot / pointed forked dragonfly toe '+str(sign)+' '+str(j)+' '+str(fork))))
 def test_abdomen_annular_collars_tail_fins_and_terminal_prongs_have_real_joins(self):
  abdomen=self.find('Abdomen /');cap=self.find('Tail cap /');self.assertTrue(self.overlap(abdomen,self.find('Thorax /')));self.assertTrue(self.overlap(abdomen,cap))
  for p in self.ps:
   if p['part'].startswith('Abdomen collar /'):self.assertTrue(self.overlap(p,abdomen),p['part'])
   if p['part'].startswith(('Tail fin /','Tail tip /')):self.assertTrue(self.overlap(p,cap),p['part'])
if __name__=='__main__':unittest.main()
