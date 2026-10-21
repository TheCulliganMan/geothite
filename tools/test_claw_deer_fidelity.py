"""Multi-view anatomy checks over decoded meshes, independent of author pins.

CRYSTAL_CLAW_DEER_FIDELITY_MODELS selects named immutable baseline/candidate
GLBs for negative controls; no exporter is used to judge the geometry.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
MODELS=Path(os.environ.get('CRYSTAL_CLAW_DEER_FIDELITY_MODELS',str(ROOT)))
def points(p):return list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
def bounds(p,k):return min(p['positions'][k::3]),max(p['positions'][k::3])
def span(p,k):a,b=bounds(p,k);return b-a
class ClawDeerFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.parts={}
  for s in ('sneasel','stantler'):
   m=Model((MODELS/(s+'.glb')).read_bytes());m=logical_model(m) if 'source_part_ranges' in m.doc['extras'] else m
   cls.models[s]=m;cls.parts[s]=m.neutral_model()['primitives']
 def matching(self,s,prefix):return [p for p in self.parts[s] if p['part'].startswith(prefix)]
 def find(self,s,prefix):
  q=self.matching(s,prefix);self.assertTrue(q,(s,'missing anatomy',prefix));return q[0]
 def test_sneasel_shin_axes_are_upright_beneath_the_hips(self):
  for p in self.matching('sneasel','Leg /'):
   centers=[]
   for y in (.09,.19):
    lo,hi=section(p,y);centers.append([(a+b)/2 for a,b in zip(lo,hi)])
   self.assertLess(abs(centers[0][1]-centers[1][1]),.012,'forward-sloping shin')
   self.assertLess(abs(centers[0][0]),.10,'splayed ankle')
 def test_sneasel_plume_is_a_sagittal_solid_with_real_side_width(self):
  p=self.find('sneasel','Plume /');self.assertLess(span(p,0),.032);self.assertGreater(span(p,2),.10)
  q=dict(p,positions=[v for x,y,z in points(p) for v in (z,y,x)])
  lo,hi=section(q,.93);self.assertGreater(hi[0]-lo[0],.07);self.assertGreater(hi[1]-lo[1],.008)
 def test_sneasel_three_tail_blades_spread_behind_the_body(self):
  ps=self.matching('sneasel','Tail / swept');self.assertEqual(len(ps),3)
  for p in ps:self.assertLess(span(p,0),.025);self.assertGreater(span(p,2),.20);self.assertLess(bounds(p,2)[1],-.07)
 def test_sneasel_round_skull_and_pear_hips_have_profile_mass(self):
  p=self.find('sneasel','Head /');self.assertGreater(span(p,1),.27);self.assertGreater(span(p,2),.28)
  body=self.find('sneasel','Body /');a,b=section(body,.31);c,d=section(body,.47);self.assertGreater(b[0]-a[0],d[0]-c[0]+.055)
 def test_sneasel_feather_root_caps_stay_inside_their_moving_supports(self):
  m=self.models['sneasel']
  for clip,t in ((None,0.),('idle',.25),('attack',.18),('hit',.31)):
   mats=m.matrices(clip,t)
   for i,n in enumerate(m.names):
    if not n.startswith(('Plume /','Tail / swept')):continue
    is_plume=n.startswith('Plume /');j=next(j for j,name in enumerate(m.names) if name.startswith('Head /' if is_plume else 'Tail / short'))
    parent=dict(positions=sum(m.posed(j,mats),[]),indices=m.indices[j]);vs=m.vertices[i]
    # Cap station is the lowest plume ring or the foremost tail ring.
    axis=1 if is_plume else 2;edge=(min if is_plume else max)(v[axis] for v in vs)
    ids=[k for k,v in enumerate(vs) if (v[axis]<edge+.006 if is_plume else v[axis]>edge-.006)]
    self.assertGreaterEqual(len(ids),6)
    for k in ids:self.assertTrue(occupied(parent,m.point(i,k,mats)),(clip,n,vs[k]))
 def test_stantler_has_a_padded_muzzle_and_small_separate_nose(self):
  p=self.find('stantler','Muzzle /');pad=self.find('stantler','Muzzle pad /');self.assertLess(span(p,0),.095);self.assertGreater(span(pad,0),1.8*span(p,0));self.assertGreater(span(pad,2),.13)
 def test_stantler_antlers_have_angular_branches_and_open_negative_space(self):
  for p in self.matching('stantler','Antler /'):
   unique=set(points(p));self.assertLess(len(unique),150,'smoothed oval loop')
   x=sum(bounds(p,0))/2;y=sum(bounds(p,1))/2;self.assertFalse(rays(p,x,y))
  for p in self.matching('stantler','Antler tine /'):self.assertGreater(span(p,1),.05)
 def test_stantler_bib_is_long_and_three_ribs_wrap_each_flank(self):
  self.assertGreater(span(self.find('stantler','Chest /'),1),.17)
  ps=self.matching('stantler','Rib /');self.assertEqual(len(ps),6)
  for side in ('-1','1'):
   ribs=[p for p in ps if p['part'].split()[-2]==side];self.assertEqual(len(ribs),3)
   for p in ribs:self.assertGreater(span(p,2),.23)
 def test_stantler_hooves_have_a_sloping_pastern_and_flat_cloven_sole(self):
  for p in self.matching('stantler','Hoof /'):
   vs=points(p);self.assertEqual(min(v[1] for v in vs),0.)
   self.assertGreaterEqual(len({(x,z) for x,y,z in vs if y==0.}),9)
   front=max(v[2] for v in vs);back=min(v[2] for v in vs)
   front_top=max(y for x,y,z in vs if z>front-.03);back_top=max(y for x,y,z in vs if z<back+.03)
   self.assertLess(front_top,.025);self.assertGreater(back_top,.05)
 def test_stantler_fitted_bib_ribs_and_gaze_clear_the_actual_posed_surface(self):
  m=self.models['stantler']
  for clip,t in ((None,0.),('idle',.25),('attack',.18),('hit',.31)):
   mats=m.matrices(clip,t)
   for i,n in enumerate(m.names):
    if not n.startswith(('Chest /','Rib /','Eye / fitted dark')):continue
    side=-1 if n.split()[-2]=='-1' else 1
    parent=next(j for j,name in enumerate(m.names) if name.startswith('Body /' if n.startswith('Rib /') else 'Neck /' if n.startswith('Chest /') else 'Head /'))
    carrier=mats[1 if n.startswith(('Rib /','Chest /')) else 3]
    def local(j):
     coords=[]
     for p in m.posed(j,mats):
      d=[p[k]-carrier[12+k] for k in range(3)];q=[sum(carrier[a*4+k]*d[k] for k in range(3)) for a in range(3)]
      coords.extend((-side*q[2],q[1],side*q[0]) if n.startswith('Rib /') else q)
     return dict(positions=coords,indices=m.indices[j])
    mark=local(i);body=local(parent);vs=points(mark)
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(v[j] for v in (a,b,c))/3 for j in range(3)];hits=rays(body,x,y);self.assertTrue(hits,(clip,n));self.assertGreater(z-max(hits),.0003,(clip,n))
if __name__=='__main__':unittest.main()
