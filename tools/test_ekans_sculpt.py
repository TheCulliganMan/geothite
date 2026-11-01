"""Independent occupied coils, fitted snake gaze and continuous collar checks."""
import copy,unittest
import ekans_sculpt as author
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,surface_distances
from sculpt_geometry import translate
from test_seahorse_glb import occupied_overlap
import test_nidorina_sculpt as fitted

class EkansTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=author.sculpture()
 def find(self,prefix,suffix=None):return next(p for p in self.model['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'ekans':self.model})
 def test_finite_corner_normals_follow_faces(self):assert_model_shading(self,{'ekans':self.model})
 def test_original_height_grounding_and_existing_budgets(self):
  ps=self.model['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),author.HEIGHT,places=6)
  self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000);self.assertEqual(len(ps),16);self.assertEqual(len({p['part'] for p in ps}),len(ps))
 def test_species_specific_gaze_collar_and_three_rattles(self):
  for prefix,n in [('Eye /',2),('Pupil /',2),('Glint /',2),('Nostril /',2),('Collar /',1),('Rattle /',3),('Belly /',1)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.model['primitives']),n,prefix)
  for side in (-1,1):
   p=self.find('Pupil /',side);self.assertGreater(max(p['positions'][1::3])-min(p['positions'][1::3]),2.5*(max(p['positions'][::3])-min(p['positions'][::3])))
 def test_continuous_coil_has_no_self_intersections(self):self.assertIsNone(crossed_faces(self.find('Body /')))
 def attachments(self):
  body=self.find('Body /');yield self.find('Head /'),body;yield self.find('Collar /'),body;yield self.find('Belly /'),body;yield self.find('Mouth /'),self.find('Head /')
  for j in range(3):yield self.find('Rattle /',j),body if j==0 else self.find('Rattle /',j-1)
 def test_head_collar_belly_and_rattles_have_occupied_roots(self):
  for child,parent in self.attachments():self.assertTrue(occupied_overlap(child,parent),(child['part'],parent['part']))
 def test_detached_negative_controls_fail_attachment(self):
  for child,parent in self.attachments():self.assertFalse(occupied_overlap(translate(copy.deepcopy(child),3.,0.,0.),parent))
 def pairs(self):
  for p in self.model['primitives']:
   n=p['part'];side=n.split()[-1]
   if n.startswith(('Eye /','Nostril /')):yield p,self.find('Head /')
   elif n.startswith('Pupil /'):yield p,self.find('Eye /',side)
   elif n.startswith('Glint /'):yield p,self.find('Pupil /',side)
 def test_fitted_features_clear_actual_parent_triangles(self):
  h=fitted.NidorinaTests();h.pairs=self.pairs;h.test_fitted_layers_follow_actual_stored_parent();h.test_fitted_face_centers_and_edges_clear_parent()
 def test_collar_and_belly_follow_actual_three_dimensional_coil(self):
  for name in ('Collar /','Belly /'):
   p=self.find(name);points={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}
   self.assertTrue(all(d<.008 for d in surface_distances(self.find('Body /'),points,.008)),name)
if __name__=='__main__':unittest.main()
