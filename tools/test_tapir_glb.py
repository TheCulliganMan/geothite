#!/usr/bin/env python3
"""Stored tapir skins: supported feet, continuous seams and hand-held pendulum."""
import copy,math,unittest
from pathlib import Path
import tapir_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class TapirSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,m,prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
 def test_canonical_limits_hierarchy_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_tapir(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());count=14 if s=='hypno' else 13;self.assertEqual(len(m.joints),count)
   self.assertEqual(m.parents[:count],[None,0,1,1,3,4,1,6,7,0,0,2,2]+([8] if s=='hypno' else []))
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(count)))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_feature_and_digit_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    expected=None
    if n.startswith('Foot /'):expected=0
    elif n.startswith('Hand /'):expected=5 if n.split()[-2 if n.startswith('Hand / curved finger') else -1]=='-1' else 8
    elif n.startswith('Ear /'):expected=11 if n.endswith('-1') else 12
    elif n.startswith(('Nose /','Eye /','Eye border /','Cheek /')):expected=2
    elif n.startswith('Pendulum /'):expected=13
    elif n.startswith('Ruff /'):expected=1
    if expected is not None:
     self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{expected},(s,n))
 def test_fixed_floor_bounded_strain_winding_and_source_clock_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith('Foot /'):
       for a,b in zip(points,m.vertices[i]):self.assertLess(math.dist(a,b),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_every_carrier_has_independent_articulation(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for joint in range(1,len(m.joints)):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,joint))
 def test_wavy_material_seam_stays_welded_through_all_clips(self):
  m=self.models['drowzee'];a,b=0,1;lookup={tuple(p):i for i,p in enumerate(m.vertices[b])};pairs=[(i,lookup[tuple(p)]) for i,p in enumerate(m.vertices[a]) if tuple(p) in lookup];self.assertEqual(len(pairs),64)
  for clip in m.clips:
   for t in (.13,.31,.58,.83):
    mats=m.matrices(clip,t)
    for i,j in pairs:self.assertLess(math.dist(m.point(a,i,mats),m.point(b,j,mats)),1e-8)
 def test_held_pendulum_remains_connected_and_moves_relative_to_hand(self):
  m=self.models['hypno'];ps=m.neutral_model()['primitives'];hand=self.find(m,'Hand / broad palm 1');cord=self.find(m,'Pendulum / hanging');ring=self.find(m,'Pendulum / open');lip=self.find(m,'Pendulum / dark')
  for clip in m.clips:
   for t in (0.,.13,.31,.58,.83):
    mats=m.matrices(clip,t);posed={i:dict(ps[i],positions=sum(m.posed(i,mats),[])) for i in (hand,cord,ring,lip)}
    for a,b in ((hand,cord),(cord,ring),(ring,lip)):self.assertTrue(parts_overlap(posed[a],posed[b]),(clip,t,m.names[a],m.names[b]))
  detached=copy.deepcopy(ps[cord]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[hand]))
  mats=m.matrices('attack',.31);self.assertGreater(max(math.dist(q,skin.transform(mats[8],p)) for p,q in zip(m.vertices[ring],m.posed(ring,mats))),.015)
 def test_strict_reader_rejects_mutation(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_tapir(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_tapir(s,changed)
if __name__=='__main__':unittest.main()
