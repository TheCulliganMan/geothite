"""Independent canonical-skin checks for Pidgeotto's full production rig."""
import copy,math,os,struct,unittest
from pathlib import Path
import pidgeotto_skin_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY,values
from pidgeotto_contours import components
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(os.environ['CRYSTAL_PIDGEOTTO_SKIN_TEST_MODELS']) if 'CRYSTAL_PIDGEOTTO_SKIN_TEST_MODELS' in os.environ else None
class PidgeottoSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'pidgeotto.glb').read_bytes() if ROOT else author.export_pidgeotto_skin('pidgeotto');cls.m=Model(cls.blob);cls.original=author.original_model(author.authoring_model('pidgeotto'))
 def test_exact_original_neutral_anatomy_and_existing_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_pidgeotto_skin('pidgeotto'));self.assertEqual(len(m.parts),34);self.assertEqual(len(m.joints),14);self.assertEqual(sum(map(len,m.vertices)),6293);self.assertEqual(sum(len(i)//3 for i in m.indices),5776)
  self.assertEqual(author.geometry_digest(self.original),author.ORIGINAL_REST_DIGEST)
  for index,(expected,actual) in enumerate(zip(self.original['primitives'],m.neutral_model()['primitives'])):
   self.assertEqual(expected['part'],actual['part'])
   for k,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
    self.assertEqual(author.packed(expected[k],c),author.packed(actual[k][:len(expected[k])],c))
    if index not in (14,23) or k=='base_color':self.assertEqual(len(actual[k]),len(expected[k]))
  for axis in range(3):
   old=[v for p in self.original['primitives'] for v in p['positions'][axis::3]];new=[p[axis] for ps in m.vertices for p in ps];self.assertEqual((min(old),max(old)),(min(new),max(new)))
  self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['pidgeotto']);self.assertLessEqual(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
 def test_stored_components_are_closed_outward_with_real_shading(self):
  models={}
  for p in self.m.neutral_model()['primitives']:
   for j,c in enumerate(components(p)):models[p['part']+'/'+str(j)]={'primitives':[c]}
  assert_closed_models(self,models);assert_model_shading(self,models)
 def test_hierarchy_inverse_binds_and_used_controls(self):
  m=self.m;self.assertEqual(m.parents[:14],[None,0,1,1,3,1,5,0,0,1,9,9,9,2]);self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(14)))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_exact_existing_shoulder_pivots_and_idle_wing_keys(self):
  import animated_glb as original
  m=self.m;groups,pivots=original.pidgeotto_groups(self.original);times,tracks=original.pidgeotto_idle_samples();clip=m.doc['animations'][0]
  self.assertEqual(clip['name'],'pidgeotto.idle')
  for j,side,track in ((3,'l',tracks[0]),(5,'r',tracks[1])):
   inverse=m.inverse[j];self.assertEqual([-inverse[i] for i in (12,13,14)],pivots[side]);channel=next(c for c in clip['channels'] if c['target']['node']==j);sampler=clip['samplers'][channel['sampler']]
   self.assertEqual(struct.pack('<65f',*values(m.doc,m.binary,sampler['input'])),struct.pack('<65f',*times));self.assertEqual(struct.pack('<260f',*values(m.doc,m.binary,sampler['output'])),struct.pack('<260f',*track))
   for t in (.125,.25,.5,.75,.875):
    mats=m.matrices('idle',t);angle=(1 if side=='l' else -1)*.42*math.sin(math.tau*t);c=math.cos(angle);sn=math.sin(angle);px,py,pz=pivots[side]
    for i in groups[side]:
     for a,b in zip(m.vertices[i][:len(self.original['primitives'][i]['positions'])//3],m.posed(i,mats)):
      expected=(px+c*(a[0]-px)-sn*(a[1]-py),py+sn*(a[0]-px)+c*(a[1]-py),a[2]);self.assertLess(math.dist(expected,b),1e-7)
 def test_coherent_head_eyes_beak_cheek_masks_and_planted_claws(self):
  m=self.m
  for i,n in enumerate(m.names):
   if n=='Bird body':self.assertTrue(all(js==[1,0,0,0] for js in m.ids[i]))
   if n=='Bird head' or n.startswith('Eye ') or n=='Beak':self.assertTrue(all(js==[2,0,0,0] for js in m.ids[i]),n)
   if n.startswith('Toe'):self.assertTrue(all(js==[0,0,0,0] for js in m.ids[i]),n)
  i=m.names.index('Light breast');model=m.neutral_model();masks=author.face_mask_points(model);self.assertTrue(masks)
  for p,js in zip(m.vertices[i],m.ids[i]):self.assertEqual(js,[2 if tuple(p) in masks else 1,0,0,0])
 def test_each_control_moves_and_head_wrist_tail_controls_are_independent(self):
  m=self.m
  for joint in range(1,14):
   qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[joint]=m.rotations('attack',.30)[joint];mats=m.matrices(rotations=qs);motion=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(motion),.0001,joint)
   if joint in (4,6):
    for i,n in enumerate(m.names):
     own=n.startswith(('Layered wing','Flight feather')) and (sum(p[0] for p in m.vertices[i])<0)==(joint==4)
     if not own:self.assertLess(motion[i],1e-7,(joint,n))
   if joint==2:
    for i,n in enumerate(m.names):
     if n.startswith(('Layered wing','Flight feather','Tail feather','Toe','Bird shin')):self.assertLess(motion[i],1e-7,n)
 def test_all_clips_keep_floor_positive_winding_and_bounded_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.125,.25,.30,.55,.75,.875,1.):
    mats=m.matrices(clip,t)
    for i,vs in enumerate(m.vertices):
     ps=m.posed(i,mats);ns=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in ps),-1e-7,(clip,t,m.names[i]))
     if m.names[i].startswith('Toe'):self.assertEqual(ps,vs)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(ps,ns,tri);self.assertGreater(cosine,0.,(clip,t,m.names[i],k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.80<math.dist(ps[a],ps[b])/math.dist(vs[a],vs[b])<1.20,(clip,t,m.names[i]))
 def test_endpoints_and_rotation_only_source_relative_cues(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  for clip in m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'});self.assertEqual(clip['extras'],{'loopSuggested':idle,'cueRelative':not idle})
 def test_real_occupied_head_wing_tail_crest_breast_and_foot_roots_survive_motion(self):
  from sculpt_test_support import parts_overlap
  m=self.m;names=m.names
  pairs=[('Bird head','Bird body'),('Light breast','Bird body'),('Layered wing','Bird body'),('Layered wing.001','Bird body')]+[(n,'Bird body') for n in names if n.startswith(('Tail feather','Bird shin'))]+[(n,'Bird head') for n in names if n.startswith('Head crest')]+[(n,'Bird shin' if j<3 else 'Bird shin.001') for j,n in enumerate(n for n in names if n.startswith('Toe'))]
  for clip in m.clips:
   for t in (0.,.18,.30,.55,.80):
    mats=m.matrices(clip,t);ps={n:dict(part=n,positions=sum(m.posed(i,mats),[]),normals=sum(m.posed(i,mats,True),[]),indices=m.indices[i],base_color=[1.,1.,1.,1.]) for i,n in enumerate(names)}
    for a,b in pairs:self.assertTrue(parts_overlap(ps[a],ps[b]),(clip,t,a,b))
    for i in (14,23):
     p=ps[names[i]];source=self.original['primitives'][i];n=len(source['positions'])//3;k=len(source['indices']);old=dict(p,positions=p['positions'][:3*n],normals=p['normals'][:3*n],indices=p['indices'][:k]);cover=dict(p,positions=p['positions'][3*n:],normals=p['normals'][3*n:],indices=[j-n for j in p['indices'][k:]])
     self.assertTrue(parts_overlap(cover,ps['Bird body']),(clip,t,'rooted shoulder cover'));self.assertTrue(parts_overlap(cover,old),(clip,t,'cover reaches original wing'))
    for c in components(ps['Light breast']):
     if min(c['positions'][1::3])>.65:self.assertTrue(parts_overlap(c,ps['Bird head']),(clip,t,'cheek mask'))
 def test_detached_head_wing_and_tail_negative_controls_are_rejected(self):
  from sculpt_test_support import parts_overlap
  from sculpt_geometry import translate
  ps={p['part']:p for p in self.m.neutral_model()['primitives']}
  for child in ('Bird head','Layered wing','Tail feather'):self.assertFalse(parts_overlap(translate(copy.deepcopy(ps[child]),3.,0.,0.),ps['Bird body']))
 def test_decode_is_canonical_and_malformed_bytes_are_rejected(self):
  decoded=author.decode_pidgeotto_skin(self.blob);self.assertEqual(author.geometry_digest(decoded),author.REST_DIGESTS['pidgeotto']);self.assertEqual(author.geometry_digest(author.original_model(decoded)),author.ORIGINAL_REST_DIGEST);self.assertEqual(author.export_pidgeotto_skin('pidgeotto',decoded),self.blob)
  model=copy.deepcopy(self.original);model['primitives'][0]['positions'][0]+=.001
  with self.assertRaises(ValueError):author.export_pidgeotto_skin('pidgeotto',model)
  changed=bytearray(self.blob);changed[-8]^=1
  for b in (bytes(changed),self.blob[:-1],self.blob+b'\0',b'bad',b'\0'*(1024*1024+1)):
   with self.assertRaises(ValueError):author.decode_pidgeotto_skin(b)
if __name__=='__main__':unittest.main()
