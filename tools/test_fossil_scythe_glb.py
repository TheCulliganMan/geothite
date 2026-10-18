#!/usr/bin/env python3
"""Decoded fossil armor, fitted gaze, scythes and planted claw/foot supports."""
import copy,math,os,unittest
from pathlib import Path
import fossil_scythe_glb as author
import test_fossil_scythe_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(os.environ.get('CRYSTAL_FOSSIL_SCYTHE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class FossilScytheSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_source_packing_hierarchy_inverse_binds_names_and_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_fossil_scythe(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_fossil_scythe(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(len(m.joints),6 if s=='kabuto' else 13);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertEqual(len(set(self.runtime[s].names)),len(self.runtime[s].names));self.assertTrue(all(0<len(n.encode())<=160 for n in self.runtime[s].names));self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_stored_corner_normals(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_all_controls_move_independently_and_faces_ribs_scythes_and_digits_share_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(author.JOINTS[s]))));rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith(('Foot /','Claw /','Toe /')):owner=0
    elif s=='kabuto' and not n.startswith('Leg /'):owner=1
    elif s=='kabutops':
     if n.startswith(('Body /','Neck /','Rib plate /','Back spur /')):owner=1
     elif n.startswith(('Head /','Head wing /','Eye socket /','Eye lens /','Pupil /')):owner=2
     elif n.startswith(('Upper arm /','Elbow /')):owner=3 if n.endswith('-1') else 4
     elif n.startswith('Forearm /'):owner=5 if n.endswith('-1') else 6
     elif n.startswith(('Wrist /','Scythe /')):owner=7 if n.endswith('-1') else 8
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),n);self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
 def test_fixed_soles_positive_winding_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if m.names[i].startswith(('Foot /','Claw /','Toe /')):self.assertEqual(posed,vs)
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
     mats=m.matrices(clip,t);helper=anatomy.FossilScytheTests();helper.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])}
     helper.test_complete_anatomy_has_occupied_volume_roots()
 def test_posed_gaze_pores_ribs_and_rear_inset_keep_parent_support(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=anatomy.FossilScytheTests();helper.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])};indices={n:i for i,n in enumerate(m.names)}
     def pairs():
      for child,parent in helper.pairs(s):
       back=child['part'].startswith('Rear inset /')
       if back:child=anatomy.reverse_front(child);parent=anatomy.reverse_front(parent)
       i=indices[child['part']];j=m.ids[i][0][0];self.assertTrue(all(ids==[j,0,0,0] for ids in m.ids[i]));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]));child=inverse_rigid(child,mats[j]);parent=inverse_rigid(parent,mats[j]);yield (anatomy.reverse_front(child),anatomy.reverse_front(parent)) if back else (child,parent)
     fitted=anatomy.fitted.NidorinaTests();fitted.pairs=pairs;fitted.test_fitted_layers_follow_actual_stored_parent();fitted.test_fitted_face_centers_and_edges_clear_parent()
 def test_front_white_gaze_and_negative_attachment_controls_remain_real(self):
  for s,m in self.logical.items():
   helper=anatomy.FossilScytheTests();helper.models={s:self.models[s]};helper.test_detached_limbs_and_tail_negative_controls_are_rejected()
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper.models={s:dict(primitives=[inverse_rigid(self.part(m,i,mats),mats[1 if s=='kabuto' else 2]) for i in range(len(m.parts))])};helper.test_both_gazes_are_visible_ahead_of_complete_head_or_shell()
 def test_source_bytes_and_species_mutations_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_fossil_scythe(bad,s)
   with self.assertRaises(ValueError):author.decode_fossil_scythe(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_fossil_scythe(s,model)

if __name__=='__main__':unittest.main()
