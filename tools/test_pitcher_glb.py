"""Independent botanical skin carriers, flexible leaves, true roots and hollow mouths."""
import copy,math,os,unittest
from pathlib import Path
import pitcher_glb as author
import test_pitcher_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(os.environ.get('CRYSTAL_PITCHER_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class PitcherSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_source_packing_hierarchy_inverse_binds_names_and_original_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_pitcher(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_pitcher(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(len(m.joints),9 if s=='weepinbell' else 10);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertEqual(len(set(self.runtime[s].names)),len(self.runtime[s].names));self.assertTrue(all(0<len(n.encode())<=160 for n in self.runtime[s].names));self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_corner_normals(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_all_controls_move_and_complete_faces_mouths_fangs_and_hood_share_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(author.JOINTS[s]))));rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated);self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=3 if s=='bellsprout' and n.startswith(('Head /','Mouth /','Eye /')) else 1 if n.startswith(('Eye /','Pigment /')) else 2 if n.startswith(('Mouth /','Fang /')) else 7 if n.startswith('Hood /') else None
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),n);self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
 def test_fixed_floor_positive_winding_bounded_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.8<ratio<1.2,(s,clip,t,m.names[i],ratio))
     for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_actual_roots_and_leaf_veins_stay_attached_through_poses(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=anatomy.PitcherTests();helper.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])};helper.test_complete_stalk_roots_leaves_mouth_lids_and_tendrils_have_occupied_roots();helper.test_curved_stalks_roots_leaves_and_tendrils_do_not_cross()
 def test_posed_fitted_gaze_and_pigment_retain_parent_support(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);helper=anatomy.PitcherTests();helper.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])};indices={n:i for i,n in enumerate(m.names)}
     def pairs():
      for child,parent in helper.pairs(s):
       i=indices[child['part']];j=m.ids[i][0][0];self.assertTrue(all(ids==[j,0,0,0] for ids in m.ids[i]));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]));yield inverse_rigid(child,mats[j]),inverse_rigid(parent,mats[j])
     fitted=anatomy.fitted.NidorinaTests();fitted.pairs=pairs;fitted.test_fitted_layers_follow_actual_stored_parent();fitted.test_fitted_face_centers_and_edges_clear_parent()
 def test_visible_gaze_and_recessed_cavities_remain_real_through_articulation(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.46,.76):
     mats=m.matrices(clip,t);parts=[]
     for i,n in enumerate(m.names):
      j=3 if s=='bellsprout' and n.startswith(('Head /','Mouth /','Eye /')) else 2 if n.startswith(('Mouth /','Fang /')) else 1
      parts.append(inverse_rigid(self.part(m,i,mats),mats[j]))
     helper=anatomy.PitcherTests();helper.models={s:dict(primitives=parts)};helper.test_frontal_gaze_stays_in_front_of_actual_head_or_body();helper.test_mouth_axis_reaches_dark_recess_before_outer_wall()
 def test_mutated_source_bytes_and_species_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_pitcher(bad,s)
   with self.assertRaises(ValueError):author.decode_pitcher(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_pitcher(s,model)
if __name__=='__main__':unittest.main()
