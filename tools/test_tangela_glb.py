#!/usr/bin/env python3
"""Decoded rooted curls, coherent eye frame, visible pupils and planted shoe regressions."""
import copy,math,unittest
from pathlib import Path
import tangela_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class TangelaSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'tangela.glb').read_bytes();cls.runtime=skin.Model(cls.blob);cls.m=logical_model(cls.runtime);cls.ps=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending))
 def posed(self,mats):return [dict(p,positions=sum(self.m.posed(i,mats),[])) for i,p in enumerate(self.ps)]
 def local(self,index,mats):
  matrix=mats[1];points=[]
  for p in self.m.posed(index,mats):
   delta=[p[k]-matrix[12+k] for k in range(3)];points += [sum(matrix[4*j+k]*delta[k] for k in range(3)) for j in range(3)]
  return dict(self.ps[index],positions=points)
 def test_exact_logical_geometry_lossless_material_packing_and_limits(self):
  m=self.m;r=self.runtime;self.assertEqual(self.blob,author.export_tangela());self.assertFalse((ROOT/'tangela.mesh.json').exists());self.assertEqual((len(m.parts),len(r.parts),len(m.joints)),(49,10,39))
  self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['tangela']);self.assertEqual(author.geometry_digest(r.neutral_model()),author.PACKED_REST_DIGESTS['tangela']);self.assertEqual(author.geometry_digest(author.decode_vine_tangle(self.blob,'tangela')),author.REST_DIGESTS['tangela'])
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024);self.assertEqual(m.parents[:39],[None,0]+[1]*37)
  self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(39)))
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  bad=copy.deepcopy(author.authoring_model());bad['primitives'][0]['positions'][0]+=.01
  with self.assertRaises(ValueError):author.export_tangela(bad)
 def test_shoes_eye_frame_and_root_caps_keep_their_anatomical_owners(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith('Shoe /'):self.assertEqual(owners,{0})
   elif n in author.VINES:
    joint=2+author.VINES.index(n);self.assertEqual(owners,{1,joint});count=10 if n.startswith('Rear vine /') else 12
    for v in set(m.indices[i][:(count-2)*3]):self.assertEqual(m.ids[i][v],[1,0,0,0],n)
   else:self.assertEqual(owners,{1},n)
 def test_all_curls_retain_actual_occupied_roots_and_shoes_stay_attached(self):
  m=self.m;body=self.find('Body /')
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t);parts=self.posed(mats)
    for i,n in enumerate(m.names):
     if n in author.VINES:
      path=author.VINE_PATHS[n];point=tuple((a+b)/2 for a,b in zip(path[0],path[1]));witness=transform(mats[1],author.normalized('tangela',point));self.assertTrue(occupied(parts[body],witness),(clip,n));self.assertTrue(occupied(parts[i],witness),(clip,n))
     elif n.startswith(('Shoe /','Eye /')) or 'eye-frame rope' in n:self.assertTrue(parts_overlap(parts[i],parts[body]),(clip,n))
  first=author.VINES[0];index=m.names.index(first);path=author.VINE_PATHS[first];point=author.normalized('tangela',tuple((a+b)/2 for a,b in zip(path[0],path[1])));bad=copy.deepcopy(self.ps[index]);bad['positions']=[v+(2. if i%3==2 else 0.) for i,v in enumerate(bad['positions'])];self.assertFalse(occupied(bad,point))
 def test_all_posed_vines_leave_both_pupils_unobstructed(self):
  m=self.m;vines=[i for i,n in enumerate(m.names) if n.startswith(('Front vine /','Front weave /','Rear vine /','Rear weave /'))]
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80):
    mats=m.matrices(clip,t);parts={i:self.local(i,mats) for i in vines}
    for side in ('-1','1'):
     pupil=self.local(self.find('Pupil /',[side]),mats)
     for k in range(0,len(pupil['positions']),3):
      x,y,z=pupil['positions'][k:k+3]
      for i in vines:
       hits=rays(parts[i],x,y)
       if hits:self.assertLess(max(hits),z,(clip,t,m.names[i]))
 def test_floor_endpoints_winding_strain_and_independent_vine_articulation(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);floor=float('inf')
    for i,n in enumerate(m.names):
     points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(clip,n))
     if n.startswith('Shoe /'):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    self.assertLess(abs(floor),1e-7)
    if step in (0,8):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  rotations=m.rotations('attack',.31)
  for joint,n in enumerate(author.VINES,2):
   isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate);index=m.names.index(n);self.assertGreater(max(math.dist(p,q) for p,q in zip(m.vertices[index],m.posed(index,mats))),.0005,n)
   other=m.names.index(author.VINES[(joint-1)%37]);self.assertLess(max(math.dist(p,q) for p,q in zip(m.vertices[other],m.posed(other,mats))),1e-7,n)
 def test_independent_idle_phases_do_not_collapse_into_one_rigid_tangle(self):
  rotations=self.m.rotations('idle',.25);values={tuple(round(v,6) for v in rotations[j]) for j in range(2,39)};self.assertEqual(len(values),37)
 def test_clips_keep_source_cue_rotation_only_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertTrue(all(c['target']['path']=='rotation' and c['target']['node']!=0 for c in clip['channels']))

if __name__=='__main__':unittest.main()
