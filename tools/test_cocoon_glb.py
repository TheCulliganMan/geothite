#!/usr/bin/env python3
"""Decoded grounded chrysalis skin, coherent faces and restrained fold articulation."""
import copy,math,unittest
from pathlib import Path
import cocoon_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class CocoonSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,parts,prefix,ending=None):return next(p for p in parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_skin_schema_and_distinct_anatomy(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_cocoon(s));self.assertEqual(len(m.parts),15 if s=='metapod' else 14);self.assertEqual(len(m.joints),4 if s=='metapod' else 5)
   self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertEqual(sum(n.startswith('Shell /') for n in m.names),1)
   self.assertEqual(sum(n.startswith('Pupil /') for n in m.names),2 if s=='metapod' else 0)
   self.assertEqual(sum(n.startswith('Arm fold /') for n in m.names),2 if s=='kakuna' else 0)
 def test_coherent_faces_fold_owners_and_fixed_lower_shell(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith(('Eye /','Pupil /','Glint /','Brow /')):self.assertEqual(owners,{2},(s,n))
    elif n.startswith(('Arm fold /','Arm crease /')):self.assertEqual(owners,{3 if n.endswith('-1') else 4},n)
    elif n.startswith('Shell /'):self.assertEqual(owners,{0,1,2,3} if s=='metapod' else {0,1,2},s)
   for i,p in enumerate(m.vertices[0]):
    if p[1]<1e-7:self.assertEqual(m.ids[0][i],[0,0,0,0])
 def test_fitted_faces_and_shell_panels_clear_actual_stored_surfaces(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives']
   for mark in parts[1:]:
    n=mark['part'];side=[n.split()[-1]]
    if n.startswith('Pupil /'):parent=self.find(parts,'Eye /',side)
    elif n.startswith('Glint /'):parent=self.find(parts,'Eye /',side)
    elif n.startswith(('Eye /','Brow /','Shell fold /','Arm fold /','Shell keel /')):parent=parts[0]
    else:continue
    vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
     q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,(s,n));self.assertGreater(q[2]-max(hits),.0001,(s,n))
 def test_posed_fold_roots_remain_seated(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives'];prefixes=('Shell fold /',) if s=='metapod' else ('Arm fold /','Shell keel /')
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)]
     for p in posed:
      if p['part'].startswith(prefixes):self.assertTrue(parts_overlap(p,posed[0]),(s,clip,t,p['part']))
   detached=copy.deepcopy(next(p for p in parts if p['part'].startswith(prefixes)));detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,parts[0]))
 def test_ground_neutral_endpoints_winding_strain_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8)
     for i,n in enumerate(m.names):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,n))
      for a,b in zip(posed,m.vertices[i]):
       if b[1]<1e-7:self.assertLess(math.dist(a,b),1e-7)
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))>.001:moved.add(n.split(' /')[0])
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     if step in (0,8):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Shell','Eye'}<=moved,(s,moved))
   mats=m.matrices('attack',.31)
   if s=='kakuna':
    for i,n in enumerate(m.names):
     if n.startswith('Arm fold /'):self.assertGreater(max(math.dist(m.posed(i,mats)[v],skin.transform(mats[1],p)) for v,p in enumerate(m.vertices[i])),.001,n)
   else:
    self.assertGreater(max(math.dist(m.posed(0,mats)[v],skin.transform(mats[1],p)) for v,p in enumerate(m.vertices[0]) if any(j==3 and w>.5 for j,w in zip(m.ids[0][v],m.weights[0][v]))),.001)
 def test_source_cue_contract(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
