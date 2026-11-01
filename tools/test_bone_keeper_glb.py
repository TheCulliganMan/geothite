#!/usr/bin/env python3
"""Actual skinned bone keepers: intact recessed skulls, supported feet and gripped clubs."""
import copy,math,unittest
from pathlib import Path
import bone_keeper_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BoneKeeperTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,m,prefix,suffix=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (suffix is None or n.endswith(' '+str(suffix))))
 def test_canonical_limits_hierarchy_weights_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_bone_keeper(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertEqual(len(m.joints),14);self.assertEqual(m.parents[:14],[None,0,1,2,1,4,5,1,7,8,0,0,1,12])
   self.assertEqual(author.geometry_digest(author.decode_bone_keeper(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(14)))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if not w))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_skull_features_gripping_digits_and_club_are_coherent(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith(('Foot /','Toe claw /')):owner=0
    elif n.startswith(('Head /','Skull /','Snout /','Orbit /','Eye /','Iris /','Pupil /','Eye glint /','Nostril /','Horn /','Cheek /')):owner=3
    elif n.startswith(('Bone shaft /','Bone lobe /','Thumb /')):owner=6
    elif n.startswith(('Hand /','Finger /','Hand claw /')):owner=6 if n.split()[-2 if n.startswith('Finger /') else -1]=='-1' else 9
    elif n.startswith('Tail spine /'):owner=13
    if owner is not None:self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{owner},(s,n))
 def test_planted_floor_strain_winding_and_cue_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Toe claw /')):
       for a,b in zip(points,m.vertices[i]):self.assertLess(math.dist(a,b),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_all_carriers_act_independently(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for j in range(1,14):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_occupied_anatomical_joins_survive_motion_and_club_is_held(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];pairs=[]
   for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Skull /','Head /'),('Snout /','Skull /'),('Tail /','Body /'),('Bone shaft /','Thumb /'),('Bone shaft /','Hand /')):pairs.append((self.find(m,child),self.find(m,parent,-1 if parent=='Hand /' else None)))
   if s=='marowak':pairs.append((self.find(m,'Tail spine /'),self.find(m,'Tail /')))
   for side in (-1,1):
    for child,parent in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand /','Arm /'),('Haunch /','Body /'),('Foot /','Haunch /'),('Toe claw /','Foot /'),('Horn /','Skull /'),('Cheek /','Skull /')):pairs.append((self.find(m,child,side),self.find(m,parent,side if parent in ('Shoulder /','Arm /','Haunch /','Foot /') else None)))
    for digit in range(3):pairs.append((self.find(m,'Finger /',f'{side} {digit}'),self.find(m,'Hand /',side)))
   for i,n in enumerate(m.names):
    if n.startswith('Bone lobe /'):pairs.append((i,self.find(m,'Bone shaft /')))
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[self.find(m,'Bone shaft /')]);detached['positions']=[v+(2 if i%3==0 else 0) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[self.find(m,'Hand /',-1)]))
 def test_strict_reader_rejects_mutated_geometry(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_bone_keeper(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_bone_keeper(s,changed)
if __name__=='__main__':unittest.main()
