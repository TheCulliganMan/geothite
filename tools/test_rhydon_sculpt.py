#!/usr/bin/env python3
"""Stored drill-biped anatomy, surface attachments, roots and mouth clearance."""
import copy,json,os,unittest
from pathlib import Path
from rhydon_sculpt import anatomy,sculpture,HEIGHT
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import indexed_ray_intersections as rays
from test_kingdra_sculpt import crossed_faces
from test_steelix_sculpt import occupied
from steelix_sculpt import block,INK
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_RHYDON_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class RhydonTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from rhino_glb import read_rhydon
  cls.model=read_rhydon(ROOT/'rhydon.glb')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'rhydon':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'rhydon':self.m})
 def test_recipe_parity_height_floor_budget_and_anatomy(self):
  from rhino_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Belly plate /',4),('Ear /',2),('Hand claw /',6),('Foot claw /',4),('Fang /',2),('Horn /',1),('Horn spiral /',1),('Tail armor /',3),('Cheek armor /',6)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_head_muzzle_jaw_and_oral_chamber_have_volume_roots(self):
  body=self.find('Body /');head=self.find('Head /');self.assertTrue(self.overlap(head,body));self.assertTrue(self.overlap(self.find('Muzzle /'),head));self.assertTrue(self.overlap(self.find('Jaw /'),body))
  self.assertTrue(self.overlap(self.find('Mouth back /'),self.find('Muzzle /')));self.assertTrue(self.overlap(self.find('Tongue /'),self.find('Jaw /')))
 def test_ears_horn_and_cheek_armor_have_actual_roots(self):
  head=self.find('Head /')
  for p in self.ps:
   if p['part'].startswith(('Ear /','Cheek armor /')):self.assertTrue(self.overlap(p,head),p['part'])
  self.assertTrue(self.overlap(self.find('Horn /'),self.find('Muzzle /')));self.assertTrue(self.overlap(self.find('Horn spiral /'),self.find('Horn /')))
 def test_limbs_claws_and_teeth_have_actual_roots(self):
  body=self.find('Body /')
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /'),('Thigh /','Body /'),('Leg /','Thigh /'),('Foot /','Leg /')):self.assertTrue(self.overlap(self.find(child,s),body if parent=='Body /' else self.find(parent,s)),(child,s))
   for j in range(3):self.assertTrue(self.overlap(self.find('Hand claw /',str(s)+' '+str(j)),self.find('Hand /',s)))
   for j in range(2):self.assertTrue(self.overlap(self.find('Foot claw /',str(s)+' '+str(j)),self.find('Foot /',s)))
   self.assertTrue(self.overlap(self.find('Fang /',s),self.find('Muzzle /')))
 def test_tail_dorsal_armor_and_collars_have_volume_roots(self):
  tail=self.find('Tail /');body=self.find('Body /');self.assertTrue(self.overlap(tail,body))
  for p in self.ps:
   if p['part'].startswith('Tail armor /'):self.assertTrue(self.overlap(p,tail),p['part'])
   if p['part'].startswith('Back armor /'):self.assertTrue(self.overlap(p,body),p['part'])
 def test_tail_band_faces_clear_and_root_into_actual_tail(self):
  tail=self.find('Tail /')
  for collar in (p for p in self.ps if p['part'].startswith('Tail armor /')):
   vertices=[collar['positions'][i:i+3] for i in range(0,len(collar['positions']),3)];indices=collar['indices']
   # Two longitudinal intervals, 32 radial quads, paired outer/inner faces.
   for offset in range(0,2*32*12,12):
    for layer,inside in ((0,False),(6,True)):
     for t in (0,3):
      tri=[vertices[k] for k in indices[offset+layer+t:offset+layer+t+3]];point=tuple(sum(v[k] for v in tri)/3 for k in range(3))
      self.assertEqual(occupied(tail,point),inside,(collar['part'],offset,layer,t))
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Belly plate /','Body /'),('Collar /','Body /'),('Ear inset /','Ear /'),('Socket /','Head /'),('Eye /','Socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Nostril /','Muzzle /')):
    if p['part'].startswith(child):yield p,self.find(parent,None if parent in ('Body /','Head /','Muzzle /') else s)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_horn_tail_and_claws_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Horn /','Horn spiral /','Tail /','Hand claw /','Foot claw /','Arm /','Leg /','Cheek armor /')):self.assertIsNone(crossed_faces(p),p['part'])
 def aperture_points(self):
  # Locate the current lips from stored vertical ray hits. Former world-space
  # probes can test empty air when a muzzle is shortened or its jaw raised.
  upper=self.find('Muzzle /');lower=self.find('Jaw /')
  def vertical(p):return dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,z,y)])
  up,down=vertical(upper),vertical(lower);front=max(upper['positions'][2::3]);points=[]
  for x in (-.035*HEIGHT,0.,.035*HEIGHT):
   for z in (front*.72,front*.79):
    a=rays(down,x,z);b=rays(up,x,z);self.assertTrue(a and b,(x,z,'missing actual lip'))
    floor=max(a);ceiling=min(b);self.assertGreater(ceiling-floor,.006*HEIGHT)
    points += [(x,floor+(ceiling-floor)*t,z) for t in (.65,.85)]
  return points
 def test_open_jaw_has_an_actual_unoccupied_aperture(self):
  for point in self.aperture_points():self.assertFalse(any(occupied(p,point) for p in self.ps),point)
 def test_filled_jaw_negative_is_rejected(self):
  points=self.aperture_points();xs,ys,zs=zip(*points);fill=block('Filled mouth negative',INK,tuple((min(q)+max(q))/2 for q in (xs,ys,zs)),tuple((max(q)-min(q))/2+.01 for q in (xs,ys,zs)))
  self.assertTrue(all(occupied(fill,q) for q in points))
 def test_detached_digits_horn_and_tail_are_rejected(self):
  for child,parent in (('Hand claw /','Hand /'),('Horn /','Muzzle /'),('Tail /','Body /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
