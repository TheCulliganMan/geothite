#!/usr/bin/env python3
"""Decoded nautilus controls, actual shell support, grounded fans and open adult beak."""
import copy,math,os,unittest
from pathlib import Path
import nautilus_glb as author
import test_omanyte_sculpt as juvenile
import test_omastar_sculpt as adult
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import occupied_overlap,inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(os.environ.get('CRYSTAL_NAUTILUS_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class NautilusSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i])
 def find(self,m,prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
 def test_exact_source_packing_hierarchy_inverse_binds_and_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_nautilus(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_nautilus(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(len(m.names),32 if s=='omanyte' else 43);self.assertEqual(len(m.joints),15 if s=='omanyte' else 19);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_actual_corner_normals(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_independent_controls_and_coherent_rigid_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(1,len(author.JOINTS[s]))))
   rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=author.owner(s,n);self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]),(s,n))
 def test_ground_contact_positive_winding_rigid_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if not m.names[i].startswith('Beak /'):
       for p,q in zip(posed,vs):self.assertAlmostEqual(p[1],q[1],places=6)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertAlmostEqual(math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]),1.,places=5)
     for i,k in floor:self.assertAlmostEqual(m.point(i,k,mats)[1],m.vertices[i][k][1],places=6)
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_actual_shell_eye_tentacle_and_beak_roots_through_poses(self):
  for s,m in self.logical.items():
   body=self.find(m,'Body /');shell=self.find(m,'Shell /');pairs=[(shell,body)]
   for i,n in enumerate(m.names):
    if n.startswith(('Eye /','Tentacle /','Beak /')):pairs.append((i,body))
    elif n.startswith(('Shell seam /','Shell spiral /','Spine /')):pairs.append((i,shell))
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76):
     mats=m.matrices(clip,t);parts={i:self.part(m,i,mats) for i in {v for p in pairs for v in p}}
     for child,parent in pairs:self.assertTrue(occupied_overlap(parts[child],parts[parent]),(s,clip,t,m.names[child],m.names[parent]))
 def test_posed_gaze_keeps_fitted_parent_support_and_visible_pupils(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=(juvenile.OmanyteTests if s=='omanyte' else adult.OmastarTests)();helper.ps=[inverse_rigid(self.part(m,i,mats),mats[1]) for i in range(len(m.parts))]
     if s=='omanyte':helper.test_mantle_shell_and_tentacles_leave_every_stored_iris_vertex_clear()
     else:helper.test_mantle_shell_spines_and_tentacles_leave_pupils_clear()
     helper.ps=[inverse_rigid(self.part(m,i,mats),mats[m.ids[i][0][0]]) for i in range(len(m.parts))]
     if s=='omanyte':helper.test_fitted_eyes_follow_actual_stored_parents();helper.test_fitted_eye_centers_and_edges_clear_actual_parents()
     else:helper.test_fitted_eyes_follow_actual_stored_parents();helper.test_fitted_eye_centers_and_edges_clear_actual_parents()
 def test_independent_beak_quadrants_preserve_actual_open_cross_and_negative_control(self):
  m=self.logical['omastar']
  for clip in m.clips:
   for t in (.18,.34,.46,.76):
    mats=m.matrices(clip,t);parts=[inverse_rigid(self.part(m,i,mats),mats[1]) for i in range(len(m.parts))];self.assertTrue(adult.open_cross(parts),(clip,t))
  helper=adult.OmastarTests();helper.ps=self.models['omastar']['primitives'];helper.test_filled_beak_negative_is_rejected()
 def test_source_byte_and_species_mutations_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_nautilus(bad,s)
   with self.assertRaises(ValueError):author.decode_nautilus(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_nautilus(s,model)

if __name__=='__main__':unittest.main()
