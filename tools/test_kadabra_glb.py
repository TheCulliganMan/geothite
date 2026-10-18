#!/usr/bin/env python3
"""Stored Kadabra neck, exact reviewed geometry, lossless skin and held spoon."""
import copy,math,unittest
from pathlib import Path
import kadabra_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,assert_closed_models,assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class KadabraSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'kadabra.glb').read_bytes();cls.runtime=skin.Model(cls.blob);cls.m=logical_model(cls.runtime);cls.ps=cls.m.neutral_model()['primitives']
 def find(self,name):return self.m.names.index(name)
 def test_canonical_limits_lossless_ranges_hierarchy_and_identity(self):
  m=self.m;r=self.runtime;self.assertEqual(self.blob,author.export_kadabra());self.assertFalse((ROOT/'kadabra.mesh.json').exists());self.assertEqual(len(m.parts),60);self.assertEqual(len(r.parts),38);self.assertEqual(len(m.joints),18)
  self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['kadabra']);self.assertEqual(author.geometry_digest(r.neutral_model()),author.PACKED_REST_DIGESTS['kadabra']);self.assertEqual(author.geometry_digest(author.decode_kadabra(self.blob)),author.REST_DIGESTS['kadabra'])
  self.assertEqual(author.geometry_digest({'primitives':[p for p in self.ps if p['part'] not in ('Narrow neck','Long swept moustache','Long swept moustache.001','Broad tail saddle band')]}),'47b493c8110ead2c4fcc676f31ef36c83f2b594c11de7dd8216592f96f888734')
  self.assertLessEqual(sum(map(len,r.vertices)),20000);self.assertLessEqual(sum(map(len,r.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(m.parents[:18],[None,0,1,1,3,4,1,6,7,0,0,2,2,2,2,1,15,8])
  self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(18)))
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'kadabra':self.m.neutral_model()})
 def test_authored_triangle_shading(self):assert_model_shading(self,{'kadabra':self.m.neutral_model()})
 def test_coherent_features_and_fixed_complete_supports(self):
  m=self.m
  for i,n in enumerate(m.names):
   expected=None;left=sum(p[0] for p in m.vertices[i])<0
   if n.startswith(('Splayed hind foot','Three long hind toes','Ivory toe claw')):expected=0
   elif n.startswith(('Long tapered fox ear','Recessed ochre ear plane')):expected=11 if left else 12
   elif n.startswith('Long swept moustache'):expected=13 if left else 14
   elif n.startswith(('Held silver spoon','Hollow silver spoon')):expected=17
   elif n.startswith(('Three-fingered hand palm','Curled hand finger','Pale finger claw')):expected=5 if left else 8
   elif n.startswith(('Sculpted fox skull','Narrow focused white eye','Focused slit pupil','Angular brow plane','Five-point red forehead star')):expected=2
   if expected is not None:self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{expected},n)
 def test_neck_and_anatomical_roots_remain_physically_connected(self):
  m=self.m;find=self.find;body=find('Tapered abdomen');chest=find('Angular chest cuirass');neck=find('Narrow neck');head=find('Sculpted fox skull and projecting tapered muzzle');pairs=[(body,chest),(chest,neck),(neck,head),(body,find('Heavy curved psychic tail')),(find('Heavy curved psychic tail'),find('Broad tail saddle band'))]
  for side in range(2):
   suffix='.001' if side else '';armor=find('Sculpted shoulder armor'+suffix);upper=find('Bent upper arm'+suffix);fore=find('Sculpted tapered forearm'+suffix);hand=find('Three-fingered hand palm'+suffix);leg=find('Bent muscular hind leg'+suffix);foot=find('Splayed hind foot'+suffix)
   pairs += [(chest,armor),(armor,upper),(upper,fore),(fore,hand),(body,leg),(leg,foot),(head,find('Long tapered fox ear'+suffix)),(head,find('Long swept moustache'+suffix))]
   for j in range(3):
    index=side*3+j;end='.'+str(index).zfill(3) if index else '';finger=find('Curled hand finger'+end);claw=find('Pale finger claw'+end);toe=find('Three long hind toes'+end);tip=find('Ivory toe claw'+end);pairs += [(hand,finger),(finger,claw),(foot,toe),(toe,tip)]
  for clip in m.clips:
   for t in (0.,.31):
    mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.ps)]
    for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(clip,t,m.names[a],m.names[b]))
  old=copy.deepcopy(self.ps[neck]);low=min(old['positions'][1::3]);scale=.14/.225;old['positions']=[low+(v-low)*scale if i%3==1 else v for i,v in enumerate(old['positions'])];self.assertFalse(parts_overlap(old,self.ps[head]),'pre-fix neck must expose the actual gap')
 def test_spoon_is_supported_by_hand_and_bowl_is_joined_to_stem(self):
  m=self.m;stem=self.find('Held silver spoon handle');bowl=self.find('Hollow silver spoon bowl');hand=self.find('Three-fingered hand palm.001');fingers=[self.find('Curled hand finger.'+str(j).zfill(3)) for j in (3,4,5)]
  for clip in m.clips:
   for t in (0.,.13,.31,.58,.83):
    mats=m.matrices(clip,t);posed={i:dict(self.ps[i],positions=sum(m.posed(i,mats),[])) for i in [stem,bowl,hand]+fingers}
    self.assertTrue(any(parts_overlap(posed[stem],posed[i]) for i in [hand]+fingers),(clip,t));self.assertTrue(parts_overlap(posed[stem],posed[bowl]),(clip,t))
 def test_floor_strain_winding_endpoints_and_independent_articulation(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);floor=float('inf')
    for i,n in enumerate(m.names):
     points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(clip,n))
     if n.startswith(('Splayed hind foot','Three long hind toes','Ivory toe claw')):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    self.assertLess(abs(floor),1e-7)
    if step in (0,8):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  rotations=m.rotations('attack',.31)
  for joint in range(1,18):
   isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate)
   self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,joint)
 def test_tail_band_is_derived_from_actual_stored_tail(self):
  model=author.authoring_model();self.assertEqual(author.geometry_digest(author.fit_tail_band(model)),author.REST_DIGESTS['kadabra'])
 def test_strict_reader_rejects_mutated_geometry_and_animation(self):
  for index in (-8,-100):
   blob=bytearray(self.blob);blob[index]^=1
   with self.assertRaises(ValueError):author.decode_kadabra(bytes(blob))
  changed=copy.deepcopy(author.authoring_model());changed['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_kadabra(model=changed)
if __name__=='__main__':unittest.main()
