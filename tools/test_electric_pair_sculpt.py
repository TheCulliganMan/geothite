#!/usr/bin/env python3
"""Stored electric anatomy, stripe roots and genuinely open contact blades."""
import copy,json,os,unittest
from pathlib import Path
from electric_pair_sculpt import sculpture,SPECIES,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate,ellipsoid
from bovine_sculpt import reverse_front
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_ELECTRIC_PAIR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class ElectricPairTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in SPECIES}
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 def each(self):
  for n,m in self.models.items():self.m=m;self.ps=m['primitives'];yield n
 def test_closed_connected_outward_solids(self):assert_closed_models(self,self.models)
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,self.models)
 def test_parity_floor_height_budget_and_distinct_electric_anatomy(self):
  for n in self.each():
   self.assertEqual(self.m,sculpture(n));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),115000)
   self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHTS[n],places=6)
   counts=(('Head /',1),('Crest /',2),('Antenna /',2),('Antenna ball /',2),('Jaw /',2),('Fang /',2),('Knuckle /',6),('Thumb /',2),('Wrist fur /',4),('Tail tip /',1),('Arm stripe /',6),('Leg stripe /',4),('Toe claw /',6),('Back stripe /',1),('Tail /',1),('Tail stripe /',4)) if n=='electabuzz' else (('Head /',0),('Plug /',2),('Arm root /',2),('Arm stripe /',4),('Hand claw /',6),('Belly band /',1))
   for prefix,num in counts+(('Body /',1),('Eye /',2),('Hand /',2),('Leg /',2),('Foot /',2),('Bolt /',1)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),num,prefix)
 def test_adult_head_jaws_fangs_tongue_crest_and_antennae_are_rooted(self):
  self.m=self.models['electabuzz'];self.ps=self.m['primitives'];head=self.find('Head /');neck=self.find('Neck /');upper=self.find('Jaw / rooted yellow');lower=self.find('Jaw / rooted open')
  for a,b in ((head,neck),(neck,self.find('Body /')),(upper,head),(lower,head),(self.find('Mouth back /'),head),(self.find('Tongue /'),lower)):self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):
   for a,b in ((self.find('Crest /',s),head),(self.find('Antenna /',s),head),(self.find('Antenna ball /',s),self.find('Antenna /',s)),(self.find('Fang /',s),upper)):self.assertTrue(self.overlap(a,b),a['part'])
 def test_both_arms_hands_digits_and_stripes_have_volume_roots(self):
  for n in self.each():
   for s in (-1,1):
    arm=self.find('Arm /',s);hand=self.find('Hand /',s)
    if n=='elekid':root=self.find('Arm root /',s);self.assertTrue(self.overlap(root,self.find('Body /')));self.assertTrue(self.overlap(arm,root))
    else:self.assertTrue(self.overlap(arm,self.find('Body /')))
    self.assertTrue(self.overlap(hand,arm))
    for j in range(2 if n=='elekid' else 3):
     stripe=next(p for p in self.ps if p['part'].startswith('Arm stripe /') and p['part'].endswith(f' {s} {j}'));self.assertTrue(self.overlap(stripe,arm),stripe['part'])
    for j in range(3):self.assertTrue(self.overlap(self.find('Hand claw /' if n=='elekid' else 'Knuckle /',f'{s} {j}'),hand))
    if n=='electabuzz':
     self.assertTrue(self.overlap(self.find('Thumb /',s),hand))
     for j in range(2):self.assertTrue(self.overlap(self.find('Wrist fur /',f'{s} {j}'),arm))
 def test_both_legs_feet_and_adult_claws_are_rooted_and_grounded(self):
  for n in self.each():
   for s in (-1,1):
    leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(leg,self.find('Body /')));self.assertTrue(self.overlap(foot,leg));self.assertLess(min(foot['positions'][1::3]),.002)
    if n=='electabuzz':
     for j in range(3):self.assertTrue(self.overlap(self.find('Toe claw /',f'{s} {j}'),foot))
     for j in range(2):
      stripe=next(p for p in self.ps if p['part'].startswith('Leg stripe /') and p['part'].endswith(f' {s} {j}'));self.assertTrue(self.overlap(stripe,leg))
 def test_tail_stripes_and_baby_waist_band_have_volume_roots(self):
  self.m=self.models['electabuzz'];self.ps=self.m['primitives'];tail=self.find('Tail /');self.assertTrue(self.overlap(tail,self.find('Body /')))
  self.assertTrue(self.overlap(self.find('Tail tip /'),tail))
  for j in range(4):self.assertTrue(self.overlap(self.find('Tail stripe /',j),tail))
  self.m=self.models['elekid'];self.ps=self.m['primitives'];self.assertTrue(self.overlap(self.find('Belly band /'),self.find('Body /')))
 def test_baby_plug_blades_are_rooted_and_have_real_unfilled_contact_holes(self):
  self.m=self.models['elekid'];self.ps=self.m['primitives']
  for s in (-1,1):
   blade=self.find('Plug /',s);self.assertTrue(self.overlap(blade,self.find('Body /')))
   x=(min(blade['positions'][::3])+max(blade['positions'][::3]))/2;low=min(blade['positions'][1::3]);high=max(blade['positions'][1::3]);y=low+(high-low)*(1.153-.760)/(1.285-.760)
   for dx,dy in ((0,0),(.008,0),(-.008,0),(0,.011),(0,-.011)):
    self.assertFalse(ray_intersections(blade,x+dx,y+dy),'contact hole is capped')
    self.assertFalse(any(ray_intersections(p,x+dx,y+dy) for p in self.ps),'other anatomy blocks the contact hole')
   filled=ellipsoid('filled contact negative',(.5,.5,.5),(x,y,.010),(.02,.03,.04),32,22);self.assertTrue(ray_intersections(filled,x,y))
 def test_adult_open_jaw_has_unoccupied_front_cavity_and_rejects_a_fill(self):
  self.m=self.models['electabuzz'];self.ps=self.m['primitives'];upper=self.find('Jaw / rooted yellow');lower=self.find('Jaw / rooted open')
  y=(min(upper['positions'][1::3])+max(lower['positions'][1::3]))/2;z=.95*max(lower['positions'][2::3])
  def occupied(p,x,y,z):
   hits=ray_intersections(p,x,y);return any(a+.0001<z<b-.0001 for a,b in zip(hits[::2],hits[1::2]))
  for dx in (-.025,0,.025):
   for dy in (0,-.007,-.012):self.assertFalse(any(occupied(p,dx,y+dy,z) for p in self.ps),'jaw cavity is filled')
  filled=ellipsoid('filled jaw negative',(.5,.5,.5),(0,y,z),(.04,.024,.025),32,22);self.assertTrue(occupied(filled,0,y,z))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye socket /','Body /' if self.m['name']=='elekid' else 'Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /')):
    if n.startswith(child):parent=self.find(base,s if base not in ('Body /','Head /') else None)
   if n.startswith(('Bolt /','Flank stripe /','Back stripe /')):parent=self.find('Body /')
   if n.startswith('Forehead /'):parent=self.find('Head /')
   if n.startswith('Mouth /'):parent=self.find('Body /')
   if n.startswith('Nostril /'):parent=self.find('Jaw / rooted yellow')
   if parent:
    if n.startswith('Back stripe /'):yield reverse_front(p),reverse_front(parent)
    else:yield p,parent
 def test_fitted_layers_follow_actual_stored_parent(self):
  for n in self.each():attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_parent(self):
  for n in self.each():attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_stripes_tail_claws_and_plugs_do_not_cross_themselves(self):
  for n in self.each():
   for p in self.ps:
    if p['part'].startswith(('Arm /','Arm root /','Arm stripe /','Leg /','Leg stripe /','Tail /','Tail stripe /','Plug /','Hand claw /','Toe claw /','Crest /','Fang /','Wrist fur /')):self.assertIsNone(crossed_faces(p),(n,p['part']))
 def test_detached_tail_plug_hand_and_toe_are_rejected(self):
  for n in self.each():
   for child,parent in ((('Tail /','Body /'),('Toe claw /','Foot /')) if n=='electabuzz' else (('Plug /','Body /'),('Hand claw /','Hand /'))):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
