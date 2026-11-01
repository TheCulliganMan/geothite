#!/usr/bin/env python3
"""Stored sea-guardian hand wings, anatomical roots and genuinely open jaw."""
import copy,json,os,unittest
from pathlib import Path
from lugia_sculpt import sculpture,HEIGHT
from lugia_glb import read_lugia,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import indexed_ray_intersections as ray_intersections
from sculpt_test_support import crossed_faces
from sculpt_geometry import translate,ellipsoid
ROOT=Path(os.environ.get('CRYSTAL_LUGIA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class LugiaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_lugia(ROOT/'lugia.glb');cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'lugia':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'lugia':self.m})
 def test_recipe_parity_grounding_height_budget_and_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),80000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Wing arm /',2),('Wing palm /',2),('Wing digit /',10),('Eye flag /',2),('Head crest /',1),('Back plate /',10),('Tail /',1),('Tail fin /',2),('Haunch /',2),('Leg /',2),('Foot /',2),('Toe /',6)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_body_neck_wings_plates_and_tail_have_no_nonadjacent_crossings(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Neck /','Wing arm /','Wing digit /','Eye flag /','Head crest /','Back plate /','Tail /','Tail fin /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_neck_head_crest_eye_flags_and_jaws_have_real_volume_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');self.assertTrue(self.overlap(body,neck));self.assertTrue(self.overlap(neck,head))
  for p in self.ps:
   if p['part'].startswith(('Eye flag /','Head crest /','Jaw /','Mouth /')):self.assertTrue(self.overlap(p,head),p['part'])
  self.assertTrue(self.overlap(self.find('Tongue /'),self.find('Jaw / open')))
 def test_all_ten_broad_wing_digits_are_rooted_in_palm_and_body(self):
  for s in (-1,1):
   arm=self.find('Wing arm /',s);palm=self.find('Wing palm /',s);self.assertTrue(self.overlap(arm,self.find('Body /')));self.assertTrue(self.overlap(arm,palm))
   for j in range(5):self.assertTrue(self.overlap(self.find('Wing digit /',f'{s} {j}'),palm),(s,j))
 def test_back_plates_tail_and_both_terminal_fins_are_supported(self):
  body=self.find('Body /');tail=self.find('Tail /');self.assertTrue(self.overlap(tail,body))
  for p in self.ps:
   if p['part'].startswith('Back plate /'):self.assertTrue(self.overlap(p,body),p['part'])
   elif p['part'].startswith('Tail fin /'):self.assertTrue(self.overlap(p,tail),p['part'])
 def test_powerful_feet_and_all_six_blunt_toes_are_supported_and_grounded(self):
  for s in (-1,1):
   thigh=self.find('Haunch /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(thigh,self.find('Body /')));self.assertTrue(self.overlap(leg,thigh));self.assertTrue(self.overlap(leg,foot));self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):
    toe=self.find('Toe /',f'{s} {j}');self.assertTrue(self.overlap(toe,foot));self.assertLess(min(toe['positions'][1::3]),.006)
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   if n.startswith('Belly /'):parent=self.find('Body /')
   elif n.startswith('Mask /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Mask /',s)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',s)
   if parent:yield p,parent
 def test_blue_belly_and_fitted_eyes_follow_actual_stored_parent_skin(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.018,p['part'])
 def test_fitted_front_centers_and_edges_clear_their_parent(self):
  for p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
     hits=ray_intersections(parent,q[0],q[1]);self.assertTrue(hits);self.assertGreater(q[2]-max(hits),.0001,(p['part'],k//3))
 def jaw_gap(self,parts):
  upper=next(p for p in parts if p['part'].startswith('Jaw / hooked'));lower=next(p for p in parts if p['part'].startswith('Jaw / open'))
  lo=max(min(upper['positions'][2::3]),min(lower['positions'][2::3]));hi=min(max(upper['positions'][2::3]),max(lower['positions'][2::3]));rotated=[]
  for p in parts:
   q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i],p['positions'][i+2],-p['positions'][i+1])];rotated.append(q)
  u=rotated[parts.index(upper)];l=rotated[parts.index(lower)]
  for t in (.60,.70,.80):
   z=lo+(hi-lo)*t;uh=[-y for y in ray_intersections(u,0,z)];lh=[-y for y in ray_intersections(l,0,z)]
   if not uh or not lh:return False
   top=min(uh);bottom=max(lh)
   if top-bottom<.020:return False
   tongue=next(p for p in rotated if p['part'].startswith('Tongue /'));tongue_hits=[-y for y in ray_intersections(tongue,0,z)];floor=max([bottom]+tongue_hits)
   if top-floor<.020:return False
   # Test occupied intervals, not just surface crossings: a solid filler whose
   # boundaries lie beyond both jaws must still invalidate the mouth void.
   for p in rotated:
    # Shared triangle edges are one surface crossing, including the x=0
    # meridian of a float32 ellipsoid. Pair unique occupied boundaries.
    hits=[]
    for hit in sorted(-y for y in ray_intersections(p,0,z)):
     if not hits or abs(hit-hits[-1])>1e-7:hits.append(hit)
    for low,high in zip(hits[::2],hits[1::2]):
     if min(high,top-.0002)-max(low,floor+.0002)>.0001:return False
  return True
 def test_dark_oral_back_is_visible_and_recessed_behind_the_upper_beak(self):
  mouth=self.find('Mouth /');head=self.find('Head /');y=(min(mouth['positions'][1::3])+max(mouth['positions'][1::3]))/2
  mh=ray_intersections(mouth,0,y);hh=ray_intersections(head,0,y);self.assertTrue(mh);self.assertTrue(hh);self.assertGreater(max(mh)-max(hh),.008)
  self.assertGreater(max(self.find('Jaw / hooked')['positions'][2::3])-max(mh),.08)
 def test_open_roar_has_actual_empty_space_above_the_lower_jaw_and_tongue(self):self.assertTrue(self.jaw_gap(self.ps))
 def test_filled_jaw_and_detached_wing_negative_controls_are_rejected(self):
  upper=self.find('Jaw / hooked');lower=self.find('Jaw / open');lo=max(min(upper['positions'][2::3]),min(lower['positions'][2::3]));hi=min(max(upper['positions'][2::3]),max(lower['positions'][2::3]));z=lo+(hi-lo)*.7;ymin=min(lower['positions'][1::3]);ymax=max(upper['positions'][1::3]);filler=ellipsoid('Invalid jaw filler',(.5,.5,.5),(0,(ymin+ymax)/2,z),(.02,(ymax-ymin)*.55,.02),24,16)
  self.assertFalse(self.jaw_gap(self.ps+[filler]));self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Wing arm /',1)),0,0,1),self.find('Body /')))

if __name__=='__main__':unittest.main()
