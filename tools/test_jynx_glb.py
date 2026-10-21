"""Decoded grounded gown, independent five-digit hands and coherent dancer hair."""
import copy,hashlib,math,os,unittest
from pathlib import Path
import jynx_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
import test_ice_dancers_sculpt as anatomy
ROOT=Path(os.environ.get('CRYSTAL_JYNX_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class JynxSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'jynx.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def part(self,i,mats):
  m=self.m;return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_canonical_f32_anatomy_logical_ranges_and_runtime_limits(self):
  self.assertEqual(self.blob,author.export_jynx());self.assertEqual(author.geometry_digest(self.m.neutral_model()),author.REST_DIGEST);self.assertEqual(author.geometry_digest(author.decode_jynx(self.blob,'jynx')),author.REST_DIGEST)
  self.assertEqual((len(self.m.parts),len(self.runtime.parts),sum(map(len,self.m.vertices)),sum(len(i)//3 for i in self.m.indices),len(self.m.joints)),(45,35,18473,30004,22));self.assertLessEqual(len(self.blob),1024*1024);self.assertLessEqual(len(self.m.doc['accessors']),256)
 def test_hierarchy_inverse_binds_and_used_controls(self):
  m=self.m;self.assertEqual(m.parents[:12],[None,0,1,2,3,3,1,6,7,1,9,10]);self.assertEqual(m.parents[12:22],[8]*5+[11]*5)
  self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(22)))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_face_bodice_palms_and_independent_finger_owners(self):
  m=self.m
  for i,n in enumerate(m.names):
   if n.startswith(('Head /','Eye socket /','Eye /','Pupil /','Lips /','Mouth /','Hair back /','Hair crown /','Hair part /','Back hair strand /')):owners={3}
   elif n.startswith(('Chest trim /','Chest bridge /','Bodice /')):owners={1}
   elif n.startswith('Palm /'):owners={8 if n.endswith('-1') else 11}
   elif n.startswith('Finger /'):
    side,j=map(int,n.split()[-2:]);owners={8 if side==-1 else 11,12+j if side==-1 else 17+j}
   else:continue
   self.assertTrue(all({j for j,w in zip(js,ww) if w}<=owners for js,ww in zip(m.ids[i],m.weights[i])),n)
 def test_every_control_articulates_and_fingers_do_not_drag_neighbors(self):
  m=self.m
  for j in range(1,22):
   qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,j)
   if j>=12:
    own=(j-12)//5;finger=(j-12)%5;side=-1 if own==0 else 1
    for i,n in enumerate(m.names):
     if n.startswith('Finger /') and n.endswith(f' {side} {finger}'):continue
     self.assertLess(movement[i],1e-7,(j,n))
 def test_grounded_hem_winding_and_bounded_strain_across_clips(self):
  m=self.m;floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertGreater(len(floor),40)
  for clip in m.clips:
   for t in (0.,.18,.30,.55,.80,1.):
    mats=m.matrices(clip,t)
    for i,vs in enumerate(m.vertices):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(clip,t,m.names[i]))
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,m.names[i],k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(clip,t,m.names[i],ratio))
    for i,k in floor:self.assertLess(math.dist(m.vertices[i][k],m.point(i,k,mats)),1e-7)
 def test_actual_posed_hair_throat_hands_digits_and_kiss_rim_roots(self):
  for clip in self.m.clips:
   for t in (.18,.30,.55,.80):
    mats=self.m.matrices(clip,t);ps=[self.part(i,mats) for i in range(len(self.m.parts))];h=anatomy.IceDancersTests();h.models={'jynx':dict(primitives=ps)};h.test_neck_head_hair_crown_and_clothes_have_real_roots();h.test_both_arms_hands_and_every_digit_have_real_roots();h.test_lips_are_rooted_and_leave_a_real_recessed_open_center()
 def test_normal_local_pack_palette(self):
  for p,n in zip(self.m.parts,self.m.names):
   rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /')) else (255,255,255) if n.startswith(('Eye /','Hair','Back hair strand /','Chest trim /','Chest bridge /','Sleeve /')) else (239,49,156) if n.startswith(('Gown /','Bodice /','Lips /')) else (115,16,123)
   color=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,c in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(c,expected,places=7,msg=n)
 def test_neutral_endpoints_and_rotation_only_source_cues(self):
  for clip in self.m.clips:
   for t in (0.,1.):
    for matrix in self.m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  for animation in self.m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
 def test_mutated_geometry_bytes_species_rejected_and_smoochum_remains_canonical(self):
  m=copy.deepcopy(self.m.neutral_model());m['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_jynx(m)
  b=bytearray(self.blob);b[-8]^=1
  with self.assertRaises(ValueError):author.decode_jynx(bytes(b),'jynx')
  with self.assertRaises(ValueError):author.export_jynx_model('smoochum')
  from smoochum_glb import export_smoochum
  p=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/smoochum.glb';blob=p.read_bytes();self.assertEqual(blob,export_smoochum())
if __name__=='__main__':unittest.main()
