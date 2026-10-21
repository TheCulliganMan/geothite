"""Reference-volume checks over actual decoded meshes, without author pins.

CRYSTAL_MANTIS_SLEEPER_FIDELITY_MODELS selects named immutable baselines or
candidate GLBs for rejected-geometry controls.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays,crossed_faces
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
MODELS=Path(os.environ.get('CRYSTAL_MANTIS_SLEEPER_FIDELITY_MODELS',str(ROOT)))
def points(p):return list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
def bounds(p,k):return min(p['positions'][k::3]),max(p['positions'][k::3])
def span(p,k):a,b=bounds(p,k);return b-a
def mean(p,k):return sum(bounds(p,k))/2
class MantisSleeperFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.parts={}
  for s in ('scyther','snorlax'):
   m=Model((MODELS/(s+'.glb')).read_bytes());m=logical_model(m) if 'source_part_ranges' in m.doc['extras'] else m
   cls.models[s]=m;cls.parts[s]=m.neutral_model()['primitives']
 def matching(self,s,prefix):return [p for p in self.parts[s] if p['part'].startswith(prefix)]
 def find(self,s,prefix):
  ps=self.matching(s,prefix);self.assertTrue(ps,(s,'missing anatomy',prefix));return ps[0]
 def test_scyther_blades_sweep_forward_in_depth_and_retain_real_thickness(self):
  for p in self.matching('scyther','Scythe /'):
   self.assertGreater(span(p,2),.55);self.assertLess(span(p,0),.30)
   side=-1 if p['part'].endswith('-1') else 1;a=math.radians(20.);normal=(-math.cos(a),0.,side*math.sin(a));d=[sum(x*y for x,y in zip(v,normal)) for v in points(p)]
   self.assertTrue(.025<max(d)-min(d)<.075,(p['part'],'blade cross-section'))
 def test_scyther_wings_have_backward_reach_and_broad_profiles(self):
  ps=self.matching('scyther','Wing case /');self.assertEqual(len(ps),4)
  for p in ps:
   self.assertLess(bounds(p,2)[0],-.63);self.assertGreater(span(p,2),.55);self.assertGreater(span(p,1),.17)
   vs=points(p);area=0.
   for k in range(0,len(p['indices']),3):
    a,b,c=[vs[j] for j in p['indices'][k:k+3]];area+=abs((b[1]-a[1])*(c[2]-a[2])-(b[2]-a[2])*(c[1]-a[1]))/2
   self.assertGreater(area,.055,'a long narrow profile bounds box does not establish a broad wing')
 def test_scyther_has_three_short_green_crown_points_above_a_deep_skull(self):
  head=self.find('scyther','Head /');crests=self.matching('scyther','Crest /');self.assertEqual(len(crests),3)
  center=min(crests,key=lambda p:abs(mean(p,0)));self.assertLess(bounds(center,1)[1]-bounds(head,1)[1],.10)
  r,g,b,_=center['base_color'];self.assertGreater(g,3*r);self.assertGreater(span(head,2),.28)
  self.assertGreater(max(bounds(p,0)[1] for p in crests)-min(bounds(p,0)[0] for p in crests),.40)
 def test_scyther_lower_greave_axes_are_upright_and_separate_from_toes(self):
  for p in self.matching('scyther','Greave /'):
   centers=[]
   for y in (.09,.21):
    lo,hi=section(p,y);centers.append([(a+b)/2 for a,b in zip(lo,hi)])
   self.assertLess(abs(centers[0][1]-centers[1][1]),.014,'forward-sloping greave')
 def test_scyther_all_three_abdominal_seams_wrap_the_rear_shell(self):
  body=self.find('scyther','Abdomen /');seams=self.matching('scyther','Abdomen seam /');self.assertEqual(len(seams),3)
  for p in seams:
   lo,hi=section(body,mean(p,1));self.assertLess(bounds(p,2)[0],lo[1]-.0003);self.assertGreater(bounds(p,2)[1],hi[1]+.0003)
 def test_snorlax_has_a_deep_rounded_seated_barrel(self):
  body=self.find('snorlax','Body /');lo,hi=section(body,.46);self.assertGreater(hi[1]-lo[1],1.0);self.assertGreater(hi[0]-lo[0],1.20)
 def test_snorlax_hood_has_real_mass_in_front_and_profile(self):
  body=self.find('snorlax','Body /');head=self.find('snorlax','Head /');self.assertGreater(span(head,0)/span(body,0),.62);self.assertGreater(span(head,2),.54)
 def test_snorlax_bends_its_resting_paws_toward_the_upper_belly(self):
  for p in self.matching('snorlax','Hand /'):
   self.assertGreater(mean(p,2),.23);self.assertGreater(mean(p,1),.71);self.assertLess(abs(mean(p,0)),.65)
 def test_snorlax_has_large_soles_and_full_round_brown_pads(self):
  for p in self.matching('snorlax','Foot /'):self.assertGreater(span(p,0),.43);self.assertGreater(span(p,2),.43)
  for p in self.matching('snorlax','Foot pad /'):self.assertGreater(span(p,0),.24);self.assertGreater(span(p,1),.19)
 def test_snorlax_head_ear_and_arm_root_caps_remain_in_occupied_tissue(self):
  m=self.models['snorlax'];head=next(i for i,n in enumerate(m.names) if n.startswith('Head /'));body=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
  for clip,t in ((None,0.),('idle',.25),('attack',.18),('hit',.31)):
   mats=m.matrices(clip,t);posed=[dict(positions=sum(m.posed(i,mats),[]),indices=m.indices[i]) for i in range(len(m.names))]
   for i,n in enumerate(m.names):
    if n.startswith('Head /'):parents=[body]
    elif n.startswith('Ear /'):parents=[head]
    elif n.startswith('Arm /'):parents=[body,head]
    else:continue
    vs=m.vertices[i];a,b,c=[vs[k] for k in m.indices[i][:3]];u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);length=math.sqrt(sum(q*q for q in normal));normal=[q/length for q in normal]
    ids={k for k,p in enumerate(vs) if abs(sum(normal[j]*(p[j]-a[j]) for j in range(3)))<.000002};self.assertGreaterEqual(len(ids),20,n)
    for k in ids:self.assertTrue(any(occupied(posed[j],m.point(i,k,mats)) for j in parents),(clip,n,m.vertices[i][k]))
 def test_snorlax_belly_mask_and_sole_pads_clear_the_actual_posed_parent(self):
  m=self.models['snorlax']
  for clip,t in ((None,0.),('idle',.25),('attack',.18),('hit',.31)):
   mats=m.matrices(clip,t)
   for i,n in enumerate(m.names):
    if not n.startswith(('Belly /','Face /','Foot pad /')):continue
    prefix='Body /' if n.startswith('Belly /') else 'Head /' if n.startswith('Face /') else 'Foot /'
    parent=next(j for j,name in enumerate(m.names) if name.startswith(prefix) and (prefix!='Foot /' or name.split()[-1]==n.split()[-1]));carrier=mats[1 if prefix=='Body /' else 2 if prefix=='Head /' else 0]
    def local(j):
     coords=[]
     for p in m.posed(j,mats):
      d=[p[k]-carrier[12+k] for k in range(3)];coords.extend(sum(carrier[a*4+k]*d[k] for k in range(3)) for a in range(3))
     return dict(positions=coords,indices=m.indices[j])
    mark=local(i);skin=local(parent);vs=points(mark)
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(p[j] for p in (a,b,c))/3 for j in range(3)];hits=rays(skin,x,y);self.assertTrue(hits,(clip,n));self.assertGreater(z-max(hits),.0003,(clip,n))
 def test_snorlax_bent_forelimbs_do_not_fold_through_themselves(self):
  for p in self.matching('snorlax','Arm /'):self.assertIsNone(crossed_faces(p),p['part'])
if __name__=='__main__':unittest.main()
