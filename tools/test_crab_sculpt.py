#!/usr/bin/env python3
"""Independent stored anatomy, solid joins and fitted crab faces."""
import json,os,unittest
from pathlib import Path
from crab_sculpt import sculpture,SPECIES,HEIGHT
from crab_glb import read_crab,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('CRAB_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class CrabTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_crab(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,ps,prefix,suffix=None):return next(p for p in ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward(self):assert_closed_models(self,self.models)
 def test_stored_normals_and_winding(self):assert_model_shading(self,self.models)
 def test_recipe_ground_height_budget(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),40000)
 def test_four_legs_two_open_claws_and_distinct_crowns(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for prefix,n in (('Walking leg /',4),('Leg knee /',4),('Foot /',4),('Pincer arm /',2),('Claw palm /',2),('Claw jaw /',2),('Crown /',5 if s=='kingler' else 2)):
    self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n)
   palms=[self.find(ps,'Claw palm /',sign) for sign in (-1,1)];sizes=[max(p['positions'][1::3])-min(p['positions'][1::3]) for p in palms]
   self.assertGreater(sizes[0]/sizes[1],1.4) if s=='kingler' else self.assertAlmostEqual(sizes[0],sizes[1],places=6)
 def test_articulations_and_crown_have_actual_solid_joins(self):
  for s,m in self.models.items():
   ps=m['primitives'];body=self.find(ps,'Carapace /');belly=self.find(ps,'Belly /');pairs=[(body,belly)]
   for sign in (-1,1):
    arm=self.find(ps,'Pincer arm /',sign);wrist=self.find(ps,'Wrist /',sign);palm=self.find(ps,'Claw palm /',sign);jaw=self.find(ps,'Claw jaw /',sign)
    pairs += [(body,arm),(arm,wrist),(wrist,palm),(palm,jaw),(belly,self.find(ps,'Cheek peak /',sign))]
   for p in ps:
    if p['part'].startswith('Walking leg /'):pairs.append((belly,p))
    elif p['part'].startswith('Crown /'):pairs.append((body,p))
   for a,b in pairs:self.assertTrue(joins.DonphanTests.overlap(self,a,b),(s,a['part'],b['part']))
 def test_faces_follow_stored_support_and_clear_front(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    n=p['part']
    if n.startswith('Eye white /'):parent=self.find(ps,'Carapace /')
    elif n.startswith('Pupil /'):parent=self.find(ps,'Eye white /',n.split()[-1])
    elif n.startswith('Mouth /'):parent=self.find(ps,'Belly /')
    else:continue
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.020)
    for k in range(0,len(p['indices']),3):
     a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
     x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(s,n))
if __name__=='__main__':unittest.main()
