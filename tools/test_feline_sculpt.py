"""Independent closed anatomy, attachment and fitted-surface checks for the two cats."""
import copy,json,math,os,unittest
from pathlib import Path
from feline_sculpt import sculpture,SPECIES,HEIGHTS,TAILS,normalized
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections,crossed_faces
import test_nidorina_sculpt as attachment
ROOT=Path(os.environ.get('CRYSTAL_FELINE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class FelineTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={}
  for s in SPECIES:
   path=ROOT/(s+'.mesh.json')
   if path.exists():cls.models[s]=json.loads(path.read_bytes())
   else:
    from feline_glb import read_feline
    cls.models[s]=read_feline(ROOT/(s+'.glb'),s)
 find=attachment.NidorinaTests.find
 overlap=attachment.NidorinaTests.overlap
 def each(self):
  for s,m in self.models.items():self.m=m;self.ps=m['primitives'];yield s
 def test_closed_outward_solids(self):assert_closed_models(self,self.models)
 def test_shading_follows_actual_faces(self):assert_model_shading(self,self.models)
 def test_regeneration_grounded_dimensions_and_limits(self):
  from steelix_glb import geometry_digest
  for s in self.each():
   self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture(s)));self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHTS[s],places=6);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
 def test_distinct_biped_and_long_quadruped_anatomy(self):
  for s in self.each():
   legs=2 if s=='meowth' else 4
   for prefix,count in (('Body /',1),('Head /',1),('Neck /',1),('Eye /',2),('Pupil /',2),('Ear /',2),('Ear inset /',2),('Muzzle /',2),('Whisker /',4 if s=='meowth' else 6),('Tail /',1),('Leg /',legs),('Foot /',legs),('Toe /',legs*3),('Coin /',int(s=='meowth')),('Gem /',int(s=='persian')),('Arm /',2 if s=='meowth' else 0),('Finger /',6 if s=='meowth' else 0)):
    self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),count,(s,prefix))
   body=self.find('Body /');span=[max(body['positions'][i::3])-min(body['positions'][i::3]) for i in range(3)]
   self.assertGreater(span[1] if s=='meowth' else span[2],span[2] if s=='meowth' else span[1])
 def test_every_actual_anatomical_root_overlaps_parent_volume(self):
  for s in self.each():
   body=self.find('Body /');head=self.find('Head /');neck=self.find('Neck /')
   for a,b in ((head,neck),(neck,body),(self.find('Tail /'),body),(self.find('Nose /'),self.find('Muzzle /'))):self.assertTrue(self.overlap(a,b),(s,a['part'],b['part']))
   for side in (-1,1):
    for prefix in ('Ear /','Muzzle /'):self.assertTrue(self.overlap(self.find(prefix,side),head),(s,prefix,side))
    for j in range(2 if s=='meowth' else 3):self.assertTrue(self.overlap(self.find('Whisker /',f'{side} {j}'),self.find('Muzzle /',side)),(s,side,j))
    if s=='meowth':
     self.assertTrue(self.overlap(self.find('Brow whisker /',side),head))
     arm=self.find('Arm /',side);hand=self.find('Hand /',side)
     self.assertTrue(self.overlap(arm,body));self.assertTrue(self.overlap(hand,arm))
     for j in range(3):self.assertTrue(self.overlap(self.find('Finger /',f'{side} {j}'),hand))
   for leg in (p for p in self.ps if p['part'].startswith('Leg /')):
    label=leg['part'].split('support ')[1];foot=self.find('Foot /',label);self.assertTrue(self.overlap(leg,body),(s,label));self.assertTrue(self.overlap(foot,leg),(s,label))
    for j in range(3):self.assertTrue(self.overlap(self.find('Toe /',label+' '+str(j)),foot))
   self.assertTrue(self.overlap(self.find('Coin /' if s=='meowth' else 'Gem /'),head))
   if s=='meowth':
    for groove in (p for p in self.ps if p['part'].startswith('Coin groove /')):self.assertTrue(self.overlap(groove,self.find('Coin /')))
 def pairs(self):
  for p in self.ps:
   n=p['part'];side=n.split()[-1]
   if n.startswith(('Eye socket /','Mouth /')):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',side)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',side)
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',side)
   elif n.startswith('Fang /'):parent=self.find('Mouth /')
   elif n.startswith('Gem glint /'):parent=self.find('Gem /')
   else:continue
   yield p,parent
 def test_fitted_features_follow_stored_parent_vertices(self):
  for s in self.each():attachment.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_front_edges_and_centers_clear_parent(self):
  for s in self.each():attachment.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_curved_limbs_tails_whiskers_and_ears_do_not_self_cross(self):
  for s in self.each():
   for p in self.ps:
    if p['part'].startswith(('Leg /','Arm /','Tail /','Whisker /','Brow whisker /','Ear /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_pupil_front_area_remains_visible_above_head_muzzle_coin_and_eyes(self):
  def project(p,angle):
   c=math.cos(angle);sn=math.sin(angle);q=dict(p);q['positions']=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,c*y-sn*z,sn*y+c*z)];return q
  for s,m in self.models.items():
   for angle in (0.,.15,.35):
    parts=[project(p,angle) for p in m['primitives']];visible=total=0.
    for p in parts:
     if not p['part'].startswith('Pupil /'):continue
     parents=[q for q in parts if q['part'].startswith(('Head /','Muzzle /','Coin /','Gem /')) or q['part'].startswith('Eye /') and q['part'].split()[-1]==p['part'].split()[-1]];vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
     for i in range(0,len(p['indices']),3):
      a,b,c=[vs[j] for j in p['indices'][i:i+3]];area=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
      if area<=1e-10:continue
      mid=[sum(v[k] for v in (a,b,c))/3 for k in range(3)];hits=[z for parent in parents for z in ray_intersections(parent,*mid[:2])];total+=area
      if not hits or mid[2]>max(hits):visible+=area
    self.assertGreater(total,.0001);self.assertAlmostEqual(visible/total,1.,places=6,msg=(s,angle))
 def test_detached_necks_whiskers_paws_and_charms_are_rejected(self):
  from sculpt_geometry import translate
  for s in self.each():
   for child,parent in (('Neck /','Body /'),('Whisker /','Muzzle /'),('Foot /','Leg /'),('Coin /' if s=='meowth' else 'Gem /','Head /')):self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)),2.,0.,0.),self.find(parent)))
if __name__=='__main__':unittest.main()
