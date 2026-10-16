#!/usr/bin/env python3
"""Stored bulldog skins: fixed clefts, coherent spots, folded ears and lower tusks."""
import math,unittest
from pathlib import Path
import bulldog_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BulldogSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()}
 def test_canonical_bind_and_existing_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_bulldog(s,author.decode_bulldog(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(42,14140,19048,13) if s=='granbull' else (39,16555,19516,13))
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_normal_source_colors(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    rgb=(123,33,140) if n.startswith('Mouth / fitted rosy') else (0,0,0) if n.startswith(('Nose /','Pupil /','Eye / fitted','Ear marking /','Mouth /','Wrist /','Hand fold /','Toe groove /','Collar / fitted')) else (57,82,107) if n.startswith(('Spot /','Collar / continuous')) else (255,255,255) if n.startswith(('Fang /','Tusk /','Eye /')) or (s=='snubbull' and n.startswith('Foot /')) else ((189,99,181) if s=='granbull' else (231,115,107))
    color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for v,a in zip(rgb,color):
     x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_fitted_face_spots_clefts_and_folded_ears_share_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Toe groove /')) else 1 if n.startswith('Spot /') else 2 if n.startswith(('Head /','Jowl /','Eye /','Pupil /','Muzzle /','Nose /','Mouth /','Fang /')) else 3 if n.startswith(('Jaw /','Tusk /')) else 8 if n.startswith('Tail /') else (9 if n.endswith('-1') else 10) if n.startswith(('Ear /','Ear marking /')) else None
    if n.startswith(('Hand /','Hand fold /','Wrist /')):
     side=int(n.split()[-2] if n.startswith('Hand fold /') else n.split()[-1]);owner=5 if side<0 else 7
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_jaw_and_lower_tusks_move_as_one_unit(self):
  m=self.models['granbull'];jaw=next(p for p,n in enumerate(m.names) if n.startswith('Jaw /'))
  self.assertTrue(all(ids==[3,0,0,0] for ids in m.ids[jaw]))
  for p,n in enumerate(m.names):
   if n.startswith('Tusk /'):self.assertTrue(all(ids==[3,0,0,0] for ids in m.ids[p]))
  for clip in m.clips:
   self.assertGreater(max(math.dist(a,b) for p,n in enumerate(m.names) if n.startswith('Tusk /') for a,b in zip(m.vertices[p],m.posed(p,m.matrices(clip,.31)))),.0005)
 def test_floor_fixed_paws_neutral_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set();fixed=0
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      if n.startswith(('Foot /','Toe groove /')):self.assertEqual(posed,m.vertices[p]);fixed+=1
      if n.startswith('Leg /'):
       for a,b in zip(m.vertices[p],posed):self.assertAlmostEqual(a[1],b[1],places=7)
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertGreater(fixed,0);self.assertTrue({'Body','Head','Ear','Arm','Hand','Jaw','Tail'}<=moved,(s,moved))
 def test_winding_and_edge_strain_through_all_clips(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,.18,.31,.55,.80,1.):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,n,ratio))
 def test_identity_root_rotation_only_delayed_fists(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
   arm=[m.rotations('attack',i/100)[6][0] for i in range(101)];paw=[m.rotations('attack',i/100)[7][0] for i in range(101)]
   self.assertGreater(paw.index(max(paw)),arm.index(max(arm)))
if __name__=='__main__':unittest.main()
