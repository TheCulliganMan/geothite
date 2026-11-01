#!/usr/bin/env python3
"""Stored tongue-bearer articulation, fitted groove clearance and real anatomical roots."""
import copy,math,os,unittest
from pathlib import Path
from collections import defaultdict
import lickitung_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,crossed_faces,indexed_ray_intersections as rays
from chikorita_sculpt import cross,sub,unit,dot
ROOT=Path(os.environ.get('CRYSTAL_LICKITUNG_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class LickitungSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'lickitung.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def find(self,prefix,side=None):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix) and (side is None or n.endswith(' '+str(side))))
 def test_exact_geometry_hierarchy_inverse_binds_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_lickitung());self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['lickitung']);self.assertEqual(author.geometry_digest(self.runtime.neutral_model()),author.PACKED_REST_DIGESTS['lickitung'])
  self.assertEqual((len(m.parts),len(self.runtime.parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),(33,26,17028,29256,18));self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(m.parents[:18],[None,0,1,2,1,4,5,1,7,8,0,0,1,12,13,3,15,16])
  self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(18)))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_rigid_recess_gaze_nails_and_floor_owners(self):
  m=self.m
  for i,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Toe /')) else 3 if n.startswith(('Head /','Mouth lining /','Eye /','Eye glint /','Nostril /')) else (6 if n.endswith('-1') else 9) if n.startswith('Hand tip /') else None
   if owner is not None:self.assertEqual({j for ids,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(ids,ws) if w},{owner},n)
 def test_floor_strain_winding_and_neutral_endpoints(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);floor=1e9
    for i,n in enumerate(m.names):
     vs=m.posed(i,mats);ns=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(clip,n))
     if n.startswith(('Foot /','Toe /')):
      for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,ns,tri);self.assertGreater(cosine,0.,(clip,n,step));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n,step))
    self.assertLess(abs(floor),1e-7)
    if step in (0,8):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_each_of_seventeen_carriers_has_independent_motion(self):
  m=self.m;rotations=m.rotations('attack',.31)
  for j in range(1,18):
   isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
   self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,j)
 def test_actual_roots_and_uncrossed_surfaces_survive_source_clips(self):
  m=self.m;ps=m.neutral_model()['primitives'];pairs=[]
  def add(a,b,side=None,parent_side=None):pairs.append((self.find(a,side),self.find(b,parent_side)))
  for a,b in (('Neck /','Body /'),('Head /','Neck /'),('Tongue /','Head /'),('Tail /','Body /')):add(a,b)
  for side in (-1,1):
   for a,b in (('Shoulder /','Body /'),('Arm /','Shoulder /'),('Hand tip /','Arm /'),('Thigh /','Body /'),('Foot /','Thigh /'),('Toe /','Foot /')):add(a,b,side,None if b=='Body /' else side)
  for clip in m.clips:
   for t in (0.,.13,.31,.67,.83):
    mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(clip,t,m.names[a],m.names[b]))
    for i,n in enumerate(m.names):
     if n.startswith(('Arm /','Tongue /','Tail /','Head /')):self.assertIsNone(crossed_faces(posed[i]),(clip,t,n))
 def test_flexible_groove_and_haunch_rings_stay_above_actual_posed_parent(self):
  m=self.m;ps=m.neutral_model()['primitives'];pairs=[]
  for i,n in enumerate(m.names):
   for child,parent in (('Tongue groove /','Tongue /'),('Belly band /','Body /'),('Thigh ring /','Thigh /'),('Thigh center /','Thigh ring /')):
    if n.startswith(child):pairs.append((i,self.find(parent,n.split()[-1] if parent in ('Thigh /','Thigh ring /') else None)))
  for clip in m.clips:
   for t in (0.,.31,.67):
    mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for child,parent in pairs:
     points=m.posed(child,mats)
     for k in range(0,len(m.indices[child]),3):
      ids=m.indices[child][k:k+3];a,b,c=[m.vertices[child][i] for i in ids]
      # The authored front cap must clear its parent. Closed markings also
      # have intentionally buried backs and walls; a small pose rotation can
      # turn a wall toward +Z without making it an exposed cap.
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
      tri=[points[i] for i in ids]
      for ws in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
       x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(posed[parent],x,y);self.assertTrue(hits,(clip,t,m.names[child]));gap=z-max(hits);self.assertGreater(gap,.0001,(clip,t,m.names[child],gap));self.assertLess(gap,.02,(clip,t,m.names[child],gap))
 def test_continuous_palms_have_no_large_wrist_closing_faces(self):
  m=self.m;self.assertFalse(any(n.startswith('Hand /') for n in m.names))
  for side,owner in ((-1,4),(1,7)):
   i=self.find('Arm /',side);groups=defaultdict(list);root=[-v for v in m.inverse[owner][12:15]]
   for k in range(0,len(m.indices[i]),3):
    tri=[m.vertices[i][v] for v in m.indices[i][k:k+3]];a,b,c=tri;normal=unit(cross(sub(b,a),sub(c,a)));groups[tuple(round(v,4) for v in normal)+(round(dot(normal,a),4),)].append(tri)
   for triangles in groups.values():
    points={tuple(q) for tri in triangles for q in tri}
    if len(points)<14:continue
    common=set.intersection(*({tuple(q) for q in tri} for tri in triangles))
    if not common:continue
    area=sum(math.sqrt(dot(n,n))*.5 for a,b,c in triangles for n in [cross(sub(b,a),sub(c,a))])
    if area<.001:continue
    a,b,c=triangles[0];normal=unit(cross(sub(b,a),sub(c,a)));self.assertLess(abs(dot(normal,sub(root,a))),.00001,(side,'large closing plane outside shoulder'))
 def test_strict_reader_rejects_geometry_and_byte_mutations(self):
  changed=bytearray(self.blob);changed[-8]^=1
  with self.assertRaises(ValueError):author.decode_lickitung(bytes(changed))
  changed=copy.deepcopy(author.authoring_model());changed['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_lickitung(changed)
if __name__=='__main__':unittest.main()
