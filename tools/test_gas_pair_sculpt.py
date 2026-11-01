#!/usr/bin/env python3
"""Solid crater geometry, fitted faces and authoritative joining stalks."""
import copy,json,os,unittest
from pathlib import Path
from gas_pair_sculpt import sculpture,anatomy,HEIGHTS,VENTS,unit,PURPLE
from gas_pair_glb import read_gas,geometry_digest
from sculpt_geometry import translate,ellipsoid
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_steelix_sculpt import occupied
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(os.environ.get('CRYSTAL_GAS_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class GasTests(unittest.TestCase):
 raw={}
 @classmethod
 def setUpClass(cls):cls.models={n:read_gas(ROOT/(n+'.glb'),n) for n in HEIGHTS}
 find=attachments.NidorinaTests.find
 overlap=attachments.NidorinaTests.overlap
 def select(self,n):self.m=self.models[n];self.ps=self.m['primitives']
 def test_closed_outward_surfaces(self):assert_closed_models(self,self.models)
 def test_face_aligned_normals(self):assert_model_shading(self,self.models)
 def test_recipe_floor_height_budget_and_distinct_anatomy(self):
  for n,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(n)));self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHTS[n],places=6);self.assertLessEqual(sum(len(p['indices'])//3 for p in m['primitives']),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in m['primitives']),20000);self.assertEqual(len(m['primitives']),len({p['part'] for p in m['primitives']}))
   for prefix,count in (('Eye /',2 if n=='koffing' else 4),('Fang /',2 if n=='koffing' else 4),('Skull mark /',1 if n=='koffing' else 2),('Crossbone /',2),('Tongue /',1 if n=='koffing' else 0),('Small body /',0 if n=='koffing' else 1),('Rear node /',0 if n=='koffing' else 1),('Gas cloud /',2),('Gas lobe /',8)):self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),count,(n,prefix))
 def test_actual_weezing_connectors_join_all_three_orbs(self):
  self.select('weezing')
  for child,parent in (('Main stalk /','Body /'),('Main stalk /','Small body /'),('Rear stalk /','Body /'),('Rear stalk /','Rear node /')):self.assertTrue(self.overlap(self.find(child),self.find(parent)),(child,parent))
 def test_solid_fangs_have_actual_lip_roots(self):
  for n in HEIGHTS:
   self.select(n)
   for p in self.ps:
    if p['part'].startswith('Fang /'):self.assertTrue(self.overlap(p,self.find('Small body /' if 'small Weezing' in p['part'] else 'Body /')),p['part'])
 def pairs(self):
  for p in self.ps:
   if p['part'].startswith(('Body /','Small body /','Fang /','Main stalk /','Rear stalk /','Rear node /','Gas cloud /','Gas lobe /')):continue
   label='small Weezing' if 'small Weezing' in p['part'] else ('large Weezing' if 'large Weezing' in p['part'] else 'Koffing')
   parent=self.find('Small body /' if label=='small Weezing' else 'Body /')
   for child,pre in (('Pupil /','Eye /'),('Tongue /','Mouth /'),('Skull center /','Skull mark /')):
    if p['part'].startswith(child):parent=next(q for q in self.ps if q['part'].startswith(pre) and label in q['part'] and (pre!='Eye /' or q['part'].split()[-1]==p['part'].split()[-1]))
   yield p,parent
 def test_fitted_vertices_follow_actual_parent(self):
  for n in HEIGHTS:self.select(n);attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)
 def test_fitted_faces_and_edges_clear_actual_parent(self):
  for n in HEIGHTS:self.select(n);attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)
 def test_fangs_and_stalks_are_uncrossed(self):
  for n in HEIGHTS:
   self.select(n)
   for p in self.ps:
    if p['part'].startswith(('Fang /','Main stalk /','Rear stalk /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_each_floating_cloud_has_connected_lobes(self):
  for n in HEIGHTS:
   self.select(n)
   for j in (0,1):
    cloud=self.find('Gas cloud /',j)
    for k in range(4):self.assertTrue(self.overlap(self.find('Gas lobe /',f'{j} {k}'),cloud),(n,j,k))
 def fixture(self,n):
  if n not in self.raw:self.raw[n]=anatomy(n)
  raw=self.raw[n];lo=min(v for p in raw for v in p['positions'][1::3]);hi=max(v for p in raw for v in p['positions'][1::3]);scale=HEIGHTS[n]/(hi-lo)
  bodies=[('Body /',(0,.47,0),(.43,.41,.37))] if n=='koffing' else [('Body /',(-.22,.51,0),(.43,.42,.375)),('Small body /',(.40,.86,-.025),(.27,.28,.25))]
  def convert(q):return tuple((v-(lo if i==1 else 0))*scale for i,v in enumerate(q))
  return bodies,convert,scale
 def test_thirteen_actual_radial_crater_recesses_on_each_orb(self):
  for n in HEIGHTS:
   self.select(n);bodies,convert,_=self.fixture(n)
   for prefix,c,r in bodies:
    for d in VENTS:
     q=convert(tuple(c[i]+r[i]*unit(d)[i]*.98 for i in range(3)));self.assertFalse(occupied(self.find(prefix),q),(n,prefix,d))
 def test_actual_front_expression_recess_space(self):
  for n in HEIGHTS:
   self.select(n);bodies,convert,_=self.fixture(n)
   for prefix,c,r in bodies:
    q=convert((c[0],c[1]+r[1]*(.075 if n=='koffing' else .055),c[2]+r[2]*.87));self.assertFalse(any(occupied(p,q) for p in self.ps),(n,prefix))
 def test_filled_craters_and_detached_teeth_stalks_are_rejected(self):
  for n in HEIGHTS:
   self.select(n);bodies,convert,scale=self.fixture(n)
   for prefix,c,r in bodies:
    for d in VENTS:
     q=convert(tuple(c[i]+r[i]*unit(d)[i]*.98 for i in range(3)));self.assertTrue(occupied(ellipsoid('Filled crater negative',PURPLE,q,(.025*scale,)*3,24,18),q))
   for p in self.ps:
    if p['part'].startswith('Fang /'):self.assertFalse(self.overlap(translate(copy.deepcopy(p),3,0,0),self.find('Small body /' if 'small Weezing' in p['part'] else 'Body /')))
  self.select('weezing');self.assertFalse(self.overlap(translate(copy.deepcopy(self.find('Main stalk /')),3,0,0),self.find('Body /')))
if __name__=='__main__':unittest.main()
