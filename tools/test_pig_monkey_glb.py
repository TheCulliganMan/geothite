"""Decoded monkey skins: fixed paws, coherent faces/fists and physical fur/limb roots."""
import copy,math,os,unittest
from pathlib import Path
import pig_monkey_glb as author
import test_pig_monkey_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_PIG_MONKEY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class PigMonkeySkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_canonical_f32_geometry_logical_ranges_and_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_pig_monkey(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_pig_monkey(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   self.assertEqual((len(m.parts),len(self.runtime[s].parts),len(m.joints)),(52,38,12) if s=='mankey' else (59,45,10))
 def test_hierarchy_inverse_binds_and_all_used_controls(self):
  for s,m in self.models.items():
   self.assertEqual(m.parents[:10],[None,0,1,2,1,4,1,1,0,0])
   if s=='mankey':self.assertEqual(m.parents[10:12],[1,10])
   self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(len(m.joints))))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_pelt_face_fists_and_fixed_digit_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Toe /')) else 1 if n.startswith(('Body /','Fur /','Belly /','Eye socket /','Eye /','Pupil /','Snout /','Nostril /')) else None
    if n.startswith(('Hand /','Thumb /','Knuckle /','Wrist cuff /')):
     side=int(n.split()[-2] if n.startswith('Knuckle /') else n.split()[-1]);owner=3 if side==-1 else 5
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n))
 def test_each_control_moves_and_side_controls_do_not_drag_neighbors(self):
  for s,m in self.models.items():
   for j in range(1,len(m.joints)):
    qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,(s,j))
    if j in (3,5):
     side=-1 if j==3 else 1
     for i,n in enumerate(m.names):
      own=n.startswith(('Arm /','Hand /','Thumb /','Knuckle /','Wrist cuff /')) and int(n.split()[-2] if n.startswith('Knuckle /') else n.split()[-1])==side
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
 def test_actual_fur_ear_fist_toe_cuff_and_tail_roots_survive_poses(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     mats=m.matrices(clip,t);ps=[self.part(m,i,mats) for i in range(len(m.parts))];h=anatomy.PigMonkeyTests();h.models={s:dict(primitives=ps)};h.test_snout_limbs_fists_thumbs_knuckles_feet_toes_and_cuffs_are_rooted();h.m=h.models[s];h.ps=ps;body=h.find('Body /')
     for p in ps:
      if p['part'].startswith(('Fur /','Ear /')):self.assertTrue(h.overlap(p,body),(s,clip,t,p['part']))
 def test_normal_local_pack_four_color_palette(self):
  for s,m in self.models.items():
   rgb=((255,255,255),(0,0,0),(239,165,90),(156,74,57)) if s=='mankey' else ((255,255,255),(0,0,0),(255,148,99),(132,66,41));colors=[]
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
   with self.assertRaises(ValueError):author.export_pig_monkey(s,model)
   blob=bytearray(self.blobs[s]);blob[-8]^=1
   with self.assertRaises(ValueError):author.decode_pig_monkey(bytes(blob),s)
  with self.assertRaises(ValueError):author.export_pig_monkey('rattata')
if __name__=='__main__':unittest.main()
