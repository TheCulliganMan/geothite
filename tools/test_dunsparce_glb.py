#!/usr/bin/env python3
"""Stored Dunsparce skin: grounded belly, fitted wings and tilted drill axis."""
import math,unittest
from pathlib import Path
import dunsparce_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class DunsparceSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'dunsparce.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_source_bind_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_dunsparce(author.decode_dunsparce(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(19,11833,12248,7))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_source_gold_blue_white_and_blue_drill_groove(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(255,198,49) if n.startswith(('Body /','Tail /','Drill /')) else (255,255,255) if n.startswith(('Wing /','Eye / pale')) else (0,0,0) if n.startswith('Eye / fitted') else (66,123,189)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_fitted_face_back_bands_and_drill_spiral_share_rigid_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=6 if n.startswith(('Drill /','Drill groove /')) else 1 if n.startswith(('Body /','Belly /','Eye /','Jaw /','Back band /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
   used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w}
   if n.startswith(('Wing /','Wing crease /')):
    self.assertEqual(used,{2 if n.endswith('-1') else 3},n)
   if n.startswith('Tail /'):self.assertEqual(used,{1,4,5,6},n)
 def test_belly_height_neutral_endpoints_and_articulated_motion(self):
  m=self.m;moved=set();support=0
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Body /','Belly /')):
      for before,after in zip(m.vertices[p],posed):self.assertAlmostEqual(before[1],after[1],places=7)
      support+=1
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(support,0);self.assertTrue({'Body','Belly','Eye','Jaw','Back band','Wing','Wing crease','Tail','Drill','Drill groove'}<=moved,moved)
 def test_winding_and_edge_strain_through_wings_and_tail(self):
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
 def test_drill_spins_about_its_authored_tilted_axis(self):
  m=self.m;turns=0
  for clip in m.clips:
   for i in range(21):
    x,y,z,w=m.rotations(clip,i/20)[6];self.assertAlmostEqual(x,0.,places=7)
    self.assertAlmostEqual(.445*z+.092*y,0.,places=7)
    if abs(y)>.001:turns+=1
  self.assertGreater(turns,0)
 def test_identity_root_rotation_only_and_delayed_drill(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  body=[m.rotations('attack',i/100)[1][1] for i in range(101)];drill=[m.rotations('attack',i/100)[6][1] for i in range(101)]
  self.assertGreater(drill.index(max(drill)),body.index(max(body)))
if __name__=='__main__':unittest.main()
