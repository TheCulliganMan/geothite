#!/usr/bin/env python3
"""Stored botanical anatomy, actual roots, fitted gaze, cavities and ground supports."""
import copy,os,unittest
from pathlib import Path
from pitcher_sculpt import sculpture,SPECIES,HEIGHT
from pitcher_glb import read_pitcher,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
from test_seahorse_glb import occupied_overlap
from sculpt_geometry import translate
import test_nidorina_sculpt as fitted
ROOT=Path(os.environ.get('CRYSTAL_PITCHER_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class PitcherTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_pitcher(ROOT/(s+'.glb'),s) for s in SPECIES}
 def find(self,s,prefix,suffix=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_connected_outward_anatomy(self):assert_closed_models(self,self.models)
 def test_finite_corner_normals_follow_actual_triangles(self):assert_model_shading(self,self.models)
 def test_exact_recipe_floor_height_and_bounded_geometry(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives'];self.assertEqual(len(ps),len({p['part'] for p in ps}));self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),7000)
 def test_real_recesses_rooted_veins_white_gaze_and_distinct_anatomy(self):
  for s,m in self.models.items():
   ps=m['primitives'];axis=1 if s=='victreebel' else 2;rim=next(p for p in ps if 'rolled' in p['part']);throat=next(p for p in ps if 'recessed' in p['part'] or 'shaded' in p['part']);self.assertGreater(max(rim['positions'][axis::3])-max(throat['positions'][axis::3]),.055)
   for prefix,count in [('Root /',2 if s=='bellsprout' else 0),('Stem /',1 if s=='bellsprout' else 0),('Body /',0 if s=='bellsprout' else 1),('Fang /',2 if s=='victreebel' else 0),('Hood /',2 if s=='victreebel' else 0),('Hook /',1 if s=='weepinbell' else 0),('Vine /',1 if s=='victreebel' else 0),('Pigment /',0 if s=='bellsprout' else 5),('Eye / fitted ivory',0 if s=='bellsprout' else 2)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count,(s,prefix))
   self.assertEqual(sum('/ raised midrib' in p['part'] for p in ps),3 if s=='victreebel' else 2)
 def pairs(self,s):
  for p in self.models[s]['primitives']:
   n=p['part'];side=n.split()[-1];parent=None
   if n.startswith('Pigment /'):parent=self.find(s,'Body /')
   elif n.startswith('Eye / fitted ivory'):parent=self.find(s,'Body /')
   elif n.startswith('Eye /') and 'black' in n:parent=self.find(s,'Head /') if s=='bellsprout' else self.find(s,'Eye / fitted ivory',side)
   elif n.startswith('Eye /') and ('catchlight' in n or 'glint' in n):parent=next(q for q in self.models[s]['primitives'] if q['part'].startswith('Eye /') and 'black' in q['part'] and q['part'].endswith(' '+side))
   if parent:yield p,parent
 def test_gaze_and_pigment_vertices_centres_and_edges_follow_actual_parents(self):
  for s in self.models:
   helper=fitted.NidorinaTests();helper.pairs=lambda:self.pairs(s);helper.test_fitted_layers_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_parent()
 def test_mouth_axis_reaches_dark_recess_before_outer_wall(self):
  for s in self.models:
   mouth=self.find(s,'Head /' if s=='bellsprout' else 'Mouth / hollow' if s=='victreebel' else 'Mouth / flared')
   throat=next(p for p in self.models[s]['primitives'] if p['part'].startswith('Mouth /') and ('recessed' in p['part'] or 'shaded' in p['part']))
   if s=='victreebel':
    def upward(p):return dict(p,positions=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i],p['positions'][i+2],p['positions'][i+1])])
    mouth,throat=upward(mouth),upward(throat);cy=(min(throat['positions'][1::3])+max(throat['positions'][1::3]))/2
   else:cy=(min(throat['positions'][1::3])+max(throat['positions'][1::3]))/2
   dark=rays(throat,0.,cy);wall=rays(mouth,0.,cy);self.assertTrue(dark,s)
   self.assertTrue(not wall or max(wall)<max(dark)+.003,(s,'outer wall masks the cavity floor'))
 def attachments(self,s):
  ps=self.models[s]['primitives'];body=self.find(s,'Stem /' if s=='bellsprout' else 'Body /');mouth=self.find(s,'Head /' if s=='bellsprout' else 'Mouth / hollow' if s=='victreebel' else 'Mouth / flared')
  if s=='bellsprout':
   yield mouth,body
   for side in (-1,1):yield self.find(s,'Root /',side),body
  else:yield mouth,body
  for p in ps:
   n=p['part']
   if n.startswith('Leaf /'):
    if n.endswith('cupped blade'):yield p,body
    else:yield p,next(q for q in ps if q['part']==n.replace('raised midrib','cupped blade'))
   elif n.startswith('Hood /'):
    if n.endswith('cupped blade'):yield p,mouth
    else:yield p,next(q for q in ps if q['part']==n.replace('raised midrib','cupped blade'))
   elif n.startswith(('Hook /','Vine /')):yield p,body
   elif n.startswith('Mouth /') and p is not mouth:yield p,mouth
   elif n.startswith('Fang /'):yield p,mouth
 def test_complete_stalk_roots_leaves_mouth_lids_and_tendrils_have_occupied_roots(self):
  for s in self.models:
   for child,parent in self.attachments(s):self.assertTrue(occupied_overlap(child,parent),(s,child['part'],parent['part']))
 def test_frontal_gaze_stays_in_front_of_actual_head_or_body(self):
  for s,m in self.models.items():
   parent=self.find(s,'Head /' if s=='bellsprout' else 'Body /')
   for side in (-1,1):
    eye=next(p for p in m['primitives'] if p['part'].startswith('Eye /') and ('black' in p['part'] if s=='bellsprout' else 'ivory' in p['part']) and p['part'].endswith(' '+str(side)));visible=0
    for k in range(0,len(eye['indices']),3):
     a,b,c=tri=[eye['positions'][j*3:j*3+3] for j in eye['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(v[j] for v in tri)/3 for j in range(3)];hits=rays(parent,x,y)
     if hits and z>max(hits)+.001:visible+=1
    self.assertGreaterEqual(visible,3,(s,side))
 def test_curved_stalks_roots_leaves_and_tendrils_do_not_cross(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    if p['part'].startswith(('Stem /','Root /','Leaf /','Hood /','Hook /','Vine /','Fang /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_anatomy_negative_controls_are_rejected(self):
  for s in self.models:
   for child,parent in self.attachments(s):self.assertFalse(occupied_overlap(translate(copy.deepcopy(child),4.,0.,0.),parent))
if __name__=='__main__':unittest.main()
