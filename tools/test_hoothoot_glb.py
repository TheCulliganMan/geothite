#!/usr/bin/env python3
"""Stored owl anatomy, palette, skin ownership and deformation regressions."""
import math,os,unittest
from pathlib import Path
import hoothoot_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ASSET=Path(os.environ.get('HOOTHOOT_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/hoothoot.glb'));ROOT=ASSET.parent
class HoothootSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_owl(cls.blob,'hoothoot')
 def test_canonical_anatomy_reader_bounds_and_unique_species(self):
  m=self.m
  self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['hoothoot']);self.assertEqual(self.blob,author.export_hoothoot())
  self.assertEqual((len(m.parts),sum(len(p) for p in m.vertices),sum(len(p)//3 for p in m.indices),len(m.joints)),(38,7029,8684,12))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  self.assertEqual(min(v[1] for p in m.vertices for v in p),0.);self.assertAlmostEqual(max(v[1] for p in m.vertices for v in p),1.,places=6)
  self.assertFalse((ROOT/'hoothoot.mesh.json').exists())
 def test_closed_outward_parts_and_stored_shading(self):
  from pidgeotto_contours import components
  ps=[q for p in self.neutral['primitives'] for q in (components(p) if p['part'].startswith(('Eye mask /','Leg band /')) or 'scalloped' in p['part'] else [p])]
  assert_closed_models(self,{'hoothoot':dict(primitives=ps)});assert_model_shading(self,{'hoothoot':self.neutral})
 def test_body_and_clock_use_inspected_full_colour_game_pigments(self):
  # Crystal reuses one shadow index; it does not make the clock hands blue.
  for prefix,rgb in [('Body /',(.67,.44,.25)),('Clock brow /',(.045,.038,.033))]:
   expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for p,name in zip(self.m.parts,self.m.names):
    if name.startswith(prefix):
     actual=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
 def test_visible_eye_caps_do_not_cut_into_curved_body_or_mask(self):
  ps=self.neutral['primitives'];body=ps[0]
  for part in ps:
   if not part['part'].startswith(('Eye mask /','Eye /')):continue
   parent=body
   if part['part'].startswith('Eye /'):
    side=part['part'].split()[-1];parent=next(p for p in ps if p['part'].startswith('Eye mask /') and p['part'].endswith(' '+side))
   points=[part['positions'][i:i+3] for i in range(0,len(part['positions']),3)]
   front=0
   for i in range(0,len(part['indices']),3):
    a,b,c=[points[j] for j in part['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=0:continue
    front+=1
    if front%16!=1:continue
    x,y,z=[(a[k]+b[k]+c[k])/3 for k in range(3)];hits=ray_intersections(parent,x,y)
    self.assertTrue(hits,(part['part'],i));self.assertGreater(z-max(hits),1e-6,(part['part'],i))

 def test_two_anatomical_feet_with_one_tucked_behind_body(self):
  m=self.m;body=next(p for p in self.neutral['primitives'] if p['part'].startswith('Body /'))
  feet=[p for p in self.neutral['primitives'] if p['part'].startswith('Foot /')];self.assertEqual(len(feet),2)
  planted=next(p for p in feet if 'planted' in p['part']);tucked=next(p for p in feet if 'tucked' in p['part'])
  self.assertLess(min(planted['positions'][1::3]),1e-6)
  x=sum(tucked['positions'][::3])/len(tucked['positions'][::3]);y=sum(tucked['positions'][1::3])/len(tucked['positions'][1::3]);hits=ray_intersections(body,x,y)
  self.assertTrue(hits);self.assertGreater(max(hits),max(tucked['positions'][2::3]))
  self.assertEqual(sum(n.startswith('Clock brow /') for n in m.names),4)
  self.assertEqual(sum(n.startswith('Toe /') for n in m.names),3)
 def test_hierarchy_binds_and_fitted_face_ownership(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['hoothoot']])
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  for p,name in enumerate(m.names):
   for js,ws in zip(m.ids[p],m.weights[p]):self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<12 for j in js))
   if name.startswith(('Eye mask /','Eye /','Pupil /','Glint /','Upper beak /')):self.assertTrue(all(js==[1,0,0,0] for js in m.ids[p]),name)
   if name.startswith('Lower beak /'):self.assertTrue(all(js==[11,0,0,0] for js in m.ids[p]))
 def test_floor_contacts_winding_and_skin_strain(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith(('Leg /','Leg band /','Foot /','Toe /','Claw /','Rear toe /')):self.assertEqual(posed,m.vertices[p],name)
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
      self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_identity_endpoints_and_cue_relative_body_brace(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  values=[m.rotations('attack',i/100)[1][0] for i in range(101)];self.assertGreater(values[20],max(values)*.9)
  for a in m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle);self.assertEqual(a['extras']['cueRelative'],not idle)
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
