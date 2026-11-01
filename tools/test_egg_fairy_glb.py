#!/usr/bin/env python3
"""Egg fairy skins retain hollow rings, rooted limbs and fixed floor contacts."""
import math,unittest
from pathlib import Path
import egg_fairy_glb as author
import test_gengar_glb as skin
import test_togepi_sculpt as joins
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class EggFairySkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def test_canonical_full_geometry_bind_and_budget(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_egg_fairy(s,author.decode_egg_fairy(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'togepi':(27,17825,28896,7),'togetic':(37,16755,26728,10)}[s])
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_fitted_details_and_wing_seams(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Foot point /','Foot crease /','Toe crease /')) else author.BANKS[s]['head'] if n.startswith(('Head /','Eye /','Eye glint /','Eye crease /','Brow /','Mouth /','Tongue /')) else (4 if n.split()[-2]=='-1' else 5) if n.startswith(('Wing feather /','Feather crease /')) else 1 if s=='togepi' and n.startswith(('Shell /','Shell ring /','Shell fleck /')) else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_floor_fixed_feet_neutral_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Foot point /','Foot crease /','Toe crease /')):self.assertEqual(posed,m.vertices[p])
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Head','Arm','Eye','Shell' if s=='togepi' else 'Wing feather'}<=moved,(s,moved))
 def test_winding_and_edge_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.31,.55,.80):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,n,ratio))
 def test_actual_posed_head_wing_arm_and_leg_contacts(self):
  for s,m in self.models.items():
   pairs=[]
   def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
   body=find('Shell /' if s=='togepi' else 'Body /');pairs.append((find('Head /'),body))
   for p,n in enumerate(m.names):
    if n.startswith(('Arm /','Leg /','Wing feather /')):pairs.append((p,body))
    if n.startswith('Leg /'):pairs.append((p,find(('Foot / grounded rounded cream pad ' if s=='togepi' else 'Foot / rounded grounded ivory leaf pad ')+n.split()[-1])))
    if n.startswith('Foot point /'):pairs.append((p,find('Foot / rounded grounded ivory leaf pad '+n.split()[-1])))
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t)
     for a,b in pairs:
      def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
      self.assertTrue(joins.TogepiTests.overlap(self,part(a),part(b)),(s,clip,t,m.names[a],m.names[b]))
 def test_noncanonical_data_rejected(self):
  for s,blob in self.blobs.items():
   with self.assertRaises(ValueError):author.decode_egg_fairy(blob[:-1],s)
if __name__=='__main__':unittest.main()
