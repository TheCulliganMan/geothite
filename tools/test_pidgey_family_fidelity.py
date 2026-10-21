"""Reference-shape regressions decoded without the authoring digest pins.

CRYSTAL_PIDGEY_FIDELITY_MODELS selects the immutable rejected models for negative
verification. Game volume, neutral posture and motion are checked separately.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from pidgeotto_contours import components
from sculpt_test_support import horizontal_section_bounds as section,ray_intersections
from test_steelix_sculpt import occupied
ROOT=Path(os.environ.get('CRYSTAL_PIDGEY_FIDELITY_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
HEIGHTS={'pidgey':.82,'pidgeotto':1.,'pidgeot':1.35}
def span(p,k):return max(p['positions'][k::3])-min(p['positions'][k::3])
def area(p,u,v):
 ps=list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]));total=0.
 for i in range(0,len(p['indices']),3):
  a,b,c=[ps[j] for j in p['indices'][i:i+3]];total+=abs((b[u]-a[u])*(c[v]-a[v])-(b[v]-a[v])*(c[u]-a[u]))*.5
 return total
class PidgeyFamilyFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.parts={}
  for s in HEIGHTS:
   m=Model((ROOT/(s+'.glb')).read_bytes());cls.models[s]=logical_model(m) if 'source_part_ranges' in m.doc['extras'] else m;cls.parts[s]=cls.models[s].neutral_model()['primitives']
 def find(self,s,prefix):
  found=[p for p in self.parts[s] if p['part'].startswith(prefix)];self.assertTrue(found,(s,prefix));return found
 def test_pidgey_has_a_smaller_bird_skull_over_its_full_breast(self):
  head=self.find('pidgey','Head /')[0];body=self.find('pidgey','Body /')[0];self.assertLess(span(head,1),span(body,1)*.74);self.assertGreater(span(body,2),span(body,1)*.98)
 def test_pidgey_has_three_actual_short_crown_feathers(self):
  crowns=self.find('pidgey','Crown /');self.assertEqual(sum(len(components(p)) for p in crowns),3)
 def test_pidgey_resting_flight_vanes_have_broad_profile_surfaces(self):
  vanes=self.find('pidgey','Flight feather /');self.assertEqual(sum(len(components(p)) for p in vanes),8)
  self.assertGreater(sum(area(p,1,2) for p in vanes),sum(area(p,0,1) for p in vanes)*5.)
 def test_pidgey_folded_resting_wings_stay_below_its_head(self):
  wings=self.find('pidgey','Wing /');head=self.find('pidgey','Head /')[0]
  self.assertLess(max(v for p in wings for v in p['positions'][1::3]),max(head['positions'][1::3])*.80)
 def test_pidgey_beak_has_the_game_references_broad_short_proportions(self):
  bill=self.find('pidgey','Upper beak /')[0];head=self.find('pidgey','Head /')[0]
  self.assertGreater(span(bill,0),span(head,0)*.40);self.assertLess(span(bill,0),span(head,0)*.58)
  self.assertLess(span(bill,2),span(bill,0))
 def test_pidgey_eyes_are_near_the_beak_under_narrow_lids(self):
  head=self.find('pidgey','Head /')[0]
  for eye in self.find('pidgey','Eye /'):
   self.assertLess(min(abs(x) for x in eye['positions'][::3]),span(head,0)*.30)
   self.assertLess(span(eye,1),span(head,1)*.14)
 def test_pidgey_inner_eye_gap_is_smaller_than_its_bill(self):
  bill=self.find('pidgey','Upper beak /')[0]
  gap=sum(min(abs(x) for x in eye['positions'][::3]) for eye in self.find('pidgey','Eye /'))
  self.assertGreater(gap/span(bill,0),.10)
  self.assertLess(gap/span(bill,0),.45)
 def test_pidgey_pupils_fill_the_front_eye_field(self):
  for side in (-1,1):
   eye=next(p for p in self.find('pidgey','Eye /') if p['part'].endswith(' '+str(side)))
   pupil=next(p for p in self.find('pidgey','Pupil /') if p['part'].endswith(' '+str(side)))
   fill=area(pupil,0,1)/area(eye,0,1)
   self.assertGreater(fill,.25);self.assertLess(fill,.45)
 def test_pidgey_lining_stays_above_the_lower_bill_floor(self):
  lining=self.find('pidgey','Mouth /')[0];lower=self.find('pidgey','Lower beak /')[0]
  self.assertGreater(min(lining['positions'][1::3])-min(lower['positions'][1::3]),.002*.82)
 def test_pidgey_upper_breast_carries_the_throat_in_profile(self):
  head=self.find('pidgey','Head /')[0];body=self.find('pidgey','Body /')[0]
  head_front=ray_intersections(head,0,.62*.82);body_front=ray_intersections(body,0,.62*.82)
  self.assertTrue(head_front and body_front)
  self.assertLess(max(head_front)-max(body_front),.10*.82)
 def test_pidgey_cream_brows_frame_the_brown_forehead_and_sweep_back(self):
  head=self.find('pidgey','Head /')[0]
  for brow in self.find('pidgey','Brow /'):
   self.assertLess(min(abs(x) for x in brow['positions'][::3]),span(head,0)*.085)
   self.assertLess(min(brow['positions'][2::3]),min(head['positions'][2::3]))
 def test_pidgey_beak_opens_in_front_of_recessed_occupied_lining(self):
  upper=self.find('pidgey','Upper beak /')[0];lower=self.find('pidgey','Lower beak /')[0];lining=self.find('pidgey','Mouth /')[0];head=self.find('pidgey','Head /')[0]
  # Cast upward through the real jaws. The aperture moves with the revised bill;
  # a fixed probe at the previous mouth position could test unrelated empty air.
  def vertical(p,x,z):
   turned=dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,-z,y)])
   return ray_intersections(turned,x,-z)
  back=max(lining['positions'][2::3]);tip=min(max(p['positions'][2::3]) for p in (upper,lower))
  for x in (-.015,0.,.015):
   for f in (.35,.60):
    z=back+(tip-back)*f;u=vertical(upper,x,z);l=vertical(lower,x,z)
    self.assertTrue(u and l,(x,z));bottom=max(l);top=min(u);self.assertGreater(top-bottom,.005*.82)
    y=(bottom+top)*.5;point=(x,y,z)
    self.assertFalse(any(occupied(p,point) for p in self.parts['pidgey']),point)
    hits=ray_intersections(lining,x,y);self.assertTrue(hits,point)
    backing=max(hits);self.assertGreater(z-backing,.01*.82)
    self.assertTrue(occupied(lining,(x,y,backing-.005*.82)))
    self.assertLess(max(ray_intersections(head,x,y)),backing)
 def test_pidgey_shanks_do_not_slope_forward_above_the_toes(self):self.shanks('pidgey','Leg /')
 def test_pidgeotto_has_a_deep_stocky_torso(self):self.assertGreater(span(self.find('pidgeotto','Bird body')[0],2),.43)
 def test_pidgeot_has_a_deep_full_breast(self):self.assertGreater(span(self.find('pidgeot','Body /')[0],2),.43*1.35)
 def test_pidgeotto_wing_vanes_are_broad_profiles_in_the_folded_pose(self):
  ps=self.find('pidgeotto','Flight feather');self.assertGreater(sum(area(p,1,2) for p in ps),sum(area(p,0,1) for p in ps)*5.)
 def test_pidgeot_wing_vanes_are_broad_profiles_in_the_folded_pose(self):
  ps=self.find('pidgeot','Primary /');self.assertGreater(sum(area(p,1,2) for p in ps),sum(area(p,0,1) for p in ps)*3.)
 def test_pidgeot_wing_cases_follow_the_downward_fold_in_profile(self):
  for p in self.find('pidgeot','Wing /'):
   self.assertGreater(span(p,1),span(p,2)*.80)
   self.assertGreater(area(p,1,2),.02*1.35**2)
 def test_pidgeot_folded_flight_tips_end_below_the_lower_breast(self):
  ps=self.find('pidgeot','Primary /')
  self.assertLess(min(v for p in ps for v in p['positions'][1::3]),.30*1.35)
 def test_pidgeotto_resting_wings_stay_below_its_head(self):
  wings=self.find('pidgeotto','Layered wing');head=self.find('pidgeotto','Bird head')[0];self.assertLess(max(v for p in wings for v in p['positions'][1::3]),max(head['positions'][1::3])*.85)
 def test_pidgeot_resting_wings_stay_below_its_head(self):
  wings=self.find('pidgeot','Wing /');head=self.find('pidgeot','Head /')[0];self.assertLess(max(v for p in wings for v in p['positions'][1::3]),max(head['positions'][1::3])*.85)
 def test_pidgeotto_shanks_do_not_slope_forward_above_the_toes(self):self.shanks('pidgeotto','Bird shin')
 def test_pidgeot_shanks_do_not_slope_forward_above_the_toes(self):self.shanks('pidgeot','Leg /')
 def shanks(self,s,prefix):
  h=HEIGHTS[s]
  for p in self.find(s,prefix):
   centers=[]
   for y in (.065*h,.120*h):
    lo,hi=section(p,y);centers.append((lo[1]+hi[1])*.5)
   self.assertGreaterEqual(centers[1]-centers[0],-.006*h,(s,p['part'],centers))
 def test_pidgeotto_has_a_five_blade_swept_crest(self):self.assertEqual(sum(len(components(p)) for p in self.find('pidgeotto','Head crest')),5)
 def test_pidgeot_has_three_gold_and_two_red_crest_blades(self):
  ps=self.find('pidgeot','Crest /');self.assertEqual(len(ps),5);gold=[p for p in ps if p['base_color'][1]>p['base_color'][0]*.45];self.assertEqual(len(gold),3)
 def test_pidgey_stored_idle_has_no_near_loop_jump(self):self.loop('pidgey')
 def test_pidgeotto_stored_idle_has_no_near_loop_jump(self):self.loop('pidgeotto')
 def test_pidgeot_stored_idle_has_no_near_loop_jump(self):self.loop('pidgeot')
 def loop(self,s):
  m=self.models[s];duration=m.clips['idle'][0][1][-1]
  for t in (.00001/duration,1.-.00001/duration):
   mats=m.matrices('idle',t)
   for i,vs in enumerate(m.vertices):
    for a,b in zip(vs,m.posed(i,mats)):self.assertLess(max(abs(x-y) for x,y in zip(a,b)),.00001,(s,t,m.names[i],a,b))
if __name__=='__main__':unittest.main()
