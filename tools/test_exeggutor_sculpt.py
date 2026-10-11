#!/usr/bin/env python3
"""Stored coconut palm: closed blades, solid roots and fitted distinct faces."""
import json,os,unittest
from pathlib import Path
from exeggutor_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bovine_sculpt import reverse_front
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('EXEGGUTOR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class PalmTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'exeggutor.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'exeggutor':self.m})
 def test_unit_normals_and_actual_winding(self):assert_model_shading(self,{'exeggutor':self.m})
 def test_recipe_grounding_height_budget(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),75000)
 def test_distinct_heads_clawed_feet_and_folded_canopy(self):
  for prefix,n in (('Trunk /',1),('Head /',3),('Leg /',2),('Foot /',2),('Sole /',2),('Claw /',6),('Eye white /',6),('Pupil /',6),('Mouth /',3),('Tongue /',1),('Slack tongue /',1),('Fang /',2),('Frond /',11),('Midrib /',11),('Trunk crease /',6)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_heads_limbs_claws_and_fronds_have_real_roots(self):
  trunk=self.find('Trunk /');pairs=[]
  for p in self.ps:
   if p['part'].startswith(('Head /','Leg /','Frond /')):pairs.append((trunk,p))
   elif p['part'].startswith('Claw /'):pairs.append((self.find('Foot /',p['part'].split()[-2]),p))
  for sign in (-1,1):pairs.append((self.find('Leg /',sign),self.find('Foot /',sign)))
  for a,b in pairs:self.assertTrue(joins.DonphanTests.overlap(self,a,b),(a['part'],b['part']))
 def pairs(self):
  for p in self.ps:
   n=p['part'];parent=None
   if n.startswith(('Eye white /','Mouth /')):parent=self.find('Head /',n.split()[-2] if n.startswith('Eye white /') else n.split()[-1])
   elif n.startswith(('Pupil /','Lid /')):parent=next(q for q in self.ps if q['part'].startswith('Eye white /') and q['part'].split()[-2:]==n.split()[-2:])
   elif n.startswith(('Tongue /','Fang /')):parent=self.find('Mouth /',1)
   elif n.startswith('Slack tongue /'):parent=self.find('Mouth /',0)
   elif n.startswith('Smile lining /'):parent=self.find('Mouth /',2)
   elif n.startswith('Sole /'):parent=self.find('Foot /',n.split()[-1])
   if parent:yield p,parent
 def test_marking_vertices_follow_actual_parents(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,p['part']);self.assertLess(abs(z-max(h)),.018,p['part'])
 def test_front_marking_triangles_clear_stored_support(self):
  for p,parent in self.pairs():
   if p['part'].startswith('Lid /'):continue
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,p['part'])
 def test_leaf_midribs_and_trunk_segments_are_supported(self):
  for p in self.ps:
   n=p['part']
   if n.startswith('Midrib /'):parent=self.find('Frond /',n.split()[-1]);mark=p
   elif n.startswith('Trunk crease /'):
    parent=self.find('Trunk /');mark=p
    if 'rear ' in n:parent=reverse_front(parent);mark=reverse_front(p)
   else:continue
   for i in range(0,len(mark['positions']),3):
    x,y,z=mark['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,n);self.assertLess(abs(z-max(h)),.014,n)
if __name__=='__main__':unittest.main()
