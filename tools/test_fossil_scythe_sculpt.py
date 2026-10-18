"""Fossil silhouettes, occupied attachments, surface-supported gaze and shell anatomy."""
import copy,os,unittest
from pathlib import Path
from fossil_scythe_sculpt import sculpture,SPECIES,HEIGHTS
from fossil_scythe_glb import read_fossil_scythe,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
from sculpt_geometry import translate
from bovine_sculpt import reverse_front
import test_nidorina_sculpt as fitted
ROOT=Path(os.environ.get('CRYSTAL_FOSSIL_SCYTHE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class FossilScytheTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_fossil_scythe(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_anatomy(self):assert_closed_models(self,self.models)
 def test_finite_normals_follow_actual_triangles(self):assert_model_shading(self,self.models)
 def test_exact_recipe_floor_height_and_original_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
 def test_dome_four_claws_and_hammer_head_paired_scythes_are_distinct(self):
  for s,m in self.models.items():
   counts={'Shell /':1 if s=='kabuto' else 0,'Underside /':1 if s=='kabuto' else 0,'Shell pore /':2 if s=='kabuto' else 0,'Claw /':4 if s=='kabuto' else 0,'Leg /':4 if s=='kabuto' else 0,'Foot /':4 if s=='kabuto' else 2,'Head /':1 if s=='kabutops' else 0,'Head wing /':2 if s=='kabutops' else 0,'Scythe /':2 if s=='kabutops' else 0,'Rib plate /':5 if s=='kabutops' else 0,'Toe /':6 if s=='kabutops' else 0,'Back spur /':3 if s=='kabutops' else 0,'Tail /':1 if s=='kabutops' else 0,'Eye lens /':2,'Pupil /':2}
   for prefix,n in counts.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),n,(s,prefix))
 def attachments(self,s):
  if s=='kabuto':
   yield self.find(s,'Underside /'),self.find(s,'Shell /')
   for side in (-1,1):
    for j in range(2):
     suffix=f'{side} {j}'
     for child,parent in [('Leg /','Underside /'),('Foot /','Leg /'),('Claw /','Foot /')]:yield self.find(s,child,suffix),self.find(s,parent,None if parent=='Underside /' else suffix)
  else:
   for child,parent in [('Neck /','Body /'),('Head /','Neck /'),('Tail /','Body /')]:yield self.find(s,child),self.find(s,parent)
   for side in (-1,1):
    for child,parent in [('Head wing /','Head /'),('Upper arm /','Body /'),('Elbow /','Upper arm /'),('Forearm /','Elbow /'),('Wrist /','Forearm /'),('Scythe /','Wrist /'),('Thigh /','Body /'),('Shin /','Thigh /'),('Foot /','Shin /')]:yield self.find(s,child,side),self.find(s,parent,None if parent in ('Head /','Body /') else side)
    for j in range(3):yield self.find(s,'Toe /',f'{side} {j}'),self.find(s,'Foot /',side)
   for j in range(3):yield self.find(s,'Back spur /',j),self.find(s,'Body /')
 def test_complete_anatomy_has_occupied_volume_roots(self):
  for s in self.models:
   for child,parent in self.attachments(s):self.assertTrue(occupied_overlap(child,parent),(s,child['part'],parent['part']))
 def pairs(self,s):
  for p in self.models[s]['primitives']:
   n=p['part'];side=n.split()[-1];parent=None
   if n.startswith('Rear inset /'):yield reverse_front(p),reverse_front(self.find(s,'Shell /'));continue
   if n.startswith('Eye socket /'):parent=self.find(s,'Head /')
   elif n.startswith('Eye lens /'):parent=self.find(s,'Underside /') if s=='kabuto' else self.find(s,'Eye socket /',side)
   elif n.startswith('Iris /'):parent=self.find(s,'Eye lens /',side)
   elif n.startswith('Pupil /'):parent=self.find(s,'Iris /' if s=='kabuto' else 'Eye lens /',side)
   elif n.startswith('Shell pore /'):parent=self.find(s,'Shell /')
   elif n.startswith('Rib plate /'):parent=self.find(s,'Body /')
   if parent:yield p,parent
 def test_faces_pores_ribs_and_rear_inset_follow_actual_parents(self):
  for s in self.models:
   helper=fitted.NidorinaTests();helper.pairs=lambda:self.pairs(s);helper.test_fitted_layers_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_parent()
 def test_both_gazes_are_visible_ahead_of_complete_head_or_shell(self):
  for s in self.models:
   parent=self.find(s,'Shell /' if s=='kabuto' else 'Head /')
   for side in (-1,1):
    eye=self.find(s,'Eye lens /',side);visible=0
    for k in range(0,len(eye['indices']),3):
     tri=[eye['positions'][i*3:i*3+3] for i in eye['indices'][k:k+3]];a,b,c=tri
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(v[j] for v in tri)/3 for j in range(3)];hits=rays(parent,x,y)
     if not hits or z>max(hits)+.001:visible+=1
    self.assertGreaterEqual(visible,3,(s,side))
 def test_curved_legs_claws_scythes_crowns_tail_and_spurs_are_uncrossed(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    if p['part'].startswith(('Leg /','Claw /','Scythe /','Head wing /','Tail /','Toe /','Back spur /','Upper arm /','Forearm /','Shin /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_limbs_and_tail_negative_controls_are_rejected(self):
  for s in self.models:
   for child,parent in self.attachments(s):self.assertFalse(occupied_overlap(translate(copy.deepcopy(child),3.,0.,0.),parent))
if __name__=='__main__':unittest.main()
