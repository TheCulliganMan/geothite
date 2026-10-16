#!/usr/bin/env python3
"""Stored Murkrow anatomy, planted digits and coherent animated feature units."""
import math,unittest
from pathlib import Path
import murkrow_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MurkrowSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'murkrow.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_bind_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_murkrow(author.decode_murkrow(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(62,13359,16616,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_normal_source_colors(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(214,214,24) if n.startswith(('Leg /','Foot /','Toe /','Talon /','Binding /','Eye / red')) or (n.startswith('Bill /') and 'seam' not in n) else (255,255,255) if n.startswith(('Eye / fitted ivory','Eye / tiny')) else (0,0,0) if n.startswith(('Eye / fitted sly','Pupil /','Nostril /')) or 'beak seam' in n else (57,57,140)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_fitted_features_and_folded_wings_share_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Toe /','Talon /')) else 2 if n.startswith(('Head /','Hat /','Eye /','Pupil /','Nostril /')) else 3 if n.startswith('Bill /') and ('lower' in n or 'seam' in n) else 2 if n.startswith('Bill /') else 7 if n.startswith(('Tail fan /','Tail fold /')) else 6 if n.startswith('Binding /') else 8+int(n.split()[-1]) if n.startswith('Crest /') else None
   if n.startswith(('Wing /','Wing feather /')):
    side=int(n.split()[-2] if n.startswith('Wing feather /') else n.split()[-1]);owner=4 if side<0 else 5
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_fixed_digits_neutral_endpoints_and_feature_motion(self):
  m=self.m;moved=set();fixed=0
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Foot /','Toe /','Talon /')):self.assertEqual(posed,m.vertices[p]);fixed+=1
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(fixed,0);self.assertTrue({'Body','Neck','Head','Hat','Crest','Bill','Wing','Wing feather','Tail','Binding','Tail fan','Tail fold'}<=moved,moved)
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
 def test_identity_root_rotation_only_delayed_broom(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  head=[m.rotations('attack',i/100)[2][0] for i in range(101)];fan=[m.rotations('attack',i/100)[7][0] for i in range(101)]
  self.assertGreater(fan.index(max(fan)),head.index(max(head)))
if __name__=='__main__':unittest.main()
