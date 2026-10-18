"""Decoded runner skins: independent necks, coherent gaze and true lower-bill hinges."""
import copy,math,os,unittest
from pathlib import Path
import dodo_glb as author
import test_dodo_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_DODO_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class DodoSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_canonical_f32_geometry_logical_ranges_and_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_dodo(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_dodo(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   self.assertEqual((len(m.parts),len(self.runtime[s].parts),len(m.joints)),(37,19,12) if s=='doduo' else (61,32,19))
 def test_hierarchy_inverse_binds_and_all_used_controls(self):
  for s,m in self.models.items():
   self.assertEqual(m.parents[:len(m.joints)],[None,0,1,2,3,4,1,6,7,8,0,0] if s=='doduo' else [None,0,1,2,3,4,1,6,7,8,1,10,11,12,0,0,1,1,1])
   self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(len(m.joints))))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_separate_head_gaze_crests_jaws_and_fixed_claws(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Toe /','Claw /','Hind toe /','Hind claw /')) else 1 if n.startswith(('Body /','Underbelly /')) else 4+4*int(n.split()[-2]) if n.startswith(('Eye /','Pupil /','Glint /','Crest /')) else 4+4*int(n.split()[-1]) if n.startswith(('Head /','Upper beak /','Mouth /')) else 5+4*int(n.split()[-1]) if n.startswith(('Lower beak /','Tongue /')) else None
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n))
 def test_each_control_moves_and_one_head_does_not_drag_other_heads(self):
  for s,m in self.models.items():
   for j in range(1,len(m.joints)):
    qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,(s,j))
    if j in [4+4*k for k in range(2 if s=='doduo' else 3)]:
     head=(j-4)//4
     for i,n in enumerate(m.names):
      if n.startswith('Head /') and int(n.split()[-1])!=head:self.assertLess(movement[i],1e-7,(s,j,n))
 def test_fixed_clawed_feet_positive_winding_and_bounded_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,.18,.30,.55,.80,1.):
     mats=m.matrices(clip,t)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if m.names[i].startswith(('Foot /','Toe /','Claw /','Hind toe /','Hind claw /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,m.names[i],ratio))
 def test_actual_head_neck_bill_crest_leg_digit_and_fan_roots_survive_poses(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     mats=m.matrices(clip,t);ps=[self.part(m,i,mats) for i in range(len(m.parts))];h=anatomy.DodoTests();h.models={s:dict(primitives=ps)};h.test_heads_necks_beaks_crests_and_tail_have_actual_solid_roots();h.test_long_legs_join_feet_and_all_clawed_digits()
 def test_jaw_hinges_follow_authored_yaw_axes_and_leave_other_geometry_fixed(self):
  for s,m in self.models.items():
   for head in range(2 if s=='doduo' else 3):
    joint=5+4*head;qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[joint]=m.rotations('attack',.30)[joint];mats=m.matrices(rotations=qs);q=qs[joint]
    yaw=anatomy.POSES[s][head][3];v=math.sqrt(sum(x*x for x in q[:3]));self.assertGreater(v,.005);self.assertAlmostEqual(q[0]/v,math.cos(yaw),places=6);self.assertAlmostEqual(q[1],0.,places=7);self.assertAlmostEqual(q[2]/v,-math.sin(yaw),places=6)
    for i,n in enumerate(m.names):
     own=n.startswith(('Lower beak /','Tongue /')) and int(n.split()[-1])==head
     movement=max(math.dist(a,b) for a,b in zip(m.vertices[i],m.posed(i,mats)))
     if own:self.assertGreater(movement,.0001,(s,head,n))
     else:self.assertLess(movement,1e-7,(s,head,n))
 def test_normal_local_pack_four_color_palette(self):
  for s,m in self.models.items():
   rgb=((255,255,255),(0,0,0),(173,107,74),(99,66,49)) if s=='doduo' else ((255,255,255),(0,0,0),(214,132,66),(148,66,49));colors=[]
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
   with self.assertRaises(ValueError):author.export_dodo(s,model)
   blob=bytearray(self.blobs[s]);blob[-8]^=1
   with self.assertRaises(ValueError):author.decode_dodo(bytes(blob),s)
  with self.assertRaises(ValueError):author.export_dodo('rattata')
if __name__=='__main__':unittest.main()
