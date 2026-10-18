#!/usr/bin/env python3
"""Physical roots, fitted markings and solid anatomy for the tongue bearer."""
import copy,json,os,unittest
from pathlib import Path
from lickitung_sculpt import sculpture,HEIGHT,PINK
from sculpt_geometry import translate,linear
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_LICKITUNG_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class LickitungTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from lickitung_glb import read_lickitung
  cls.model=read_lickitung(ROOT/'lickitung.glb')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_outward_surfaces(self):assert_closed_models(self,{'lickitung':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'lickitung':self.m})
 def test_recipe_palette_floor_height_and_budget(self):
  from lickitung_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000)
  self.assertEqual(self.find('Head /')['base_color'],linear(PINK));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Arm /',2),('Hand /',0),('Tongue /',1),('Tongue groove /',1),('Belly band /',4),('Thigh ring /',2),('Thigh center /',2),('Toe /',2),('Hand tip /',2)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_oral_bowl_is_an_actual_recess_in_the_closed_head(self):
  from sculpt_test_support import indexed_ray_intersections as rays
  mouth=self.find('Mouth lining /');head=self.find('Head /');xs=mouth['positions'][::3];ys=mouth['positions'][1::3];x=(min(xs)+max(xs))/2;y=(min(ys)+max(ys))/2
  center=sorted(set(rays(head,x,y)));self.assertGreaterEqual(len(center),2);self.assertGreater(max(center)-min(center),.10)
  for edge in (x+.75*(min(xs)-x),x+.75*(max(xs)-x)):self.assertGreater(max(rays(head,edge,y))-max(center),.025)
 def test_oral_lining_stays_close_to_the_actual_recess(self):
  from sculpt_test_support import indexed_ray_intersections as rays
  mouth=self.find('Mouth lining /');head=self.find('Head /');vs=[mouth['positions'][i:i+3] for i in range(0,len(mouth['positions']),3)]
  for k in range(0,len(mouth['indices']),3):
   tri=[vs[i] for i in mouth['indices'][k:k+3]];a,b,c=tri
   if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
   for ws in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
    x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(head,x,y);self.assertTrue(hits);gap=z-max(hits);self.assertGreater(gap,.0001);self.assertLess(gap,HEIGHT*.001)
 def test_head_neck_tongue_and_tail_have_actual_roots(self):
  for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Tongue /','Head /'),('Tail /','Body /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),child)
 def test_all_limbs_tips_and_toes_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand tip /','Arm /'),('Thigh /','Body /'),('Foot /','Thigh /'),('Toe /','Foot /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,s if parent!='Body /' else None)),(s,child))
   self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Mouth lining /','Head /'),('Tongue groove /','Tongue /'),('Belly band /','Body /'),('Eye /','Head /'),('Eye glint /','Eye /'),('Nostril /','Head /'),('Thigh ring /','Thigh /'),('Thigh center /','Thigh ring /')):
    if p['part'].startswith(child):yield p,self.find(parent,s if parent in ('Eye /','Thigh /','Thigh ring /') else None)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_faces_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_tongue_and_tail_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Tongue /','Tail /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_long_broad_tongue_and_heavy_tail(self):
  tongue=self.find('Tongue /');tail=self.find('Tail /');self.assertGreater(max(tongue['positions'][2::3]),max(self.find('Head /')['positions'][2::3])+.4);self.assertGreater(max(tongue['positions'][::3])-min(tongue['positions'][::3]),.20);self.assertGreater(max(tail['positions'][1::3]),.7);self.assertLess(min(tail['positions'][2::3]),-.5)
 def test_detached_tongue_and_appendages_are_rejected(self):
  for child,parent in (('Tongue /','Head /'),('Tail /','Body /'),('Hand tip /','Arm /'),('Toe /','Foot /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
