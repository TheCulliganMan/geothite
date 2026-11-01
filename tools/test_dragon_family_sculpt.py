#!/usr/bin/env python3
"""Stored dragon-family anatomy, continuous skin support, coils and wing roots."""
import json,os,unittest
from pathlib import Path
from dragon_family_sculpt import sculpture,SPECIES,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
ROOT=Path(os.environ.get('CRYSTAL_DRAGON_FAMILY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
from sculpt_test_support import surface_distances

class DragonFamilyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from dragon_family_glb import read_dragon_family
  cls.ms={s:read_dragon_family(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.ms[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:return True
  return False
 def test_closed_connected_outward_parts(self):assert_closed_models(self,self.ms)
 def test_finite_unit_normals_follow_actual_faces(self):assert_model_shading(self,self.ms)
 def test_recipe_parity_height_grounding_and_budget(self):
  for s,m in self.ms.items():
   from dragon_family_glb import geometry_digest
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
   self.assertLessEqual(sum(len(p['positions'])//3 for p in m['primitives']),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in m['primitives']),40000);self.assertEqual(len(m['primitives']),len({p['part'] for p in m['primitives']}))
 def test_species_specific_forehead_orbs_antennae_wings_and_digits(self):
  for s,counts in {'dratini':{'Forehead /':1,'Horn /':0,'Ear fin /':2,'Orb /':0},'dragonair':{'Forehead /':0,'Horn /':1,'Ear fin /':2,'Orb /':3},'dragonite':{'Horn /':1,'Antenna /':2,'Wing /':2,'Wing border /':2,'Wing spar /':2,'Wing rib /':4,'Belly fold /':8,'Foot claw /':6,'Hand claw /':6}}.items():
   for prefix,n in counts.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ms[s]['primitives']),n,(s,prefix))
 def test_serpent_bodies_and_guardian_tail_have_no_crossed_faces(self):
  for s in ('dratini','dragonair'):self.assertIsNone(crossed_faces(self.find(s,'Body /')),s)
  self.assertIsNone(crossed_faces(self.find('dragonite','Tail /')))
 def test_heads_muzzles_forehead_ears_and_three_pearls_have_real_roots(self):
  for s in SPECIES:
   body=self.find(s,'Body /');head=self.find(s,'Head /');self.assertTrue(self.overlap(body,head),s);self.assertTrue(self.overlap(head,self.find(s,'Muzzle /')),s)
   for p in self.ms[s]['primitives']:
    if p['part'].startswith(('Horn /','Antenna /','Ear fin /')):self.assertTrue(self.overlap(p,head),(s,p['part']))
    if p['part'].startswith('Orb /'):self.assertTrue(self.overlap(p,body),p['part'])
 def pairs(self):
  for s,m in self.ms.items():
   for p in m['primitives']:
    name=p['part'];last=name.split()[-1];parent=None
    if name.startswith(('Eye socket /','Forehead /')):parent=self.find(s,'Head /')
    elif name.startswith('Iris /'):parent=self.find(s,'Eye socket /',last)
    elif name.startswith('Eye white /'):parent=self.find(s,'Eye socket /',last)
    elif name.startswith('Pupil /'):parent=self.find(s,'Eye white /' if s=='dragonite' else 'Iris /',last)
    elif name.startswith('Glint /'):parent=self.find(s,'Pupil /',last)
    elif name.startswith(('Mouth /','Nostril /')):parent=self.find(s,'Muzzle /')
    elif name.startswith('Orb glint /'):parent=self.find(s,'Orb /',last)
    elif name.startswith('Belly fold /'):parent=self.find(s,'Belly /')
    elif name.startswith('Belly /') and s=='dragonite':parent=self.find(s,'Body /')
    if parent:yield s,p,parent
 def test_fitted_faces_orb_glints_and_plate_seams_have_actual_support(self):
  for s,p,parent in self.pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    h=ray_intersections(parent,x,y);self.assertTrue(h,(s,p['part']));self.assertLess(abs(z-max(h)),.019,(s,p['part']))
 def test_fitted_front_triangles_clear_actual_stored_parents(self):
  for s,p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(0,.5,.5),(.5,0,.5)):
     x,y,z=[sum(v[q]*w for v,w in zip((a,b,c),weights)) for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertGreater(z-max(hits),.0001,(s,p['part'],k//3,weights))
 def test_closed_ventral_ribbons_follow_actual_three_dimensional_tubes(self):
  for s in SPECIES:
   ribbon=self.find(s,'Tail underside /' if s=='dragonite' else 'Belly /');parent=self.find(s,'Tail /' if s=='dragonite' else 'Body /');self.assertTrue(self.overlap(ribbon,parent),s)
   points={tuple(ribbon['positions'][i:i+3]) for i in range(0,len(ribbon['positions']),3)}
   for gap in surface_distances(parent,points,.007):self.assertLess(gap,.007,s)
 def test_detached_ventral_skin_is_rejected_by_actual_surface_distance(self):
  body=self.find('dratini','Body /');ribbon=self.find('dratini','Belly /')
  points=[tuple(ribbon['positions'][i+k]+(HEIGHTS['dratini']*2 if k==2 else 0) for k in range(3)) for i in range(0,len(ribbon['positions']),57)]
  self.assertTrue(all(gap>.007 for gap in surface_distances(body,points,.007)))
 def test_guardian_arms_hands_claws_and_grounded_legs_are_rooted(self):
  s='dragonite';body=self.find(s,'Body /')
  for sign in (-1,1):
   arm=self.find(s,'Arm /',sign);hand=self.find(s,'Hand /',sign);thigh=self.find(s,'Haunch /',sign);leg=self.find(s,'Leg /',sign);foot=self.find(s,'Foot /',sign)
   for a,b in ((body,arm),(arm,hand),(body,thigh),(thigh,leg),(leg,foot)):self.assertTrue(self.overlap(a,b),(a['part'],b['part']))
   self.assertEqual(min(foot['positions'][1::3]),0.)
   for p in self.ms[s]['primitives']:
    if p['part'].startswith(('Hand claw /','Foot claw /')) and p['part'].split()[-2]==str(sign):self.assertTrue(self.overlap(p,hand if p['part'].startswith('Hand') else foot),p['part'])
  self.assertTrue(self.overlap(body,self.find(s,'Tail /')))
 def test_guardian_teal_membranes_and_orange_bones_share_roots(self):
  s='dragonite'
  for sign in (-1,1):
   border=self.find(s,'Wing border /',sign);wing=self.find(s,'Wing /',sign);spar=self.find(s,'Wing spar /',sign)
   self.assertTrue(self.overlap(border,self.find(s,'Body /')));self.assertTrue(self.overlap(wing,border));self.assertTrue(self.overlap(spar,border));self.assertTrue(self.overlap(spar,self.find(s,'Body /')))
   for p in self.ms[s]['primitives']:
    if p['part'].startswith('Wing rib /') and p['part'].split()[-2]==str(sign):self.assertTrue(self.overlap(p,wing),p['part']);self.assertTrue(self.overlap(p,spar),p['part'])
if __name__=='__main__':unittest.main()
