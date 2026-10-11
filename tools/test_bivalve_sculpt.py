#!/usr/bin/env python3
"""Stored hollow valves, rooted armor spikes and fitted bivalve faces."""
import json,os,unittest
from pathlib import Path
from bivalve_sculpt import sculpture,SPECIES,HEIGHT,top_view
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bovine_sculpt import reverse_front
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('BIVALVE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class BivalveTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,ps,prefix,suffix=None):return next(p for p in ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,self.models)
 def test_finite_unit_normals_follow_actual_winding(self):assert_model_shading(self,self.models)
 def test_recipe_height_grounding_and_budget(self):
  for s,m in self.models.items():
   self.assertEqual(m,sculpture(s));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT[s],places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),50000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
 def test_distinct_hollow_valves_tongue_and_spiked_armor(self):
  for s,m in self.models.items():
   ps=m['primitives']
   counts=(('Shell /',1),('Body /',1),('Eye white /',2),('Pupil /',2))
   counts += (('Jaw /',1),('Lining /',1),('Jaw lining /',1),('Hinge /',1),('Horn /',2),('Rear horn /',2),('Tongue /',1),('Shell groove /',3)) if s=='shellder' else (('Inner hood /',1),('Outer spike /',8),('Rear crown /',1),('Inner horn /',1),('Grin border /',1),('Teeth /',1),('Tooth gap /',3))
   for prefix,n in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,prefix))
   if s=='cloyster':self.assertFalse(any(p['part'].startswith('Tongue /') for p in ps))
 def pairs(self,s):
  ps=self.models[s]['primitives']
  for p in ps:
   n=p['part'];parent=None
   if n.startswith(('Eye white /','Grin border /')):parent=self.find(ps,'Body /')
   elif n.startswith('Pupil /'):parent=self.find(ps,'Eye white /',n.split()[-1])
   elif n.startswith('Teeth /'):parent=self.find(ps,'Grin border /')
   if parent:yield p,parent
 def test_marking_vertices_follow_actual_stored_parents(self):
  for s in SPECIES:
   for p,parent in self.pairs(s):
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part'],x,y));self.assertLess(abs(z-max(hits)),.018)
 def test_front_marking_triangles_clear_actual_support(self):
  for s in SPECIES:
   for p,parent in self.pairs(s):
    for k in range(0,len(p['indices']),3):
     a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
     x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(s,p['part'],k//3,z-max(h)))
 def test_shells_and_linings_have_real_recessed_cavities(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    if not p['part'].startswith(('Shell /','Lining /','Jaw /','Jaw lining /','Inner hood /')):continue
    y=(min(p['positions'][1::3])+max(p['positions'][1::3]))/2;h=ray_intersections(p,0,y);self.assertTrue(h,(s,p['part']));self.assertGreater(max(p['positions'][2::3])-max(h),HEIGHT[s]*.065,(s,p['part']));self.assertGreater(max(h)-min(h),.002,(s,p['part']))
 def test_shell_layers_body_hinge_and_tongue_have_solid_joins(self):
  for s,m in self.models.items():
   ps=m['primitives'];shell=self.find(ps,'Shell /');body=self.find(ps,'Body /');pairs=[(shell,body)]
   if s=='shellder':pairs += [(shell,self.find(ps,'Lining /')),(shell,self.find(ps,'Hinge /')),(body,self.find(ps,'Jaw /')),(self.find(ps,'Jaw /'),self.find(ps,'Jaw lining /')),(body,self.find(ps,'Tongue /'))]
   else:pairs += [(shell,self.find(ps,'Inner hood /')),(body,self.find(ps,'Inner hood /'))]
   for a,b in pairs:self.assertTrue(joins.DonphanTests.overlap(self,a,b),(s,a['part'],b['part']))
 def test_every_horn_and_outer_lance_has_real_shell_root(self):
  for s,m in self.models.items():
   ps=m['primitives'];shell=self.find(ps,'Shell /')
   for p in ps:
    if p['part'].startswith(('Horn /','Rear horn /','Outer spike /','Rear crown /')):parent=shell
    elif p['part'].startswith('Inner horn /'):parent=self.find(ps,'Inner hood /')
    else:continue
    self.assertTrue(joins.DonphanTests.overlap(self,parent,p),(s,p['part']))
 def test_tongue_fold_shell_grooves_and_tooth_divisions_are_supported(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    if p['part'].startswith('Tongue crease /'):parent=top_view(self.find(ps,'Tongue /'));mark=top_view(p)
    elif p['part'].startswith('Shell groove /'):parent=reverse_front(self.find(ps,'Shell /'));mark=reverse_front(p)
    elif p['part'].startswith('Tooth gap /'):parent=self.find(ps,'Teeth /');mark=p
    else:continue
    for x,y,z in {tuple(mark['positions'][i:i+3]) for i in range(0,len(mark['positions']),3)}:
     h=ray_intersections(parent,x,y);self.assertTrue(h,(s,p['part'],x,y));self.assertLess(abs(z-max(h)),.014,(s,p['part']))
if __name__=='__main__':unittest.main()
