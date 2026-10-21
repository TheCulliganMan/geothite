#!/usr/bin/env python3
"""Stored-volume guards for the electric and live-coal posture corrections."""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds,indexed_ray_intersections as rays

ROOT=Path(os.environ.get('CRYSTAL_ELEMENTAL_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
HEIGHTS={'elekid':.91,'electabuzz':1.28,'magby':.77,'magmar':1.30}

def bounds(p):
 return [(min(p['positions'][k::3]),max(p['positions'][k::3])) for k in range(3)]

class ElementalFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={}
  for name in HEIGHTS:
   runtime=Model((ROOT/(name+'.glb')).read_bytes())
   cls.models[name]=logical_model(runtime) if 'source_part_ranges' in runtime.doc['extras'] else runtime
 def part(self,name,prefix,side=None):
  return next(p for p in self.models[name].neutral_model()['primitives'] if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+str(side))))
 def test_actual_shin_sections_are_upright_in_both_profiles(self):
  for name,height in HEIGHTS.items():
   for side in (-1,1):
    p=self.part(name,'Shin band /' if name=='magmar' else 'Leg /',side);low,high=bounds(p)[1];centres=[]
    for f in (.25,.5,.75):
     a,b=horizontal_section_bounds(p,low+f*(high-low));centres.append((a[1]+b[1])/2)
    self.assertLess(max(centres)-min(centres),.008*height,(name,side,'shaft pitches toward the toes'))
 def test_electric_bodies_have_round_front_and_profile_mass(self):
  for name in ('elekid','electabuzz'):
   b=bounds(self.part(name,'Body /'));width=b[0][1]-b[0][0];height=b[1][1]-b[1][0];depth=b[2][1]-b[2][0]
   self.assertGreater(width/height,.90 if name=='elekid' else .88,name)
   self.assertGreater(depth/height,.73 if name=='elekid' else .70,name)
 def test_elekid_has_alternating_fists_and_two_complete_waist_bands(self):
  a=bounds(self.part('elekid','Hand /',-1))[1];b=bounds(self.part('elekid','Hand /',1))[1]
  self.assertGreater((sum(a)-sum(b))/2,.28*HEIGHTS['elekid'])
  bands=[p for p in self.models['elekid'].neutral_model()['primitives'] if p['part'].startswith('Belly band /')]
  self.assertEqual(len(bands),2)
  for band in bands:
   self.assertLess(bounds(band)[2][0],-.10);self.assertGreater(bounds(band)[2][1],.10)
 def test_electabuzz_both_fists_are_beside_its_head(self):
  head=bounds(self.part('electabuzz','Head /'));body=bounds(self.part('electabuzz','Body /'))
  for side in (-1,1):
   hand=bounds(self.part('electabuzz','Hand /',side));self.assertGreater(sum(hand[1])/2,sum(body[1])/2+.28*HEIGHTS['electabuzz'])
   self.assertGreater(hand[1][1],head[1][0]);self.assertLess(hand[1][0],head[1][1])
 def test_magby_palms_are_low_and_pear_widens_at_hips(self):
  name='magby';head=bounds(self.part(name,'Head /'));body=self.part(name,'Body /');low,high=bounds(body)[1]
  for side in (-1,1):self.assertLess(bounds(self.part(name,'Hand /',side))[1][1],head[1][0]+.01*HEIGHTS[name])
  hip=horizontal_section_bounds(body,low+.24*(high-low));chest=horizontal_section_bounds(body,low+.79*(high-low))
  self.assertGreater((hip[1][0]-hip[0][0])/(chest[1][0]-chest[0][0]),1.4)
 def test_magmar_flames_have_profile_volume_and_lower_torso_is_yellow_all_around(self):
  name='magmar';height=HEIGHTS[name]
  for prefix,side in (('Crown flame /',-1),('Crown flame /',1),('Tail flame /',None)):
   flame=self.part(name,prefix,side);low,high=bounds(flame)[1];a,b=horizontal_section_bounds(flame,low+.38*(high-low))
   self.assertGreater(b[1]-a[1],.065*height,(prefix,side,'flame is a flat bank'))
  body=self.part(name,'Body /');field=self.part(name,'Belly /');low,high=bounds(body)[1];y=low+.25*(high-low)
  for x in (-.08,0.,.08):
   tissue=sorted(rays(body,x,y));color=sorted(rays(field,x,y));self.assertGreaterEqual(len(color),4)
   self.assertLess(color[0],tissue[0]);self.assertGreater(color[-1],tissue[-1])
   self.assertLess(tissue[0]-color[0],.012*height);self.assertLess(color[-1]-tissue[-1],.012*height)
  self.assertLess(bounds(field)[1][0],low)

if __name__=='__main__':unittest.main()
