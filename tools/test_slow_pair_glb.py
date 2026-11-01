#!/usr/bin/env python3
"""Stored shell bipeds: coherent held assemblies, rooted anatomy and occupied oral gaps."""
import copy,math,os,unittest
from collections import defaultdict
from chikorita_sculpt import cross,sub,unit,dot
from pathlib import Path
import slow_pair_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,crossed_faces
from test_steelix_sculpt import occupied
import test_slowbro_sculpt as anatomy_tests
ROOT=Path(os.environ.get('CRYSTAL_SLOW_PAIR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class SlowPairSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def find(self,m,prefix,side=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (side is None or n.endswith(' '+str(side))))
 def test_exact_geometry_hierarchy_inverse_binds_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_shell_biped(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(self.runtime[s].neutral_model()),author.PACKED_REST_DIGESTS[s])
   self.assertEqual((len(m.parts),len(self.runtime[s].parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),(72,38,19322,32532,18) if s=='slowbro' else (60,33,18831,31512,18));self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual(m.parents[:18],[None,0,1,2,1,4,5,1,7,8,0,0,1,12]+([13,3,3,14] if s=='slowbro' else [3,2,2,2]))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_rigid_faces_claws_shells_and_floor_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Foot claw /')) else 17 if n.startswith(('Shell jaw /','Shell lower tooth /')) else 14 if n.startswith(('Shell ','Crown ','Jewel')) else (15 if n.endswith('-1') else 16) if n.startswith(('Ear /','Ear inset /','Ear curl /')) else 3 if n.startswith(('Head /','Muzzle /','Jaw /','Mouth back /','Mouth seam /','Tongue /','Eye /','Pupil /','Nostril /','Fang /','Tooth /')) else (6 if n.split()[-2 if n.startswith('Hand claw /') else -1]=='-1' else 9) if n.startswith(('Hand /','Hand claw /')) else None
    if owner is not None:self.assertEqual({j for ids,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(ids,ws) if w},{owner},(s,n))
 def test_floor_strain_winding_and_neutral_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=1e9
     for i,n in enumerate(m.names):
      vs=m.posed(i,mats);ns=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Foot claw /')):
       for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,ns,tri);self.assertGreater(cosine,0.,(s,clip,n,step));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n,step))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_each_of_seventeen_anatomical_carriers_has_independent_motion(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for j in range(1,18):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_actual_roots_remain_occupied_through_source_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];pairs=[]
   def add(a,b,side=None,parent_side=None):pairs.append((self.find(m,a,side),self.find(m,b,parent_side)))
   add('Tail /','Body /');add('Muzzle /','Body /' if s=='slowbro' else 'Head /')
   for side in (-1,1):
    for a,b in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Thigh /','Body /'),('Leg /','Thigh /'),('Foot /','Leg /')):add(a,b,side,None if b=='Body /' else side)
    for j in range(3):add('Hand claw /','Arm /',f'{side} {j}',side)
    for j in range(2):add('Foot claw /','Foot /',f'{side} {j}',side)
    if s=='slowbro':add('Ear /','Body /',side);add('Fang /','Muzzle /',side)
    else:add('Tooth /','Muzzle /',side)
   if s=='slowbro':
    for a,b in (('Jaw /','Body /'),('Tongue /','Jaw /'),('Shell body /','Tail /'),('Shell jaw /','Shell body /'),('Shell rear wall /','Shell body /'),('Shell mouth back /','Shell jaw /')):add(a,b)
    for i,n in enumerate(m.names):
     if n.startswith(('Shell coil /','Shell spine /','Shell upper tooth /')):pairs.append((i,self.find(m,'Shell body /')))
     elif n.startswith('Shell lower tooth /'):pairs.append((i,self.find(m,'Shell jaw /')))
   else:
    for a,b in (('Head /','Body /'),('Crown body /','Head /'),('Crown brim /','Head /'),('Jewel rim /','Crown body /'),('Jewel /','Jewel rim /'),('Jewel glint /','Jewel /')):add(a,b)
    for i,n in enumerate(m.names):
     if n.startswith('Collar panel /'):pairs.append((i,self.find(m,'Body /')))
     elif n.startswith(('Crown horn /','Crown crest /','Crown turn /')):pairs.append((i,self.find(m,'Crown body /')))
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
     for i,n in enumerate(m.names):
      if n.startswith('Arm /'):self.assertIsNone(crossed_faces(posed[i]),(s,clip,t,n))
 def test_both_slowbro_mouths_stay_empty_against_every_posed_solid(self):
  m=self.models['slowbro'];ps=m.neutral_model()['primitives'];probe=anatomy_tests.SlowbroTests()
  for clip in m.clips:
   for t in (0.,.13,.31,.67,.83):
    mats=m.matrices(clip,t)
    for shell in (False,True):
     matrix=mats[14 if shell else 3]
     def local(q):
      q=[q[k]-matrix[12+k] for k in range(3)];return [sum(matrix[k*4+j]*q[j] for j in range(3)) for k in range(3)]
     for i,p in enumerate(ps):
      p=dict(p,positions=sum((local(q) for q in m.posed(i,mats)),[]))
      for q in probe.mouth_points(shell):self.assertFalse(occupied(p,q),(clip,t,shell,m.names[i],q))
 def test_continuous_forelimbs_have_no_large_wrist_closing_faces(self):
  # Find actual planar caps in the stored surface. A large closing disk is
  # allowed only at the buried shoulder, not across the wrist or palm.
  for s,m in self.models.items():
   self.assertFalse(any(n.startswith('Hand /') for n in m.names))
   for side,owner in ((-1,4),(1,7)):
    i=self.find(m,'Arm /',side);groups=defaultdict(list);root=[-v for v in m.inverse[owner][12:15]]
    for k in range(0,len(m.indices[i]),3):
     tri=[m.vertices[i][v] for v in m.indices[i][k:k+3]];a,b,c=tri;normal=unit(cross(sub(b,a),sub(c,a)))
     groups[tuple(round(v,4) for v in normal)+(round(dot(normal,a),4),)].append(tri)
    for triangles in groups.values():
     points={tuple(q) for tri in triangles for q in tri}
     if len(points)<14:continue
     common=set.intersection(*({tuple(q) for q in tri} for tri in triangles))
     if not common:continue
     area=sum(math.sqrt(dot(n,n))*.5 for a,b,c in triangles for n in [cross(sub(b,a),sub(c,a))])
     if area<.001:continue
     a,b,c=triangles[0];normal=unit(cross(sub(b,a),sub(c,a)))
     self.assertLess(abs(dot(normal,sub(root,a))),.00001,(s,side,'large closing plane outside shoulder'))
 def test_strict_reader_rejects_geometry_and_byte_mutations(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_shell_biped(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_shell_biped(s,changed)
if __name__=='__main__':unittest.main()
