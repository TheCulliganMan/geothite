#!/usr/bin/env python3
"""Stored Yanma skin, lossless feature packing, wing coherence and planted toes."""
import math,unittest
from pathlib import Path
import yanma_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class YanmaSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'yanma.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_bind_and_existing_budgets(self):
  m=self.m;decoded=author.decode_yanma(self.blob);self.assertEqual(self.blob,author.export_yanma(decoded));self.assertEqual(len(decoded['primitives']),66);self.assertEqual(m.doc['extras']['packed_sha256'],author.PACKED_REST_DIGEST)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(58,19976,26452,16))
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_source_red_green_white_and_black(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(74,214,82) if n.startswith(('Head /','Leg /','Foot /')) else (0,0,0) if n.startswith(('Eye /','Mouth /','Wing vein /','Abdomen collar /')) else (255,255,255) if n.startswith(('Eye glint /','Wing /','Tail tip /')) else (165,57,57)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_wing_units_face_terminal_fins_and_toes_share_rigid_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith('Foot /') else 1 if n.startswith('Thorax /') else 2 if n.startswith(('Head /','Eye /','Eye glint /','Crown /','Mandible /','Forehead /','Mouth /')) else 15 if n.startswith(('Tail cap /','Tail fin /','Tail tip /')) else None
   if n.startswith(('Wing /','Wing band /','Wing vein /')):
    words=n.split();side,rear=map(int,words[-3:-1] if n.startswith('Wing vein /') else words[-2:]);owner=3+(0 if side<0 else 2)+rear
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_vein_groups_preserve_all_twelve_named_tubes(self):
  groups=[p['source_parts'] for p in self.m.doc['extras']['source_part_ranges'] if 'source_parts' in p]
  self.assertEqual(len(groups),4);self.assertTrue(all(len(g)==3 for g in groups));self.assertEqual(len({s['part'] for g in groups for s in g}),12)
  decoded=author.decode_yanma(self.blob);self.assertEqual(author.geometry_digest(decoded),author.REST_DIGEST)
 def test_floor_height_neutral_endpoints_and_articulated_motion(self):
  m=self.m;moved=set();fixed=0
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith('Foot /'):self.assertEqual(posed,m.vertices[p]);fixed+=1
     if n.startswith(('Thorax /','Leg /')):
      for before,after in zip(m.vertices[p],posed):self.assertAlmostEqual(before[1],after[1],places=7)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(fixed,0);self.assertTrue({'Thorax','Head','Wing','Wing band','Wing vein','Leg','Abdomen','Abdomen collar','Tail cap','Tail fin','Tail tip'}<=moved,moved)
 def test_winding_and_edge_strain_through_all_clips(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80,1.):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,t,n,ratio))
 def test_identity_root_rotation_only_and_delayed_rear_wings(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  front=[-m.rotations('attack',i/100)[3][2] for i in range(101)];rear=[-m.rotations('attack',i/100)[4][2] for i in range(101)];self.assertGreater(rear.index(max(rear)),front.index(max(front)))
if __name__=='__main__':unittest.main()
