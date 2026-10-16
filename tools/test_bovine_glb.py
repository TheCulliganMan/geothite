#!/usr/bin/env python3
"""Stored bovine skins: fixed hooves, fitted udders and three separate tail chains."""
import math,unittest
from pathlib import Path
import bovine_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BovineSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()}
 def test_canonical_bind_and_existing_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_bovine(s,author.decode_bovine(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(42,14025,16504,17) if s=='tauros' else (45,17281,17976,14))
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_normal_source_colors(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    if s=='miltank':
     rgb=(0,0,0) if n.startswith(('Eye / fitted dark','Eye / dark','Mouth / fitted cheerful','Hand / fitted','Foot / fitted','Ear / pale')) else (49,49,107) if n.startswith(('Head /','Nose /','Spot /','Hand / dark','Foot / grounded','Eye / fitted blue','Ear / broad','Tail / rounded')) else (255,255,255) if n.startswith(('Udder /','Horn /','Eye / fitted ivory','Eye / white')) else (222,123,148)
    else:
     rgb=(0,0,0) if n.startswith(('Eye / fitted dark','Eye / dark','Nostril /','Rivet / dark','Hoof groove /','Mane crease /')) else (255,255,255) if n.startswith(('Horn /','Rivet / gray','Eye / fitted ivory')) else (82,57,33) if n.startswith(('Mane /','Nose /','Hoof /','Tail / pointed')) else (206,123,41)
    color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for v,a in zip(rgb,color):
     x=v/255;b=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,b,places=7)
 def test_tauros_body_reference_excludes_raised_tufts(self):
  m=self.models['tauros']
  body=max(v[1] for p,n in enumerate(m.names) if not n.startswith('Tail /') for v in m.vertices[p])
  full=max(v[1] for points in m.vertices for v in points)
  self.assertAlmostEqual(m.doc['extras']['body_reference_height'],body,places=6)
  self.assertLess(body,full);self.assertAlmostEqual(m.doc['extras']['full_silhouette_height'],full,places=6)
 def test_fitted_face_udder_markings_mane_and_cloven_grooves_share_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    if s=='miltank':
     owner=0 if n.startswith('Foot /') else 1 if n.startswith(('Body /','Udder /','Teat /','Spot /')) else 2 if n.startswith(('Head /','Face /','Eye /','Muzzle /','Nose /','Mouth /','Horn /')) else (7 if n.endswith('-1') else 8) if n.startswith('Ear /') else (4 if n.endswith('-1') else 6) if n.startswith('Hand /') else 11 if n.startswith('Tail / rounded') else None
    else:
     owner=0 if n.startswith(('Hoof /','Hoof groove /')) else 1 if n.startswith('Body /') else 2 if n.startswith(('Mane /','Mane crease /')) else 3 if n.startswith(('Head /','Eye /','Nose /','Nostril /','Rivet /','Horn /')) else 10+3*int(n.split()[-1]) if n.startswith('Tail / pointed') else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_three_tail_tufts_have_separate_articulated_chains(self):
  m=self.models['tauros'];owners=[]
  for p,n in enumerate(m.names):
   if n.startswith('Tail / pointed'):owners.append(next(ids[0] for ids in m.ids[p]));self.assertTrue(all(ids==[owners[-1],0,0,0] for ids in m.ids[p]))
  self.assertEqual(owners,[10,13,16])
  rotations=m.rotations('attack',.31);self.assertEqual(len({tuple(rotations[j-1]) for j in owners}),3)
 def test_floor_fixed_hooves_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set();fixed=0
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      if n.startswith(('Foot /','Hoof /','Hoof groove /')):self.assertEqual(posed,m.vertices[p]);fixed+=1
      if n.startswith('Leg /'):
       for a,b in zip(m.vertices[p],posed):self.assertAlmostEqual(a[1],b[1],places=7)
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertGreater(fixed,0);self.assertTrue({'Body','Head','Horn','Leg','Tail'}<=moved,(s,moved))
   self.assertTrue(({'Udder','Teat','Arm','Hand','Ear','Spot'} if s=='miltank' else {'Mane','Mane crease','Rivet'})<=moved,(s,moved))
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
 def test_identity_root_rotation_only_delayed_tail(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
   arm=[m.rotations('attack',i/100)[1][1] for i in range(101)];paw=[m.rotations('attack',i/100)[10 if s=='miltank' else 9][0] for i in range(101)]
   self.assertGreater(paw.index(max(paw)),arm.index(max(arm)))
if __name__=='__main__':unittest.main()
