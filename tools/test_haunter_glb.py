#!/usr/bin/env python3
"""Stored ghost face, independent shroud hinges and detached six-claw hand checks."""
import copy,math,unittest
from pathlib import Path
import haunter_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'haunter.glb'
class HaunterTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def test_canonical_schema_limits_and_recognizable_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_haunter());self.assertEqual(len(m.parts),23);self.assertEqual(len(m.joints),16)
  self.assertFalse((ROOT/'haunter.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.10,places=6)
  for prefix,count in [('Body /',1),('Eye /',2),('Pupil /',2),('Upper tooth /',4),('Lower tooth /',4),('Hand /',2),('Finger /',6)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'haunter':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'haunter':self.m.neutral_model()})
 def test_eye_caps_have_well_conditioned_convex_faces(self):
  for p in self.parts:
   if not p['part'].startswith('Eye /'):continue
   vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
   for k in range(0,len(p['indices']),3):
    a,b,c=[vs[j] for j in p['indices'][k:k+3]];u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)]
    n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
    if n[2]<=1e-14:continue
    q=[sum(w[j] for w in (a,b,c))/3/author.SCALE for j in range(3)];q[1]+=author.LOW
    normal=[q[0]/.34**2,(q[1]-.55)/.30**2,(q[2]+.025)/.275**2]
    cosine=sum(x*y for x,y in zip(n,normal))/math.sqrt(sum(x*x for x in n)*sum(x*x for x in normal))
    self.assertGreater(cosine,.90,(p['part'],k//3,'eye cap folds away from its convex surface'))
 def test_fitted_face_clears_actual_stored_parents(self):
  for mark in self.parts:
   n=mark['part']
   if n.startswith(('Eye /','Mouth rim /','Upper tooth /','Lower tooth /')):parent=self.parts[0]
   elif n.startswith('Pupil /'):parent=next(p for p in self.parts if p['part'].startswith('Eye /') and p['part'].split()[-1]==n.split()[-1])
   elif n.startswith('Mouth /'):parent=next(p for p in self.parts if p['part'].startswith('Mouth rim /'))
   else:continue
   positions=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
   for k in range(0,len(mark['indices']),3):
    a,b,c=[positions[j] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    samples=[[sum(q[j] for q in (a,b,c))/3 for j in range(3)],*[[.5*(u[j]+v[j]) for j in range(3)] for u,v in ((a,b),(b,c),(c,a))]]
    for q in samples:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.0001,(n,q))
 def test_coherent_face_six_shroud_hinges_and_six_fingers(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith('Body /'):self.assertEqual(owners,set(range(1,8)))
   elif n.startswith('Hand /'):self.assertEqual(owners,{8 if n.endswith('-1') else 12})
   elif n.startswith('Finger /'):
    side,j=map(int,n.split()[-2:]);self.assertEqual(owners,{(9 if side<0 else 13)+j})
   else:self.assertEqual(owners,{1},n)
  for j in range(2,8):self.assertEqual(m.parents[j],1)
  for j in range(9,12):self.assertEqual(m.parents[j],8)
  for j in range(13,16):self.assertEqual(m.parents[j],12)
 def test_detached_hands_and_occupied_finger_roots_survive_motion(self):
  m=self.m
  for clip in m.clips:
   for t in (.0,.18,.31,.55):
    mats=m.matrices(clip,t);parts=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)]
    for side in (-1,1):
     palm=next(p for p in parts if p['part'].startswith('Hand /') and p['part'].split()[-1]==str(side))
     self.assertFalse(parts_overlap(palm,parts[0]),(clip,t,'hand must remain detached',side))
     for p in parts:
      if p['part'].startswith('Finger /') and p['part'].split()[-2]==str(side):self.assertTrue(parts_overlap(p,palm),(clip,t,p['part']))
  palm=next(p for p in self.parts if p['part'].startswith('Hand /'));finger=copy.deepcopy(next(p for p in self.parts if p['part'].startswith('Finger /')))
  finger['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(finger['positions'])];self.assertFalse(parts_overlap(finger,palm))
 def test_floor_neutral_endpoints_winding_strain_and_independent_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(q[1] for q in posed),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))>.002:moved.add(n.split(' /')[0])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    if step in (0,8):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Eye','Hand','Finger'}<=moved,moved)
  mats=m.matrices('attack',.31)
  for i,n in enumerate(m.names):
   if n.startswith('Finger /'):
    parent=8 if n.split()[-2]=='-1' else 12
    self.assertGreater(max(math.dist(m.point(i,v,mats),skin.transform(mats[parent],p)) for v,p in enumerate(m.vertices[i])),.002,n)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
