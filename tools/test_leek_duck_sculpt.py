#!/usr/bin/env python3
"""Stored duck: true wing-held leek, closed webbed feet and fitted markings."""
import json,os,unittest
from pathlib import Path
from leek_duck_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bivalve_sculpt import top_view
import test_magnet_sculpt as joins
RIG_READY=os.environ.get('LEEK_DUCK_TEST_RIG_READY')=='1'
ROOT=Path(os.environ.get('LEEK_DUCK_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class LeekDuckTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'farfetch_d.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'farfetch_d':self.m})
 def test_finite_unit_normals_and_actual_winding(self):assert_model_shading(self,{'farfetch_d':self.m})
 def test_exact_recipe_grounding_height_budget(self):
  self.assertEqual(self.m,sculpture(rig_ready=RIG_READY));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000 if RIG_READY else 85000);
  if RIG_READY:self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
 def test_distinct_identity_details(self):
  for prefix,count in (('Body /',1),('Head /',1),('Underbody /',1),('Eye border /',2),('Eye white /',2),('Pupil /',2),('Glint /',2),('Brow /',1),('Bill /',1),('Bill seam /',1),('Crest /',3),('Leg /',2),('Foot /',2),('Web crease /',6),('Raised wing /',1),('Raised feather /',3),('Holding wing /',1),('Holding feather /',3),('Grip /',2),('Leek stem /',1),('Leek neck /',1),('Leek leaf /',3),('Tail /',3)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,prefix)
 def test_body_head_bill_crests_wings_tail_and_feet_have_real_roots(self):
  body=self.find('Body /');head=self.find('Head /');pairs=[(body,self.find('Underbody /')),(body,head),(head,self.find('Bill /')),(body,self.find('Raised wing /')),(body,self.find('Holding wing /'))]
  for p in self.ps:
   n=p['part']
   if n.startswith('Crest /'):pairs.append((head,p))
   elif n.startswith('Leg /'):pairs.append((self.find('Underbody /'),p))
   elif n.startswith('Tail /'):pairs.append((self.find('Underbody /'),p))
   elif n.startswith('Foot /'):pairs.append((self.find('Leg /',n.split()[-1]),p))
   elif n.startswith('Raised feather /'):pairs.append((self.find('Raised wing /'),p))
   elif n.startswith(('Holding feather /','Grip /')):pairs.append((self.find('Holding wing /'),p))
  for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(a['part'],b['part']))
 def test_leek_is_held_and_leaves_have_physical_stalk_roots(self):
  stem=self.find('Leek stem /');neck=self.find('Leek neck /');pairs=[(stem,neck)]
  pairs += [(stem,p) for p in self.ps if p['part'].startswith('Grip /')]
  pairs += [(neck,p) for p in self.ps if p['part'].startswith('Leek leaf /')]
  for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(a['part'],b['part']))
 def markings(self):
  for p in self.ps:
   n=p['part'];parent=None
   if n.startswith(('Brow /','Eye border /')):parent=self.find('Head /')
   elif n.startswith('Eye white /'):parent=self.find('Eye border /',n.split()[-1])
   elif n.startswith('Pupil /'):parent=self.find('Eye white /',n.split()[-1])
   elif n.startswith('Glint /'):parent=self.find('Pupil /',n.split()[-1])
   elif n.startswith('Bill seam /'):parent=self.find('Bill /')
   if parent:yield p,parent
 def test_marking_vertices_follow_actual_parent_triangles(self):
  for p,parent in self.markings():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,p['part']);self.assertLess(abs(z-max(h)),.017,p['part'])
 def test_front_marking_triangles_clear_stored_support(self):
  for p,parent in self.markings():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,p['part'])
 def test_web_folds_are_supported_by_actual_upward_foot_surfaces(self):
  for p in self.ps:
   if not p['part'].startswith('Web crease /'):continue
   parent=top_view(self.find('Foot /',p['part'].split()[-2]));mark=top_view(p)
   for i in range(0,len(mark['positions']),3):
    x,y,z=mark['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,p['part']);self.assertLess(abs(z-max(h)),.010,p['part'])
if __name__=='__main__':unittest.main()
