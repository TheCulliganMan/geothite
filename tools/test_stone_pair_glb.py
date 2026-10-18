#!/usr/bin/env python3
"""Decoded stone skins: actual roots, fitted face/armor support, digits and empty jaw."""
import copy,math,os,unittest
from pathlib import Path
import stone_pair_glb as author
import test_graveler_sculpt as gravel
import test_golem_sculpt as adult
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import occupied_overlap,inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_steelix_sculpt import occupied
ROOT=Path(os.environ.get('CRYSTAL_STONE_PAIR_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class StoneSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i])
 def find(self,m,prefix,suffix=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (suffix is None or n.endswith(' '+str(suffix))))
 def test_exact_source_packing_hierarchy_inverse_binds_and_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_stone_pair(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_stone_pair(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(len(m.names),40 if s=='graveler' else 77);self.assertEqual(len(m.joints),10 if s=='graveler' else 12);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   self.assertEqual(len(set(self.runtime[s].names)),len(self.runtime[s].names))
   self.assertTrue(all(0<len(n.encode('utf-8'))<=160 for n in self.runtime[s].names))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_actual_corner_normals(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_independent_controls_and_complete_fixed_support_and_digit_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(author.JOINTS[s]))))
   rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith(('Foot /','Foot claw /','Heel claw /')):owner=0
    elif s=='graveler' and n.startswith(('Hand /','Finger /')):
     side=int(n.split()[-3] if n.startswith('Finger /') else n.split()[-1]);owner=6+(0 if ' upper ' in n else 2)+(0 if side<0 else 1)
    elif s=='golem' and n.startswith(('Hand /','Hand claw /')):owner=8+(0 if int(n.split()[-2] if n.startswith('Hand claw /') else n.split()[-1])<0 else 1)
    elif s=='golem' and n.startswith(('Shell /','Shell plate /')):owner=1
    elif s=='golem' and n.startswith(('Jaw /','Tongue /','Lower tooth /')):owner=3
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
 def test_fixed_floor_positive_winding_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if m.names[i].startswith(('Foot /','Foot claw /','Heel claw /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.8<ratio<1.2,(s,clip,t,m.names[i],ratio))
     for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_actual_arm_wrist_digit_rock_and_jaw_roots_through_poses(self):
  for s,m in self.logical.items():
   body=self.find(m,'Body /' if s=='graveler' else 'Shell /');pairs=[]
   if s=='graveler':
    for side in (-1,1):
     for level in ('upper','lower'):
      arm=self.find(m,'Arm / rooted '+level,side);hand=self.find(m,'Hand / broad '+level,side);wrist=arm if level=='upper' else self.find(m,'Forearm /',side);pairs.extend(((arm,body),(hand,wrist)))
      if level=='lower':pairs.append((wrist,arm))
      for j in range(3):pairs.append((self.find(m,'Finger /',f'{side} {level} {j}'),hand))
    for i,n in enumerate(m.names):
     if n.startswith(('Foot /','Rubble /')):pairs.append((i,body))
   else:
    for side in (-1,1):
     for child,parent in (('Arm /','Shell /'),('Forearm /','Arm /'),('Hand /','Forearm /'),('Thigh /','Shell /'),('Leg /','Thigh /'),('Foot /','Leg /'),('Heel claw /','Foot /')):pairs.append((self.find(m,child,side),body if parent=='Shell /' else self.find(m,parent,side)))
     for j in range(3):pairs.extend(((self.find(m,'Hand claw /',f'{side} {j}'),self.find(m,'Hand /',side)),(self.find(m,'Foot claw /',f'{side} {j}'),self.find(m,'Foot /',side))))
    pairs.extend(((self.find(m,'Head /'),body),(self.find(m,'Muzzle /'),self.find(m,'Head /')),(self.find(m,'Jaw /'),body),(self.find(m,'Mouth back /'),self.find(m,'Muzzle /')),(self.find(m,'Tongue /'),self.find(m,'Jaw /'))))
    for i,n in enumerate(m.names):
     if n.startswith('Shell plate /'):pairs.append((i,body))
     elif n.startswith('Fang /'):pairs.append((i,self.find(m,'Muzzle /')))
     elif n.startswith('Lower tooth /'):pairs.append((i,self.find(m,'Jaw /')))
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76):
     mats=m.matrices(clip,t);parts={i:self.part(m,i,mats) for i in {v for p in pairs for v in p}}
     for child,parent in pairs:self.assertTrue(occupied_overlap(parts[child],parts[parent]),(s,clip,t,m.names[child],m.names[parent]))
 def test_posed_fitted_face_and_polygon_armor_keep_actual_support(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=(gravel.GravelerTests if s=='graveler' else adult.GolemTests)();helper.ps=[inverse_rigid(self.part(m,i,mats),mats[m.ids[i][0][0]]) for i in range(len(m.parts))]
     helper.test_fitted_vertices_follow_actual_stored_parent();helper.test_fitted_face_centers_and_edges_clear_actual_parent()
   if s=='golem':
    helper=adult.GolemTests();helper.ps=self.models[s]['primitives'];helper.test_shell_plate_outer_faces_clear_and_inner_faces_root_into_body()
 def test_posed_jaw_aperture_remains_actually_empty_and_filled_negative_fails(self):
  m=self.logical['golem'];helper=adult.GolemTests();points=helper.aperture_points()
  for clip in m.clips:
   for t in (.18,.34,.46,.76):
    mats=m.matrices(clip,t);parts=[inverse_rigid(self.part(m,i,mats),mats[1]) for i in range(len(m.parts))]
    for point in points:self.assertFalse(any(occupied(p,point) for p in parts),(clip,t,point))
  helper.ps=self.models['golem']['primitives'];helper.test_filled_jaw_and_detached_armor_negatives()
 def test_source_byte_and_species_mutations_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_stone_pair(bad,s)
   with self.assertRaises(ValueError):author.decode_stone_pair(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_stone_pair(s,model)

if __name__=='__main__':unittest.main()
