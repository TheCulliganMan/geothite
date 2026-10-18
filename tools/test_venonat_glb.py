#!/usr/bin/env python3
"""Fuzzy insect anatomy, occupied roots and independently articulated planted-foot skin checks."""
import copy,math,unittest
from pathlib import Path
import venonat_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'venonat.glb'
class VenonatTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_distinctive_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_venonat());self.assertEqual(len(m.parts),33);self.assertEqual(len(m.joints),10)
  self.assertFalse((ROOT/'venonat.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.10,places=6)
  for prefix,count in [('Eye /',2),('Antenna /',2),('Antenna tip /',2),('Fang /',2),('Arm /',2),('Hand /',2),('Finger /',6),('Leg /',2),('Foot /',2),('Fur crest /',3)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'venonat':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'venonat':self.m.neutral_model()})
 def test_lens_glints_follow_actual_faceted_surface(self):
  for mark in self.parts:
   if not mark['part'].startswith('Eye sheen /'):continue
   parent=self.find('Eye /',[mark['part'].split()[-1]])
   vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
   for k in range(0,len(mark['indices']),3):
    a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits);self.assertGreater(q[2]-max(hits),.0001)
 def test_anatomical_owners(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith(('Foot /','Toe crease /')):expected={0}
   elif n.startswith('Body /'):expected={0,1}
   elif n.startswith(('Antenna /','Antenna tip /')):expected={2 if n.endswith('-1') else 3}
   elif n.startswith('Arm /'):expected={1,4 if n.endswith('-1') else 5}
   elif n.startswith('Hand /'):expected={6 if n.endswith('-1') else 7}
   elif n.startswith('Finger /'):expected={6 if n.split()[-2]=='-1' else 7}
   elif n.startswith('Leg /'):expected={0,1,8 if n.endswith('-1') else 9}
   else:expected={1}
   self.assertEqual(owners,expected,n)
 def test_occupied_roots_survive_motion(self):
  m=self.m;body=self.find('Body /')['part'];pairs=[]
  for p in self.parts:
   n=p['part'];side=[n.split()[-1]]
   if n.startswith(('Eye /','Antenna /','Arm /','Leg /','Fur crest /','Nose /')):pairs.append((n,body))
   if n.startswith('Fang /'):pairs.append((n,self.find('Nose /')['part']))
   if n.startswith('Antenna tip /'):pairs.append((n,self.find('Antenna /',side)['part']))
   if n.startswith('Hand /'):pairs.append((n,self.find('Arm /',side)['part']))
   if n.startswith('Finger /'):pairs.append((n,self.find('Hand /',[n.split()[-2]])['part']))
   if n.startswith('Leg /'):pairs.append((n,self.find('Foot /',side)['part']))
  for clip in m.clips:
   for t in (0.,.31):
    mats=m.matrices(clip,t);parts={p['part']:dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)}
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  detached=copy.deepcopy(self.find('Nose /'));detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,self.find('Body /')))
 def test_floor_neutral_endpoints_winding_strain_and_independent_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(q[1] for q in posed),-1e-7,(clip,n))
     displacement=max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))
     if n.startswith(('Foot /','Toe crease /')):self.assertLess(displacement,1e-7,n)
     elif displacement>.002:moved.add(n.split(' /')[0])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    if step in (0,8):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Antenna','Arm','Hand','Leg'}<=moved,moved)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
