#!/usr/bin/env python3
"""Stored steel orbs: closed magnets, real mechanical roots and fitted slots."""
import json,math,os,unittest
from pathlib import Path
from magnet_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bivalve_sculpt import top_view
ROOT=Path(os.environ.get('MAGNET_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class MagnetTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,ps,prefix,unit,ending=None):return next(p for p in ps if p['part'].startswith(prefix) and p['part'].endswith(' / unit '+str(unit)) and (ending is None or p['part'].split(' / unit ')[0].endswith(' '+str(ending))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    x,y=xl+(xh-xl)*i/19,yl+(yh-yl)*j/23;aa=sorted(ray_intersections(a,x,y));bb=sorted(ray_intersections(b,x,y))
    for al,ah in zip(aa[::2],aa[1::2]):
     for bl,bh in zip(bb[::2],bb[1::2]):
      if min(ah,bh)-max(al,bl)>.00025:return True
  return False
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,self.models)
 def test_unit_normals_and_actual_winding(self):assert_model_shading(self,self.models)
 def test_exact_recipe_grounding_height_budget(self):
  for s,m in self.models.items():
   self.assertEqual(m,sculpture(s));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),25000 if s=='magnemite' else 75000)
 def test_each_unit_has_complete_mechanical_details(self):
  for s,m in self.models.items():
   for j in range(1 if s=='magnemite' else 3):
    ps=[p for p in m['primitives'] if p['part'].endswith(' / unit '+str(j))]
    for prefix,n in (('Sphere /',1),('Eye border /',1),('Eye white /',1),('Pupil /',1),('Face bolt /',2),('Face slot /',2),('Crown shaft /',1),('Thread /',1),('Crown head /',1),('Crown slot /',1),('Magnet /',2),('Pole /',4)):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,j,prefix))
 def test_screws_magnets_and_poles_have_actual_solid_roots(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for j in range(1 if s=='magnemite' else 3):
    body=self.find(ps,'Sphere /',j);shaft=self.find(ps,'Crown shaft /',j)
    pairs=[(body,shaft),(shaft,self.find(ps,'Crown head /',j)),(shaft,self.find(ps,'Thread /',j))]
    pairs += [(body,self.find(ps,'Face bolt /',j,k)) for k in (0,1)]
    for sign in (-1,1):
     magnet=self.find(ps,'Magnet /',j,sign);pairs.append((body,magnet))
     pairs += [(magnet,p) for p in ps if p['part'].startswith('Pole /') and p['part'].endswith(' / unit '+str(j)) and p['part'].split(' / unit ')[0].split()[-2]==str(sign)]
    for a,b in pairs:self.assertTrue(self.overlap(a,b),(s,a['part'],b['part']))
 def test_magneton_bodies_physically_join(self):
  ps=self.models['magneton']['primitives'];b=[self.find(ps,'Sphere /',j) for j in range(3)]
  for i,j in ((0,1),(0,2),(1,2)):self.assertTrue(self.overlap(b[i],b[j]),(i,j))
 def markings(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    n=p['part'];j=int(n.split(' / unit ')[1]);parent=None
    if n.startswith('Eye border /'):parent=self.find(ps,'Sphere /',j)
    elif n.startswith('Eye white /'):parent=self.find(ps,'Eye border /',j)
    elif n.startswith('Pupil /'):parent=self.find(ps,'Eye white /',j)
    elif n.startswith('Face slot /'):parent=self.find(ps,'Face bolt /',j,n.split(' / unit ')[0].split()[-1])
    elif n.startswith('Crown slot /'):
     parent=self.find(ps,'Crown head /',j);angle=0 if j==0 else (2 if j==1 else -2);c,t=math.cos(angle),math.sin(angle)
     def unturn(q):
      r=dict(q);r['positions']=[v for i in range(0,len(q['positions']),3) for x,y,z in [q['positions'][i:i+3]] for v in (x*c+y*t,y*c-x*t,z)];return top_view(r)
     p,parent=unturn(p),unturn(parent)
    if parent:yield s,p,parent
 def test_markings_fit_actual_parent_triangles(self):
  for s,p,parent in self.markings():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,(s,p['part']));self.assertLess(abs(z-max(h)),.020,(s,p['part']))
 def test_marking_fronts_clear_stored_surfaces(self):
  for s,p,parent in self.markings():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(s,p['part']))
 def test_horseshoe_notches_open_outward_between_solid_arms(self):
  for species,m in self.models.items():
   ps=m['primitives']
   for j in range(1 if species=='magnemite' else 3):
    body=self.find(ps,'Sphere /',j);scale=(max(body['positions'][2::3])-min(body['positions'][2::3]))/.36;cx=(max(body['positions'][::3])+min(body['positions'][::3]))/2;cy=(max(body['positions'][1::3])+min(body['positions'][1::3]))/2;angle=0 if j==0 else (2 if j==1 else -2)
    for sign in (-1,1):
     p=self.find(ps,'Magnet /',j,sign)
     def ray(x,y):
      root=sign*.17;a=sign*.28;c,t=math.cos(a),math.sin(a);x,y=root+(sign*x-root)*c-y*t,(sign*x-root)*t+y*c;c,t=math.cos(angle),math.sin(angle);return ray_intersections(p,cx+(x*c-y*t)*scale,cy+(x*t+y*c)*scale)
     for x in (.32,.38,.44,.50,.56):self.assertFalse(ray(x,0),(species,j,sign,x))
     for y in (-.105,.105):self.assertTrue(ray(.40,y),(species,j,sign,y))
if __name__=='__main__':unittest.main()
