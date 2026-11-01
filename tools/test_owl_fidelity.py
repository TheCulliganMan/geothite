"""Stored owl identity, support and volume guards independent of source pins."""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from pidgeotto_contours import components
from sculpt_test_support import parts_overlap,ray_intersections,horizontal_section_bounds as section
ROOT=Path(os.environ.get('CRYSTAL_OWL_FIDELITY_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def span(p,a):return max(p['positions'][a::3])-min(p['positions'][a::3])
def color(p):return p['base_color'][:3]
class OwlFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={s:Model((ROOT/(s+'.glb')).read_bytes()) for s in ('hoothoot','noctowl')};cls.parts={s:m.neutral_model()['primitives'] for s,m in cls.models.items()}
 def find(self,s,prefix):
  ps=[p for p in self.parts[s] if p['part'].startswith(prefix)];self.assertTrue(ps,(s,prefix));return ps
 def test_hoothoot_has_a_broad_pale_belly_instead_of_small_chevrons(self):
  p=max(self.find('hoothoot','Breast mark /'),key=lambda p:span(p,0)*span(p,1))
  self.assertGreater(span(p,0),.45);self.assertGreater(span(p,1),.30)
 def test_hoothoot_has_coral_irises_and_dark_clock_rings(self):
  for p in self.find('hoothoot','Eye /'):
   r,g,b=color(p);self.assertGreater(r,g*3);self.assertGreater(g,b)
  for p in self.find('hoothoot','Eye mask /'):self.assertLess(max(color(p)),.01)
 def test_hoothoot_has_three_actual_lower_lashes_per_eye(self):
  for p in self.find('hoothoot','Eye mask /'):self.assertEqual(len(components(p)),4)
 def test_hoothoot_wings_are_short_outward_flaps(self):
  for p in self.find('hoothoot','Wing /'):
   self.assertGreater(span(p,0),span(p,1)*1.5);self.assertLess(span(p,1),.16)
 def test_hoothoot_tail_feathers_have_occupied_body_roots_in_motion(self):
  m=self.models['hoothoot'];body=m.names.index(self.find('hoothoot','Body /')[0]['part']);tails=[m.names.index(p['part']) for p in self.find('hoothoot','Tail /')]
  for clip in m.clips:
   for t in (.18,.40,.70):
    mats=m.matrices(clip,t);b=dict(self.parts['hoothoot'][body],positions=[v for q in m.posed(body,mats) for v in q])
    for i in tails:
     p=dict(self.parts['hoothoot'][i],positions=[v for q in m.posed(i,mats) for v in q]);self.assertTrue(parts_overlap(b,p),(clip,t,p['part']))
 def test_hoothoot_tucked_foot_stays_inside_the_body(self):
  m=self.models['hoothoot'];bi=m.names.index(self.find('hoothoot','Body /')[0]['part']);fi=next(i for i,n in enumerate(m.names) if n.startswith('Foot /') and 'tucked' in n)
  for clip in m.clips:
   for t in (0.,.2,.6):
    mats=m.matrices(clip,t);body=dict(self.parts['hoothoot'][bi],positions=[v for q in m.posed(bi,mats) for v in q])
    for x,y,z in m.posed(fi,mats):
     hits=ray_intersections(body,x,y);self.assertTrue(hits);self.assertLess(min(hits),z);self.assertGreater(max(hits),z)
 def test_both_owls_keep_upright_shanks_above_the_forward_toes(self):
  for s,h in (('hoothoot',1.),('noctowl',1.3)):
   legs=[p for p in self.find(s,'Leg /') if 'tucked' not in p['part']]
   for p in legs:
    centers=[]
    for y in (.055*h,.120*h):
     lo,hi=section(p,y);centers.append((lo[1]+hi[1])*.5)
    self.assertLess(abs(centers[0]-centers[1]),.005*h,(s,p['part'],centers))
 def test_noctowl_has_two_broad_gold_plumes_with_real_profile_thickness(self):
  crowns=self.find('noctowl','Crown /');self.assertEqual(len(crowns),2)
  for p in crowns:
   r,g,b=color(p);self.assertGreater(g,b*1.8);self.assertGreater(span(p,2),.12)
 def test_noctowl_has_red_irises_in_its_stern_face(self):
  ps=self.find('noctowl','Iris /');self.assertEqual(len(ps),2)
  for p in ps:
   r,g,b=color(p);self.assertGreater(r,max(g,b)*5.)
 def test_noctowl_has_a_deep_breast_and_folded_wing_cases(self):
  body_depth=span(self.find('noctowl','Body /')[0],2);self.assertGreater(body_depth,.35*1.3)
  for p in self.find('noctowl','Wing /'):self.assertGreater(span(p,2),body_depth*.50)
 def test_complete_wing_root_caps_stay_buried_in_the_body(self):
  from test_steelix_sculpt import occupied
  for s,m in self.models.items():
   caps=[]
   for i,name in enumerate(m.names):
    if not name.startswith('Wing /'):continue
    side=-1 if sum(v[0] for v in m.vertices[i])<0 else 1;j=(6 if side<0 else 8) if s=='hoothoot' else (7 if side<0 else 8)
    pivot=[-x for x in m.inverse[j][12:15]];center=[sum(v[a] for v in m.vertices[i])/len(m.vertices[i]) for a in range(3)];along=[center[a]-pivot[a] for a in range(3)];ids=set()
    for k in range(0,len(m.indices[i]),3):
     tri=m.indices[i][k:k+3];a,b,c=[m.vertices[i][v] for v in tri];u=[b[q]-a[q] for q in range(3)];v=[c[q]-a[q] for q in range(3)];n=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);length=math.sqrt(sum(x*x for x in n));n=[x/length for x in n]
     if abs(sum(n[q]*(a[q]-pivot[q]) for q in range(3)))<.00001 and sum(n[q]*along[q] for q in range(3))<-.01:ids.update(tri)
    self.assertGreater(len(ids),5,(s,name));caps.append((i,ids))
   bi=m.names.index(self.find(s,'Body /')[0]['part'])
   for clip in m.clips:
    for t in (0.,.18,.4,.7):
     mats=m.matrices(clip,t);body=dict(self.parts[s][bi],positions=[v for q in m.posed(bi,mats) for v in q])
     for i,ids in caps:
      posed=m.posed(i,mats)
      for j in ids:self.assertTrue(occupied(body,posed[j]),(s,clip,t,m.names[i],j,posed[j]))
 def test_noctowl_gold_bridges_join_the_actual_head_and_plumes(self):
  m=self.models['noctowl'];body_i=m.names.index(self.find('noctowl','Body /')[0]['part'])
  for clip in m.clips:
   for t in (0.,.18,.7):
    mats=m.matrices(clip,t)
    def posed(i):return dict(self.parts['noctowl'][i],positions=[v for q in m.posed(i,mats) for v in q])
    for side in (-1,1):
     brow=next(i for i,n in enumerate(m.names) if n.startswith('Brow /') and n.endswith(' '+str(side)));plume=next(i for i,n in enumerate(m.names) if n.startswith('Crown /') and n.endswith(' '+str(side)))
     self.assertTrue(parts_overlap(posed(brow),posed(body_i)),(clip,t,side,'head'));self.assertTrue(parts_overlap(posed(brow),posed(plume)),(clip,t,side,'plume'))
 def test_both_stored_idle_loops_close_at_absolute_production_offsets(self):
  for s,m in self.models.items():
   d=m.clips['idle'][0][1][-1]
   for t in (.00001/d,1.-.00001/d):
    mats=m.matrices('idle',t)
    for i,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(i,mats)):self.assertLess(max(abs(x-y) for x,y in zip(a,b)),.00001,(s,t,m.names[i]))
 def test_fitted_face_and_belly_layers_stay_on_the_actual_posed_parent(self):
  from sculpt_test_support import indexed_ray_intersections
  for s,m in self.models.items():
   parts=self.parts[s]
   for clip,t in (('idle',.25),('attack',.18),('attack',.55),('hit',.30)):
    mats=m.matrices(clip,t)
    for i,p in enumerate(parts):
     name=p['part'];prefix=next((q for q in ('Eye mask /','Eye ring /','Eye /','Iris /','Pupil /','Glint /','Breast mark /') if name.startswith(q)),None)
     if prefix is None or 'scalloped' in name:continue
     side=name.split()[-1];parent_prefix={'Eye mask /':'Body /','Eye ring /':'Eye mask /','Eye /':'Eye ring /' if s=='noctowl' else 'Eye mask /','Iris /':'Eye /','Pupil /':'Iris /' if s=='noctowl' else 'Eye /','Glint /':'Pupil /','Breast mark /':'Body /'}[prefix]
     parents=[j for j,n in enumerate(m.names) if n.startswith(parent_prefix) and (parent_prefix=='Body /' or n.endswith(' '+side))];self.assertTrue(parents,(s,name,parent_prefix));j=parents[0]
     common=mats[1 if s=='hoothoot' else 2 if prefix=='Breast mark /' else 4]
     def local(points):return [tuple(sum(common[4*k+a]*(q[a]-common[12+a]) for a in range(3)) for k in range(3)) for q in points]
     parent=dict(parts[j],positions=[v for q in local(m.posed(j,mats)) for v in q]);posed=local(m.posed(i,mats));rest=m.vertices[i]
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];a,b,c=[rest[v] for v in tri];u=[b[n]-a[n] for n in range(3)];v=[c[n]-a[n] for n in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
      if normal[2]<=.5*math.sqrt(sum(x*x for x in normal)):continue
      q=tuple(sum(posed[v][a] for v in tri)/3 for a in range(3));hits=indexed_ray_intersections(parent,q[0],q[1]);self.assertTrue(hits,(s,clip,t,name,k));clearance=q[2]-max(hits);self.assertGreater(clearance,.00001,(s,clip,t,name,k));self.assertLess(clearance,.018,(s,clip,t,name,k))
if __name__=='__main__':unittest.main()
