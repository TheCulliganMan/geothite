#!/usr/bin/env python3
"""Six stored psychic eggs, distinct faces, supported cracks and hollow shell."""
import json,os,unittest
from pathlib import Path
from exeggcute_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bivalve_sculpt import top_view
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('EXEGGCUTE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class EggTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'exeggcute.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_parts(self):assert_closed_models(self,{'exeggcute':self.m})
 def test_unit_normals_and_actual_winding(self):assert_model_shading(self,{'exeggcute':self.m})
 def test_recipe_height_ground_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000)
 def test_six_eggs_five_distinct_faces_and_one_yolk(self):
  for prefix,n in (('Egg /',6),('Eye white /',10),('Pupil /',10),('Lid /',10),('Mouth /',5),('Crack /',4),('Yolk /',1)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
  self.assertEqual(len({tuple(self.find('Mouth /',j)['positions']) for j in (0,1,2,4,5)}),5)
 def test_broken_shell_is_hollow_and_yolk_is_rooted(self):
  shell=self.find('Egg /',3);yolk=self.find('Yolk /');rot=top_view(shell)
  x=(min(rot['positions'][::3])+max(rot['positions'][::3]))/2;y=(min(rot['positions'][1::3])+max(rot['positions'][1::3]))/2
  hits=ray_intersections(rot,x,y);self.assertTrue(hits);self.assertGreater(max(rot['positions'][2::3])-max(hits),HEIGHT*.25)
  self.assertTrue(joins.DonphanTests.overlap(self,shell,yolk),'yolk must join actual shell floor')
 def test_raised_rear_eggs_join_front_cluster(self):
  for j in (3,4,5):
   self.assertTrue(any(joins.DonphanTests.overlap(self,self.find('Egg /',j),self.find('Egg /',i)) for i in (0,1,2)),('unsupported raised egg',j))
 def test_each_expression_has_an_unoccluded_front_sample(self):
  eggs=[p for p in self.ps if p['part'].startswith('Egg /')]
  for j in (0,1,2,4,5):
   mouth=self.find('Mouth /',j);fronts=[]
   for k in range(0,len(mouth['indices']),3):
    a,b,c=[mouth['positions'][i*3:i*3+3] for i in mouth['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])>1e-10:fronts.append(tuple((a[t]+b[t]+c[t])/3 for t in range(3)))
   samples=fronts[::max(1,len(fronts)//25)];visible=0
   for x,y,z in samples:
    hits=[h for egg in eggs for h in ray_intersections(egg,x,y)];self.assertTrue(hits)
    visible += z-max(hits)>.0001
   self.assertGreater(visible,len(samples)*.70,('occluded expression',j,visible,len(samples)))
 def pairs(self):
  for p in self.ps:
   n=p['part'];parent=None
   if n.startswith(('Eye white /','Mouth /','Crack /')):j=n.split()[-2] if n.startswith('Eye white /') else n.split()[-1];parent=self.find('Egg /',j)
   elif n.startswith(('Pupil /','Lid /')):parent=next(q for q in self.ps if q['part'].startswith('Eye white /') and q['part'].split()[-2:]==n.split()[-2:])
   if parent:yield p,parent
 def test_markings_and_lines_follow_actual_stored_parents(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,p['part']);self.assertLess(abs(z-max(h)),.023,p['part'])
 def test_front_marking_triangles_clear_stored_support(self):
  for p,parent in self.pairs():
   if p['part'].startswith(('Lid /','Crack /')):continue
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,p['part'])
if __name__=='__main__':unittest.main()
