#!/usr/bin/env python3
"""Stored Gligar anatomy, hooked gaps, sail roots and fitted face checks."""
import json,math,unittest
from pathlib import Path
from gligar_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class GligarTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'gligar.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,{'gligar':self.m})
 def test_finite_unit_normals_follow_winding(self):assert_model_shading(self,{'gligar':self.m})
 def test_recipe_parity_height_budget_and_distinct_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Ear /',2),('Pincer /',2),('Wing /',2),('Wing spar /',2),('Foot claw /',2),('Fang /',2),('Tongue /',1),('Stinger /',1)):
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
  for p in self.ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith(('Eye /','Mouth /')):parent=self.find('Head /')
   elif n.startswith('Pupil /'):parent=self.find('Eye / broad ivory triangular glider eye '+sign)
   elif n.startswith('Eye glint /'):parent=self.find('Pupil / narrow dark vertical glider slit '+sign)
   elif n.startswith('Tongue crease /'):parent=self.find('Tongue /')
   if parent:yield p,parent
 def test_facial_vertices_follow_actual_stored_support(self):
  for p,parent in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,(p['part'],x,y));self.assertLess(abs(z-max(hits)),.015)
 def test_front_facial_triangles_clear_parent_surfaces(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_ears_head_arms_and_hooked_pincers_have_real_roots(self):
  body=self.find('Body /');head=self.find('Head /');self.assertTrue(self.overlap(body,head))
  for sign in (-1,1):
   self.assertTrue(self.overlap(self.find('Ear / long swept tapered scorpion point '+str(sign)),head))
   arm=self.find('Arm / curved jointed pincer support '+str(sign));claw=self.find('Pincer / thick hooked crescent claw '+str(sign))
   self.assertTrue(self.overlap(arm,body));self.assertTrue(self.overlap(arm,claw),claw['part'])
 def test_both_pincers_have_actual_open_centers_and_open_jaws(self):
  for sign in (-1,1):
   claw=self.find('Pincer / thick hooked crescent claw '+str(sign));cx=(max(claw['positions'][::3])+min(claw['positions'][::3]))/2;cy=(max(claw['positions'][1::3])+min(claw['positions'][1::3]))/2
   self.assertFalse(ray_intersections(claw,cx,cy))
   angle=.27 if sign<0 else -1.10
   for t in (.04,.08,.12):self.assertFalse(ray_intersections(claw,cx-t*math.sin(angle),cy+t*math.cos(angle)),(sign,t))
 def test_scalloped_blue_sails_and_upper_struts_are_attached(self):
  body=self.find('Body /')
  for sign in (-1,1):
   wing=self.find('Wing / scalloped blue gliding sail '+str(sign));spar=self.find('Wing spar / pink upper sail strut '+str(sign));arm=self.find('Arm / curved jointed pincer support '+str(sign))
   self.assertTrue(self.overlap(wing,body));self.assertTrue(self.overlap(spar,wing));self.assertTrue(self.overlap(spar,arm))
 def test_grounded_feet_and_curved_talons_join_bent_legs(self):
  body=self.find('Body /')
  for sign in (-1,1):
   leg=self.find('Leg / bent pink rear haunch '+str(sign));foot=self.find('Foot / grounded rounded scorpion pad '+str(sign));claw=self.find('Foot claw / pointed curved forward talon '+str(sign))
   self.assertTrue(self.overlap(leg,body));self.assertTrue(self.overlap(leg,foot));self.assertTrue(self.overlap(foot,claw));self.assertEqual(min(foot['positions'][1::3]),0.)
 def test_tongue_fangs_and_complete_curled_venom_tail_are_rooted(self):
  head=self.find('Head /');tongue=self.find('Tongue /');self.assertTrue(self.overlap(tongue,head));self.assertTrue(self.overlap(tongue,self.find('Mouth /')))
  for p in self.ps:
   if p['part'].startswith('Fang /'):self.assertTrue(self.overlap(p,head))
  tail=self.find('Tail /');bulb=self.find('Tail bulb /');stinger=self.find('Stinger /')
  self.assertTrue(self.overlap(tail,self.find('Body /')));self.assertTrue(self.overlap(tail,bulb));self.assertTrue(self.overlap(bulb,stinger))
if __name__=='__main__':unittest.main()
