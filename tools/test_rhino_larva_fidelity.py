#!/usr/bin/env python3
"""Independent stored-volume guards for the rhino/larva reference audit.

CRYSTAL_RHINO_LARVA_FIDELITY_MODELS can point at an immutable earlier batch;
these measurements deliberately do not require the current exporter's pin.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section
from sculpt_test_support import indexed_ray_intersections as rays,parts_overlap
from test_steelix_sculpt import occupied

ROOT=Path(os.environ.get('CRYSTAL_RHINO_LARVA_FIDELITY_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

def bounds(p,axis):return min(p['positions'][axis::3]),max(p['positions'][axis::3])
def middle(p,axis):return sum(bounds(p,axis))/2
def vertical(p):return dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,z,y)])

class RhinoLarvaFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:logical_model(Model((ROOT/(s+'.glb')).read_bytes())).neutral_model() for s in ('rhyhorn','rhydon','larvitar')}
 def find(self,s,prefix,suffix=None):
  found=[p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix)))]
  self.assertTrue(found,(s,prefix,suffix,'missing anatomical volume'))
  return found[0]
 def height(self,s):return max(bounds(p,1)[1] for p in self.models[s]['primitives'])
 def test_rhyhorn_shields_wrap_both_flanks_with_real_depth(self):
  s='rhyhorn';h=self.height(s)
  for p in self.models[s]['primitives']:
   if not p['part'].startswith('Back armor /'):continue
   self.assertLess(bounds(p,1)[0],.35*h)
   lo,hi=section(p,.38*h)
   self.assertLess(lo[0],-.20*h);self.assertGreater(hi[0],.20*h)
   self.assertGreater(hi[1]-lo[1],.15*h)
 def test_rhyhorn_wedge_has_fangs_and_a_real_backed_mouth(self):
  s='rhyhorn';h=self.height(s);head=self.find(s,'Head /');jaw=self.find(s,'Jaw /');back=self.find(s,'Mouth back /')
  fangs=[p for p in self.models[s]['primitives'] if p['part'].startswith('Fang /')];self.assertEqual(len(fangs),2)
  for p in fangs:self.assertTrue(parts_overlap(p,head))
  for x in (-.025*h,0.,.025*h):
   for z in (bounds(head,2)[1]*.84,bounds(head,2)[1]*.89):
    low=rays(vertical(jaw),x,z);high=rays(vertical(head),x,z);self.assertTrue(low and high)
    a,b=max(low),min(high);self.assertGreater(b-a,.006*h)
    point=(x,(a+b)/2,z);self.assertFalse(any(occupied(p,point) for p in self.models[s]['primitives']))
    lining=rays(back,x,point[1]);self.assertTrue(lining);self.assertLess(max(lining),z-.08*h)
 def test_rhyhorn_dorsal_crests_peak_instead_of_forming_flat_roofs(self):
  s='rhyhorn';h=self.height(s)
  for p in self.models[s]['primitives']:
   if not p['part'].startswith('Back armor /'):continue
   points=list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]));peak=max(points,key=lambda q:q[1]);v=vertical(p)
   for z in (peak[2]-.06*h,peak[2]+.06*h):
    hits=rays(v,0.,z);self.assertTrue(hits);self.assertGreater(peak[1]-max(hits),.03*h)
 def test_rhydon_resting_hands_are_lower_and_asymmetric(self):
  s='rhydon';h=self.height(s);ys=[]
  for side in (-1,1):
   hand=self.find(s,'Hand /',side);shoulder=self.find(s,'Shoulder /',side)
   self.assertLess(middle(hand,1),middle(shoulder,1)-.06*h)
   self.assertGreater(middle(hand,2),middle(shoulder,2)+.10*h);ys.append(middle(hand,1))
  self.assertGreater(abs(ys[0]-ys[1]),.06*h)
 def test_rhydon_shin_shafts_stand_over_complete_heels(self):
  s='rhydon';h=self.height(s)
  for side in (-1,1):
   leg=self.find(s,'Leg /',side);centers=[]
   for y in (.085*h,.145*h):
    lo,hi=section(leg,y);centers.append((lo[1]+hi[1])/2)
   self.assertLess(abs(centers[1]-centers[0])/(.06*h),.20)
   foot=self.find(s,'Foot /',side);lo,hi=section(foot,.012*h)
   self.assertLess(lo[1],min(centers)-.035*h)
   self.assertGreater(hi[0]-lo[0],.14*h)
 def test_rhydon_nasal_drill_projects_forward_from_the_skull(self):
  s='rhydon';horn=self.find(s,'Horn /');v=list(zip(horn['positions'][::3],horn['positions'][1::3],horn['positions'][2::3]));zl,zh=bounds(horn,2);span=zh-zl
  groups=[[p for p in v if (p[2]-zl)/span<.10],[p for p in v if (p[2]-zl)/span>.90]]
  a,b=[tuple(sum(p[k] for p in group)/len(group) for k in range(3)) for group in groups]
  self.assertGreater(b[2]-a[2],.15*self.height(s));self.assertLess((b[1]-a[1])/(b[2]-a[2]),.65)
 def test_rhydon_open_mouth_is_between_current_lips_with_occupied_lining(self):
  s='rhydon';h=self.height(s);upper=self.find(s,'Muzzle /');lower=self.find(s,'Jaw /');back=self.find(s,'Mouth back /');front=bounds(upper,2)[1]
  for x in (-.035*h,0.,.035*h):
   for z in (front*.72,front*.79):
    a,b=rays(vertical(lower),x,z),rays(vertical(upper),x,z);self.assertTrue(a and b)
    floor,ceiling=max(a),min(b);self.assertTrue(.006*h<ceiling-floor<.065*h)
    point=(x,floor+(ceiling-floor)*.75,z)
    self.assertFalse(any(occupied(p,point) for p in self.models[s]['primitives']))
    lining=rays(back,x,point[1]);self.assertTrue(lining);self.assertLess(max(lining),z-.06*h)
 def test_larvitar_full_skull_and_shorter_crown_replace_the_flat_bill(self):
  s='larvitar';h=self.height(s);head=self.find(s,'Head /');horn=self.find(s,'Horn /');head_height=bounds(head,1)[1]-bounds(head,1)[0]
  self.assertGreater(head_height,.37*h)
  self.assertLess((bounds(horn,1)[1]-bounds(horn,1)[0])/head_height,.70)
  lo,hi=section(head,.64*h);self.assertGreater(hi[1]-lo[1],.27*h)
 def test_larvitar_forelimbs_are_short_tapered_stubs(self):
  s='larvitar';h=self.height(s);names=[p['part'] for p in self.models[s]['primitives']]
  self.assertFalse(any(n.startswith(('Hand /','Finger /')) for n in names))
  for side in (-1,1):
   arm=self.find(s,'Arm /',side)
   self.assertLess(bounds(arm,2)[1]-bounds(arm,2)[0],.12*h)
   points=list(zip(arm['positions'][::3],arm['positions'][1::3],arm['positions'][2::3]));outer=max(side*p[0] for p in points)
   tip=[p for p in points if side*p[0]>outer-.012*h]
   self.assertLess(max(p[1] for p in tip)-min(p[1] for p in tip),.04*h)
   self.assertTrue(parts_overlap(arm,self.find(s,'Body /')))
 def test_larvitar_short_radial_fan_has_downward_as_well_as_upward_teeth(self):
  s='larvitar';h=self.height(s);hub=self.find(s,'Tail hub /');cx,cy=middle(hub,0),middle(hub,1);teeth=[p for p in self.models[s]['primitives'] if p['part'].startswith('Tail prong /')]
  self.assertEqual(len(teeth),6);angles=[]
  for p in teeth:
   points=list(zip(p['positions'][::3],p['positions'][1::3]));tip=max(points,key=lambda q:math.hypot(q[0]-cx,q[1]-cy));angles.append(math.atan2(tip[1]-cy,tip[0]-cx)%math.tau)
   self.assertTrue(parts_overlap(p,hub))
  angles.sort();gaps=[(b-a)%math.tau for a,b in zip(angles,angles[1:]+angles[:1])]
  self.assertTrue(all(.70<g<1.40 for g in gaps),gaps)
  self.assertLess(min(bounds(p,1)[0] for p in teeth),cy-.12*h);self.assertGreater(max(bounds(p,1)[1] for p in teeth),cy+.12*h)
  self.assertLess(bounds(self.find(s,'Body /'),2)[0]-min(bounds(p,2)[0] for p in teeth),.30*h)

if __name__=='__main__':unittest.main()
