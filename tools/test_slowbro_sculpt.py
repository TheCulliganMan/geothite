#!/usr/bin/env python3
"""Stored shell-tailed anatomy, surface layers, oral gaps and real roots."""
import copy,json,os,unittest
from pathlib import Path
from slowbro_sculpt import sculpture,anatomy,HEIGHT
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_steelix_sculpt import occupied
from steelix_sculpt import block,INK
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_SLOWBRO_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class SlowbroTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=json.loads((ROOT/'slowbro.mesh.json').read_bytes())
 def setUp(self):self.m=self.model;self.ps=self.m['primitives']
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,{'slowbro':self.m})
 def test_normals_follow_actual_faces(self):assert_model_shading(self,{'slowbro':self.m})
 def test_recipe_parity_floor_height_budget_and_anatomy(self):
  self.assertEqual(self.m,sculpture());self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for label,n in (('Body /',1),('Ear /',2),('Eye /',2),('Belly band /',4),('Hand claw /',6),('Foot claw /',4),('Fang /',2),('Shell coil /',3),('Shell spine /',7),('Shell upper tooth /',4),('Shell lower tooth /',4),('Shell eye /',2)):self.assertEqual(sum(p['part'].startswith(label) for p in self.ps),n,label)
 def test_face_ears_jaw_and_tail_have_actual_roots(self):
  body=self.find('Body /')
  for label in ('Muzzle /','Jaw /','Tail /'):self.assertTrue(self.overlap(self.find(label),body),label)
  self.assertTrue(self.overlap(self.find('Mouth back /'),self.find('Muzzle /')));self.assertTrue(self.overlap(self.find('Tongue /'),self.find('Jaw /')))
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Ear /',s),body));self.assertTrue(self.overlap(self.find('Fang /',s),self.find('Muzzle /')))
 def test_limbs_and_all_white_claws_have_actual_roots(self):
  for s in (-1,1):
   for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /'),('Thigh /','Body /'),('Leg /','Thigh /'),('Foot /','Leg /')):self.assertTrue(self.overlap(self.find(child,s),self.find(parent,None if parent=='Body /' else s)),(child,s))
   for label,parent,n in (('Hand claw /','Hand /',3),('Foot claw /','Foot /',2)):
    for j in range(n):self.assertTrue(self.overlap(self.find(label,f'{s} {j}'),self.find(parent,s)),(label,s,j))
 def test_both_feet_support_the_model(self):
  for s in (-1,1):self.assertEqual(min(self.find('Foot /',s)['positions'][1::3]),0)
 def test_shell_is_attached_to_tail_and_has_rooted_jaws_teeth_spines(self):
  shell=self.find('Shell body /');jaw=self.find('Shell jaw /');tail=self.find('Tail /')
  self.assertTrue(self.overlap(shell,tail));self.assertTrue(self.overlap(jaw,shell));self.assertTrue(self.overlap(self.find('Shell mouth back /'),shell));self.assertTrue(self.overlap(self.find('Shell mouth back /'),jaw))
  wall=self.find('Shell rear wall /');self.assertTrue(self.overlap(wall,shell));self.assertTrue(self.overlap(wall,jaw))
  for p in self.ps:
   for child,parent in (('Shell coil /',shell),('Shell spine /',shell),('Shell upper tooth /',shell),('Shell lower tooth /',jaw)):
    if p['part'].startswith(child):self.assertTrue(self.overlap(p,parent),p['part'])
 def test_shell_coil_faces_wrap_the_actual_parent(self):
  shell=self.find('Shell body /')
  for p in self.ps:
   if not p['part'].startswith('Shell coil /'):continue
   count=len(p['positions'])//(3*2*40);vertices=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
   for offset in range(0,(count-1)*40*12,12):
    for layer,inside in ((0,False),(6,True)):
     for t in (0,3):
      tri=[vertices[k] for k in p['indices'][offset+layer+t:offset+layer+t+3]];point=tuple(sum(v[k] for v in tri)/3 for k in range(3));self.assertEqual(occupied(shell,point),inside,(p['part'],offset,layer,t))
 def pairs(self):
  for p in self.ps:
   s=p['part'].split()[-1]
   for child,parent in (('Belly band /','Body /'),('Eye /','Body /'),('Pupil /','Eye /'),('Ear inset /','Ear /'),('Ear curl /','Ear inset /'),('Nostril /','Muzzle /'),('Shell socket /','Shell body /'),('Shell eye /','Shell socket /'),('Shell pupil /','Shell eye /')):
    if p['part'].startswith(child):yield p,self.find(parent,None if parent in ('Body /','Muzzle /','Shell body /') else s)
 def test_fitted_vertices_follow_actual_stored_parent(self):attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_face_centers_and_edges_clear_actual_parent(self):attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def mouth_points(self,shell=False):
  raw=anatomy();low=min(v for p in raw for v in p['positions'][1::3]);high=max(v for p in raw for v in p['positions'][1::3]);k=HEIGHT/(high-low)
  coords=[(x,y,z) for x in (.225,.515) for y in (.405,.419) for z in (-.335,-.355)] if shell else [(x,y,z) for x in (-.04,0,.04) for y in (1.04,1.055) for z in (.43,.46)]
  return [(x*k,(y-low)*k,z*k) for x,y,z in coords]
 def test_hero_and_shell_jaws_have_actual_empty_apertures(self):
  for shell in (False,True):
   for q in self.mouth_points(shell):self.assertFalse(any(occupied(p,q) for p in self.ps),(shell,q))
 def test_filled_apertures_are_rejected(self):
  for shell in (False,True):
   points=self.mouth_points(shell);axes=list(zip(*points));fill=block('Filled mouth negative',INK,tuple((min(a)+max(a))/2 for a in axes),tuple((max(a)-min(a))/2+.01 for a in axes));self.assertTrue(all(occupied(fill,q) for q in points))
 def test_curved_limbs_tail_and_shell_spines_are_uncrossed(self):
  for p in self.ps:
   if p['part'].startswith(('Arm /','Leg /','Hand claw /','Foot claw /','Tail /','Shell spine /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_tail_shell_and_claw_negatives(self):
  for child,parent in (('Tail /','Body /'),('Shell body /','Tail /'),('Hand claw /','Hand /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),3,0,0),self.find(parent)))
if __name__=='__main__':unittest.main()
