"""Decoded raptor skins: rooted wing banks, coherent gaze, necks and planted digits."""
import copy,importlib.util,json,math,os,struct,sys,tempfile,unittest
from pathlib import Path
import raptor_bird_glb as author
import test_raptor_bird_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_RAPTOR_BIRD_SKIN_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class RaptorSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def posed(self,m,mats):return dict(primitives=[dict(part=n,positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor']) for i,n in enumerate(m.names)])
 def test_exact_authored_geometry_ranges_and_existing_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_raptor_bird(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_raptor_bird(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.runtime[s].parts),64)
   self.assertEqual(len(m.parts),105 if s=='pidgeot' else 101);self.assertEqual(len(m.joints),20 if s=='pidgeot' else 19)
 def test_original_neutral_bounds_and_uniform_physical_span(self):
  for s,m in self.models.items():
   self.assertEqual(min(v[1] for vs in m.vertices for v in vs),0.);self.assertAlmostEqual(max(v[1] for vs in m.vertices for v in vs),author.HEIGHTS[s],places=6)
   self.assertEqual(m.doc['extras']['reference_span'],author.HEIGHTS[s])
 def test_hierarchy_inverse_binds_and_used_controls(self):
  for s,m in self.models.items():
   self.assertEqual(m.parents[:len(m.joints)],[None,0,1,2,3,4,1,6,1,8]+[1]*5+([0,0,4,4,4] if s=='pidgeot' else [1,1,4,2]))
   self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(0 if s=='pidgeot' else 1,len(m.joints))))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_head_facial_layers_bill_and_fixed_digits(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Toe /','Talon /')) else 4 if n.startswith(('Head /','Cheek /','Mask /','Eye /','Eye socket /','Pupil /','Glint /')) else 5 if n.startswith('Bill /') and any(k in n for k in ('lower','mandible','seam')) else 4 if n.startswith('Bill /') else None
    if s=='fearow' and n.startswith(('Foot /','Toe /','Talon /','Leg /')):owner=15 if sum(v[0] for v in m.vertices[i])<0 else 16
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n))
 def test_every_control_moves_and_independent_wrist_leaves_opposite_bank_fixed(self):
  for s,m in self.models.items():
   for j in range(1,len(m.joints)):
    qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,(s,j))
    if j in (7,9):
     for i,n in enumerate(m.names):
      if n.startswith(('Wing /','Primary /','Rear primary /','Covert /','Rear covert /')) and (sum(v[0] for v in m.vertices[i])>0)==(j==7):self.assertLess(movement[i],1e-7,(s,j,n))
 def test_planted_claws_positive_winding_and_bounded_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     mats=m.matrices(clip,t)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if s=='pidgeot' and m.names[i].startswith(('Foot /','Toe /','Talon /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,m.names[i],ratio))
 def test_actual_neck_head_bill_crest_mane_wing_tail_and_digit_roots(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     h=anatomy.RaptorBirdTests();h.ms={s:self.posed(m,m.matrices(clip,t))}
     h.test_neck_head_bill_crest_and_mane_have_physical_support();h.test_all_wing_layers_and_rear_tail_feathers_are_rooted()
     # Feet routine enumerates both species; independently use the current one.
     for sign in (-1,1):
      leg=h.find(s,'Leg /',sign);foot=h.find(s,'Foot /',sign);self.assertTrue(h.overlap(leg,h.find(s,'Body /')));self.assertTrue(h.overlap(leg,foot))
 def test_fitted_face_breast_and_feather_tips_keep_surface_support_in_attack(self):
  for s,m in self.models.items():
   for t in (.18,.55):
    mats=m.matrices('attack',t);model=self.posed(m,mats)
    # Front-heightfield oracles must follow each parent region's rigid frame.
    # Keep all differential neck/wrist skinning after removing that common
    # motion; occupied-volume joins are checked separately in world space.
    for p in model['primitives']:
     n=p['part'];j=4 if n.startswith(('Head /','Cheek /','Mask /','Eye /','Eye socket /','Pupil /','Glint /')) else 2 if n.startswith(('Neck /','Throat /')) else 6 if n.startswith(('Wing /','Wing field /','Primary /','Rear primary /','Covert /','Rear covert /','Primary tip /','Rear primary tip /')) and sum(p['positions'][::3])<0 else 8 if n.startswith(('Wing /','Wing field /','Primary /','Rear primary /','Covert /','Rear covert /','Primary tip /','Rear primary tip /')) else 1
     frame=mats[j];p['positions']=[sum(frame[4*i+k]*(q[k]-frame[12+k]) for k in range(3)) for q in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for i in range(3)]
    h=anatomy.RaptorBirdTests();h.ms={s:model}
    h.test_fitted_faces_breasts_and_both_sides_of_pale_tips_have_mesh_support()
    rest=anatomy.RaptorBirdTests();rest.ms={s:m.neutral_model()};original={p['part']:p for _,p,parent in rest.pairs()}
    for _,p,parent in h.pairs():
     source=original[p['part']]
     for k in range(0,len(p['indices']),3):
      ids=p['indices'][k:k+3];a,b,c=[source['positions'][3*i:3*i+3] for i in ids]
      # Select actual authored front caps. A buried sidewall may acquire a
      # positive XY area when the neck bends; it must remain a rooted wall,
      # rather than being mistaken for a newly exposed front marking.
      u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
      if normal[2]<=.5*math.sqrt(sum(x*x for x in normal)):continue
      a,b,c=[p['positions'][3*i:3*i+3] for i in ids]
      for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
       hits=anatomy.ray_intersections(parent,q[0],q[1]);self.assertTrue(hits,(s,p['part'],k//3));self.assertGreater(q[2]-max(hits),.0001,(s,p['part'],k//3))
 def test_fearow_open_bill_survives_source_cues(self):
  m=self.models['fearow'];h=anatomy.RaptorBirdTests()
  for clip in m.clips:
   for t in (.18,.30,.55,.80):
    mats=m.matrices(clip,t);parts=self.posed(m,mats)['primitives'];head=mats[4]
    # The clearance oracle casts along the authored bill axis. Undo only the
    # decoded rigid head transform; retain the lower mandible's actual hinge
    # motion and all other posed geometry that could fill the mouth.
    for p in parts:
     p['positions']=[sum(head[4*i+j]*(q[j]-head[12+j]) for j in range(3)) for q in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for i in range(3)]
    self.assertTrue(h.bill_gap(parts),(clip,t))
 def test_neutral_endpoints_and_source_relative_rotation_only_cues(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   for animation in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
 def test_public_selection_duplicate_rejection_and_exact_neutral_stl(self):
  from battle_model_assets import species_paths,read_species_model
  spec=importlib.util.spec_from_file_location('raptor_stl',Path(__file__).with_name('export-model-stl.py'));stl=importlib.util.module_from_spec(spec);sys.modules[spec.name]=stl;spec.loader.exec_module(stl)
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);folder=root/'battle_species';folder.mkdir()
   for s in author.SPECIES:
    path=folder/(s+'.glb');path.write_bytes(self.blobs[s]);self.assertEqual(species_paths(folder)[s],path)
    model=read_species_model(path);self.assertEqual(author.geometry_digest(model),author.REST_DIGESTS[s]);reader=stl.ModelReader(root);canonical=reader.read(path)
    neutral=root/(s+'.mesh.json');neutral.write_text(json.dumps(model));self.assertEqual(stl.encode_stl(canonical),stl.encode_stl(reader.read(neutral)))
    duplicate=folder/(s+'.mesh.json');duplicate.write_text(json.dumps(model))
    with self.assertRaises(ValueError):species_paths(folder)
    duplicate.unlink()
 def test_geometry_and_malformed_binary_rejected(self):
  for s,m in self.models.items():
   bad=copy.deepcopy(m.neutral_model());bad['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_raptor_bird(s,bad)
   for b in (self.blobs[s][:-4],b'bad',self.blobs[s]+b' '):
    with self.assertRaises(ValueError):author.decode_raptor_bird(b,s)
if __name__=='__main__':unittest.main()
