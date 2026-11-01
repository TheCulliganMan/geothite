#!/usr/bin/env python3
"""Stored Qwilfish skin: coherent spines/face, articulated tail and floor height."""
import math,unittest
from pathlib import Path
import qwilfish_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class QwilfishSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'qwilfish.glb').read_bytes();cls.m=Model(cls.blob)
 def test_bind_canonical_source_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_qwilfish(author.decode_qwilfish(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(36,9014,13644,7))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_source_navy_yellow_and_yellow_lips(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(33,49,132) if n.startswith('Body / closed teal') or n.startswith('Spine / rooted teal') or n.startswith('Tail spot /') else (255,255,255) if n.startswith('Tail fin / fitted') else (222,239,90) if n.startswith(('Body / closed cream','Spine / rooted cream','Fin /','Lips /')) else None
   if rgb is None:continue
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_body_spines_face_and_spotted_paddle_have_coherent_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=6 if n.startswith(('Tail fin /','Tail spot /')) else 1 if not n.startswith(('Tail /','Fin /','Fin crease /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
   if n.startswith(('Fin /','Fin crease /')):
    used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w}
    self.assertTrue({1,3 if n.endswith('-1') else 2}<=used,n)
   if n.startswith('Tail /'):
    used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w};self.assertEqual(used,{1,4,5,6})
 def test_yaw_only_clips_preserve_every_vertex_height_and_neutral_endpoints(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats)
     for before,after in zip(m.vertices[p],posed):self.assertAlmostEqual(before[1],after[1],places=7)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Spine','Fin','Fin crease','Tail','Tail fin','Tail spot'}<=moved,moved)
 def test_winding_and_edge_strain_through_tail_and_fin_strokes(self):
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
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.75<ratio<1.25,(clip,t,n,ratio))
 def test_identity_root_rotation_only_and_delayed_tail(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  body=[m.rotations('attack',i/100)[1][1] for i in range(101)];tail=[m.rotations('attack',i/100)[6][1] for i in range(101)]
  self.assertGreater(tail.index(max(tail)),body.index(max(body)))
if __name__=='__main__':unittest.main()
