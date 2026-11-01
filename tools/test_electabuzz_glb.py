#!/usr/bin/env python3
"""Decode stored Electabuzz geometry, articulate actual owners and preserve anatomical voids."""
import copy,math,os,unittest
from pathlib import Path
import electabuzz_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,crossed_faces,indexed_ray_intersections
ROOT=Path(os.environ.get('CRYSTAL_ELECTABUZZ_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class ElectabuzzSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'electabuzz.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def find(self,prefix,side=None):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix) and (side is None or n.endswith(' '+str(side))))
 def test_exact_geometry_hierarchy_inverse_binds_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_electabuzz());self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['electabuzz']);self.assertEqual(author.geometry_digest(self.runtime.neutral_model()),author.PACKED_REST_DIGESTS['electabuzz'])
  self.assertEqual((len(m.parts),len(self.runtime.parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),(70,41,19747,33880,19));self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(m.parents[:19],[None,0,1,2,1,4,5,1,7,8,0,0,3,3,3,3,1,16,17])
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_rigid_jaws_gaze_fists_features_and_floor_owners(self):
  m=self.m
  for i,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Toe claw /')) else 3 if n.startswith(('Head /','Eye socket /','Eye /','Pupil /','Forehead /','Jaw /','Mouth back /','Tongue /','Fang /','Nostril /')) else (6 if n.split()[-2 if n.startswith('Knuckle /') else -1]=='-1' else 9) if n.startswith(('Hand /','Knuckle /','Thumb /')) else (12 if n.endswith('-1') else 13) if n.startswith(('Antenna /','Antenna ball /')) else (14 if n.endswith('-1') else 15) if n.startswith('Crest /') else 18 if n.startswith('Tail tip /') else None
   if owner is not None:self.assertEqual({j for ids,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(ids,ws) if w},{owner},n)
 def test_floor_edge_strain_winding_and_neutral_endpoints(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);floor=1e9
    for i,n in enumerate(m.names):
     vs=m.posed(i,mats);ns=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(clip,n))
     if n.startswith(('Foot /','Toe claw /')):
      for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,ns,tri);self.assertGreater(cosine,0.,(clip,n,step));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n,step))
    self.assertLess(abs(floor),1e-7)
    if step in (0,8):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_all_eighteen_anatomical_carriers_move_independently(self):
  m=self.m;rotations=m.rotations('attack',.31)
  for j in range(1,19):
   isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
   self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,j)
 def test_actual_anatomical_roots_and_stripe_wraps_remain_occupied(self):
  m=self.m;ps=m.neutral_model()['primitives'];pairs=[]
  for a,b in (('Neck /','Body /'),('Head /','Neck /'),('Jaw / rooted yellow','Head /'),('Jaw / rooted open','Head /'),('Tongue /','Jaw / rooted open'),('Tail /','Body /'),('Tail tip /','Tail /')):pairs.append((self.find(a),self.find(b)))
  for s in (-1,1):
   for a,b in (('Crest /','Head /'),('Antenna /','Head /'),('Antenna ball /','Antenna /'),('Fang /','Jaw / rooted yellow'),('Arm /','Body /'),('Hand /','Arm /'),('Thumb /','Hand /'),('Leg /','Body /'),('Foot /','Leg /')):pairs.append((self.find(a,s),self.find(b,None if b in ('Body /','Head /','Jaw / rooted yellow') else s)))
   for j in range(3):
    for child,parent in (('Knuckle /','Hand /'),('Toe claw /','Foot /'),('Arm stripe /','Arm /')):pairs.append((self.find(child,f'{s} {j}'),self.find(parent,s)))
   for j in range(2):
    for child,parent in (('Wrist fur /','Arm /'),('Leg stripe /','Leg /')):pairs.append((self.find(child,f'{s} {j}'),self.find(parent,s)))
  for j in range(4):pairs.append((self.find('Tail stripe /',j),self.find('Tail /')))
  for clip in m.clips:
   for t in (0.,.13,.31,.67,.83):
    mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(clip,t,m.names[a],m.names[b]))
 def test_curved_surfaces_do_not_cross(self):
  for p in self.m.neutral_model()['primitives']:
   if p['part'].startswith(('Arm /','Arm stripe /','Leg /','Leg stripe /','Tail /','Tail stripe /','Wrist fur /','Toe claw /','Crest /','Fang /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_open_jaw_space_remains_empty_against_all_posed_solids(self):
  m=self.m;ps=m.neutral_model()['primitives'];upper=self.find('Jaw / rooted yellow');lower=self.find('Jaw / rooted open');y=(min(v[1] for v in m.vertices[upper])+max(v[1] for v in m.vertices[lower]))/2;z=.95*max(v[2] for v in m.vertices[lower])
  for clip in m.clips:
   for t in (0.,.13,.31,.67,.83):
    mats=m.matrices(clip,t);matrix=mats[3]
    def neutral_head(q):
     q=[q[k]-matrix[12+k] for k in range(3)];return [sum(matrix[k*4+j]*q[j] for j in range(3)) for k in range(3)]
    for i,p in enumerate(ps):
     local=dict(p,positions=sum((neutral_head(q) for q in m.posed(i,mats)),[]))
     for dx,dy in ((0.,0.),(.025,0.),(-.025,0.),(0.,-.012)):
      hits=sorted(set(indexed_ray_intersections(local,dx,y+dy)));self.assertFalse(any(a+.0001<z<b-.0001 for a,b in zip(hits[::2],hits[1::2])),(clip,t,m.names[i]))
 def test_strict_reader_rejects_geometry_and_byte_mutations(self):
  changed=bytearray(self.blob);changed[-8]^=1
  with self.assertRaises(ValueError):author.decode_electabuzz(bytes(changed))
  changed=copy.deepcopy(author.authoring_model('electabuzz'));changed['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_electabuzz(changed)
if __name__=='__main__':unittest.main()
