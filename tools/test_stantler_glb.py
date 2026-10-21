#!/usr/bin/env python3
"""Decoded Stantler skin: grounded hooves, coherent antler loops and independently curled tail."""
import os,math,unittest
from pathlib import Path
import stantler_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/stantler.glb'
ROOT=Path(os.environ.get('STANTLER_GLB',str(ROOT)))
class StantlerSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_stantler(author.decode_stantler(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(56,13113,12428,12))
  self.assertEqual(author.geometry_digest(author.decode_stantler(self.blob)),author.REST_DIGEST)
  self.assertEqual(len(self.runtime.parts),51);self.assertEqual(author.geometry_digest(self.runtime.neutral_model()),self.runtime.doc['extras']['packed_sha256'])
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_full_color_game_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(30,25,29) if n.startswith(('Eye / fitted dark','Eye / dark','Nostril /','Hoof groove /')) else (100,62,42) if n.startswith(('Muzzle /','Ear / fitted')) else (80,51,95) if n.startswith(('Hoof /','Antler orb /')) else (242,218,143) if n.startswith(('Antler /','Antler stem /','Antler tine /')) else (245,239,217) if n.startswith(('Eye / fitted ivory','Chest /','Brow spot /','Spot /','Rib /')) else (200,156,120)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_fitted_gaze_antlers_orbs_ears_and_flank_markings_share_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Hoof /','Hoof groove /')) else 3 if n.startswith(('Head /','Eye /','Brow /','Brow spot /','Muzzle /','Muzzle pad /','Nostril /','Antler')) else (4 if n.endswith('-1') else 5) if n.startswith('Ear /') else 11 if n.startswith('Tail / large') else 1 if n.startswith(('Body /','Spot /','Rib /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_fixed_hooves_endpoints_and_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Hoof /','Hoof groove /')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Eye','Muzzle','Antler','Ear','Tail','Leg'}<=moved,moved)
 def test_winding_and_edge_strain(self):
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
 def test_rotation_only_root_and_delayed_tail_curl(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  torso=[m.rotations('attack',i/100)[1][1] for i in range(101)];held=[m.rotations('attack',i/100)[11][1] for i in range(101)]
  self.assertGreater(held.index(max(held)),torso.index(max(torso)))
if __name__=='__main__':unittest.main()
