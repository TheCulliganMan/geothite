#!/usr/bin/env python3
"""Independent stored Wobbuffet skin, support and coherent watchful tail checks."""
import math,unittest
from pathlib import Path
import wobbuffet_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class WobbuffetSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'wobbuffet.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_source_bind_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_wobbuffet(author.decode_wobbuffet(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(19,11791,19260,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_source_blue_black_mouth_and_shaded_tongue(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(99,173,255) if n.startswith(('Body /','Arm /','Hand /','Foot /')) else (57,115,132) if n.startswith('Tongue /') else (255,255,255) if n.startswith('Tail eye /') else (0,0,0)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_face_hands_tail_eyes_and_feet_have_coherent_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith('Foot /') else 1 if n.startswith(('Eye /','Mouth /','Tongue /')) else 10 if n.startswith(('Tail paddle /','Tail eye /','Tail pupil /')) else (4 if 'salute' in n else 7) if n.startswith('Hand /') else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
   used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w}
   if n.startswith('Arm /'):self.assertEqual(used,{1,2,3,4} if 'salute' in n else {1,5,6,7},n)
   if n.startswith('Tail /'):self.assertEqual(used,{0,8,9,10},n)
 def test_planted_basal_lobes_neutral_endpoints_and_articulated_motion(self):
  m=self.m;moved=set();supports=0
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith('Foot /'):
      self.assertEqual(posed,m.vertices[p]);supports+=1
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(supports,0);self.assertTrue({'Body','Arm','Hand','Tail','Tail paddle','Tail eye','Tail pupil'}<=moved,moved)
 def test_winding_and_elastic_edge_strain_through_all_clips(self):
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
 def test_identity_root_rotation_only_and_delayed_watchful_tail(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  body=[m.rotations('attack',i/100)[1][0] for i in range(101)];tail=[m.rotations('attack',i/100)[10][1] for i in range(101)]
  self.assertGreater(tail.index(max(tail)),body.index(max(body)))
if __name__=='__main__':unittest.main()
