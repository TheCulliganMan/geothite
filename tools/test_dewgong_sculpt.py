#!/usr/bin/env python3
"""Stored Dewgong curve, fan roots, fitted face and non-overlapping body sections."""
import json,os,unittest
from pathlib import Path
from dewgong_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as ray_intersections
ROOT=Path(os.environ.get('DEWGONG_TEST_MODEL',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/dewgong.glb'))
class DewgongTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from seal_glb import read_seal
  cls.m=read_seal(ROOT,'dewgong');cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'dewgong':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'dewgong':self.m})
 def test_recipe_parity_height_grounding_and_budget(self):
  from seal_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
 def test_distinct_adult_seal_anatomy(self):
  for prefix,count in (('Body /',1),('Flipper /',2),('Flipper fold /',4),('Tail fluke /',2),('Muzzle /',1),('Nose /',1),('Eye /',2),('Eye glint /',2),('Fang /',2),('Horn /',1),('Mouth /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count)
  self.assertFalse(any(p['part'].startswith(('Tongue /','Head /')) for p in self.ps))
  self.assertTrue(all(v>.85 for v in self.find('Muzzle /')['base_color'][:3]))
 def pairs(self):
  for p in self.ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith('Eye /'):parent=self.find('Body /')
   elif n.startswith(('Eye sheen /','Eye glint /')):parent=self.find('Eye /',sign)
   elif n.startswith('Nose /'):parent=self.find('Muzzle /')
   elif n.startswith('Nose glint /'):parent=self.find('Nose /')
   if parent:yield p,parent
 def test_marking_vertices_fit_actual_stored_support(self):
  for p,parent in self.pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    h=ray_intersections(parent,x,y);self.assertTrue(h,(p['part'],x,y));self.assertLess(abs(z-max(h)),.017)
 def test_front_marking_triangles_clear_actual_support(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(p['part'],k//3,z-max(h)))
 def test_appendages_and_white_muzzle_have_solid_roots(self):
  body=self.find('Body /');muzzle=self.find('Muzzle /');pairs=[(body,muzzle)]
  pairs += [(body,p) for p in self.ps if p['part'].startswith(('Flipper /','Tail fluke /','Horn /'))]
  pairs += [(muzzle,p) for p in self.ps if p['part'].startswith('Fang /')]
  for a,b in pairs:self.assertTrue(parts_overlap(a,b),(a['part'],b['part']))
 def test_smile_and_fan_folds_follow_actual_surfaces(self):
  for p in self.ps:
   if p['part'].startswith('Mouth /'):parent=self.find('Muzzle /')
   elif p['part'].startswith('Flipper fold /'):parent=self.find('Flipper /',p['part'].split()[-2])
   else:continue
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    h=ray_intersections(parent,x,y);self.assertTrue(h,(p['part'],x,y));self.assertLess(abs(z-max(h)),.012)
 def test_body_curve_does_not_fold_into_itself(self):
  body=self.find('Body /');vs=[body['positions'][i:i+3] for i in range(0,len(body['positions']),3)];tris=[]
  for i in range(0,len(body['indices']),3):
   a,b,c=[vs[j] for j in body['indices'][i:i+3]];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
   if abs(den)>1e-12:tris.append((a,b,c,den))
  xmin=min(v[0] for v in vs);xmax=max(v[0] for v in vs);ymin=min(v[1] for v in vs);ymax=max(v[1] for v in vs)
  # Ordered signed ray crossings distinguish disjoint U-shaped sections from
  # nested overlap or inverted inner bend surfaces. Avoid shared-edge rays.
  for i in range(21):
   for j in range(37):
    x=xmin+(xmax-xmin)*(i+.371)/21;y=ymin+(ymax-ymin)*(j+.619)/37;hits=[]
    for a,b,c,den in tris:
     u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
     if min(u,v,1-u-v)>1e-9:hits.append((u*a[2]+v*b[2]+(1-u-v)*c[2],1 if den>0 else -1))
    hits.sort();self.assertEqual([s for z,s in hits],[-1,1]*(len(hits)//2),(x,y,hits))
 def test_flukes_are_high_behind_the_face_and_flippers_extend_laterally(self):
  for p in self.ps:
   if p['part'].startswith('Tail fluke /'):self.assertGreater(min(p['positions'][1::3]),HEIGHT*.75);self.assertLess(max(p['positions'][2::3]),-.3)
   if p['part'].startswith('Flipper /'):self.assertGreater(max(abs(v) for v in p['positions'][::3]),HEIGHT*.35)
if __name__=='__main__':unittest.main()
