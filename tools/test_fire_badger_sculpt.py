#!/usr/bin/env python3
"""Independent fire-badger solids, shared coat seams, fitted gaze and rooted fire."""
import copy,os,unittest
from pathlib import Path
from fire_badger_sculpt import sculpture,SPECIES,HEIGHTS
from fire_badger_glb import read_fire_badger,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
from test_steelix_sculpt import occupied
from sculpt_geometry import translate,ellipsoid
import test_nidorina_sculpt as fitted
ROOT=Path(os.environ.get('CRYSTAL_FIRE_BADGER_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def joined(parts):
 # Actual stored triangles from both color halves form the fitted skull parent.
 out=dict(part='complete stored skull',positions=[],normals=[],indices=[])
 for p in parts:
  offset=len(out['positions'])//3;out['positions']+=p['positions'];out['normals']+=p['normals'];out['indices'] += [i+offset for i in p['indices']]
 return out
def vertical(p):return dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,z,y)])
class FireBadgerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_fire_badger(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_outward_connected_solids(self):assert_closed_models(self,self.models)
 def test_actual_corner_normals_are_finite_and_outward(self):assert_model_shading(self,self.models)
 def test_recipe_height_floor_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000)
 def test_actual_two_tone_body_and_head_share_surface_positions_and_normals(self):
  for s,m in self.models.items():
   for prefix in ('Body /','Head /'):
    a,b=[p for p in m['primitives'] if p['part'].startswith(prefix)];points=lambda p:{tuple(p['positions'][i:i+3]+p['normals'][i:i+3]) for i in range(0,len(p['positions']),3)};self.assertGreater(len(points(a)&points(b)),12,(s,prefix));self.assertNotEqual(a['base_color'],b['base_color'])
 def test_crystal_upright_ferret_and_badger_keep_complete_digits_and_distinct_fire_banks(self):
  for s,m in self.models.items():
   counts={'Body /':2,'Head /':2,'Ear /':2,'Eye socket /':2,'Foot /':2,'Toe /':6,'Finger /':6,'Fire /':35 if s=='typhlosion' else 30}
   for prefix,n in counts.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),n,(s,prefix))
   body=[p for p in m['primitives'] if p['part'].startswith('Body /')];span=lambda axis:max(v for p in body for v in p['positions'][axis::3])-min(v for p in body for v in p['positions'][axis::3]);self.assertGreater(span(1)/span(0),1.4);self.assertGreater(span(1)/span(2),1.7)
   fires=[p for p in m['primitives'] if p['part'].endswith(' / ember silhouette')]
   self.assertEqual({p['part'].split(' / ')[1].split()[0] for p in fires},{'mantle'} if s=='typhlosion' else {'neck','rump'})
 def test_skull_ears_nose_and_adult_jaw_are_occupied(self):
  for s,m in self.models.items():
   heads=[p for p in m['primitives'] if p['part'].startswith('Head /')];body=[p for p in m['primitives'] if p['part'].startswith('Body /')];self.assertTrue(any(occupied_overlap(h,b) for h in heads for b in body),s)
   for p in m['primitives']:
    if p['part'].startswith(('Ear /','Nose /','Fang /')):self.assertTrue(any(occupied_overlap(p,h) for h in heads),(s,p['part']))
   if s=='typhlosion':self.assertTrue(any(occupied_overlap(self.find(s,'Jaw /'),h) for h in heads));self.assertTrue(occupied_overlap(self.find(s,'Tongue /'),self.find(s,'Jaw /')))
 def test_arms_palms_digits_and_supports_have_actual_roots(self):
  for s,m in self.models.items():
   body=[p for p in m['primitives'] if p['part'].startswith('Body /')]
   for p in m['primitives']:
    n=p['part']
    if n.startswith(('Leg /','Arm /','Haunch /')):self.assertTrue(any(occupied_overlap(p,b) for b in body),(s,n))
    if n.startswith('Hand /'):self.assertTrue(occupied_overlap(p,self.find(s,'Arm /',n.split()[-1])),(s,n))
    if n.startswith('Finger /'):self.assertTrue(occupied_overlap(p,self.find(s,'Hand /',n.split()[-2])),(s,n))
    if n.startswith('Foot /'):
     suffix=' '.join(n.split()[-2:]);parent=self.find(s,'Haunch /',n.split()[-2]) if s=='typhlosion' else self.find(s,'Leg /',suffix);self.assertTrue(occupied_overlap(p,parent),(s,n));self.assertAlmostEqual(min(p['positions'][1::3]),0.,places=7)
    if n.startswith('Toe /'):self.assertTrue(occupied_overlap(p,self.find(s,'Foot /',' '.join(n.split()[-3:-1]))),(s,n))
 def test_each_fire_blade_and_all_folds_have_occupied_support(self):
  for s,m in self.models.items():
   parents=[p for p in m['primitives'] if p['part'].startswith('Body /')]
   embers=[p for p in m['primitives'] if p['part'].endswith(' / ember silhouette')]
   for ember in embers:
    self.assertTrue(any(occupied_overlap(ember,p) for p in parents),(s,ember['part']));prefix=ember['part'].rsplit(' / ',1)[0]
    for p in m['primitives']:
     if p is not ember and p['part'].startswith(prefix+' / '):self.assertTrue(occupied_overlap(p,ember),(s,p['part']))
 def pairs(self,s):
  ps=self.models[s]['primitives']
  for p in ps:
   n=p['part'];side=n.split()[-1]
   if n.startswith('Lip /'):yield p,self.find(s,'Head / joined warm');continue
   parent=None
   if n.startswith('Eye socket /'):yield p,joined([h for h in ps if h['part'].startswith('Head /')]);continue
   for prefix,base in [('Eye white /','Eye socket /'),('Iris /','Eye white /'),('Pupil /','Iris /'),('Glint /','Pupil /')]:
    if n.startswith(prefix):parent=self.find(s,base,None if base.startswith('Head /') else side)
   if parent:yield p,parent
 def test_fitted_gaze_and_lip_follow_actual_vertices_edges_and_centres(self):
  for s in self.models:
   helper=fitted.NidorinaTests();helper.pairs=lambda:self.pairs(s);helper.test_fitted_layers_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_parent()
 def test_both_white_gazes_are_visible_in_front_of_the_complete_skull(self):
  for s,m in self.models.items():
   heads=[p for p in m['primitives'] if p['part'].startswith('Head /')]
   for side in (-1,1):
    eye=self.find(s,'Eye white /',side);visible=0
    for k in range(0,len(eye['indices']),3):
     tri=[eye['positions'][i*3:i*3+3] for i in eye['indices'][k:k+3]];a,b,c=tri
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(v[j] for v in tri)/3 for j in range(3)];hits=[v for h in heads for v in rays(h,x,y)]
     if hits and z>max(hits)+.001:visible+=1
    self.assertGreaterEqual(visible,3,(s,side,'gaze hidden behind the skull from the front'))
 def aperture_points(self,parts=None):
  ps=parts if parts is not None else self.models['typhlosion']['primitives'];heads=[p for p in ps if p['part'].startswith('Head /')];jaw=next(p for p in ps if p['part'].startswith('Jaw /'))
  h=max(v for p in ps for v in p['positions'][1::3]);front=max(v for p in heads for v in p['positions'][2::3]);points=[];upper=[vertical(p) for p in heads];lower=vertical(jaw)
  for x in (-.015*h,0.,.015*h):
   for z in (front*.72,front*.79):
    a=rays(lower,x,z);b=[v for p in upper for v in rays(p,x,z)];self.assertTrue(a and b,('missing actual lips',x,z));low,high=max(a),min(b);self.assertGreater(high-low,.008*h)
    for fraction in (.35,.65):
     point=(x,low+(high-low)*fraction,z);back=[v for p in ps if p['part'].startswith(('Body /','Head /')) for v in rays(p,x,point[1]) if v<z-.02*h]
     self.assertTrue(back,('mouth has no occupied backing',point));points.append(point)
  return points
 def test_adult_jaw_has_real_empty_space_and_filled_negative_control(self):
  for point in self.aperture_points():self.assertFalse(any(occupied(p,point) for p in self.models['typhlosion']['primitives']),point)
  point=self.aperture_points()[0];self.assertTrue(occupied(ellipsoid('filled mouth negative',(.1,.1,.1),point,(.02,.02,.02),16,12),point))
 def test_continuous_limbs_and_all_digits_have_uncrossed_faces(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    if p['part'].startswith(('Leg /','Arm /','Finger /','Toe /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_fire_and_ears_negative_controls_are_real(self):
  for s,m in self.models.items():
   for prefix in ('Ear /','Fire /'):
    moved=translate(copy.deepcopy(self.find(s,prefix)),3,0,0);self.assertFalse(any(occupied_overlap(moved,p) for p in m['primitives']))
if __name__=='__main__':unittest.main()
