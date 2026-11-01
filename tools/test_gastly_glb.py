#!/usr/bin/env python3
"""Stored ghost face, fitted fangs, coherent vapor and independent gas-sector checks."""
import copy,math,unittest
from pathlib import Path
import gastly_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'gastly.glb'
class GastlyTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def test_canonical_schema_limits_and_recognizable_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_gastly());self.assertEqual(len(m.parts),19);self.assertEqual(len(m.joints),11)
  self.assertFalse((ROOT/'gastly.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.05,places=6)
  for prefix,count in [('Body /',1),('Gas halo /',1),('Gas curl /',9),('Eye /',2),('Pupil /',2),('Fang /',2)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'gastly':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'gastly':self.m.neutral_model()})
 def test_fitted_face_clears_actual_stored_parents(self):
  for mark in self.parts:
   n=mark['part']
   if n.startswith(('Eye /','Mouth rim /','Fang /')):parent=self.parts[0]
   elif n.startswith('Pupil /'):parent=next(p for p in self.parts if p['part'].startswith('Eye /') and p['part'].split()[-1]==n.split()[-1])
   elif n.startswith('Mouth /'):parent=next(p for p in self.parts if p['part'].startswith('Mouth rim /'))
   else:continue
   positions=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
   for k in range(0,len(mark['indices']),3):
    a,b,c=[positions[j] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n)
    self.assertGreater(q[2]-max(hits),.0001,(n,q))
 def test_coherent_face_and_eight_blended_gas_sectors(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if not n.startswith('Gas '):self.assertEqual(owners,{1},n)
   elif n.startswith('Gas halo /'):self.assertEqual(owners,set(range(3,11)))
  self.assertEqual(m.parents[1],0);self.assertEqual(m.parents[2],0)
  for j in range(3,11):self.assertEqual(m.parents[j],2)
 def test_occupied_orb_halo_and_curl_roots_survive_motion(self):
  m=self.m
  for clip in m.clips:
   mats=m.matrices(clip,.31);parts=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)]
   self.assertTrue(parts_overlap(parts[0],parts[1]),clip)
   for i,n in enumerate(m.names):
    if n.startswith('Gas curl /'):self.assertTrue(parts_overlap(parts[i],parts[1]),(clip,n))
  detached=copy.deepcopy(self.parts[2]);detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])]
  self.assertFalse(parts_overlap(detached,self.parts[1]))
 def test_floor_neutral_endpoints_winding_strain_and_independent_gas(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(q[1] for q in posed),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))>.002:moved.add(n.split(' /')[0])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    if step in (0,8):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Eye','Fang','Gas halo','Gas curl'}<=moved,moved)
  mats=m.matrices('attack',.31)
  self.assertGreater(max(math.dist(m.point(1,v,mats),skin.transform(mats[1],p)) for v,p in enumerate(m.vertices[1])),.002)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
