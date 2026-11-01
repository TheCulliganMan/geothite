#!/usr/bin/env python3
"""Stored mime articulation, ten complete digits and actual rooted anatomy."""
import copy,math,os,unittest
from pathlib import Path
import mr_mime_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,crossed_faces,indexed_ray_intersections as rays
ROOT=Path(os.environ.get('CRYSTAL_MR_MIME_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class MimeSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'mr__mime.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def find(self,prefix,side=None):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix) and (side is None or n.endswith(' '+str(side))))
 def test_upper_smile_refinement_preserves_non_facial_solids(self):
  protected=[p for p in self.m.neutral_model()['primitives'] if not p['part'].startswith(('Head /','Mouth lining /','Cheek /'))]
  self.assertEqual(author.geometry_digest({'primitives':protected}),'313ac706496b75c1ea6d6e40938d875e802558cbee8d988ac8db64858721b2bd')
 def test_exact_geometry_hierarchy_inverse_binds_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_mr_mime());self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['mr__mime']);self.assertEqual(author.geometry_digest(self.runtime.neutral_model()),author.PACKED_REST_DIGESTS['mr__mime'])
  self.assertEqual((len(m.parts),len(self.runtime.parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),(67,34,19916,33656,26));self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(m.parents[:26],[None,0,1,2,1,4,5,1,7,8,1,1,10,11,3,3]+[6]*5+[9]*5)
  self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(26)))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_rigid_recess_gaze_glove_digits_and_floor_owners(self):
  m=self.m
  for i,n in enumerate(m.names):
   owner=0 if n.startswith(('Shoe /','Shoe curl /')) else 3 if n.startswith(('Head /','Upper lip /','Lower lip /','Mouth back /','Tongue /','Mouth lining /','Eye /','Pupil /','Eye glint /','Cheek /')) else (16 if n.split()[-2]=='-1' else 21)+int(n.split()[-1]) if n.startswith(('Finger /','Fingertip /','Finger pad /')) else (6 if n.endswith('-1') else 9) if n.startswith('Palm /') else (14 if n.endswith('-1') else 15) if n.startswith('Ear fan /') else None
   if owner is not None:self.assertEqual({j for ids,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(ids,ws) if w},{owner},n)
 def test_floor_strain_winding_and_neutral_endpoints(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);floor=1e9
    for i,n in enumerate(m.names):
     vs=m.posed(i,mats);ns=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(clip,n))
     if n.startswith(('Shoe /','Shoe curl /')):
      for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,ns,tri);self.assertGreater(cosine,0.,(clip,n,step));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n,step))
    self.assertLess(abs(floor),1e-7)
    if step in (0,8):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_each_of_twenty_five_carriers_has_independent_motion(self):
  m=self.m;rotations=m.rotations('attack',.31)
  for j in range(1,26):
   isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
   self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,j)
 def test_actual_roots_and_uncrossed_surfaces_survive_source_clips(self):
  m=self.m;ps=m.neutral_model()['primitives'];pairs=[]
  def add(a,b,side=None,parent_side=None):pairs.append((self.find(a,side),self.find(b,parent_side)))
  for a,b in (('Neck /','Body /'),('Head /','Neck /'),('Upper lip /','Head /'),('Lower lip /','Head /'),('Mouth back /','Head /'),('Tongue /','Lower lip /')):add(a,b)
  for side in (-1,1):
   for a,b in (('Ear fan /','Head /'),('Shoulder /','Body /'),('Arm /','Shoulder /'),('Palm /','Arm /'),('Hip /','Body /'),('Thigh /','Hip /'),('Knee /','Thigh /'),('Shin /','Knee /'),('Shoe /','Shin /'),('Shoe curl /','Shoe /')):add(a,b,side,None if b in ('Body /','Head /') else side)
   for i in range(5):add('Finger /','Palm /',f'{side} {i}',side);add('Fingertip /','Finger /',f'{side} {i}',f'{side} {i}')
  for clip in m.clips:
   for t in (0.,.13,.31,.67,.83):
    mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(clip,t,m.names[a],m.names[b]))
    for i,n in enumerate(m.names):
     if n.startswith(('Arm /','Finger /','Thigh /','Shin /','Ear fan /','Shoe curl /')):self.assertIsNone(crossed_faces(posed[i]),(clip,t,n))
 def test_fitted_caps_follow_actual_posed_parent(self):
  m=self.m;ps=m.neutral_model()['primitives'];pairs=[]
  for i,n in enumerate(m.names):
   for child,parent in (('Eye /','Head /'),('Pupil /','Eye /'),('Eye glint /','Pupil /'),('Cheek /','Head /'),('Mouth lining /','Head /'),('Chest /','Body /'),('Finger pad /','Fingertip /')):
    if n.startswith(child):pairs.append((i,self.find(parent,' '.join(n.split()[-2:]) if parent=='Fingertip /' else n.split()[-1] if parent in ('Eye /','Pupil /') else None)))
  for clip in m.clips:
   for t in (0.,.31,.67):
    mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for child,parent in pairs:
     points=m.posed(child,mats)
     for k in range(0,len(m.indices[child]),3):
      ids=m.indices[child][k:k+3];a,b,c=[m.vertices[child][i] for i in ids]
      # Authored exposed caps, excluding intentionally buried backs/walls.
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
      tri=[points[i] for i in ids]
      for ws in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
       x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(posed[parent],x,y);self.assertTrue(hits,(clip,t,m.names[child]));gap=z-max(hits);self.assertGreater(gap,.0001,(clip,t,m.names[child],gap));self.assertLess(gap,.02,(clip,t,m.names[child],gap))
 def test_smile_lining_has_no_raised_chords_and_aperture_stays_empty(self):
  from test_mr_mime_sculpt import MimeTests
  from test_steelix_sculpt import occupied
  m=self.m;child=self.find('Mouth lining /');parent=self.find('Head /');ps=m.neutral_model()['primitives'];points=MimeTests().mouth_points()
  for k in range(0,len(m.indices[child]),3):
   tri=[m.vertices[child][i] for i in m.indices[child][k:k+3]];a,b,c=tri
   if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
   for ws in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
    x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];self.assertLess(z-max(rays(ps[parent],x,y)),author.source.HEIGHT*.001)
  for clip in m.clips:
   for t in (0.,.31,.67):
    mats=m.matrices(clip,t);head=mats[3]
    # Express every posed solid in the moving head frame, so the original
    # empty smile probes move with the physical oral recess.
    def undo(p):
     d=[p[i]-head[12+i] for i in range(3)];return [sum(head[4*i+k]*d[k] for k in range(3)) for i in range(3)]
    posed=[dict(p,positions=sum([undo(q) for q in m.posed(i,mats)],[])) for i,p in enumerate(ps)]
    for point in points:self.assertFalse(any(occupied(p,point) for p in posed),(clip,t,point))
 def test_strict_reader_rejects_geometry_and_byte_mutations(self):
  changed=bytearray(self.blob);changed[-8]^=1
  with self.assertRaises(ValueError):author.decode_mr_mime(bytes(changed))
  changed=copy.deepcopy(author.authoring_model());changed['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_mr_mime(changed)
if __name__=='__main__':unittest.main()
