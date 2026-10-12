#!/usr/bin/env python3
"""Independent stored topology, attachment and fitted poison-pin marking checks."""
import copy,json,os,unittest
from pathlib import Path
from nidorina_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from bovine_sculpt import reverse_front
from goldfish_sculpt import zy
from sculpt_geometry import translate
ROOT=Path(os.environ.get('CRYSTAL_NIDORINA_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class NidorinaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'nidorina.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,s=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (s is None or p['part'].endswith(' '+str(s))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'nidorina':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'nidorina':self.m})
 def test_parity_floor_height_budget_and_hornless_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  self.assertFalse(any('horn' in p['part'].lower() and 'hornless' not in p['part'].lower() for p in self.ps))
  for prefix,n in (('Ear /',2),('Ear inset /',2),('Ear fold /',2),('Eye /',2),('Fang /',2),('Hand claw /',6),('Foot claw /',6),('Haunch spot /',6),('Flank spot /',2),('Back spine /',3),('Tail /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
 def test_head_muzzle_ears_fangs_tail_and_spines_have_volume_roots(self):
  body=self.find('Body /');head=self.find('Head /');muzzle=self.find('Muzzle /')
  for a,b in ((head,body),(muzzle,head),(self.find('Tail /'),body)):
   self.assertTrue(self.overlap(a,b),a['part'])
  for p in self.ps:
   if p['part'].startswith('Back spine /'):self.assertTrue(self.overlap(p,body),p['part'])
  for s in (-1,1):
   self.assertTrue(self.overlap(self.find('Ear /',s),head));self.assertTrue(self.overlap(self.find('Fang /',s),muzzle))
 def test_all_limbs_and_claws_have_volume_roots(self):
  for s in (-1,1):
   body=self.find('Body /');arm=self.find('Arm /',s);hand=self.find('Hand /',s);thigh=self.find('Haunch /',s);leg=self.find('Leg /',s);foot=self.find('Foot /',s)
   for a,b in ((arm,body),(hand,arm),(thigh,body),(leg,thigh),(foot,leg)):
    self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(foot['positions'][1::3]),.002)
   for j in range(3):
    self.assertTrue(self.overlap(self.find('Hand claw /',f'{s} {j}'),hand));self.assertTrue(self.overlap(self.find('Foot claw /',f'{s} {j}'),foot))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Ear inset /','Ear /'),('Ear fold /','Ear inset /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Nostril /','Muzzle /')):
    if n.startswith(child):parent=self.find(base,s if base not in ('Head /','Muzzle /') else None)
   if n.startswith('Breast /'):parent=self.find('Body /')
   if n.startswith('Mouth /'):parent=self.find('Muzzle /')
   if n.startswith(('Haunch spot /','Flank spot /')):
    s=int(n.split()[-2] if n.startswith('Haunch spot /') else n.split()[-1]);parent=self.find('Haunch /',s) if n.startswith('Haunch spot /') else self.find('Body /')
    a,b=zy(p),zy(parent)
    if s==1:a,b=reverse_front(a),reverse_front(b)
    yield a,b;continue
   if parent:yield p,parent
 def test_fitted_layers_follow_actual_stored_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.018,p['part'])
 def test_fitted_face_centers_and_edges_clear_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['indices']),3):
    vv=[p['positions'][3*j:3*j+3] for j in p['indices'][i:i+3]];a,b,c=vv
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
     x,y,z=[sum(vv[k][axis]*weights[k] for k in range(3)) for axis in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(z-max(hits),.0001,p['part'])
 def test_ears_tail_spines_and_limbs_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Ear /','Ear inset /','Ear fold /','Tail /','Back spine /','Arm /','Leg /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_tail_ear_and_spine_are_rejected(self):
  for prefix,parent in (('Tail /','Body /'),('Ear /','Head /'),('Back spine /','Body /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),1,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
