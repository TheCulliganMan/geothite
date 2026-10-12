#!/usr/bin/env python3
"""Stored Steelix topology, rooted armor, fitted gaze and actual jaw aperture."""
import copy,json,os,re,unittest
from pathlib import Path
from steelix_sculpt import anatomy,sculpture,HEIGHT,CHAIN,HEAD_X,block,INK,body_reference_span
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_slime_sculpt import ray_intersections
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_STEELIX_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

def occupied(p,point):
 x,y,z=point;hits=sorted(ray_intersections(p,x,y));unique=[]
 for q in hits:
  if not unique or abs(q-unique[-1])>1e-7:unique.append(q)
 return sum(q>z+1e-8 for q in unique)%2==1

class SteelixTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'steelix.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_battle_reference_matches_normalized_authored_centerline(self):
  code=(Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/src/battle_layout.rs').read_text();match=re.search(r'"STEELIX" => ([0-9._]+) / ([0-9.]+)',code);self.assertIsNotNone(match);self.assertAlmostEqual(float(match[1].replace('_','')),body_reference_span(),places=6);self.assertEqual(float(match[2]),HEIGHT)
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'steelix':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'steelix':self.m})
 def test_recipe_parity_height_budget_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),45000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Segment /',10),('Joint /',9),('Spike /',8),('Upper tooth /',4),('Lower tooth /',3),('Iris /',2),('Head /',1),('Jaw /',1),('Tail /',1)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
  self.assertFalse(any('prong' in p['part'].lower() for p in self.ps))
 def test_adjacent_rocks_and_buried_joints_have_volume_roots(self):
  for j in range(len(CHAIN)-1):
   a=self.find('Segment /',j);b=self.find('Segment /',j+1);joint=self.find('Joint /',j)
   for u,v in ((a,b),(joint,a),(joint,b)):self.assertTrue(self.overlap(u,v),(u['part'],v['part']))
 def test_spikes_tail_head_and_jaw_hinges_have_actual_roots(self):
  for p in self.ps:
   if p['part'].startswith('Spike /'):self.assertTrue(self.overlap(p,self.find('Segment /',p['part'].split()[-2])))
  self.assertTrue(self.overlap(self.find('Tail /'),self.find('Segment /',0)))
  head=self.find('Head /');jaw=self.find('Jaw /');self.assertTrue(self.overlap(head,self.find('Segment /',9)))
  for s in (-1,1):
   hinge=self.find('Hinge /',s)
   for parent in (head,jaw):self.assertTrue(self.overlap(hinge,parent),parent['part'])
  for parent in (head,jaw):self.assertTrue(self.overlap(self.find('Mouth back /'),parent))
 def test_all_squared_teeth_have_actual_jaw_roots(self):
  for p in self.ps:
   for child,parent in (('Upper tooth /','Head /'),('Lower tooth /','Jaw /')):
    if p['part'].startswith(child):self.assertTrue(self.overlap(p,self.find(parent)),p['part'])
 def test_tail_boulders_support_the_raised_head(self):
  for j in (0,1):self.assertLess(min(self.find('Segment /',j)['positions'][1::3]),.007)
  self.assertGreater(min(self.find('Head /')['positions'][1::3]),.8)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Socket /','Head /'),('Eye /','Socket /'),('Iris /','Eye /'),('Pupil /','Iris /')):
    if p['part'].startswith(child):yield p,self.find(parent,None if parent=='Head /' else s)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def aperture_points(self):
  raw=anatomy();lo=min(v for p in raw for v in p['positions'][1::3]);hi=max(v for p in raw for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  return [(x*scale,(y-lo)*scale,z*scale) for x in (HEAD_X-.077,HEAD_X,HEAD_X+.077) for y in (.965,.977) for z in (.435,.49)]
 def test_front_jaw_aperture_is_actually_empty(self):
  for point in self.aperture_points():self.assertFalse(any(occupied(p,point) for p in self.ps),point)
 def test_filled_jaw_and_detached_armor_negatives(self):
  points=self.aperture_points();xs,ys,zs=zip(*points);fill=block('Filled jaw negative',INK,tuple((min(q)+max(q))/2 for q in (xs,ys,zs)),tuple((max(q)-min(q))/2+.01 for q in (xs,ys,zs)))
  self.assertTrue(all(occupied(fill,q) for q in points))
  for child,parent in (('Tail /','Segment /'),('Spike /','Segment /'),('Upper tooth /','Head /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
 def test_all_spike_and_joint_surfaces_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Spike /','Joint /','Tail /')):self.assertIsNone(crossed_faces(p),p['part'])
if __name__=='__main__':unittest.main()
