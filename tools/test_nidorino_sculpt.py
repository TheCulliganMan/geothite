#!/usr/bin/env python3
"""Independent stored poison-pin quadruped attachment and fitted surface checks."""
import copy,json,os,unittest
from pathlib import Path
from nidorino_sculpt import sculpture,HEIGHT,side_chart
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate
ROOT=Path(os.environ.get('CRYSTAL_NIDORINO_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class NidorinoTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=json.loads((ROOT/'nidorino.mesh.json').read_bytes());cls.ps=cls.m['primitives']
 def find(self,prefix,s=None):return next(p for p in self.ps if p['part'].startswith(prefix) and (s is None or p['part'].endswith(' '+str(s))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'nidorino':self.m})
 def test_normals_track_actual_faces(self):assert_model_shading(self,{'nidorino':self.m})
 def test_recipe_parity_grounding_height_budget_and_quadruped_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),110000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Horn /',1),('Ear /',2),('Ear inset /',2),('Haunch /',4),('Leg /',4),('Foot /',4),('Claw /',12),('Leg spot /',8),('Flank spot /',10),('Head spot /',4),('Fang /',2),('Back spine /',4),('Tail /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
  head=self.find('Head /');self.assertGreater(max(head['positions'][2::3])-min(head['positions'][2::3]),.5)
 def test_head_jaw_horn_ears_fangs_tail_and_spines_have_volume_roots(self):
  body=self.find('Body /');head=self.find('Head /')
  for a,b in ((head,body),(self.find('Jaw /'),head),(self.find('Horn /'),head),(self.find('Tail /'),body)):
   self.assertTrue(self.overlap(a,b),a['part'])
  for p in self.ps:
   if p['part'].startswith('Back spine /'):self.assertTrue(self.overlap(p,body),p['part'])
  for s in (-1,1):
   self.assertTrue(self.overlap(self.find('Ear /',s),head));self.assertTrue(self.overlap(self.find('Fang /',s),head))
 def test_all_four_leg_chains_and_twelve_claws_have_volume_roots(self):
  for s in (-1,1):
   for placement in ('front','rear'):
    tag=f'{s} {placement}';thigh=self.find('Haunch /',tag);leg=self.find('Leg /',tag);foot=self.find('Foot /',tag)
    for a,b in ((thigh,self.find('Body /')),(leg,thigh),(foot,leg)):self.assertTrue(self.overlap(a,b),a['part'])
    self.assertLess(min(foot['positions'][1::3]),.002)
    for j in range(3):self.assertTrue(self.overlap(self.find('Claw /',tag+' '+str(j)),foot))
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye socket /','Head /'),('Eye /','Eye socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Nostril /','Head /'),('Ear inset /','Ear /')):
    if n.startswith(child):parent=self.find(base,s if base not in ('Head /',) else None)
   if n.startswith('Mouth /'):parent=self.find('Jaw /')
   if n.startswith('Head spot /'):parent=self.find('Head /')
   if n.startswith('Leg spot /'):
    tokens=n.split();s=int(tokens[-3]);tag=' '.join(tokens[-3:-1]);yield side_chart(p,s),side_chart(self.find('Leg /',tag),s);continue
   if n.startswith('Flank spot /'):
    s=int(n.split()[-2]);yield side_chart(p,s),side_chart(self.find('Body /'),s);continue
   if parent:yield p,parent
 def test_fitted_layers_follow_actual_stored_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.016,p['part'])
 def test_fitted_triangle_centers_and_edges_clear_parent(self):
  for p,parent in self.pairs():
   for i in range(0,len(p['indices']),3):
    vv=[p['positions'][3*j:3*j+3] for j in p['indices'][i:i+3]];a,b,c=vv
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
     x,y,z=[sum(vv[k][axis]*weights[k] for k in range(3)) for axis in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(z-max(hits),.0001,p['part'])
 def test_horn_ears_tail_spines_and_legs_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Horn /','Ear /','Ear inset /','Tail /','Back spine /','Leg /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_horn_ear_and_tail_are_rejected(self):
  for prefix,parent in (('Horn /','Head /'),('Ear /','Head /'),('Tail /','Body /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),1,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
