"""Independent stored-volume checks for Charizard and Dragonite.

No author digest is used. CRYSTAL_DRAGON_FIDELITY_MODELS can select immutable
baseline assets as rejected-geometry controls.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays,surface_distances,crossed_faces
from test_steelix_sculpt import occupied
ROOT=Path(os.environ.get('CRYSTAL_DRAGON_FIDELITY_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def points(p):return list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
def bounds(p,k):return min(p['positions'][k::3]),max(p['positions'][k::3])
def span(p,k):a,b=bounds(p,k);return b-a
def middle(p,k):return sum(bounds(p,k))/2
class DragonFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={s:logical_model(Model((ROOT/(s+'.glb')).read_bytes())) for s in ('charizard','dragonite')};cls.parts={s:m.neutral_model()['primitives'] for s,m in cls.models.items()}
 def matching(self,s,prefix):return [p for p in self.parts[s] if p['part'].startswith(prefix)]
 def find(self,s,prefix):
  ps=self.matching(s,prefix);self.assertTrue(ps,(s,'missing actual anatomy',prefix));return ps[0]
 def test_dragonite_has_a_deep_pear_and_substantial_skull(self):
  body=self.find('dragonite','Body /');lo,hi=section(body,.45);self.assertGreater(hi[0]-lo[0],.67);self.assertGreater(hi[1]-lo[1],.57)
  self.assertGreater(span(self.find('dragonite','Head /'),2),.35)
 def test_dragonite_wings_are_compact_back_swept_profile_surfaces(self):
  ps=self.matching('dragonite','Wing border /');self.assertEqual(len(ps),2)
  for p in ps:
   self.assertLess(span(p,0),.23);self.assertLess(bounds(p,2)[0],-.44);vs=points(p);area=0.
   for k in range(0,len(p['indices']),3):
    a,b,c=[vs[j] for j in p['indices'][k:k+3]];area+=abs((b[1]-a[1])*(c[2]-a[2])-(b[2]-a[2])*(c[1]-a[1]))/2
   self.assertGreater(area,.055,'bounds alone cannot prove broad wing profiles')
 def test_dragonite_feelers_are_shorter_than_the_old_wide_springs(self):
  ps=self.matching('dragonite','Antenna /');self.assertEqual(len(ps),2);width=max(bounds(p,0)[1] for p in ps)-min(bounds(p,0)[0] for p in ps);self.assertTrue(1.3<width/span(self.find('dragonite','Head /'),0)<1.8)
 def test_dragonite_holds_bent_forearms_forward_of_its_belly(self):
  ps=self.matching('dragonite','Hand /');self.assertEqual(len(ps),2)
  for p in ps:self.assertGreater(middle(p,1),.65);self.assertGreater(middle(p,2),.30)
  for p in self.matching('dragonite','Arm /'):self.assertIsNone(crossed_faces(p))
 def test_dragonite_ankles_stay_upright_above_projecting_toes(self):
  for p in self.matching('dragonite','Leg /'):
   centers=[sum(v[1] for v in section(p,y))/2 for y in (.08,.16)];self.assertLess(abs(centers[1]-centers[0]),.012)
 def test_both_dragons_have_long_low_tails(self):
  for s in self.models:
   tail=self.find(s,'Tail /');self.assertGreater(span(tail,2),1.25);far=[p for p in points(tail) if p[2]<-1.30];self.assertTrue(far,s);self.assertLess(max(p[1] for p in far),.43,s)
 def test_charizard_has_a_fuller_barrel_and_a_bib_below_its_neck(self):
  lo,hi=section(self.find('charizard','Body /'),.42);self.assertGreater(hi[1]-lo[1],.49);self.assertLess(bounds(self.find('charizard','Belly /'),1)[1],.80)
 def test_charizard_has_raised_complete_palms_and_upright_ankles(self):
  palms=self.matching('charizard','Hand /');self.assertEqual(len(palms),2)
  for p in palms:self.assertGreater(middle(p,1),.74)
  shins=self.matching('charizard','Shin /');self.assertEqual(len(shins),2)
  for p in shins:
   centers=[sum(v[1] for v in section(p,y))/2 for y in (.075,.14)];self.assertLess(abs(centers[1]-centers[0]),.012)
 def test_charizard_wings_have_orange_backs_and_fitted_teal_fronts(self):
  cases=self.matching('charizard','Wing back /');self.assertEqual(len(cases),2)
  for case in cases:
   side=int(case['part'].split()[-1]);wing=next(p for p in self.matching('charizard','Wing /') if p['part'].split()[-1]==str(side));r,g,b,_=case['base_color'];self.assertGreater(r,b+.5)
   a=side*math.radians(30.);c=math.cos(a);sn=math.sin(a)
   def local(p):return dict(p,positions=[v for x,y,z in points(p) for v in (c*x-sn*z,y,sn*x+c*z)])
   skin=local(case);mark=local(wing);vs=points(mark)
   for k in range(0,len(mark['indices']),3):
    a,b,c=[vs[j] for j in mark['indices'][k:k+3]];u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];n=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
    if n[2]<.5*math.sqrt(sum(x*x for x in n)):continue
    x,y,z=[sum(p[j] for p in (a,b,c))/3 for j in range(3)];hits=rays(skin,x,y);self.assertTrue(hits,(side,k,x,y,z,n));self.assertGreater(z-max(hits),.0001)
 def test_both_tail_undersides_follow_actual_tubes_through_motion(self):
  for s,m in self.models.items():
   parent=next(i for i,n in enumerate(m.names) if n.startswith('Tail /'));ribbons=[i for i,n in enumerate(m.names) if n.startswith('Tail underside /')];self.assertEqual(len(ribbons),1,s);i=ribbons[0]
   for clip,t in (('idle',.25),('attack',.34),('hit',.46)):
    mats=m.matrices(clip,t);skin=dict(positions=sum(m.posed(parent,mats),[]),indices=m.indices[parent]);pts={tuple(p) for p in m.posed(i,mats)}
    for gap in surface_distances(skin,pts,.007):self.assertLess(gap,.007,(s,clip))
 def test_charizard_gape_is_empty_in_front_of_occupied_recessed_backing(self):
  wall=self.find('charizard','Throat /');x,y,z=[middle(wall,k) for k in range(3)]
  for dx in (-.025,0.,.025):
   self.assertTrue(occupied(wall,(x+dx,y,z)))
   self.assertFalse(any(occupied(p,(x+dx,y,z+.08)) for p in self.parts['charizard']),'front gape must stay open')
 def test_charizard_stored_loop_returns_without_a_near_join_snap(self):
  m=self.models['charizard'];duration=m.clips['idle'][0][1][-1];epsilon=.00001/duration
  for t in (epsilon,1.-epsilon):
   mats=m.matrices('idle',t)
   for i,vertices in enumerate(m.vertices):
    for p,q in zip(vertices,m.posed(i,mats)):self.assertLess(math.dist(p,q),.00001,(t,m.names[i],p,q))
 def test_charizard_mouth_floor_stays_fitted_to_its_actual_moving_jaw(self):
  self.find('charizard','Oral floor /');m=self.models['charizard'];i=next(i for i,n in enumerate(m.names) if n.startswith('Oral floor /'));j=next(i for i,n in enumerate(m.names) if n.startswith('Jaw /'))
  for clip,t in (('idle',.25),('attack',.34),('hit',.46)):
   mats=m.matrices(clip,t);carrier=mats[3]
   def local(index):
    coords=[]
    for p in m.posed(index,mats):
     d=[p[k]-carrier[12+k] for k in range(3)];x,y,z=[sum(carrier[a*4+k]*d[k] for k in range(3)) for a in range(3)];coords.extend((x,-z,y))
    return dict(positions=coords,indices=m.indices[index])
   skin=local(j);mark=local(i);vs=points(mark)
   for k in range(0,len(mark['indices']),3):
    a,b,c=[vs[v] for v in mark['indices'][k:k+3]];u=[b[v]-a[v] for v in range(3)];v=[c[v]-a[v] for v in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
    if normal[2]<.5*math.sqrt(sum(q*q for q in normal)):continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
     x,y,z=[sum(p[v]*w for p,w in zip((a,b,c),weights)) for v in range(3)];hits=rays(skin,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001)
 def test_complete_arm_antenna_and_tail_caps_stay_buried_through_poses(self):
  for s,m in self.models.items():
   body=next(i for i,n in enumerate(m.names) if n.startswith('Body /'));head=next(i for i,n in enumerate(m.names) if n.startswith('Head /'))
   for clip,t in ((None,0.),('idle',.25),('attack',.34),('hit',.46)):
    mats=m.matrices(clip,t);parents={j:dict(positions=sum(m.posed(j,mats),[]),indices=m.indices[j]) for j in (body,head)}
    for i,name in enumerate(m.names):
     if not name.startswith(('Arm /','Antenna /','Tail /')):continue
     vs=m.vertices[i];a,b,c=[vs[j] for j in m.indices[i][:3]];u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)];n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];length=math.sqrt(sum(x*x for x in n));n=[x/length for x in n];ids={j for j,p in enumerate(vs) if abs(sum(n[k]*(p[k]-a[k]) for k in range(3)))<.000002};self.assertGreaterEqual(len(ids),12)
     for j in ids:self.assertTrue(occupied(parents[head if name.startswith('Antenna /') else body],m.point(i,j,mats)),(s,clip,name,vs[j]))
if __name__=='__main__':unittest.main()
