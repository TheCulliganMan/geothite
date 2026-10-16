#!/usr/bin/env python3
"""Independent stored Bellossom skin, grounded leaf and source palette checks."""
import math,unittest
from pathlib import Path
import bellossom_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BellossomSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'bellossom.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_bind_and_existing_loader_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_bellossom(author.decode_bellossom(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(45,18524,26980,25))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_source_primary_colors_and_white_skirt(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(49,140,66) if n.startswith(('Body /','Head /','Arm /','Flower center /','Flower well /')) else (222,66,49) if n.startswith('Flower petal /') or 'broad red outer leaf' in n else (255,255,255) if 'curved white inner leaf' in n else None
   if rgb is None:continue
   material=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,material):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_fitted_face_and_complete_flower_crowns_share_rigid_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=3 if n.startswith(('Head /','Eye /','Eye glint /','Cheek /','Mouth /','Tongue /')) else (8 if n.split()[-2 if n.startswith('Flower petal /') else -1]=='-1' else 9) if n.startswith('Flower ') else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_all_leaf_layers_and_both_arms_are_weighted(self):
  m=self.m
  for p,n in enumerate(m.names):
   used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w}
   if n.startswith('Skirt /'):
    layer=0 if 'underskirt' in n else 1 if 'inner leaf' in n else 2
    self.assertIn(10+int(n.rsplit(' ',1)[1])*3+layer,used,n);self.assertIn(0,used,n)
   if n.startswith('Arm /'):self.assertTrue(({1,4,5} if 'raised right' in n else {1,6,7})<=used,n)
 def test_floor_fixed_tips_neutral_endpoints_winding_and_strain(self):
  m=self.m;supports=0;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     for before,after in zip(m.vertices[p],posed):
      if before[1]<.001:self.assertEqual(before,after);supports+=1
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step%6==0:
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
       self.assertGreater(cosine,0.,(clip,step,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,step,n,ratio))
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(supports,0);self.assertTrue({'Body','Head','Arm','Skirt','Flower petal','Flower center','Flower well'}<=moved,moved)
 def test_delayed_crowns_rotation_only_and_identity_root(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  body=[m.rotations('attack',i/100)[1][0] for i in range(101)];flower=[m.rotations('attack',i/100)[8][0] for i in range(101)]
  self.assertGreater(flower.index(max(flower)),body.index(max(body)))
if __name__=='__main__':unittest.main()
