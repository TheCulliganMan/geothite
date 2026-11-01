"""Decoded feline skins: coherent gaze, whiskers and charms, planted paws and rooted limbs."""
import copy,math,os,unittest
from pathlib import Path
import feline_glb as author
import test_feline_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_FELINE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class FelineSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_canonical_f32_geometry_logical_ranges_and_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_feline(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_feline(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   self.assertEqual((len(m.parts),len(self.runtime[s].parts),len(m.joints)),(51,39,15) if s=='meowth' else (48,36,13))
 def test_hierarchy_inverse_binds_and_all_used_controls(self):
  for s,m in self.models.items():
   self.assertEqual(m.parents[:len(m.joints)],[None,0,1,2,3,3,1,6,1,8,0,0,1,12,13] if s=='meowth' else [None,0,1,2,3,3,0,0,0,0,1,10,11])
   self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(len(m.joints))))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_head_face_whiskers_charms_and_fixed_paws(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Toe /')) else 1 if n.startswith('Body /') else 3 if n.startswith(('Head /','Eye socket /','Eye /','Pupil /','Muzzle /','Whisker /','Brow whisker /','Nose /','Mouth /','Fang /','Coin /','Coin groove /','Gem /','Gem glint /')) else None
    if n.startswith(('Hand /','Finger /')):
     side=int(n.split()[-2] if n.startswith('Finger /') else n.split()[-1]);owner=7 if side==-1 else 9
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n))
 def test_each_control_moves_and_palms_do_not_drag_head_or_opposite_arm(self):
  for s,m in self.models.items():
   for j in range(1,len(m.joints)):
    qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,(s,j))
    if s=='meowth' and j in (7,9):
     side=-1 if j==7 else 1
     for i,n in enumerate(m.names):
      own=n.startswith(('Arm /','Hand /','Finger /')) and int(n.split()[-2] if n.startswith('Finger /') else n.split()[-1])==side
      if not own:self.assertLess(movement[i],1e-7,(s,j,n))
 def test_fixed_paws_positive_winding_and_bounded_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,.18,.30,.55,.80,1.):
     mats=m.matrices(clip,t)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if m.names[i].startswith(('Foot /','Toe /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,m.names[i],ratio))
 def test_actual_neck_ear_whisker_charm_paw_and_tail_roots_survive_poses(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     mats=m.matrices(clip,t);ps=[self.part(m,i,mats) for i in range(len(m.parts))];h=anatomy.FelineTests();h.models={s:dict(primitives=ps)};h.test_every_actual_anatomical_root_overlaps_parent_volume()
 def test_normal_local_pack_four_color_palette(self):
  for s,m in self.models.items():
   rgb=((255,255,255),(0,0,0),(255,222,107),(165,66,24)) if s=='meowth' else ((255,255,255),(0,0,0),(255,230,82),(99,74,33));colors=[]
   for color in rgb:
    colors.append([v/255/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in color])
   for p in m.parts:
    value=m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor'][:3];self.assertTrue(any(max(abs(a-b) for a,b in zip(value,c))<1e-7 for c in colors),(s,value))
 def test_neutral_endpoints_rotation_only_source_cues(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   for animation in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
 def test_mutated_geometry_bytes_species_rejected(self):
  for s,m in self.models.items():
   model=copy.deepcopy(m.neutral_model());model['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_feline(s,model)
   blob=bytearray(self.blobs[s]);blob[-8]^=1
   with self.assertRaises(ValueError):author.decode_feline(bytes(blob),s)
  with self.assertRaises(ValueError):author.export_feline('rattata')
if __name__=='__main__':unittest.main()
