#!/usr/bin/env python3
"""Decoded water-rabbit skins: welded bellies, fitted ears and attached buoy tails."""
import math,unittest
from pathlib import Path
import water_rabbit_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
import test_aqua_rabbit_sculpt as anatomy
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class WaterRabbitSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()}
 def test_canonical_anatomy_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_water_rabbit(s,author.decode_water_rabbit(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(19,3437,5648,12) if s=='marill' else (25,4873,7448,12))
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palettes(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    blue=(74,107,239) if s=='marill' else (82,123,198);warm=(206,66,41) if s=='marill' else (181,57,33)
    rgb=(255,255,255) if n.startswith(('Body / welded pale','Eye / small white','Belly /')) else warm if n.startswith(('Ear / fitted red','Mouth / quiet')) else (0,0,0) if n.startswith(('Eye / fitted','Mouth / fitted','Nose /','Tail / continuous')) else blue
    color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for v,a in zip(rgb,color):
     x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_face_buoy_and_foot_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=0 if n.startswith('Foot /') else 2 if n.startswith(('Eye /','Mouth /','Nose /')) else 11 if n.startswith('Tail / blue') else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_welded_body_seam_survives_all_clips(self):
  for s,m in self.models.items():
   a={tuple(v):i for i,v in enumerate(m.vertices[0])};b={tuple(v):i for i,v in enumerate(m.vertices[1])};shared=a.keys()&b.keys();self.assertEqual(len(shared),40)
   for clip in m.clips:
    for t in (0.,.18,.31,.55,.80,1.):
     mats=m.matrices(clip,t);ap=m.posed(0,mats);bp=m.posed(1,mats)
     for key in shared:self.assertEqual(ap[a[key]],bp[b[key]])
 def test_floor_fixed_paddles_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      if n.startswith('Foot /'):self.assertEqual(posed,m.vertices[p])
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Body','Eye','Mouth','Ear','Arm','Tail'}<=moved,(s,moved))
 def test_winding_and_edge_strain(self):
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
 def test_posed_ear_linings_belly_spots_and_buoy_remain_fitted(self):
  for s,m in self.models.items():
   ball=next(p for p,n in enumerate(m.names) if n.startswith('Tail / blue'));cord=next(p for p,n in enumerate(m.names) if n.startswith('Tail / continuous'))
   for clip,t in [('idle',.31),('attack',.31),('hit',.55)]:
    mats=m.matrices(clip,t);posed=[m.posed(p,mats) for p in range(len(m.names))]
    center=[(min(v[k] for v in m.vertices[ball])+max(v[k] for v in m.vertices[ball]))/2 for k in range(3)]
    # The shared rigid ball owner transforms its original center consistently.
    from skin_glb_test_support import transform
    center=transform(mats[11],center);self.assertLess(min(math.dist(v,center) for v in posed[cord]),.014)
    def tris(parts):return [[posed[p][i] for i in m.indices[p][k:k+3]] for p in parts for k in range(0,len(m.indices[p]),3)]
    body=tris([0,1])
    for p,n in enumerate(m.names):
     if n.startswith('Belly /'):support=body
     elif n.startswith('Ear / fitted'):
      side=n.split()[-1];support=tris([q for q,name in enumerate(m.names) if name.startswith(('Ear / round','Ear / bent')) and name.endswith(' '+side)])
     else:continue
     for x,y,z in {tuple(v) for v in posed[p]}:
      hits=anatomy.hits(support,x,y);self.assertTrue(hits,(s,clip,n));self.assertLess(abs(z-max(hits)),.018,(s,clip,n))
 def test_identity_root_rotation_only_delayed_ear_and_ball(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
   base=[m.rotations('attack',i/100)[3][0] for i in range(101)];tip=[m.rotations('attack',i/100)[4][0] for i in range(101)]
   self.assertGreater(tip.index(max(tip)),base.index(max(base)))
if __name__=='__main__':unittest.main()
