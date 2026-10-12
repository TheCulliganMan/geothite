#!/usr/bin/env python3
"""Long connected Onix rock chain, fitted eyes and grounded tail supports."""
import copy,json,os,re,unittest
from pathlib import Path
from onix_sculpt import sculpture,HEIGHT,CHAIN,body_reference_span
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_ONIX_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/actor_props'))
class OnixTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'big_onix.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 def test_battle_reference_span_tracks_authored_body_centerline(self):
  code=(Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/src/battle_layout.rs').read_text();match=re.search(r'"ONIX" => ([0-9._]+) / ([0-9.]+)',code);self.assertIsNotNone(match);self.assertAlmostEqual(float(match[1].replace('_','')),body_reference_span(),places=6);self.assertEqual(float(match[2]),HEIGHT)
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'onix':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'onix':self.m})
 def test_recipe_parity_bounds_and_long_chain(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),45000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Rock segment /',13),('Joint /',12),('Head /',1),('Eye /',2),('Pupil /',2),('Nose ridge /',1),('Jaw /',1),('Horn /',1)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_every_adjacent_boulder_and_buried_joint_has_a_volume_root(self):
  for j in range(len(CHAIN)-1):
   a=self.find('Rock segment /',j);b=self.find('Rock segment /',j+1);joint=self.find('Joint /',j)
   for u,v in ((a,b),(joint,a),(joint,b)):self.assertTrue(self.overlap(u,v),(u['part'],v['part']))
 def test_head_nose_jaw_horn_and_chamber_are_rooted(self):
  head=self.find('Head /');self.assertTrue(self.overlap(head,self.find('Rock segment /',12)))
  for label in ('Nose ridge /','Jaw /','Horn /','Mouth back /'):self.assertTrue(self.overlap(self.find(label),head),label)
 def test_three_tail_boulders_provide_floor_support(self):
  for j in range(3):self.assertLess(min(self.find('Rock segment /',j)['positions'][1::3]),.003)
  head=self.find('Head /');self.assertGreater(min(head['positions'][1::3]),.7)
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /')):
    if p['part'].startswith(child):yield p,self.find(parent,None if parent=='Head /' else s)
 def test_fitted_vertices_follow_actual_parent(self):attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_parent(self):attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_horn_and_all_joint_surfaces_do_not_cross(self):
  for p in self.ps:
   if p['part'].startswith(('Horn /','Joint /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_rocks_and_horn_are_rejected(self):
  for child,parent in (('Rock segment /','Rock segment /'),('Horn /','Head /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
