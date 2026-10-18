#!/usr/bin/env python3
"""Decoded hollow shell hinges, coherent gaze and supported tongue articulation."""
import copy,math,unittest
from pathlib import Path
import bivalve_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BivalveSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def test_canonical_bounded_containers_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_bivalve(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());count=6 if s=='shellder' else 5;self.assertEqual(len(m.joints),count)
   self.assertEqual(m.parents[:count],[None,0,1,1,1,4] if s=='shellder' else [None,0,1,1,1])
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(1,count)))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_anatomical_owners_keep_marks_with_their_shell_or_face(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if s=='shellder':
     if n.startswith(('Tongue /','Tongue crease /')):self.assertTrue(owners<={1,4,5});self.assertIn(5,owners);continue
     if n.startswith(('Shell /','Lining /','Hinge /','Horn /','Rear horn /','Shell groove /')):expected={2}
     elif n.startswith(('Jaw /','Jaw lining /')):expected={3}
     else:expected={1}
    else:
     if n.startswith('Shell /'):expected={2 if n.endswith('-1') else 3}
     elif n.startswith('Outer spike /'):expected={2 if n.split()[-2]=='-1' else 3}
     elif n.startswith(('Inner hood /','Mantle lining /','Inner horn /')):expected={4}
     else:expected={1}
    self.assertEqual(owners,expected,(s,n))
 def test_occupied_shell_hinge_tongue_and_armor_roots_survive_motion(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives']
   def find(prefix,end=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (end is None or n.split()[-len(end):]==end))
   body=find('Body /');shell=find('Shell /');pairs=[(shell,body)]
   if s=='shellder':
    pairs += [(shell,find('Lining /')),(shell,find('Hinge /')),(body,find('Jaw /')),(find('Jaw /'),find('Jaw lining /')),(body,find('Tongue /'))]
    pairs += [(i,shell) for i,n in enumerate(m.names) if n.startswith(('Horn /','Rear horn /'))]
   else:
    hood=find('Inner hood /');pairs += [(body,hood),(hood,find('Mantle lining /')),(hood,find('Inner horn /')),(shell,find('Rear crown /'))]
    pairs += [(i,hood) for i,n in enumerate(m.names) if n.startswith('Shell /')]
    pairs += [(i,find('Shell /',[n.split()[-2]])) for i,n in enumerate(m.names) if n.startswith('Outer spike /')]
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(parts[shell]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,parts[body]))
 def test_floor_endpoints_winding_strain_and_independent_valves(self):
  for s,m in self.models.items():
   independent=set()
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if max(math.dist(a,transform(mats[1],b)) for a,b in zip(points,m.vertices[i]))>.001:independent.add(n.split(' /')[0])
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7,(s,clip,step))
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   self.assertTrue(({'Shell','Jaw','Tongue'} if s=='shellder' else {'Shell','Inner hood'})<=independent,(s,independent))
 def test_source_cues_own_attack_and_hit_clocks(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
