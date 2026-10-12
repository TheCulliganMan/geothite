#!/usr/bin/env python3
"""Independent stored phoenix anatomy, feather support and color-layer checks."""
import copy,json,os,unittest
from pathlib import Path
from ho_oh_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate,ellipsoid
from bovine_sculpt import reverse_front
ROOT=Path(os.environ.get('CRYSTAL_HO_OH_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class HoOhTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'ho_oh.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,suffix=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'ho_oh':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'ho_oh':self.m})
 def test_recipe_parity_height_floor_budget_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),120000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Wing /',2),('Flight /',24),('Flight color /',96),('Covert /',12),('Tail /',9),('Tail color /',18),('Crown /',5),('Haunch /',2),('Toe /',8),('Talon /',8)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_body_neck_wings_feathers_tail_and_crown_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Neck /','Wing /','Flight /','Flight color /','Tail /','Tail color /','Crown /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_head_neck_jaws_crest_and_all_tail_feathers_have_volume_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');self.assertTrue(self.overlap(body,neck));self.assertTrue(self.overlap(neck,head))
  for p in self.ps:
   if p['part'].startswith(('Crown /','Jaw /','Mouth /')):self.assertTrue(self.overlap(p,head),p['part'])
   elif p['part'].startswith('Tail /'):self.assertTrue(self.overlap(p,body),p['part'])
 def test_all_primary_and_covert_feathers_have_supported_wing_roots(self):
  for s in (-1,1):
   wing=self.find('Wing /',s);self.assertTrue(self.overlap(wing,self.find('Body /')))
   for j in range(12):self.assertTrue(self.overlap(self.find('Flight /',f'{s} {j}'),wing),(s,j))
   for j in range(6):self.assertTrue(self.overlap(self.find('Covert /',f'{s} {j}'),wing),(s,j))
 def test_feet_all_digits_and_talons_have_roots_and_remain_grounded(self):
  for s in (-1,1):
   thigh=self.find('Haunch /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s);self.assertTrue(self.overlap(thigh,self.find('Body /')));self.assertTrue(self.overlap(leg,thigh));self.assertTrue(self.overlap(foot,leg));self.assertLess(min(foot['positions'][1::3]),.025)
   for j in range(3):
    toe=self.find('Toe / three',f'{s} {j}');self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / forward',f'{s} {j}')))
   toe=self.find('Toe / rear',s);self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find('Talon / rear',s)))
 def pairs(self):
  for p in self.ps:
   n=p['part'];words=n.split();s=words[-1];parent=None
   if n.startswith('Belly /'):parent=self.find('Body /')
   elif n.startswith(('Collar /','Collar edge /')):parent=self.find('Neck /')
   elif n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',s)
   elif n.startswith('Iris /'):parent=self.find('Eye /',s)
   elif n.startswith('Pupil /'):parent=self.find('Iris /',s)
   elif n.startswith('Flight color /'):
    side,j=words[3:5]
    parent=self.find('Flight /',f'{side} {j}') if 'ivory band' in n else next(q for q in self.ps if q['part']==f'Flight color / {side} {j} ivory band {s}')
   elif n.startswith('Tail color /'):parent=self.find('Tail /',words[3])
   if parent:
    if n.startswith(('Collar /','Collar edge /','Flight color /','Tail color /')) and s=='-1':yield reverse_front(p),reverse_front(parent)
    else:yield p,parent
 def test_all_fitted_colors_follow_actual_stored_parent_skin(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.016,p['part'])
 def test_fitted_front_triangle_centers_and_edges_clear_their_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['indices']),3):
    vv=[p['positions'][3*j:3*j+3] for j in p['indices'][i:i+3]]
    # Select front faces using the actual outward geometric normal.
    a,b,c=vv;z=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    if z<=1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
     x,y,d=[sum(vv[k][axis]*weights[k] for k in range(3)) for axis in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(d-max(hits),.0001,p['part'])
 def jaw_gap(self,parts):
  upper=self.find('Jaw / long');lower=self.find('Jaw / open');lo=max(min(upper['positions'][2::3]),min(lower['positions'][2::3]));hi=min(max(upper['positions'][2::3]),max(lower['positions'][2::3]));rotated=[]
  for p in parts:
   q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i],p['positions'][i+2],-p['positions'][i+1])];rotated.append(q)
  u=rotated[parts.index(upper)];l=rotated[parts.index(lower)]
  for t in (.60,.70,.80):
   z=lo+(hi-lo)*t;uh=[-y for y in ray_intersections(u,0,z)];lh=[-y for y in ray_intersections(l,0,z)]
   if not uh or not lh:return False
   top=min(uh);bottom=max(lh)
   if top-bottom<.020:return False
   for p in rotated:
    hits=sorted(-y for y in ray_intersections(p,0,z))
    for low,high in zip(hits[::2],hits[1::2]):
     if min(high,top-.0002)-max(low,bottom+.0002)>.0001:return False
  return True
 def test_phoenix_roar_contains_actual_empty_space(self):self.assertTrue(self.jaw_gap(self.ps))
 def test_filled_phoenix_jaw_is_rejected(self):
  upper=self.find('Jaw / long');lower=self.find('Jaw / open');lo=max(min(upper['positions'][2::3]),min(lower['positions'][2::3]));hi=min(max(upper['positions'][2::3]),max(lower['positions'][2::3]));z=lo+(hi-lo)*.7;ymin=min(lower['positions'][1::3]);ymax=max(upper['positions'][1::3]);filler=ellipsoid('Invalid phoenix jaw filler',(.5,.5,.5),(0,(ymin+ymax)/2,z),(.02,(ymax-ymin)*.55,.02),24,16)
  self.assertFalse(self.jaw_gap(self.ps+[filler]))
 def test_dark_oral_lining_is_visible_ahead_of_head_and_recessed_behind_beak(self):
  mouth=self.find('Mouth /');head=self.find('Head /');y=(min(mouth['positions'][1::3])+max(mouth['positions'][1::3]))/2;mh=ray_intersections(mouth,0,y);hh=ray_intersections(head,0,y);self.assertTrue(mh);self.assertTrue(hh);self.assertGreater(max(mh)-max(hh),.008);self.assertGreater(max(self.find('Jaw / long')['positions'][2::3])-max(mh),.08)
 def test_detached_flight_feather_is_rejected(self):
  feather=copy.deepcopy(self.find('Flight /','1 5'));translate(feather,0,0,1);self.assertFalse(self.overlap(feather,self.find('Wing /',1)))

if __name__=='__main__':unittest.main()
