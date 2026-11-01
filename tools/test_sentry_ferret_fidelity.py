"""Game-reference volume and support checks, independent of recipe digest pins."""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays,parts_overlap
ROOT=Path(os.environ.get('CRYSTAL_SENTRY_FERRET_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
HEIGHTS={'sentret':1.,'furret':1.2}
def span(p,a):return max(p['positions'][a::3])-min(p['positions'][a::3])
def combine(parts):
 out=dict(positions=[],indices=[])
 for p in parts:
  offset=len(out['positions'])//3;out['positions']+=p['positions'];out['indices'] += [v+offset for v in p['indices']]
 return out
def inside(p,q):
 hits=sorted(rays(p,q[0],q[1]));unique=[]
 for z in hits:
  if not unique or abs(z-unique[-1])>1e-7:unique.append(z)
 return sum(z>q[2]+1e-8 for z in unique)%2==1
def normal(a,b,c):
 u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)]
 return (u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
class SentryFerretFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={s:Model((ROOT/(s+'.glb')).read_bytes()) for s in HEIGHTS};cls.parts={s:m.neutral_model()['primitives'] for s,m in cls.models.items()}
 def find(self,s,prefix):
  ps=[p for p in self.parts[s] if p['part'].startswith(prefix)];self.assertTrue(ps,(s,prefix));return ps
 def test_sentret_has_a_round_body_instead_of_a_thin_pear(self):
  body=self.find('sentret','Body /')[0];self.assertGreater(span(body,0),span(body,1)*.85)
 def test_sentret_has_tiny_eyes_above_its_cream_annulus(self):
  body=self.find('sentret','Body /')[0]
  for eye in self.find('sentret','Eye /'):self.assertLess(span(eye,0),span(body,0)*.055)
  ring=self.find('sentret','Belly ring /')[0];r,g,b=ring['base_color'][:3];self.assertGreater(r,g);self.assertGreater(g,b)
 def test_sentret_has_charcoal_outer_ears_and_pink_inner_tissue(self):
  for p in self.find('sentret','Ear /'):
   r,g,b=p['base_color'][:3];self.assertLess(r,g*1.6);self.assertGreater(b,g*.65)
  for p in self.find('sentret','Ear inset /'):
   r,g,b=p['base_color'][:3];self.assertGreater(r,g*2.);self.assertGreater(b,g*1.3)
 def test_sentret_forelimbs_have_broad_flank_volume(self):
  for p in self.find('sentret','Arm /'):self.assertGreater(span(p,2),.075)
 def test_sentret_has_a_broad_centered_terminal_tail_support(self):
  tail=combine(self.find('sentret','Tail /')+self.find('sentret','Tail band /'));body=self.find('sentret','Body /')[0]
  self.assertGreater(span(tail,0),span(body,0)*.55)
  self.assertLess(abs(max(tail['positions'][::3])+min(tail['positions'][::3])),span(body,0)*.03)
  lo,hi=section(tail,.07);self.assertGreater(hi[0]-lo[0],span(body,0)*.40)
 def test_sentret_suspended_shanks_do_not_project_forward_into_long_feet(self):
  for p in self.find('sentret','Leg /'):
   lo=min(p['positions'][1::3]);h=span(p,1);centers=[]
   for y in (lo+h*.30,lo+h*.75):
    a,b=section(p,y);centers.append((a[1]+b[1])*.5)
   self.assertLess(abs(centers[0]-centers[1]),.025)
  for p in self.find('sentret','Foot /'):self.assertLess(span(p,2),span(p,1)*1.3)
 def test_sentret_terminal_tail_keeps_round_volume_near_its_front_end(self):
  tail=combine(self.find('sentret','Tail /')+self.find('sentret','Tail band /'))
  # Cut across depth near the front pole: a cone can share the front outline
  # while losing the rounded reference volume in both profiles.
  end=max(tail['positions'][2::3]);turned=dict(tail,positions=[v for x,y,z in zip(tail['positions'][::3],tail['positions'][1::3],tail['positions'][2::3]) for v in (x,z,-y)])
  lo,hi=section(turned,end-.020);self.assertGreater(hi[1]-lo[1],.12)
 def test_furret_has_cream_fur_instead_of_literal_yellow_shadow_indices(self):
  cream=[p for p in self.find('furret','Body /') if 'cream' in p['part']];self.assertTrue(cream)
  for p in cream:
   r,g,b=p['base_color'][:3];self.assertGreater(g,.60);self.assertGreater(b,.45)
 def test_furret_has_one_waist_belt_and_a_full_lower_body(self):
  h=HEIGHTS['furret'];body=self.find('furret','Body /');belts=[p for p in body if 'brown' in p['part'] and .25*h<(min(p['positions'][1::3])+max(p['positions'][1::3]))/2<.70*h]
  self.assertEqual(len(belts),1)
  lo,hi=section(combine(body),.26*h);self.assertGreater(hi[0]-lo[0],.38*h);self.assertGreater(hi[1]-lo[1],.35*h)
 def test_furret_has_a_continuous_brown_neck_saddle(self):
  p=self.find('furret','Back coat /')[0];self.assertGreater(span(p,1),.40*HEIGHTS['furret']);self.assertGreater(span(p,0),.25*HEIGHTS['furret'])
  m=self.models['furret'];i=next(i for i,n in enumerate(m.names) if n.startswith('Back coat /'));mid=(min(p['positions'][1::3])+max(p['positions'][1::3]))/2;j=next(i for i,n in enumerate(m.names) if n.startswith('Body /') and min(v[1] for v in m.vertices[i])<mid<max(v[1] for v in m.vertices[i]))
  front={tuple(v):(tuple(ids),tuple(ws)) for v,ids,ws in zip(m.vertices[j],m.ids[j],m.weights[j])};back={tuple(v):(tuple(ids),tuple(ws)) for v,ids,ws in zip(m.vertices[i],m.ids[i],m.weights[i])};shared=front.keys()&back.keys()
  self.assertGreater(len(shared),30,'brown and cream are closed sections of the actual body')
  for point in shared:self.assertEqual(front[point],back[point])
 def test_furret_skull_and_ears_stay_narrower_than_the_full_lower_body(self):
  h=HEIGHTS['furret'];body=combine(self.find('furret','Body /'));lo,hi=section(body,.26*h);hips=hi[0]-lo[0]
  lo,hi=section(body,.91*h);self.assertLess(hi[0]-lo[0],hips*.78)
  self.assertLess(span(combine(self.find('furret','Ear /')),0),hips*1.15)
 def test_furret_paws_are_cream_and_ear_points_have_cream_tips(self):
  for p in self.find('furret','Foot /'):
   r,g,b=p['base_color'][:3];self.assertGreater(g,.60);self.assertGreater(b,.45)
  tips=self.find('furret','Ear tip /');self.assertEqual(len(tips),2)
  for p in tips:
   self.assertGreater(p['base_color'][2],.45);self.assertGreater(span(p,2),.025*HEIGHTS['furret'])
   side=p['part'].split()[-1];ear=next(q for q in self.find('furret','Ear /') if q['part'].endswith(' '+side))
   points=lambda q:set(zip(q['positions'][::3],q['positions'][1::3],q['positions'][2::3]))
   self.assertGreaterEqual(len(points(p)&points(ear)),4)
   self.assertAlmostEqual(min(p['positions'][2::3]),min(ear['positions'][2::3]),places=6)
   self.assertAlmostEqual(max(p['positions'][2::3]),max(ear['positions'][2::3]),places=6)
 def test_furret_mouth_is_a_real_backed_recess_with_empty_air_in_front(self):
  mouth=self.find('furret','Mouth /')[0];body=combine(self.find('furret','Body /'));x=(min(mouth['positions'][::3])+max(mouth['positions'][::3]))/2;y=(min(mouth['positions'][1::3])+max(mouth['positions'][1::3]))/2
  hits=rays(mouth,x,y);self.assertTrue(hits);back=max(hits);adjacent=[max(rays(body,x+dx,y)) for dx in (-.035*1.2,.035*1.2)];gap=min(adjacent)-back;self.assertGreater(gap,.012*1.2)
  self.assertTrue(inside(mouth,(x,y,back-.0004)))
  self.assertFalse(any(inside(p,(x,y,back+gap*.5)) for p in self.parts['furret']))
 def test_complete_tail_and_forelimb_root_caps_stay_inside_their_bodies(self):
  for s,m in self.models.items():
   targets=[i for i,n in enumerate(m.names) if n.startswith('Arm /')]
   targets.append(next(i for i,n in enumerate(m.names) if n.startswith('Tail /')))
   caps=[]
   for i in targets:
    tail=m.names[i].startswith('Tail /');side=-1 if sum(p[0] for p in m.vertices[i])<0 else 1;suffix='/body' if s=='sentret' and tail else '/tail_base' if tail else '/arm_left' if side<0 else '/arm_right'
    joint=next(j for j,node in enumerate(m.doc['nodes'][:len(m.joints)]) if node['name'].endswith(suffix));pivot=[-x for x in m.inverse[joint][12:15]];center=[sum(v[k] for v in m.vertices[i])/len(m.vertices[i]) for k in range(3)];along=[center[k]-pivot[k] for k in range(3)];ids=set()
    for k in range(0,len(m.indices[i]),3):
     tri=m.indices[i][k:k+3];a,b,c=[m.vertices[i][v] for v in tri];n=normal(a,b,c);length=math.sqrt(sum(v*v for v in n));n=[v/length for v in n]
     if abs(sum(n[k]*(a[k]-pivot[k]) for k in range(3)))<.00001 and sum(n[k]*along[k] for k in range(3))<-.01:ids.update(tri)
    self.assertGreater(len(ids),5,(s,m.names[i]));caps.append((i,ids))
   for clip in m.clips:
    for t in (0.,.18,.40,.70):
     mats=m.matrices(clip,t);bodies=[dict(p,positions=[v for q in m.posed(i,mats) for v in q]) for i,p in enumerate(self.parts[s]) if p['part'].startswith(('Body /','Back coat /'))]
     for i,ids in caps:
      vs=m.posed(i,mats)
      for j in ids:self.assertTrue(any(inside(p,vs[j]) for p in bodies),(s,clip,t,m.names[i],j,vs[j]))
 def test_furret_terminal_fur_points_are_rooted_during_motion(self):
  s='furret';m=self.models[s];tufts=[i for i,n in enumerate(m.names) if n.startswith('Tail tuft /')];self.assertEqual(len(tufts),2)
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t);tails=[dict(p,positions=[v for q in m.posed(i,mats) for v in q]) for i,p in enumerate(self.parts[s]) if p['part'].startswith('Tail /')]
    for i in tufts:
     p=dict(self.parts[s][i],positions=[v for q in m.posed(i,mats) for v in q]);self.assertTrue(any(parts_overlap(p,tail) for tail in tails),(clip,t,m.names[i]))
 def test_both_stored_idle_loops_close_at_absolute_production_offsets(self):
  for s,m in self.models.items():
   duration=m.clips['idle'][0][1][-1]
   for t in (.00001/duration,1.-.00001/duration):
    mats=m.matrices('idle',t)
    for i,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(i,mats)):self.assertLess(max(abs(x-y) for x,y in zip(a,b)),.00001,(s,t,m.names[i]))
 def test_fitted_faces_ears_and_annulus_keep_surface_clearance_in_motion(self):
  for s,m in self.models.items():
   ps=self.parts[s]
   for clip,t in (('idle',.25),('attack',.18),('attack',.55),('hit',.30)):
    mats=m.matrices(clip,t)
    for i,p in enumerate(ps):
     name=p['part'];prefix=next((q for q in ('Eye /','Glint /','Ear inset /','Belly ring /','Face mark /','Forehead /','Nose /','Mouth /','Tongue /','Back coat /') if name.startswith(q)),None)
     if prefix is None or prefix=='Back coat /' or s=='sentret' and prefix=='Nose /':continue
     side=name.split()[-1];parent_prefix={'Glint /':'Eye /','Ear inset /':'Ear /','Ear tip /':'Ear /','Tongue /':'Mouth /'}.get(prefix,'Body /');parents=[j for j,n in enumerate(m.names) if n.startswith(parent_prefix) and (parent_prefix in ('Body /','Mouth /') or n.endswith(' '+side))];self.assertTrue(parents,(s,name,parent_prefix));back=prefix=='Back coat /';axis=-1 if back else 1
     if prefix in ('Ear inset /','Ear tip /'):carrier=5 if side=='-1' else 6
     elif s=='sentret':carrier=4
     else:carrier=0 if back else 4
     common=mats[carrier]
     def local(points):return [tuple(sum(common[4*k+a]*(q[a]-common[12+a]) for a in range(3)) for k in range(3)) for q in points]
     parent=combine([dict(ps[j],positions=[v for q in local(m.posed(j,mats)) for v in q]) for j in parents]);posed=local(m.posed(i,mats));rest=m.vertices[i]
     if back:parent['positions']=[v*axis if k%3==2 else v for k,v in enumerate(parent['positions'])]
     checked=0
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];a,b,c=[rest[v] for v in tri];n=normal(a,b,c)
      if n[2]*axis<=.5*math.sqrt(sum(v*v for v in n)):continue
      q=[sum(posed[v][a] for v in tri)/3 for a in range(3)];q[2]*=axis;hits=rays(parent,q[0],q[1]);self.assertTrue(hits,(s,clip,t,name,k));clearance=q[2]-max(hits);self.assertGreater(clearance,.00001,(s,clip,t,name,k,clearance));self.assertLess(clearance,.018,(s,clip,t,name,k,clearance));checked+=1
     self.assertGreater(checked,0,(s,name))
if __name__=='__main__':unittest.main()
