"""Original cobra anatomy, occupied roots, visible hood mask and open fanged mouth."""
import copy,unittest
import arbok_sculpt as author
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays,surface_distances
from sculpt_geometry import translate
from test_seahorse_glb import occupied_overlap
import test_nidorina_sculpt as fitted
class ArbokTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=author.sculpture()
 def find(self,prefix,suffix=None):return next(p for p in self.model['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'arbok':self.model})
 def test_finite_corner_normals_follow_faces(self):assert_model_shading(self,{'arbok':self.model})
 def test_original_height_floor_and_existing_budgets(self):
  ps=self.model['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),author.HEIGHT,places=6);self.assertEqual(len(ps),25);self.assertEqual(len(ps),len({p['part'] for p in ps}));self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
 def test_coil_and_rounded_hood_have_no_self_intersections(self):
  for prefix in ('Body /','Hood /'):self.assertIsNone(crossed_faces(self.find(prefix)),prefix)
 def test_species_specific_mask_fangs_and_open_jaw(self):
  for prefix,n in [('Hood /',1),('False eye rim /',2),('False eye /',2),('False pupil /',2),('Threat rim /',1),('Threat /',1),('Fang /',2),('Eye /',2),('Pupil /',2),('Jaw /',1),('Tongue /',1)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.model['primitives']),n,prefix)
  hood=self.find('Hood /');self.assertGreater(max(hood['positions'][::3])-min(hood['positions'][::3]),.85)
 def attachments(self):
  for child,parent in [('Hood /','Body /'),('Head /','Hood /'),('Jaw /','Head /'),('Mouth back /','Head /'),('Mouth back /','Jaw /'),('Tongue /','Jaw /'),('Belly /','Body /')]:yield self.find(child),self.find(parent)
  for side in (-1,1):yield self.find('Fang /',side),self.find('Head /')
 def test_complete_hood_head_jaw_fangs_and_tongue_have_occupied_roots(self):
  for child,parent in self.attachments():self.assertTrue(occupied_overlap(child,parent),(child['part'],parent['part']))
 def test_detached_negative_controls_fail_occupied_roots(self):
  for child,parent in self.attachments():self.assertFalse(occupied_overlap(translate(copy.deepcopy(child),3.,0.,0.),parent))
 def pairs(self):
  for p in self.model['primitives']:
   n=p['part'];side=n.split()[-1]
   if n.startswith(('Eye /','Nostril /')):yield p,self.find('Head /')
   elif n.startswith('Pupil /'):yield p,self.find('Eye /',side)
   elif n.startswith('Glint /'):yield p,self.find('Pupil /',side)
   elif n.startswith(('False eye rim /','Threat rim /')):yield p,self.find('Hood /')
   elif n.startswith('Threat /'):yield p,self.find('Threat rim /')
   elif n.startswith('False eye /'):yield p,self.find('False eye rim /',side)
   elif n.startswith('False pupil /'):yield p,self.find('False eye /',side)
 def test_fitted_face_and_hood_details_follow_actual_parent_triangles(self):
  h=fitted.NidorinaTests();h.pairs=self.pairs;h.test_fitted_layers_follow_actual_stored_parent();h.test_fitted_face_centers_and_edges_clear_parent()
 def test_threat_chevron_and_false_eyes_are_in_front_of_the_neck(self):
  for prefix in ('Threat /','False eye /'):
   for p in self.model['primitives']:
    if not p['part'].startswith(prefix):continue
    visible=0
    for i in range(0,len(p['indices']),3):
     a,b,c=[p['positions'][3*j:3*j+3] for j in p['indices'][i:i+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(q[k] for q in (a,b,c))/3 for k in range(3)];hits=rays(self.find('Body /'),x,y)
     self.assertTrue(not hits or z>max(hits)+.0001,(p['part'],i));visible+=1
    self.assertGreater(visible,3,p['part'])
 def test_mouth_has_real_empty_space_and_exposed_fangs(self):
  lo,k=author.authoring_transform();y=(1.02-lo)*k
  for prefix in ('Head /','Jaw /','Mouth back /','Hood /'):
   hits=rays(self.find(prefix),0.,y);self.assertTrue(not hits or max(hits)<.14*k,prefix)
  for side in (-1,1):
   hits=rays(self.find('Fang /',side),side*.094*k,y);self.assertTrue(hits,side);self.assertGreater(max(hits),.18*k)
 def test_mask_brows_have_occupied_peaks_and_real_open_notches(self):
  lo,k=author.authoring_transform()
  for side in (-1,1):
   rim=self.find('False eye rim /',side)
   for x,y in ((.148,.934),(.215,.944)):self.assertTrue(rays(rim,side*x*k,(y-lo)*k),(side,x,y))
   self.assertFalse(rays(rim,side*.179*k,(.935-lo)*k),side)
 def test_continuous_belly_remains_close_to_actual_coil(self):
  p=self.find('Belly /');points={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)};self.assertTrue(all(d<.008 for d in surface_distances(self.find('Body /'),points,.008)))
if __name__=='__main__':unittest.main()
