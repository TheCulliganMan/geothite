#!/usr/bin/env python3
"""Bellossom's stored flower cavities, layered skirt, face and arm roots."""
import json,unittest
from pathlib import Path
from bellossom_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BellossomTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'bellossom.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'bellossom':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'bellossom':self.m})
 def test_recipe_parity_height_budget_and_botanical_counts(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Skirt /',15),('Arm /',2),('Flower petal /',12),('Flower center /',2),('Flower well /',2),('Cheek /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:return True
  return False
 def fitted_pairs(self):
  head=self.find('Head /')
  for p in self.ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith(('Eye /','Cheek /','Mouth /')):parent=head
   elif n.startswith('Eye glint /'):parent=self.find('Eye / glossy dark oval '+sign)
   elif n.startswith('Tongue /'):parent=self.find('Mouth /')
   elif n.startswith('Flower well /'):parent=self.find('Flower center / recessed golden crown cup '+sign)
   if parent:yield p,parent
 def test_face_and_flower_wells_fit_actual_parent_vertices(self):
  for p,parent in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.012)
 def test_front_face_triangles_clear_actual_parent(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_all_skirt_leaves_and_dancing_arms_have_real_torso_roots(self):
  body=self.find('Body /');head=self.find('Head /');self.assertTrue(self.overlap(body,head))
  for p in self.ps:
   if p['part'].startswith(('Skirt /','Arm /')):self.assertTrue(self.overlap(p,body),p['part'])
 def test_every_crown_petal_is_rooted_and_both_flowers_attach_to_head(self):
  head=self.find('Head /')
  for sign in (-1,1):
   cup=self.find('Flower center / recessed golden crown cup '+str(sign));self.assertTrue(self.overlap(cup,head),cup['part'])
   for j in range(6):
    petal=self.find('Flower petal / broad curled coral ray '+str(sign)+' '+str(j));self.assertTrue(self.overlap(petal,cup),petal['part'])
 def test_actual_recessed_flower_centers_have_no_petals_piercing_the_wells(self):
  for sign in (-1,1):
   cup=self.find('Flower center / recessed golden crown cup '+str(sign));well=self.find('Flower well / deep olive center '+str(sign));x=(max(cup['positions'][::3])+min(cup['positions'][::3]))/2;y=(max(cup['positions'][1::3])+min(cup['positions'][1::3]))/2
   floor=max(ray_intersections(cup,x,y));self.assertGreater(max(cup['positions'][2::3])-floor,.025)
   for p in self.ps:
    if not p['part'].startswith('Flower petal /'):continue
    radius=(max(well['positions'][::3])-min(well['positions'][::3]))*.45
    for dx,dy in ((radius*i/3,radius*j/3) for i in range(-3,4) for j in range(-3,4) if i*i+j*j<=9):
     hits=ray_intersections(p,x+dx,y+dy)
     if hits:self.assertLess(max(hits),floor-.003,p['part'])
if __name__=='__main__':unittest.main()
