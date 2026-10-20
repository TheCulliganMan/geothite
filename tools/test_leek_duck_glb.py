"""Decoded duck skin: exact anatomy, real grip contact, independent carry chain and fixed webs."""
import copy,importlib.util,json,math,os,struct,sys,tempfile,unittest
from pathlib import Path
import leek_duck_glb as author
import test_leek_duck_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('LEEK_DUCK_SKIN_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class LeekDuckSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'farfetch_d.glb').read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime)
 def posed(self,mats):
  m=self.m;return dict(name='farfetch_d',version=1,coordinate_system=author.COORDINATES,primitives=[dict(part=n,positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor']) for i,n in enumerate(m.names)])
 def test_exact_canonical_geometry_ranges_and_existing_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_leek_duck('farfetch_d'));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['farfetch_d']);self.assertEqual(author.geometry_digest(author.decode_leek_duck(self.blob,'farfetch_d')),author.REST_DIGESTS['farfetch_d']);self.assertEqual(len(m.parts),45);self.assertEqual(sum(map(len,m.vertices)),17863);self.assertEqual(sum(len(v)//3 for v in m.indices),26972);self.assertLessEqual(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.runtime.parts),64)
 def test_every_original_name_and_all_35_untouched_parts_preserved_bitwise(self):
  original=author.sculpture();new=self.m.neutral_model();self.assertEqual([p['part'] for p in original['primitives']],self.m.names);unchanged=0
  for old,p in zip(original['primitives'],new['primitives']):
   self.assertEqual(author.packed(old['base_color'],5126),author.packed(p['base_color'],5126),p['part'])
   if old['part'].startswith(('Pupil /','Glint /')):
    self.assertEqual(author.packed([v for i,v in enumerate(old['positions']) if i%3!=2],5126),author.packed([v for i,v in enumerate(p['positions']) if i%3!=2],5126),p['part']);self.assertEqual(old['indices'],p['indices'])
   if old['part'].startswith(('Eye border /','Eye white /','Brow /','Bill seam /','Pupil /','Glint /')):continue
   unchanged+=1
   for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):self.assertEqual(author.packed(old[key],c),author.packed(p[key],c),(p['part'],key))
  self.assertEqual(unchanged,35)
 def test_original_six_bounds_height_and_ground_exact(self):
  m=self.m
  for axis,(lo,hi) in enumerate(((-.5527104,.2499221),(0.,1.1),(-.3956783,.3328003))):self.assertEqual((min(p[axis] for vs in m.vertices for p in vs),max(p[axis] for vs in m.vertices for p in vs)),struct.unpack('<2f',struct.pack('<2f',lo,hi)))
  self.assertEqual(m.doc['extras']['reference_span'],1.10)
 def test_hierarchy_inverse_binds_and_all_controls_used(self):
  m=self.m;self.assertEqual(len(m.joints),21);self.assertEqual(m.parents[:21],[None,0,1,1,3,1,5,6,7,8,9,9,9,1,1,1,2,2,2,0,0]);self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(21)))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_head_bill_and_fixed_webs_and_folds(self):
  m=self.m
  for i,n in enumerate(m.names):
   j=0 if n.startswith(('Foot /','Web crease /')) else 2 if n.startswith(('Head /','Eye border /','Eye white /','Pupil /','Glint /','Brow /','Bill /','Bill seam /')) else 7 if n.startswith('Grip /') else None
   if j is not None:self.assertTrue(all(ids==[j,0,0,0] and ws==[1.,0.,0.,0.] for ids,ws in zip(m.ids[i],m.weights[i])),n)
 def test_each_control_moves_and_grip_chain_has_no_unrelated_drag(self):
  m=self.m
  for j in range(1,21):
   qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,j)
   if j in (4,7,8,9):
    for i,n in enumerate(m.names):
     if n.startswith(('Foot /','Web crease /','Head /','Tail /')):self.assertLess(movement[i],1e-7,(j,n))
     if j==8 and n.startswith(('Grip /','Holding feather /')):self.assertLess(movement[i],1e-7,(j,n))
     if j==4 and n.startswith('Leek '):self.assertLess(movement[i],1e-7,(j,n))
 def test_grounded_webs_positive_winding_and_bounded_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.30,.55,.80):
    mats=m.matrices(clip,t)
    for i,vs in enumerate(m.vertices):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,t,m.names[i]))
     if m.names[i].startswith(('Foot /','Web crease /')):self.assertEqual(posed,vs)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,m.names[i],k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(clip,t,m.names[i],ratio))
 def test_actual_body_head_wing_fingers_grip_stalk_leaf_tail_and_foot_roots(self):
  for clip in self.m.clips:
   for t in (.18,.30,.55,.80):
    h=anatomy.LeekDuckTests();h.m=self.posed(self.m.matrices(clip,t));h.ps=h.m['primitives'];h.test_body_head_bill_crests_wings_tail_and_feet_have_real_roots();h.test_leek_is_held_and_leaves_have_physical_stalk_roots();h.test_web_folds_are_supported_by_actual_upward_foot_surfaces()
 def test_fitted_face_markings_clear_parent_in_the_actual_head_frame(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t);model=self.posed(mats);head=mats[2]
    for p in model['primitives']:p['positions']=[sum(head[4*i+j]*(q[j]-head[12+j]) for j in range(3)) for q in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for i in range(3)]
    h=anatomy.LeekDuckTests();h.m=model;h.ps=model['primitives'];h.test_marking_vertices_follow_actual_parent_triangles();h.test_front_marking_triangles_clear_stored_support()
 def test_detached_stalk_and_leaf_negative_controls_fail_contact_oracle(self):
  for prefix in ('Leek stem /','Leek leaf /'):
   h=anatomy.LeekDuckTests();h.m=self.posed(self.m.matrices('attack',.30));h.ps=h.m['primitives'];p=h.find(prefix);p['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(p['positions'])]
   with self.assertRaises(AssertionError):h.test_leek_is_held_and_leaves_have_physical_stalk_roots()
 def test_neutral_endpoints_and_rotation_only_source_relative_cues(self):
  m=self.m
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
    path=folder/(s+'.glb');path.write_bytes(self.blob);self.assertEqual(species_paths(folder)[s],path)
    model=read_species_model(path);self.assertEqual(author.geometry_digest(model),author.REST_DIGESTS[s]);reader=stl.ModelReader(root);canonical=reader.read(path)
    neutral=root/(s+'.mesh.json');neutral.write_text(json.dumps(model));self.assertEqual(stl.encode_stl(canonical),stl.encode_stl(reader.read(neutral)))
    duplicate=folder/(s+'.mesh.json');duplicate.write_text(json.dumps(model))
    with self.assertRaises(ValueError):species_paths(folder)
    duplicate.unlink()
 def test_mutated_geometry_species_and_malformed_binary_are_rejected(self):
  bad=copy.deepcopy(self.m.neutral_model());bad['name']='farfetch_d';bad['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_leek_duck('farfetch_d',bad)
  with self.assertRaises(ValueError):author.export_leek_duck('pidgeot')
  for blob in (b'bad',self.blob[:-4],self.blob+b' '):
   with self.assertRaises(ValueError):author.decode_leek_duck(blob,'farfetch_d')
if __name__=='__main__':unittest.main()
