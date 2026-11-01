"""Independent armor carrier, hollow aperture, occupied root and floor skin checks."""
import copy,math,os,unittest
from pathlib import Path
import armored_bug_glb as author
import test_armored_bug_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(os.environ.get('CRYSTAL_ARMORED_BUG_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class ArmoredBugSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor'])
 def test_exact_source_packing_hierarchy_inverse_binds_and_original_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_armored_bug(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_armored_bug(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(len(m.joints),7);self.assertEqual(m.parents[:7],[p for n,p,v in author.JOINTS[s]])
   self.assertEqual(len(m.names),54 if s=='pineco' else 48);self.assertEqual(len(set(self.runtime[s].names)),len(self.runtime[s].names));self.assertTrue(all(0<len(n.encode())<=160 for n in self.runtime[s].names));self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_corner_normals(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_every_control_moves_and_complete_facial_armor_and_bores_share_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(7)));rotations=m.rotations('attack',.28)
   for j in range(1,7):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated);self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=None
    if s=='pineco':
     if n.startswith('Plate /'):owner=2+int(n.split()[-2])
     elif n.startswith('Crown /'):owner=5
     elif n.startswith('Stem /'):owner=6
     elif n.startswith('Eye /'):owner=1
    else:
     if n.startswith(('Core /','Belt /','Eye /')):owner=1
     elif n.startswith('Shell /'):owner=2 if 'upper' in n else 0
     elif n.startswith('Chip /'):owner=2 if n.split()[-3]=='True' else 0
     elif n.startswith(('Port /','Port recess /')):side,j=map(int,n.split()[-2:]);owner=3+(0 if side<0 else 2)+j
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),n);self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
 def test_fixed_floor_supports_positive_winding_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.28,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if s=='forretress' and (m.names[i].startswith('Shell / rocky lower') or m.names[i].startswith('Chip /') and m.names[i].split()[-3]=='False'):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.8<ratio<1.2,(s,clip,t,m.names[i],ratio))
     for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_real_shells_armor_roots_and_barrels_stay_occupied_through_poses(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.28,.46,.76):
     mats=m.matrices(clip,t);helper=anatomy.ArmoredBugTests();helper.models={s:dict(primitives=[self.part(m,i,mats) for i in range(len(m.parts))])};helper.test_hollow_shells_armor_and_barrels_have_actual_occupied_roots()
     if s=='pineco':helper.test_pineco_armor_shares_seams_and_is_rooted()
 def test_supported_gaze_stone_chips_and_unfilled_apertures_survive_articulation(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.28,.46,.76):
     mats=m.matrices(clip,t);parts=[]
     for i,n in enumerate(m.names):
      owner=1 if s=='pineco' and n.startswith('Body /') else m.ids[i][0][0]
      parts.append(inverse_rigid(self.part(m,i,mats),mats[owner]))
     helper=anatomy.ArmoredBugTests();helper.models={s:dict(primitives=parts)};helper.test_fitted_eyes_and_chips_follow_real_skin();helper.test_white_gaze_is_visible_ahead_of_the_core()
     if s=='forretress':helper.test_ports_and_eye_rims_have_real_unfilled_apertures();helper.test_stone_shell_poles_are_closed_and_equator_is_open()
 def test_every_clip_uses_source_cues_rotation_only_tracks_and_moving_upper_shell(self):
  for s,m in self.logical.items():
   for animation in m.doc['animations']:
    idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   early=author.pose_angles(s,'attack',.18);late=author.pose_angles(s,'attack',.85);self.assertGreater(sum(abs(v) for p in early for v in p),sum(abs(v) for p in late for v in p))
  self.assertLess(author.pose_angles('forretress','attack',.18)[2][0],-.04)
 def test_mutated_source_bytes_and_species_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_armored_bug(bad,s)
   with self.assertRaises(ValueError):author.decode_armored_bug(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_armored_bug(s,model)
if __name__=='__main__':unittest.main()
