#!/usr/bin/env python3
"""Stored spectral gaze, coherent necklace and swept two-stage hair locks."""
import math,os,unittest
from pathlib import Path
import misdreavus_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MisdreavusSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=Path(os.environ.get('MISDREAVUS_GLB',ROOT/'misdreavus.glb')).read_bytes();cls.m=Model(cls.blob)
 def test_canonical_bind_geometry_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_misdreavus(author.decode_misdreavus(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(41,14175,23794,15))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_normal_source_colors(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(74,74,140) if n.startswith(('Body /','Head /','Hair /')) else (247,49,156) if n.startswith(('Hair tip /','Iris /','Necklace gem /','Gem glint / soft')) else (255,255,255) if n.startswith(('Eye / warm','Eye glint /','Gem glint / tiny')) else (0,0,0)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_face_gems_highlights_and_cords_share_coherent_head_owner(self):
  m=self.m
  for p,n in enumerate(m.names):
   if not n.startswith(('Body /','Hair /','Hair tip /')):self.assertTrue(all(ids==[2,0,0,0] for ids in m.ids[p]),n)
 def test_all_five_color_seams_remain_closed_through_motion(self):
  m=self.m
  for p,n in enumerate(m.names):
   if not n.startswith('Hair tip /'):continue
   base=m.names.index('Hair / swept volumetric '+n.removeprefix('Hair tip / tapered pink curl '))
   lookup={tuple(v):i for i,v in enumerate(m.vertices[base])};shared=[(lookup[tuple(v)],i) for i,v in enumerate(m.vertices[p]) if tuple(v) in lookup]
   self.assertGreaterEqual(len({tuple(m.vertices[p][i]) for _,i in shared}),24)
   for a,b in shared:self.assertEqual((m.ids[base][a],m.weights[base][a]),(m.ids[p][b],m.weights[p][b]))
   for clip in m.clips:
    for t in (.13,.31,.55,.81):
     mats=m.matrices(clip,t);before=m.posed(base,mats);after=m.posed(p,mats)
     for a,b in shared:self.assertLess(math.dist(before[a],after[b]),1e-8)
 def test_hem_support_neutral_endpoints_and_swept_lock_motion(self):
  m=self.m;body=m.names.index('Body / continuous five-lobed flowing ghost hem and neck');fixed=[i for i,v in enumerate(m.vertices[body]) if v[1]<.015];moved=set()
  self.assertGreaterEqual(len(fixed),5)
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if p==body:
      for i in fixed:self.assertEqual(posed[i],m.vertices[p][i])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Hair','Hair tip','Necklace gem','Necklace cord','Gem glint'}<=moved,moved)
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
 def test_identity_root_rotation_only_and_delayed_curls(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  head=[m.rotations('attack',i/100)[2][0] for i in range(101)];curl=[m.rotations('attack',i/100)[9][0] for i in range(101)]
  self.assertGreater(curl.index(max(curl)),head.index(max(head)))
if __name__=='__main__':unittest.main()
