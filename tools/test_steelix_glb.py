"""Exact original steel anatomy, rigid armor carriers and rooted chain articulation."""
import copy,math,os,unittest
from pathlib import Path
import steelix_glb as author
import test_steelix_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_STEELIX_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class SteelixSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'steelix.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def part(self,i,mats):
  m=self.m;return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def helper(self,parts):
  h=anatomy.SteelixTests();h.m=dict(primitives=parts);h.ps=parts;return h
 def test_exact_original_neutral_f32_geometry_and_packing(self):
  self.assertEqual(self.blob,author.export_steelix());self.assertEqual(author.geometry_digest(self.m.neutral_model()),'87fe7010288e39368021ba4b84e9477f898cfc9e341d884f334cbaeeca40eceb');self.assertEqual(author.geometry_digest(author.decode_steelix(self.blob,'steelix')),author.REST_DIGESTS['steelix'])
  self.assertEqual((len(self.m.parts),len(self.runtime.parts),sum(map(len,self.m.vertices)),sum(len(i)//3 for i in self.m.indices)),(48,32,10405,9244));self.assertEqual(len(set(self.runtime.names)),32);self.assertTrue(all(0<len(n.encode())<=160 for n in self.runtime.names));self.assertLessEqual(len(self.blob),1024*1024);self.assertLessEqual(len(self.m.doc['accessors']),256)
 def test_hierarchy_inverse_binds_and_all_controls_move(self):
  m=self.m;self.assertEqual(m.parents[:10],[None,0,1,2,3,4,5,6,7,8]);self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(10)))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  for j in range(1,10):
   qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.27)[j];mats=m.matrices(rotations=qs);self.assertGreater(max(math.dist(a,b) for i,vs in enumerate(m.vertices) for a,b in zip(vs,m.posed(i,mats))),.0005,j)
 def test_rigid_boulders_spikes_skull_face_teeth_and_tail_have_coherent_owners(self):
  for i,n in enumerate(self.m.names):
   if n.startswith('Joint /'):continue
   owner=author.rock_owner(int(n.split()[-1])) if n.startswith('Segment /') else author.rock_owner(int(n.split()[-2])) if n.startswith('Spike /') else 0 if n.startswith('Tail /') else 9 if n.startswith(('Jaw /','Lower tooth /')) else 8
   self.assertTrue(all(js==[owner,0,0,0] for js in self.m.ids[i]),n);self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in self.m.weights[i]),n)
 def test_floor_supports_rigid_armor_positive_winding_and_bounded_joint_strain(self):
  m=self.m;floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
  for clip in m.clips:
   for t in (0.,.18,.27,.40,.52,.78,1.):
    mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
    for i,vs in enumerate(m.vertices):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(clip,t,m.names[i]))
     if m.names[i].startswith('Tail /') or m.names[i].startswith('Segment /') and int(m.names[i].split()[-1])<3:self.assertEqual(posed,vs,m.names[i])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,m.names[i],k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(clip,t,m.names[i],ratio))
    for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
 def test_actual_boulder_joint_spike_hinge_and_tooth_roots_survive_poses(self):
  for clip in self.m.clips:
   for t in (.18,.27,.52,.78):
    mats=self.m.matrices(clip,t);h=self.helper([self.part(i,mats) for i in range(len(self.m.parts))]);h.test_adjacent_rocks_and_buried_joints_have_volume_roots();h.test_spikes_tail_head_and_jaw_hinges_have_actual_roots();h.test_all_squared_teeth_have_actual_jaw_roots()
 def test_fitted_gaze_and_actual_empty_aperture_survive_jaw_articulation(self):
  for clip in self.m.clips:
   for t in (.18,.27,.52,.78):
    mats=self.m.matrices(clip,t);h=self.helper([inverse_rigid(self.part(i,mats),mats[8]) for i in range(len(self.m.parts))]);h.test_fitted_vertices_follow_actual_stored_parent();h.test_fitted_face_centers_and_edges_clear_actual_parent();h.test_front_jaw_aperture_is_actually_empty()
 def test_original_body_length_uses_real_bind_centers_and_unchanged_source_datums(self):
  m=self.m;points=m.doc['extras']['body_centerline_points'];self.assertEqual(len(points),12)
  for j in range(3,10):
   self.assertLess(math.dist([-v for v in m.inverse[j-2][12:15]],points[j]),1e-7)
  self.assertLess(math.dist([-v for v in m.inverse[8][12:15]],points[10]),1e-7);self.assertAlmostEqual(sum(math.dist(a,b) for a,b in zip(points,points[1:])),author.body_reference_span(),places=6);self.assertAlmostEqual(m.doc['extras']['body_centerline_length'],2.867623161993143,places=12)
 def test_neutral_endpoints_rotation_only_source_cues_and_real_jaw_motion(self):
  for clip in self.m.clips:
   for t in (0.,1.):
    for mat in self.m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  for a in self.m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle);self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
  self.assertGreater(author.pose_angles('steelix','attack',.18)[9][0],.05);self.assertLess(author.pose_angles('steelix','hit',.27)[9][0],0.)
 def test_mutated_source_bytes_and_species_rejected(self):
  for b in (self.blob[:-1],self.blob+b'bad',self.blob[:20]+bytes([self.blob[20]^1])+self.blob[21:]):
   with self.assertRaises(ValueError):author.decode_steelix(b,'steelix')
  with self.assertRaises(ValueError):author.decode_steelix(self.blob,'onix')
  m=copy.deepcopy(author.authoring_model('steelix'));m['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_steelix(m)
if __name__=='__main__':unittest.main()
