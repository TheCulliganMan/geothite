#!/usr/bin/env python3
"""Stored Shuckle skin: rigid open shell, planted feet and fitted soft body."""
import math,unittest
from pathlib import Path
import shuckle_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ShuckleSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'shuckle.glb').read_bytes();cls.m=Model(cls.blob)
 def test_canonical_source_bind_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_shuckle(author.decode_shuckle(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(27,14875,19836,8))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_source_red_gold_white_and_black(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(173,33,24) if n.startswith('Shell /') else (255,255,255) if n.startswith(('Rim /','Spot /')) else (255,181,41) if n.startswith(('Limb /','Foot /','Neck /','Head /')) else (0,0,0)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_hard_shell_collars_spots_feet_and_fitted_face_have_coherent_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Shell /','Interior /','Rim /','Spot /','Foot /')) else 7 if n.startswith(('Head /','Eye /','Mouth /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
   used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w}
   if n.startswith('Limb /'):
    label=n.removeprefix('Limb / smooth flexible yellow ');i=next(i for i,(name,path) in enumerate(author.LIMBS) if name==label);self.assertEqual(used,{0,1+i},n)
   if n.startswith('Neck /'):self.assertEqual(used,{0,5,6,7},n)
 def test_fixed_shell_apertures_and_feet_neutral_endpoints_and_soft_motion(self):
  m=self.m;moved=set();fixed=0
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Shell /','Interior /','Rim /','Spot /','Foot /')):
      self.assertEqual(posed,m.vertices[p]);fixed+=1
     if n.startswith('Limb /'):
      for before,after in zip(m.vertices[p],posed):self.assertAlmostEqual(before[1],after[1],places=7)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertGreater(fixed,0);self.assertTrue({'Limb','Neck','Head','Eye','Mouth'}<=moved,moved)
 def test_winding_and_soft_tissue_edge_strain_through_all_clips(self):
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
 def test_identity_root_rotation_only_and_delayed_head(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  neck=[m.rotations('attack',i/100)[5][0] for i in range(101)];head=[m.rotations('attack',i/100)[7][1] for i in range(101)]
  self.assertGreater(head.index(max(head)),neck.index(max(neck)))
if __name__=='__main__':unittest.main()
