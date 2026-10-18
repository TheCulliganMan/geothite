#!/usr/bin/env python3
"""Stored dual-star anatomy, coherent core mount and ten radial arm chains."""
import copy,math,unittest
from pathlib import Path
import starmie_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class StarmieTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'starmie.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def test_canonical_schema_and_recognizable_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_starmie());self.assertEqual(len(m.parts),23);self.assertEqual(len(m.joints),24)
  self.assertFalse((ROOT/'starmie.mesh.json').exists());self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(sum(map(len,m.vertices)),2966);self.assertEqual(sum(map(len,m.indices))//3,3344)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.03,places=6)
  for prefix,count in [('Front star /',1),('Back star /',1),('Front ridge /',5),('Back ridge /',5),('Mount spoke /',8),('Core /',1)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'starmie':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'starmie':self.m.neutral_model()})
 def test_core_ownership_and_ten_arm_chains(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith(('Mount','Core')):self.assertEqual(owners,{1},n)
   elif n.startswith('Front star /'):self.assertEqual(owners,set(range(2,13)))
   elif n.startswith('Back star /'):self.assertEqual(owners,{0,*range(13,24)})
  for j in range(3,13,2):self.assertEqual(m.parents[j],2);self.assertEqual(m.parents[j+1],j)
  for j in range(14,24,2):self.assertEqual(m.parents[j],13);self.assertEqual(m.parents[j+1],j)
 def test_occupied_core_and_spokes_survive_motion(self):
  m=self.m;names=m.names;mount=next(i for i,n in enumerate(names) if n.startswith('Mount /'));core=next(i for i,n in enumerate(names) if n.startswith('Core /'))
  for clip in m.clips:
   mats=m.matrices(clip,.31);parts=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)]
   for i,n in enumerate(names):
    if n.startswith('Mount spoke /') or i==core:self.assertTrue(parts_overlap(parts[i],parts[mount]),n)
   self.assertTrue(parts_overlap(parts[0],parts[1]),'offset stars must occupy one coherent center')
  spoke=copy.deepcopy(self.parts[13]);spoke['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(spoke['positions'])]
  self.assertFalse(parts_overlap(spoke,self.parts[mount]))
 def test_floor_neutral_endpoints_winding_strain_and_layer_motion(self):
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
  self.assertTrue({'Front star','Back star','Front ridge','Back ridge'}<=moved,moved)
 def test_core_glint_fits_actual_faceted_gem(self):
  core=next(p for p in self.parts if p['part'].startswith('Core /'));glint=next(p for p in self.parts if p['part'].startswith('Core glint /'))
  for i in range(0,len(glint['positions']),3):
   x,y,z=glint['positions'][i:i+3];hits=rays(core,x,y);self.assertTrue(hits);self.assertLess(abs(z-max(hits)),.004)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
