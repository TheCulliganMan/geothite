#!/usr/bin/env python3
"""Distinct baby fairy silhouettes, manifold surfaces and physically fitted faces."""
import json,os,unittest
from pathlib import Path
from baby_glb import read_baby,geometry_digest
from baby_fairy_sculpt import sculpture,SPECIES,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('BABY_FAIRY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class BabyFairyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_baby(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,self.models)
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,self.models)
 def test_exact_recipe_parity(self):
  for s,m in self.models.items():self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)))
 def test_grounding_height_budget_and_unique_parts(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),50000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
   for p in ps:
    if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0)
 def test_distinct_anatomy_and_eye_colors(self):
  for s in SPECIES:
   ps=self.models[s]['primitives'];count=lambda prefix:sum(p['part'].startswith(prefix) for p in ps)
   for prefix in ('Body /','Mouth /'):self.assertEqual(count(prefix),1)
   for prefix in ('Arm /','Foot /','Eye glint /'):self.assertEqual(count(prefix),2)
   if s=='cleffa':self.assertEqual(count('Ear /'),2);self.assertEqual(count('Tuft /'),1);self.assertEqual(count('Tail /'),1);self.assertEqual(count('Cheek /'),2);self.assertEqual(count('Crown /'),0)
   else:self.assertEqual(count('Crown /'),3);self.assertEqual(count('Forehead /'),1);self.assertEqual(count('Tail /'),0);self.assertEqual(count('Ear /'),0);self.assertEqual(count('Iris /'),2);self.assertGreater(self.find(s,'Iris /')['base_color'][0],self.find(s,'Iris /')['base_color'][2])
 def pairs(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    n=p['part'];sign=n.split()[-1];parent=None
    if n.startswith(('Eye /','Eye border /','Cheek /','Mouth /')):parent=self.find(s,'Body /')
    elif n.startswith('Iris /'):parent=self.find(s,'Eye border /',sign)
    elif n.startswith('Pupil /'):parent=self.find(s,'Iris /',sign)
    elif n.startswith('Eye glint /'):parent=self.find(s,'Eye /' if s=='cleffa' else 'Pupil /',sign)
    elif n.startswith('Tongue /'):parent=self.find(s,'Mouth /')
    if parent:yield p,parent
 def test_face_vertices_follow_actual_stored_parent_triangles(self):
  for p,parent in self.pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.017)
 def test_visible_face_triangles_clear_their_support(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_ears_limbs_curls_and_tail_have_physical_roots(self):
  for s,m in self.models.items():
   body=self.find(s,'Body /')
   for p in m['primitives']:
    if p['part'].startswith(('Arm /','Foot /','Ear /','Tuft /','Tail /')):self.assertTrue(joins.DonphanTests.overlap(self,body,p),(s,p['part']))
   if s=='igglybuff':
    for j in (0,1):self.assertTrue(joins.DonphanTests.overlap(self,body,self.find(s,'Crown /',j)))
    for j in (0,1):self.assertTrue(joins.DonphanTests.overlap(self,self.find(s,'Crown /',j),self.find(s,'Crown /',2)))
 def test_spirals_follow_their_actual_rounded_supports(self):
  for s,prefix,parentprefix in (('cleffa','Tuft crease /','Tuft /'),('igglybuff','Forehead /','Body /')):
   p=self.find(s,prefix);parent=self.find(s,parentprefix)
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,x,y));self.assertLess(abs(z-max(hits)),.012)
if __name__=='__main__':unittest.main()
