#!/usr/bin/env python3
"""Stored Gligar skin: coherent pincers, coupled sails and planted talons."""
import math,unittest
from pathlib import Path
import gligar_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class GligarSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'gligar.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_source_bind_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_gligar(author.decode_gligar(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(32,17422,26768,15))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_source_pink_violet_tongue_white_and_black(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(66,57,165) if n.startswith(('Wing /','Tongue /')) else (0,0,0) if n.startswith(('Pupil /','Mouth /','Tongue crease /')) else (255,255,255) if n.startswith(('Eye /','Eye glint /','Fang /')) else (206,49,148)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_fitted_face_open_pincers_tongue_and_tail_have_coherent_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Foot claw /')) else 2 if n.startswith(('Head /','Eye /','Pupil /','Eye glint /','Mouth /','Fang /')) else 5 if n.startswith(('Tongue /','Tongue crease /')) else (3 if n.endswith('-1') else 4) if n.startswith('Ear /') else (7 if n.endswith('-1') else 9) if n.startswith('Pincer /') else 13 if n.startswith('Tail bulb /') else 14 if n.startswith('Stinger /') else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_both_arm_sail_and_spar_units_use_the_same_attachment_chain(self):
  m=self.m
  for p,n in enumerate(m.names):
   used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w}
   if n.startswith(('Arm /','Wing /','Wing spar /')):self.assertEqual(used,{1,6,7} if n.endswith('-1') else {1,8,9},n)
   if n.startswith('Tail /'):self.assertEqual(used,{1,10,11,12,13},n)
   if n.startswith(('Body /','Leg /')):self.assertEqual(used,{0,1},n)
 def test_planted_feet_neutral_endpoints_and_articulated_motion(self):
  m=self.m;moved=set();support=0
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Foot /','Foot claw /')):self.assertEqual(posed,m.vertices[p]);support+=1
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(support,0);self.assertTrue({'Body','Head','Ear','Tongue','Arm','Pincer','Wing','Wing spar','Leg','Tail','Tail bulb','Stinger'}<=moved,moved)
 def test_winding_and_edge_strain_through_sails_and_tail(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80,1.):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
      self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,t,n,ratio))
 def test_tongue_is_flat_and_proportioned_to_the_head(self):
  m=self.m;head=m.vertices[m.names.index('Head / broad flattened pink glider face')];tongue=m.vertices[m.names.index('Tongue / broad rounded protruding pink tongue')]
  span=lambda points,axis:max(v[axis] for v in points)-min(v[axis] for v in points)
  self.assertLess(span(tongue,0)/span(head,0),.30)
  self.assertLess(span(tongue,1)/span(head,1),.65)
  self.assertLess(span(tongue,2)/span(tongue,0),.30)
 def test_identity_root_rotation_only_and_delayed_pincers_and_tail(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  shoulder=[-m.rotations('attack',i/100)[6][2] for i in range(101)];pincer=[-m.rotations('attack',i/100)[7][2] for i in range(101)];tail=[m.rotations('attack',i/100)[13][1] for i in range(101)]
  self.assertGreater(pincer.index(max(pincer)),shoulder.index(max(shoulder)));self.assertGreater(tail.index(max(tail)),shoulder.index(max(shoulder)))
if __name__=='__main__':unittest.main()
