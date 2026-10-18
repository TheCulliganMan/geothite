"""Stored canonical snake skin, grounded supports and independent posed anatomy."""
import copy,math,os,unittest
from pathlib import Path
import ekans_glb as author
import test_ekans_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import crossed_faces,surface_distances
ROOT=Path(os.environ.get('CRYSTAL_EKANS_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class EkansSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'ekans.glb').read_bytes();cls.m=logical_model(Model(cls.blob))
 def part(self,i,mats):
  m=self.m;return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_exact_canonical_source_hierarchy_inverse_binds_and_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_ekans());self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['ekans']);self.assertEqual(author.geometry_digest(author.decode_ekans(self.blob,'ekans')),author.REST_DIGESTS['ekans']);self.assertEqual(m.parents[:5],[None,0,1,0,3])
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  self.assertLessEqual(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertEqual(len(m.parts),16)
 def test_each_control_moves_and_complete_face_and_rattles_share_carriers(self):
  m=self.m;self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(5)))
  for j in range(1,5):
   qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.34)[j];mats=m.matrices(rotations=qs);self.assertGreater(max(math.dist(a,b) for i,vs in enumerate(m.vertices) for a,b in zip(vs,m.posed(i,mats))),.0005)
  for i,n in enumerate(m.names):
   owner=4 if n.startswith('Rattle /') else 2 if n.startswith(('Head /','Eye /','Pupil /','Glint /','Nostril /','Mouth /')) else None
   if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),n)
 def test_fixed_coil_floor_positive_winding_bounded_strain_and_neutral_endpoints(self):
  m=self.m;floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
  for clip in m.clips:
   for t in (0.,.18,.34,.46,.76,1.):
    mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
    for i,vs in enumerate(m.vertices):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(clip,t,m.names[i]))
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,m.names[i],k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.80<math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b])<1.20,(clip,t,m.names[i]))
    for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
    if t in (0.,1.):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_anatomical_roots_stay_occupied_through_poses(self):
  for clip in self.m.clips:
   for t in (.18,.34,.46,.76):
    helper=anatomy.EkansTests();helper.model=dict(primitives=[self.part(i,self.m.matrices(clip,t)) for i in range(len(self.m.parts))]);helper.test_head_collar_belly_and_rattles_have_occupied_roots()
 def test_posed_coil_stays_uncrossed_and_collar_remains_surface_supported(self):
  body=next(i for i,n in enumerate(self.m.names) if n.startswith('Body /'));collar=next(i for i,n in enumerate(self.m.names) if n.startswith('Collar /'))
  for clip in self.m.clips:
   for t in (.18,.34,.46,.76):
    mats=self.m.matrices(clip,t);parent=self.part(body,mats);p=self.part(collar,mats);self.assertIsNone(crossed_faces(parent),(clip,t))
    points={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)};self.assertTrue(all(d<.008 for d in surface_distances(parent,points,.008)),(clip,t))
 def test_rotation_only_source_cues_and_neutral_loop_endpoints(self):
  for a in self.m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle);self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
  self.assertGreater(sum(abs(v) for p in author.pose_angles('ekans','attack',.34) for v in p),sum(abs(v) for p in author.pose_angles('ekans','attack',.90) for v in p))
 def test_mutated_source_bytes_and_species_rejected(self):
  for b in (self.blob[:-1],self.blob+b'bad',self.blob[:20]+bytes([self.blob[20]^1])+self.blob[21:]):
   with self.assertRaises(ValueError):author.decode_ekans(b,'ekans')
  with self.assertRaises(ValueError):author.decode_ekans(self.blob,'arbok')
  m=copy.deepcopy(author.authoring_model('ekans'));m['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_ekans(m)
if __name__=='__main__':unittest.main()
