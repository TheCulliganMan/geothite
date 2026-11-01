#!/usr/bin/env python3
"""Reference-based shape checks over actual stored buffers, independent of pins.

CRYSTAL_FIRE_STARTER_FIDELITY_MODELS can point at immutable baseline GLBs named
cyndaquil, quilava and typhlosion for rejected-geometry checks.
"""
import os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays

MODELS=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models'
OVERRIDE=os.environ.get('CRYSTAL_FIRE_STARTER_FIDELITY_MODELS')
def bounds(p,axis):return min(p['positions'][axis::3]),max(p['positions'][axis::3])
def span(parts,axis):return max(bounds(p,axis)[1] for p in parts)-min(bounds(p,axis)[0] for p in parts)
def top(p):return dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,-z,y)])

class FireStarterFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.heights={}
  for s in ('cyndaquil','quilava','typhlosion'):
   path=Path(OVERRIDE)/(s+'.glb') if OVERRIDE else MODELS/('actor_props/battle_cyndaquil.glb' if s=='cyndaquil' else 'battle_species/'+s+'.glb')
   m=Model(path.read_bytes());m=logical_model(m) if 'source_part_ranges' in m.doc['extras'] else m
   cls.models[s]=m.neutral_model()['primitives'];cls.heights[s]=max(bounds(p,1)[1] for p in cls.models[s])
 def matching(self,s,prefix):return [p for p in self.models[s] if p['part'].startswith(prefix)]
 def find(self,s,prefix):
  ps=self.matching(s,prefix);self.assertTrue(ps,(s,'missing anatomy',prefix));return ps[0]
 def test_cyndaquil_tucks_forepaws_above_two_broad_rear_supports(self):
  s='cyndaquil';arms=self.matching(s,'Forearm /');self.assertEqual(len(arms),2)
  for p in arms:self.assertGreater(bounds(p,1)[0],.14)
  hind=self.matching(s,'Hindleg /');self.assertEqual(len(hind),2);self.assertGreater(span(hind,0),.61)
  for p in hind:self.assertEqual(bounds(p,1)[0],0.);self.assertLess(bounds(p,2)[0],-.50)
 def test_cyndaquil_rear_coat_covers_actual_upper_body_endcap(self):
  s='cyndaquil';body=self.find(s,'Cyndaquil /');coat=self.find(s,'Fitted midnight');zmin=bounds(body,2)[0]
  cap=[y for y,z in zip(body['positions'][1::3],body['positions'][2::3]) if abs(z-zmin)<1e-6];lo,hi=min(cap),max(cap)
  for x in (-.01,.005,.01):
   for f in (.60,.75,.90):
    y=lo+(hi-lo)*f;a,b=rays(body,x,y),rays(coat,x,y);self.assertTrue(a and b);self.assertLess(min(b),min(a)-.0003)
 def test_cyndaquil_four_ignition_spots_follow_the_actual_coat(self):
  s='cyndaquil';spots=self.matching(s,'Dorsal ignition spot /');self.assertEqual(len(spots),4);parent=top(self.find(s,'Fitted midnight'))
  self.assertEqual({tuple(p['part'].split()[-2:]) for p in spots},{(side,str(i)) for side in ('left','right') for i in (0,1)})
  for p in spots:
   side=-1 if p['part'].split()[-2]=='left' else 1;self.assertGreater(side*sum(bounds(p,0))/2,.04);mark=top(p);fronts=0
   for k in range(0,len(mark['indices']),3):
    a,b,c=[mark['positions'][3*i:3*i+3] for i in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    x,y,z=[sum(v[j] for v in (a,b,c))/3 for j in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,p['part']);gap=z-max(hits);self.assertTrue(.0001<gap<.012,(p['part'],gap));fronts+=1
   self.assertGreater(fronts,10)
 def test_quilava_uses_crystal_upright_stance_and_short_free_forearms(self):
  s='quilava';h=self.heights[s];body=self.matching(s,'Body /');self.assertGreater(span(body,1)/span(body,2),1.7)
  feet=self.matching(s,'Foot /');self.assertEqual(len(feet),2)
  for p in feet:self.assertEqual(bounds(p,1)[0],0.)
  arms=self.matching(s,'Arm /');self.assertEqual(len(arms),2)
  for p in arms:self.assertGreater(bounds(p,1)[0],.48*h);self.assertLess(span([p],1),.16*h)
 def test_quilava_rump_fire_stays_low_and_separate_from_neck_fire(self):
  s='quilava';h=self.heights[s];rump=self.matching(s,'Fire / rump');neck=self.matching(s,'Fire / neck');self.assertTrue(rump and neck)
  self.assertLess(max(bounds(p,1)[1] for p in rump),.25*h);self.assertGreater(min(bounds(p,1)[0] for p in neck),.70*h)
  self.assertLess(min(bounds(p,2)[0] for p in rump),-.48*h)
 def test_typhlosion_fire_mantle_fans_widely_across_shoulders(self):
  s='typhlosion';fire=self.matching(s,'Fire /');self.assertGreater(span(fire,0),.90*self.heights[s])
  self.assertEqual(sum(p['part'].endswith(' / ember silhouette') for p in fire),7)
 def test_typhlosion_compact_skull_descends_to_a_low_pointed_nose(self):
  s='typhlosion';heads=self.matching(s,'Head /');self.assertLess(span(heads,2)/span(heads,0),1.10)
  nose=self.find(s,'Nose /');self.assertGreater(max(bounds(p,1)[1] for p in heads)-sum(bounds(nose,1))/2,.15*self.heights[s])
 def test_typhlosion_flared_lower_trunk_and_coat_reach_the_heel(self):
  s='typhlosion';h=self.heights[s];body=self.matching(s,'Body /');coat=self.find(s,'Body / joined midnight');self.assertLess(bounds(coat,1)[0],.02*h)
  widths=[]
  for y in (.16*h,.65*h):
   slices=[section(p,y) for p in body];widths.append(max(hi[0] for lo,hi in slices)-min(lo[0] for lo,hi in slices))
  self.assertGreater(widths[0]/widths[1],1.35)
 def test_typhlosion_short_forearms_keep_palms_by_the_upper_body(self):
  s='typhlosion';h=self.heights[s]
  for p in self.matching(s,'Arm /'):self.assertLess(span([p],1),.22*h)
  hands=self.matching(s,'Hand /');self.assertEqual(len(hands),2)
  for p in hands:self.assertGreater(sum(bounds(p,1))/2,.44*h)

if __name__=='__main__':unittest.main()
