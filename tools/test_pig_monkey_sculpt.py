"""Rounded pig-monkey anatomy, real fur/limb roots and fitted angry faces."""
import copy,json,os,unittest
from pathlib import Path
from pig_monkey_sculpt import sculpture,SPECIES,HEIGHTS,fur_paths,normalized
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
import test_nidorina_sculpt as attachment
from test_kingdra_sculpt import crossed_faces
ROOT=Path(os.environ.get('CRYSTAL_PIG_MONKEY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class PigMonkeyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={}
  for s in SPECIES:
   if (ROOT/(s+'.glb')).exists():
    from pig_monkey_glb import read_pig_monkey
    cls.models[s]=read_pig_monkey(ROOT/(s+'.glb'),s)
   else:cls.models[s]=json.loads((ROOT/(s+'.mesh.json')).read_bytes())
 find=attachment.NidorinaTests.find
 overlap=attachment.NidorinaTests.overlap
 def each(self):
  for s,m in self.models.items():self.m=m;self.ps=m['primitives'];yield s
 def test_closed_outward_solids(self):assert_closed_models(self,self.models)
 def test_unit_normals_follow_actual_faces(self):assert_model_shading(self,self.models)
 def test_regeneration_grounded_height_and_existing_budgets(self):
  from steelix_glb import geometry_digest
  for s in self.each():
   self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture(s)));self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
 def test_species_have_distinct_crown_width_tail_and_bands(self):
  for s in self.each():
   for prefix,count in (('Body /',1),('Hand /',2),('Knuckle /',6),('Thumb /',2),('Foot /',2),('Toe /',6),('Eye /',2),('Pupil /',2),('Nostril /',2),('Ear /',2),('Tail /',1 if s=='mankey' else 0),('Wrist cuff /',0 if s=='mankey' else 2),('Ankle cuff /',0 if s=='mankey' else 2)):
    self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,(s,prefix))
   self.assertEqual(sum(p['part'].startswith('Fur / rooted pointed crown') for p in self.ps),3 if s=='mankey' else 5)
  a=self.models['mankey']['primitives'][0];b=self.models['primeape']['primitives'][0];self.assertGreater((max(b['positions'][::3])-min(b['positions'][::3]))/(max(b['positions'][1::3])-min(b['positions'][1::3])),(max(a['positions'][::3])-min(a['positions'][::3]))/(max(a['positions'][1::3])-min(a['positions'][1::3]))*1.15)
 def test_every_fur_lock_and_ear_has_an_actual_body_volume_root(self):
  for s in self.each():
   body=self.find('Body /')
   for label,path,widths,depths in fur_paths(s):
    x,y,z=normalized(s,path[0]);hits=ray_intersections(body,x,y);self.assertGreaterEqual(len(hits),2,(s,label));self.assertLess(min(hits),z);self.assertGreater(max(hits),z)
   for side in (-1,1):self.assertTrue(self.overlap(body,self.find('Ear /',side)))
 def test_snout_limbs_fists_thumbs_knuckles_feet_toes_and_cuffs_are_rooted(self):
  for s in self.each():
   body=self.find('Body /');self.assertTrue(self.overlap(self.find('Snout /'),body))
   for side in (-1,1):
    arm=self.find('Arm /',side);hand=self.find('Hand /',side);leg=self.find('Leg /',side);foot=self.find('Foot /',side)
    for a,b in ((arm,body),(hand,arm),(leg,body),(foot,leg),(self.find('Thumb /',side),hand)):self.assertTrue(self.overlap(a,b),(s,a['part'],b['part']))
    for j in range(3):
     self.assertTrue(self.overlap(self.find('Knuckle /',f'{side} {j}'),hand));self.assertTrue(self.overlap(self.find('Toe /',f'{side} {j}'),foot))
    if s=='primeape':
     self.assertTrue(self.overlap(self.find('Wrist cuff /',side),arm));self.assertTrue(self.overlap(self.find('Wrist cuff /',side),hand));self.assertTrue(self.overlap(self.find('Ankle cuff /',side),leg))
   if s=='mankey':self.assertTrue(self.overlap(self.find('Tail /'),body))
 def pairs(self):
  for p in self.ps:
   n=p['part'];side=n.split()[-1]
   if n.startswith(('Belly /','Eye socket /')):parent=self.find('Body /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',side)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',side)
   elif n.startswith('Nostril /'):parent=self.find('Snout /')
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',side)
   else:continue
   yield p,parent
 def test_fitted_face_pinna_and_belly_vertices_follow_actual_parent(self):
  for s in self.each():attachment.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_front_centers_and_edges_clear_actual_parents(self):
  for s in self.each():attachment.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_limbs_fur_and_tail_do_not_cross_themselves(self):
  for s in self.each():
   for p in self.ps:
    if p['part'].startswith(('Arm /','Leg /','Tail /','Fur /','Ear /','Knuckle /','Toe /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_fur_nose_fists_and_toes_are_detected(self):
  from sculpt_geometry import translate
  for s in self.each():
   for child,parent in (('Fur /','Body /'),('Snout /','Body /'),('Hand /','Arm /'),('Toe /','Foot /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2.,0.,0.),self.find(parent)))
if __name__=='__main__':unittest.main()
