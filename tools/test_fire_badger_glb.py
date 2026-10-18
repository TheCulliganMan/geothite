#!/usr/bin/env python3
"""Decoded fire-badger carriers, coat seams, fire roots, empty jaw and grounded supports."""
import copy,math,os,unittest
from pathlib import Path
import fire_badger_glb as author
import test_fire_badger_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_steelix_sculpt import occupied
ROOT=Path(os.environ.get('CRYSTAL_FIRE_BADGER_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class FireBadgerSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_source_packing_hierarchy_inverse_binds_names_and_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_fire_badger(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_fire_badger(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(len(m.joints),13 if s=='quilava' else 17);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertEqual(len(set(self.runtime[s].names)),len(self.runtime[s].names));self.assertTrue(all(0<len(n.encode())<=160 for n in self.runtime[s].names));self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_stored_corner_normals(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_all_controls_move_independently_and_digits_and_fire_blades_share_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(author.JOINTS[s]))));rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith(('Foot /','Toe /')):owner=0
    elif n.startswith(('Body /','Collar /')):owner=1
    elif n.startswith(('Head /','Ear /','Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Nose /','Fang /','Lip /')):owner=2
    elif n.startswith(('Jaw /','Tongue /')):owner=3
    elif n.startswith(('Hand /','Finger /')):owner=6 if int(n.split()[-2] if n.startswith('Finger /') else n.split()[-1])<0 else 7
    elif n.startswith('Fire /'):owner=(7 if s=='quilava' else 10)+[v[0] for v in author.fire_specs(s)].index(n.split(' / ')[1])
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
 def test_fixed_soles_positive_winding_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if m.names[i].startswith(('Foot /','Toe /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.8<ratio<1.2,(s,clip,t,m.names[i],ratio))
     for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_real_anatomical_roots_remain_occupied_through_poses(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=anatomy.FireBadgerTests();helper.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])}
     helper.test_skull_ears_nose_and_adult_jaw_are_occupied();helper.test_arms_palms_digits_and_supports_have_actual_roots();helper.test_each_fire_blade_and_all_folds_have_occupied_support();helper.test_actual_two_tone_body_and_head_share_surface_positions_and_normals()
 def test_posed_fitted_gaze_and_lip_keep_actual_parent_support(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=anatomy.FireBadgerTests();helper.models={s:dict(primitives=[inverse_rigid(self.part(m,i,mats),mats[m.ids[i][0][0]]) for i in range(len(m.parts))])}
     # The anatomy helper intentionally checks one complete species at a time.
     fitted=anatomy.fitted.NidorinaTests();fitted.pairs=lambda:helper.pairs(s);fitted.test_fitted_layers_follow_actual_stored_parent();fitted.test_fitted_face_centers_and_edges_clear_parent()
 def test_adult_jaw_aperture_remains_empty_through_poses_and_negative_control_is_real(self):
  helper=anatomy.FireBadgerTests();helper.models=self.models;m=self.logical['typhlosion'];points=helper.aperture_points()
  for clip in m.clips:
   for t in (.18,.34,.46,.76):
    mats=m.matrices(clip,t);parts=[inverse_rigid(self.part(m,i,mats),mats[2]) for i in range(len(m.parts))]
    for point in points:self.assertFalse(any(occupied(p,point) for p in parts),(clip,t,point))
  helper.test_adult_jaw_has_real_empty_space_and_filled_negative_control()
 def test_source_bytes_and_species_mutations_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_fire_badger(bad,s)
   with self.assertRaises(ValueError):author.decode_fire_badger(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_fire_badger(s,model)

if __name__=='__main__':unittest.main()
