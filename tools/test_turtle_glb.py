#!/usr/bin/env python3
"""Stored turtle shells, supported claws, fitted folds and independently carried cannon bores."""
import copy,math,os,unittest
from pathlib import Path
import turtle_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,crossed_faces,indexed_ray_intersections as rays
from chikorita_sculpt import cross,sub,unit,dot
from collections import defaultdict
ROOT=Path(os.environ.get('CRYSTAL_TURTLE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class TurtleSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:skin.Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def find(self,m,prefix,suffix=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (suffix is None or n.endswith(' '+str(suffix))))
 def test_lossless_geometry_limits_hierarchy_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_turtle(s));self.assertEqual(len(m.parts),66 if s=='wartortle' else 71);self.assertEqual(len(m.joints),15);self.assertEqual(m.parents[:15],[None,0,1,1,3,4,1,6,7,0,0,1 if s=='blastoise' else 2,1 if s=='blastoise' else 2,1,13])
   self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(self.runtime[s].neutral_model()),author.PACKED_REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_turtle(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(15)))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_rigid_shell_scutes_skull_claws_and_cannon_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith(('Foot /','Toe /','Foot claw /')):owner=0
    elif n.startswith(('Body /','Shell /','Shell rim /','Shell seam /','Belly /','Belly seam /','Cannon mount /')):owner=1
    elif n.startswith(('Head /','Muzzle /','Jaw /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Mouth /','Nostril /','Fang /')) or s=='blastoise' and n.startswith(('Ear /','Ear inset /')):owner=2
    elif n.startswith('Hand claw /'):owner=5 if n.split()[-2]=='-1' else 8
    elif n.startswith(('Cannon /','Cannon floor /','Cannon lining /')):owner=11 if n.endswith('-1') else 12
    if owner is not None:self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{owner},(s,n))
 def test_floor_strain_winding_and_source_cue_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Toe /','Foot claw /')):
       for a,b in zip(points,m.vertices[i]):self.assertLess(math.dist(a,b),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_every_carrier_articulates_independently(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for j in range(1,15):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_real_anatomical_roots_stay_occupied_through_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];pairs=[]
   for child,parent in (('Shell /','Body /'),('Belly /','Body /'),('Shell rim /','Shell /'),('Head /','Body /'),('Muzzle /','Head /'),('Tail /','Shell /')):pairs.append((self.find(m,child),self.find(m,parent)))
   for side in (-1,1):
    for child,parent in (('Arm /','Body /'),('Haunch /','Body /'),('Leg /','Haunch /'),('Foot /','Leg /')):pairs.append((self.find(m,child,side),self.find(m,parent,None if parent=='Body /' else side)))
    if s=='wartortle':
     for child,parent in (('Ear /','Head /'),('Ear lobe /','Ear /'),('Fang /','Muzzle /')):pairs.append((self.find(m,child,side),self.find(m,parent,None if parent in ('Head /','Muzzle /') else side)))
    else:
     for child,parent in (('Cannon mount /','Shell /'),('Cannon /','Cannon mount /'),('Cannon lining /','Cannon /'),('Ear /','Head /')):pairs.append((self.find(m,child,side),self.find(m,parent,None if parent in ('Shell /','Head /') else side)))
    for digit in range(3):
     pairs.append((self.find(m,'Hand claw /',f'{side} {digit}'),self.find(m,'Arm /',side)));pairs.append((self.find(m,'Toe /',f'{side} {digit}'),self.find(m,'Foot /',side)))
     if s=='blastoise':pairs.append((self.find(m,'Foot claw /',f'{side} {digit}'),self.find(m,'Toe /',f'{side} {digit}')))
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
 def test_limb_ear_tail_and_cannon_surfaces_do_not_cross_themselves(self):
  for s,m in self.models.items():
   for p in m.neutral_model()['primitives']:
    if p['part'].startswith(('Arm /','Leg /','Ear /','Ear lobe /','Tail /','Cannon /','Cannon lining /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_continuous_forelimbs_have_no_large_wrist_closing_faces(self):
  # Find actual planar caps in the stored surface. A large closing disk is
  # allowed only at the buried shoulder, not across the wrist or palm.
  for s,m in self.models.items():
   self.assertFalse(any(n.startswith('Hand /') for n in m.names))
   for side,owner in ((-1,3),(1,6)):
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
 def test_hollow_cannon_assemblies_remain_rigid_and_empty(self):
  # Pull all posed geometry into each stored barrel's local axis frame, then
  # check its actual open centerline against every solid, not a painted disk.
  from chikorita_sculpt import unit,sub,dot,cross
  m=self.models['blastoise'];ps=m.neutral_model()['primitives']
  for side in (-1,1):
   barrel=self.find(m,'Cannon /',side);axis=unit((side*.075,.280,.383));right=unit(cross(axis,(0,1,0)));up=cross(axis,right);front=max(dot(q,axis) for q in m.vertices[barrel]);ring={tuple(q) for q in m.vertices[barrel] if front-dot(q,axis)<1e-6};tip=[sum(q[k] for q in ring)/len(ring) for k in range(3)];owner=11 if side<0 else 12
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);matrix=mats[owner]
     def local(q):
      q=[q[k]-matrix[12+k] for k in range(3)];return [sum(matrix[k*4+j]*q[j] for j in range(3)) for k in range(3)]
     for i,p in enumerate(ps):
      points=[sub(local(q),tip) for q in m.posed(i,mats)];posed=dict(p,positions=[v for q in points for v in (dot(q,right),dot(q,up),dot(q,axis))]);hits=sorted(set(rays(posed,0.,0.)))
      for a,b in zip(hits[::2],hits[1::2]):self.assertLessEqual(min(b,-.006)-max(a,-.125),.0001,(clip,t,side,m.names[i]))
      if p['part'].startswith(('Cannon /','Cannon floor /','Cannon lining /')) and p['part'].endswith(' '+str(side)):
       for a,b in zip((local(q) for q in m.posed(i,mats)),m.vertices[i]):self.assertLess(math.dist(a,b),1e-7)
 def test_strict_reader_rejects_changed_geometry_and_bytes(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_turtle(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_turtle(s,changed)
if __name__=='__main__':unittest.main()
