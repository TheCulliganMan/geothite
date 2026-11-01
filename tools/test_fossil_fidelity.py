"""Reference-backed fossil volumes, feature planes, posture and stored motion.

The ordinary decoded reader has no author pins. Select immutable rejected
meshes with CRYSTAL_FOSSIL_FIDELITY_MODELS for negative verification.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from test_seahorse_glb import occupied_overlap
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays
ROOT=Path(os.environ.get('CRYSTAL_FOSSIL_FIDELITY_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def points(p):return list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
def bounds(p,k):return min(p['positions'][k::3]),max(p['positions'][k::3])
def span(p,k):a,b=bounds(p,k);return b-a
def area(p,u,v):
 vs=points(p);result=0.
 for k in range(0,len(p['indices']),3):
  a,b,c=[vs[i] for i in p['indices'][k:k+3]];result+=abs((b[u]-a[u])*(c[v]-a[v])-(b[v]-a[v])*(c[u]-a[u]))/2
 return result
class FossilFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={s:logical_model(Model((ROOT/(s+'.glb')).read_bytes())) for s in ('aerodactyl','kabutops')};cls.parts={s:m.neutral_model()['primitives'] for s,m in cls.models.items()}
 def matching(self,s,prefix):return [p for p in self.parts[s] if p['part'].startswith(prefix)]
 def find(self,s,prefix):
  found=self.matching(s,prefix);self.assertTrue(found,(s,prefix,'missing anatomy'));return found[0]
 def test_aerodactyl_wings_present_broad_profiles_and_narrow_fronts(self):
  wings=self.matching('aerodactyl','Wing /');self.assertEqual(len(wings),2)
  for p in wings:self.assertGreater(area(p,1,2),area(p,0,1)*8.);self.assertGreater(span(p,1),span(p,0)*1.1)
 def test_aerodactyl_gray_backs_and_purple_undersides_are_distinct(self):
  cases=self.matching('aerodactyl','Wing back /');self.assertEqual(len(cases),2);body=self.find('aerodactyl','Body /')
  for case in cases:
   wing=next(p for p in self.matching('aerodactyl','Wing /') if p['part'].split()[-1]==case['part'].split()[-1]);self.assertEqual(case['base_color'],body['base_color']);self.assertGreater(case['base_color'][1],wing['base_color'][1]*2.)
 def test_aerodactyl_gape_is_open_without_the_old_oversized_drop(self):
  head=self.find('aerodactyl','Skull /');jaw=self.find('aerodactyl','Jaw /')
  def turn(p):return dict(p,positions=[v for x,y,z in points(p) for v in (x,-z,y)])
  front=bounds(head,2)[1]
  for fraction in (.65,.76,.86):
   upper=rays(turn(head),0.,-front*fraction);lower=rays(turn(jaw),0.,-front*fraction);self.assertTrue(upper and lower);gap=min(upper)-max(lower);self.assertTrue(.035*1.30<gap<.14*1.30,gap)
 def test_aerodactyl_eyes_stay_on_the_rear_skull_in_profile(self):
  head=self.find('aerodactyl','Skull /');eyes=self.matching('aerodactyl','Eye white /');self.assertEqual(len(eyes),2)
  for p in eyes:self.assertLess(span(p,2),span(head,2)*.35);self.assertLess(sum(bounds(p,2))/2,bounds(head,2)[0]+span(head,2)*.55)
 def test_aerodactyl_ankles_do_not_repeat_the_forward_diagonal(self):
  for p in self.matching('aerodactyl','Leg /'):
   lo,hi=bounds(p,1);centers=[sum(v[1] for v in section(p,lo+(hi-lo)*t))/2 for t in (.25,.65)];self.assertLess(abs(centers[1]-centers[0]),.025*1.30)
 def test_aerodactyl_tail_is_low_and_nearly_centered(self):
  tail=self.find('aerodactyl','Tail /');self.assertLess(span(tail,0),.12*1.30);far=[p for p in points(tail) if p[2]<bounds(tail,2)[0]+.10];self.assertTrue(far);self.assertLess(max(p[1] for p in far),.25*1.30)
 def test_aerodactyl_arrowhead_is_a_profile_spade_not_a_front_fan(self):
  p=self.find('aerodactyl','Tail tip /');self.assertGreater(area(p,1,2),area(p,0,1)*3.5)
 def test_aerodactyl_stored_idle_has_no_near_loop_wing_jump(self):
  m=self.models['aerodactyl'];epsilon=.00001/m.clips['idle'][0][1][-1]
  for t in (epsilon,1.-epsilon):
   mats=m.matrices('idle',t)
   for i,vs in enumerate(m.vertices):
    for p,q in zip(vs,m.posed(i,mats)):self.assertLess(math.dist(p,q),.00001,(t,m.names[i],p,q))
 def test_aerodactyl_backing_stays_joined_to_both_moving_jaws(self):
  self.find('aerodactyl','Mouth back /');m=self.models['aerodactyl'];wall=next(i for i,n in enumerate(m.names) if n.startswith('Mouth back /'));parents=[next(i for i,n in enumerate(m.names) if n.startswith(prefix)) for prefix in ('Skull /','Jaw /')]
  for clip,t in (('idle',.25),('attack',.34),('hit',.46)):
   mats=m.matrices(clip,t)
   def posed(i):return dict(positions=sum(m.posed(i,mats),[]),indices=m.indices[i])
   for j in parents:self.assertTrue(occupied_overlap(posed(wall),posed(j)),(clip,m.names[j]))
 def test_aerodactyl_purple_tissue_clears_actual_gray_sails_through_motion(self):
  self.assertEqual(len(self.matching('aerodactyl','Wing back /')),2);m=self.models['aerodactyl']
  for clip,t in (('idle',.25),('attack',.34),('hit',.46)):
   mats=m.matrices(clip,t)
   for side in (-1,1):
    i=next(i for i,n in enumerate(m.names) if n.startswith('Wing /') and n.split()[-1]==str(side));j=next(i for i,n in enumerate(m.names) if n.startswith('Wing back /') and n.split()[-1]==str(side));carrier=mats[m.ids[i][0][0]]
    def local(index):
     vs=[]
     for p in m.posed(index,mats):
      d=[p[k]-carrier[12+k] for k in range(3)];x,y,z=[sum(carrier[a*4+k]*d[k] for k in range(3)) for a in range(3)];vs.extend((.60*x+side*.80*y,z,side*.80*x-.60*y))
     return dict(positions=vs,indices=m.indices[index])
    skin=local(j);mark=local(i);vs=points(mark)
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[v] for v in mark['indices'][k:k+3]];u=[b[v]-a[v] for v in range(3)];v=[c[q]-a[q] for q in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
     if normal[2]<.5*math.sqrt(sum(q*q for q in normal)):continue
     x,y,z=[sum(p[v] for p in (a,b,c))/3 for v in range(3)];hits=rays(skin,x,y);self.assertTrue(hits);self.assertGreater(z-max(hits),.0001)
 def test_kabutops_has_one_wide_shield_with_a_central_low_keel(self):
  head=self.find('kabutops','Head /');self.assertFalse(self.matching('kabutops','Head wing /'));lo,hi=bounds(head,1);a,b=section(head,lo+(hi-lo)*.10);self.assertLess(b[0]-a[0],span(head,0)*.30);self.assertGreater(span(head,0),.80)
 def test_kabutops_has_continuous_steel_armor_and_three_chest_lobes(self):
  breast=self.find('kabutops','Breast plate /');self.assertEqual(len(self.matching('kabutops','Rib plate /')),3);self.assertGreater(span(breast,1),.30);self.assertLess(abs(breast['base_color'][0]-breast['base_color'][1]),.10)
 def test_kabutops_has_five_paired_lateral_back_ridges(self):
  ps=self.matching('kabutops','Back spur /');self.assertEqual(len(ps),10)
  for side in (-1,1):
   group=[p for p in ps if int(p['part'].split()[-2])==side];self.assertEqual(len(group),5);self.assertTrue(all(span(p,0)>.25 for p in group))
 def test_kabutops_hocks_bend_back_above_the_forward_toes(self):
  for p in self.matching('kabutops','Shin /'):
   lo,hi=bounds(p,1);centers=[sum(v[1] for v in section(p,lo+(hi-lo)*t))/2 for t in (.25,.70)];self.assertGreater(centers[1]-centers[0],.020)
 def test_kabutops_blades_are_flat_gray_bands_reaching_below_the_hips(self):
  ps=self.matching('kabutops','Scythe /');self.assertEqual(len(ps),2)
  for p in ps:self.assertGreater(area(p,0,1),area(p,1,2)*2.);self.assertLess(bounds(p,1)[0],.25*1.20);self.assertLess(p['base_color'][0],.60)
 def test_kabutops_tail_is_a_short_downward_dagger(self):
  p=self.find('kabutops','Tail /');self.assertLess(bounds(p,1)[0],.10*1.20);self.assertLess(span(p,2),span(p,1)*.90)
if __name__=='__main__':unittest.main()
