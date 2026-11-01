#!/usr/bin/env python3
"""Actual stone-shell coverage, surface roots, reptile anatomy and open jaw."""
import copy,json,os,unittest
from pathlib import Path
from golem_sculpt import anatomy,sculpture,shell_cells,head_point,HEIGHT,INK
from steelix_sculpt import block
from test_steelix_sculpt import occupied
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_GOLEM_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class GolemTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=__import__('stone_pair_glb').read_stone_pair(ROOT/'golem.glb', 'golem')
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'golem':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'golem':self.m})
 def test_recipe_parity_height_floor_budget_and_anatomy(self):
  from stone_pair_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Shell /',1),('Shell plate /',32),('Head /',1),('Arm /',2),('Forearm /',2),('Hand claw /',6),('Foot claw /',6),('Heel claw /',2),('Fang /',2),('Lower tooth /',3)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_polygon_cell_edges_cover_a_complete_spherical_shell(self):
  cells=shell_cells();self.assertEqual(sum(len(c)==5 for c in cells),12);self.assertEqual(sum(len(c)==6 for c in cells),20)
  from collections import Counter
  edges=Counter(tuple(sorted((tuple(round(q,7) for q in a),tuple(round(q,7) for q in b)))) for c in cells for a,b in zip(c,c[1:]+c[:1]));self.assertTrue(all(n==2 for n in edges.values()));self.assertEqual(len(edges),90)
 def test_every_shell_plate_roots_into_actual_seam_body(self):
  shell=self.find('Shell /')
  for p in self.ps:
   if p['part'].startswith('Shell plate /'):self.assertTrue(self.overlap(p,shell),p['part'])
 def test_shell_plate_outer_faces_clear_and_inner_faces_root_into_body(self):
  shell=self.find('Shell /')
  for j,cell in enumerate(shell_cells()):
   p=self.find('Shell plate /',j);vertices=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)];count=7*len(cell)*3
   for layer,inside in ((0,False),(count,True)):
    for i in range(layer,layer+count,3):
     vv=[vertices[k] for k in p['indices'][i:i+3]];point=tuple(sum(v[k] for v in vv)/3 for k in range(3));self.assertEqual(occupied(shell,point),inside,(j,layer,i))
 def test_head_muzzle_jaw_chamber_tongue_and_teeth_have_actual_roots(self):
  shell=self.find('Shell /');head=self.find('Head /');upper=self.find('Muzzle /');jaw=self.find('Jaw /')
  for child,parent in ((head,shell),(upper,head),(jaw,shell),(self.find('Mouth back /'),upper),(self.find('Tongue /'),jaw)):self.assertTrue(self.overlap(child,parent),(child['part'],parent['part']))
  for p in self.ps:
   if p['part'].startswith('Fang /'):self.assertTrue(self.overlap(p,upper))
   if p['part'].startswith('Lower tooth /'):self.assertTrue(self.overlap(p,jaw))
 def test_all_limbs_and_claws_have_actual_volume_roots(self):
  shell=self.find('Shell /')
  for s in (-1,1):
   for child,parent in (('Arm /','Shell /'),('Forearm /','Arm /'),('Hand /','Forearm /'),('Thigh /','Shell /'),('Leg /','Thigh /'),('Foot /','Leg /'),('Heel claw /','Foot /')):self.assertTrue(self.overlap(self.find(child,s),shell if parent=='Shell /' else self.find(parent,s)),(child,s))
   for j in range(3):
    self.assertTrue(self.overlap(self.find('Hand claw /',f'{s} {j}'),self.find('Hand /',s)))
    self.assertTrue(self.overlap(self.find('Foot claw /',f'{s} {j}'),self.find('Foot /',s)))
 def test_both_grounded_pads_support_the_raised_shell(self):
  for s in (-1,1):self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
  self.assertGreater(min(self.find('Shell /')['positions'][1::3]),.1)
 def test_low_skull_and_forward_lowered_claws_carry_the_shell(self):
  head=self.find('Head /');shell=self.find('Shell /')
  self.assertLess(max(head['positions'][1::3]),.60*HEIGHT)
  self.assertLess(max(head['positions'][::3])-min(head['positions'][::3]),.45*(max(shell['positions'][::3])-min(shell['positions'][::3])))
  for s in (-1,1):
   hand=self.find('Hand /',s)
   self.assertLess(max(hand['positions'][1::3]),.65*HEIGHT)
   for j in range(3):
    claw=self.find('Hand claw /',f'{s} {j}')
    self.assertLess(max(claw['positions'][1::3]),max(hand['positions'][1::3]))
    self.assertGreater(max(claw['positions'][2::3]),max(hand['positions'][2::3]))
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Socket /','Head /'),('Eye /','Socket /'),('Iris /','Eye /'),('Pupil /','Iris /')):
    if p['part'].startswith(child):yield p,self.find(parent,None if parent=='Head /' else s)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def aperture_points(self):
  raw=anatomy();lo=min(v for p in raw for v in p['positions'][1::3]);hi=max(v for p in raw for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  return [(px*scale,(py-lo)*scale,pz*scale) for x in (-.04,0,.04) for y in (.735,.742) for z in (.67,.71) for px,py,pz in (head_point((x,y,z)),)]
 def test_open_jaw_aperture_is_actually_unoccupied(self):
  from sculpt_test_support import indexed_ray_intersections
  def vertical(p):return dict(p,positions=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i],p['positions'][i+2],p['positions'][i+1])])
  upper=vertical(self.find('Muzzle /'));lower=vertical(self.find('Jaw /'))
  for point in self.aperture_points():
   x,y,z=point;above=indexed_ray_intersections(upper,x,z);below=indexed_ray_intersections(lower,x,z)
   self.assertTrue(above and below,('aperture sample left the actual jaws',point))
   self.assertLess(max(below),y);self.assertLess(y,min(above))
   self.assertFalse(any(occupied(p,point) for p in self.ps),point)
 def test_filled_jaw_and_detached_armor_negatives(self):
  points=self.aperture_points();xs,ys,zs=zip(*points);fill=block('Filled mouth negative',INK,tuple((min(q)+max(q))/2 for q in (xs,ys,zs)),tuple((max(q)-min(q))/2+.01 for q in (xs,ys,zs)))
  self.assertTrue(all(occupied(fill,q) for q in points))
  for child,parent in (('Shell plate /','Shell /'),('Hand claw /','Hand /'),('Fang /','Muzzle /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
 def test_curved_limbs_and_claws_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Forearm /','Hand claw /','Foot claw /','Heel claw /','Fang /')):self.assertIsNone(crossed_faces(p),p['part'])
if __name__=='__main__':unittest.main()
