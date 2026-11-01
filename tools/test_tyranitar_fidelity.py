#!/usr/bin/env python3
"""Stored-volume regressions for Tyranitar's reference-based posture revision.

The immutable baseline can be tested through CRYSTAL_TYRANITAR_FIDELITY_MODELS;
these checks decode its real buffers without requiring a current authoring pin.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section,parts_overlap
from sculpt_test_support import indexed_ray_intersections as rays
from test_steelix_sculpt import occupied

ROOT=Path(os.environ.get('CRYSTAL_TYRANITAR_FIDELITY_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def bounds(p,axis):return min(p['positions'][axis::3]),max(p['positions'][axis::3])
def middle(p,axis):return sum(bounds(p,axis))/2
def vertical(p):return dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,z,y)])
def shaft_axis(p):
 # Short thick frustums can yield misleading horizontal bounds where a slice
 # cuts an end cap. The largest actual planar patches reveal the shaft axis.
 vertices=list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]));groups={}
 for k in range(0,len(p['indices']),3):
  a,b,c=[vertices[j] for j in p['indices'][k:k+3]];u=[b[i]-a[i] for i in range(3)];w=[c[i]-a[i] for i in range(3)]
  n=[u[1]*w[2]-u[2]*w[1],u[2]*w[0]-u[0]*w[2],u[0]*w[1]-u[1]*w[0]];area=math.sqrt(sum(x*x for x in n))
  key=tuple(round(x/area*(1 if n[1]>=0 else -1),3) for x in n)
  total,count=groups.get(key,(0.,0));groups[key]=(total+area,count+1)
 axis,(area,count)=max(groups.items(),key=lambda item:item[1][0]);return axis,count

class TyranitarFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.runtime=logical_model(Model((ROOT/'tyranitar.glb').read_bytes()));cls.model=cls.runtime.neutral_model();cls.parts=cls.model['primitives'];cls.h=max(bounds(p,1)[1] for p in cls.parts)
 def matching(self,prefix):return [p for p in self.parts if p['part'].startswith(prefix)]
 def find(self,prefix,side=None):
  ps=[p for p in self.matching(prefix) if side is None or p['part'].endswith(' '+str(side))];self.assertTrue(ps,(prefix,side));return ps[0]
 def test_ankle_shafts_stand_over_full_heel_soles(self):
  for side in (-1,1):
   shin=self.find('Shin /',side);centers=[]
   for y in (.075*self.h,.11*self.h):
    lo,hi=section(shin,y);centers.append((lo[1]+hi[1])/2)
   axis,count=shaft_axis(shin);self.assertGreater(count,8);self.assertGreater(axis[1],.5)
   self.assertLess(abs(axis[2]/axis[1]),.20)
   foot=self.find('Foot /',side);lo,hi=section(foot,.008*self.h)
   self.assertLess(lo[1],min(centers)-.04*self.h);self.assertGreater(hi[0]-lo[0],.14*self.h)
 def test_body_has_a_broad_chest_and_deep_lower_barrel(self):
  body=self.find('Body /');lo,hi=section(body,.57*self.h)
  self.assertGreater(hi[0]-lo[0],.30*self.h)
  lo,hi=section(body,.38*self.h);self.assertGreater(hi[1]-lo[1],.355*self.h)
 def test_compact_muzzle_and_descending_brow_replace_the_long_bill(self):
  head=self.find('Head /');self.assertLess(bounds(head,2)[1],.22*self.h)
  keel=self.find('Brow keel /');self.assertTrue(parts_overlap(keel,head))
  self.assertGreater(bounds(keel,1)[1]-bounds(keel,1)[0],.09*self.h)
 def test_short_forelimbs_keep_hands_beside_the_chest(self):
  for side in (-1,1):
   hand=self.find('Hand /',side);self.assertLess(middle(hand,2),.14*self.h)
   self.assertLess(middle(hand,1),.44*self.h);self.assertGreater(side*middle(hand,0),.25*self.h)
   claws=[p for p in self.matching('Hand claw /') if int(p['part'].split()[-2])==side];self.assertEqual(len(claws),3)
   self.assertLess(max(bounds(p,2)[1] for p in claws)-middle(hand,2),.065*self.h)
 def test_dorsal_armor_has_two_complete_volumetric_rows(self):
  spikes=self.matching('Back spike /');self.assertEqual(len(spikes),8);body=self.find('Body /')
  for side in (-1,1):
   row=[p for p in spikes if side*middle(p,0)>.08*self.h];self.assertEqual(len(row),4)
   for p in row:
    self.assertTrue(parts_overlap(p,body));self.assertGreater(bounds(p,0)[1]-bounds(p,0)[0],.06*self.h)
    self.assertGreater(bounds(p,2)[1]-bounds(p,2)[0],.13*self.h)
 def test_tail_stays_low_with_a_radial_terminal_crown(self):
  tail=self.find('Tail /');self.assertLess(bounds(tail,1)[1],.34*self.h)
  spikes=self.matching('Tail spike /');self.assertEqual(len(spikes),4);hub=self.find('Tail hub /');self.assertTrue(parts_overlap(hub,tail))
  # Derive the terminal axis from the stored tail's far cross-section.
  rotated=dict(tail,positions=[v for x,y,z in zip(tail['positions'][::3],tail['positions'][1::3],tail['positions'][2::3]) for v in (x,z,y)])
  lo,hi=section(rotated,-.67*self.h);cx,cy=(lo[0]+hi[0])/2,(lo[1]+hi[1])/2;angles=[]
  base_lo,base_hi=section(rotated,-.25*self.h)
  self.assertLess(middle(hub,1),(base_lo[1]+base_hi[1])/2-.005*self.h)
  cap_width=max(bounds(p,0)[1] for p in spikes)-min(bounds(p,0)[0] for p in spikes)
  self.assertGreater(cap_width,(base_hi[0]-base_lo[0])*.85)
  for p in spikes:
   self.assertLess(middle(p,2),-.61*self.h);self.assertTrue(parts_overlap(p,tail))
   vertices=list(zip(p['positions'][::3],p['positions'][1::3]));tip=max(vertices,key=lambda q:math.hypot(q[0]-cx,q[1]-cy))
   angles.append(math.atan2(tip[1]-cy,tip[0]-cx)%math.tau)
  angles.sort();gaps=[(b-a)%math.tau for a,b in zip(angles,angles[1:]+angles[:1])]
  self.assertTrue(all(1.10<g<2.1 for g in gaps),gaps)
  self.assertLess(bounds(hub,2)[1]-bounds(hub,2)[0],.09*self.h)
 def test_paired_chest_and_thigh_vents_and_rear_openings_are_retained(self):
  self.assertEqual(len(self.matching('Torso vent /')),4);self.assertEqual(len(self.matching('Thigh vent /')),4);self.assertEqual(len(self.matching('Back vent /')),2)
  for prefix in ('Torso vent /','Thigh vent /'):
   for side in (-1,1):
    ps=[p for p in self.matching(prefix) if side*middle(p,0)>0];self.assertEqual(len(ps),2)
    self.assertGreater(abs(middle(ps[0],1)-middle(ps[1],1)),.05*self.h)
 def test_belly_is_a_long_diamond_with_three_narrow_segment_seams(self):
  belly=self.find('Belly /');w=bounds(belly,0)[1]-bounds(belly,0)[0];height=bounds(belly,1)[1]-bounds(belly,1)[0]
  self.assertGreater(height/w,1.45);self.assertEqual(len(self.matching('Belly armor /')),3)
  for p in self.matching('Belly armor /'):self.assertLess(bounds(p,1)[1]-bounds(p,1)[0],.01*self.h)
 def test_empty_mouth_samples_are_between_actual_lips_and_backed_by_a_throat(self):
  head=self.find('Head /');jaw=self.find('Jaw /');back=self.find('Mouth back /');front=bounds(head,2)[1]
  for x in (-.015*self.h,0.,.015*self.h):
   for z in (front*.72,front*.79):
    a,b=rays(vertical(jaw),x,z),rays(vertical(head),x,z);self.assertTrue(a and b)
    low,high=max(a),min(b);self.assertGreater(high-low,.008*self.h)
    point=(x,low+(high-low)*.75,z);self.assertFalse(any(occupied(p,point) for p in self.parts))
    backing=rays(back,x,point[1]);self.assertTrue(backing);self.assertLess(max(backing),z-.035*self.h)

if __name__=='__main__':unittest.main()
