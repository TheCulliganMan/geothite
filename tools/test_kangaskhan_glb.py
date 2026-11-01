#!/usr/bin/env python3
"""Decoded mother/baby motion, open pouch containment, rim grips and fixed supports."""
import copy,math,unittest
from pathlib import Path
import kangaskhan_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
import test_kangaskhan_sculpt as sculpture_tests
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class KangaskhanSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'kangaskhan.glb').read_bytes();cls.runtime=skin.Model(cls.blob);cls.m=logical_model(cls.runtime);cls.ps=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending))
 def posed(self,mats):return [dict(p,positions=sum(self.m.posed(i,mats),[])) for i,p in enumerate(self.ps)]
 def fixture(self,parts):
  obj=sculpture_tests.KangaskhanTests();obj.ps=parts;obj.m={'primitives':parts};return obj
 def local(self,index,mats,owner):
  matrix=mats[owner];points=[]
  for p in self.m.posed(index,mats):
   delta=[p[k]-matrix[12+k] for k in range(3)];points += [sum(matrix[4*j+k]*delta[k] for k in range(3)) for j in range(3)]
  return dict(self.ps[index],positions=points)
 def test_exact_logical_geometry_lossless_packing_limits_and_identity_binds(self):
  m=self.m;r=self.runtime;self.assertEqual(self.blob,author.export_kangaskhan());self.assertFalse((ROOT/'kangaskhan.mesh.json').exists());self.assertEqual((len(m.parts),len(r.parts),len(m.joints)),(70,46,20))
  self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS['kangaskhan']);self.assertEqual(author.geometry_digest(r.neutral_model()),author.PACKED_REST_DIGESTS['kangaskhan']);self.assertEqual(author.geometry_digest(author.decode_parent(self.blob,'kangaskhan')),author.REST_DIGESTS['kangaskhan'])
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(m.parents[:20],[None,0,1,2,2,2,1,6,1,8,0,0,1,12,1,14,15,15,15,15]);self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(20)))
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  bad=copy.deepcopy(author.authoring_model());bad['primitives'][0]['positions'][0]+=.01
  with self.assertRaises(ValueError):author.export_kangaskhan(bad)
 def test_anatomical_owners_preserve_pouch_gaze_armor_and_independent_grips(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith(('Foot /','Foot claw /')):expected={0}
   elif n.startswith(('Body /','Belly /','Pouch /','Back plate /')):expected={1}
   elif n.startswith(('Jaw /','Tongue /')):expected={3}
   elif n.startswith('Ear /'):expected={4 if n.endswith('-1') else 5}
   elif n.startswith(('Shoulder /','Shoulder plate /')):expected={6 if n.endswith('-1') else 8}
   elif n.startswith(('Fist /','Hand claw /')):side=n.split()[-2] if n.startswith('Hand claw /') else n.split()[-1];expected={7 if side=='-1' else 9}
   elif n.startswith('Arm /'):self.assertTrue(owners<={6,7} if n.endswith('-1') else owners<={8,9});continue
   elif n.startswith(('Haunch /','Haunch armor outline /','Haunch plate /')):expected={10 if n.endswith('-1') else 11}
   elif n.startswith('Ankle /'):self.assertTrue(owners<={0,10 if n.endswith('-1') else 11});continue
   elif n.startswith('Tail /'):self.assertTrue(owners<={1,12,13});continue
   elif n.startswith('Baby body /'):expected={14}
   elif n.startswith('Baby neck /'):self.assertTrue(owners<={14,15});continue
   elif n.startswith(('Baby arm /','Baby hand /')):expected={16 if n.endswith('-1') else 17}
   elif n.startswith('Baby ear /'):expected={18 if n.endswith('-1') else 19}
   elif n.startswith('Baby '):expected={15}
   else:expected={2}
   self.assertEqual(owners,expected,n)
 def test_all_parent_baby_plate_claw_and_pouch_roots_survive_motion(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t);parts=self.posed(mats)
    for child,parent in self.fixture(parts).roots():
     if child['part'].startswith('Baby body /') and parent['part'].startswith('Pouch /'):
      # The seated bottom occupies the thin cup floor, below the first sample
      # of the generic volume grid. Test the actual shared occupied volume.
      point=transform(mats[1],author.normalized('kangaskhan',(0.,.2965,.340)))
      self.assertTrue(occupied(child,point));self.assertTrue(occupied(parent,point));continue
     self.assertTrue(parts_overlap(child,parent),(clip,t,child['part'],parent['part']))
  detached=copy.deepcopy(self.ps[self.find('Baby hand /')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,self.ps[self.find('Pouch /')]))
 def test_baby_remains_within_open_pouch_and_both_hands_keep_the_rim(self):
  m=self.m;baby=self.find('Baby body /');pouch=self.find('Pouch /');fixture=self.fixture(self.ps)
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);body=self.local(baby,mats,1);cup=self.local(pouch,mats,1);self.assertTrue(fixture.contained_baby(body,cup),(clip,step))
    for side in ('-1','1'):self.assertTrue(parts_overlap(self.local(self.find('Baby hand /',[side]),mats,1),cup),(clip,step,side))
    empty=author.normalized('kangaskhan',(0.,.610,.425));self.assertFalse(occupied(cup,empty));self.assertFalse(occupied(body,empty))
  bad=copy.deepcopy(self.ps[baby]);bad['positions']=[v+(.25 if i%3==2 else 0.) for i,v in enumerate(bad['positions'])];self.assertFalse(fixture.contained_baby(bad,self.ps[pouch]))
 def test_floor_neutral_endpoints_winding_strain_and_all_twenty_joint_carriers(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);floor=float('inf')
    for i,n in enumerate(m.names):
     points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(clip,n))
     if n.startswith(('Foot /','Foot claw /')):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    self.assertLess(abs(floor),1e-7)
    if step in (0,8):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  rotations=m.rotations('attack',.31)
  for joint in range(1,20):
   isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate);self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,joint)
 def test_actual_fitted_mother_baby_and_thigh_markings_clear_their_posed_parents(self):
  m=self.m;names={n:i for i,n in enumerate(m.names)}
  for clip in m.clips:
   mats=m.matrices(clip,.31)
   for child,parent in self.fixture(self.ps).pairs():
    i,j=names[child['part']],names[parent['part']];owner=m.ids[i][0][0];mark,support=self.local(i,mats,owner),self.local(j,mats,owner);vs=[mark['positions'][k:k+3] for k in range(0,len(mark['positions']),3)]
    for k in range(0,len(mark['indices']),3):
     tri=mark['indices'][k:k+3];a,b,c=[vs[q] for q in tri]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
     for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
      x,y,z=[sum(w*vs[q][axis] for q,w in zip(tri,weights)) for axis in range(3)];hits=rays(support,x,y);self.assertTrue(hits,child['part']);self.assertGreater(z-max(hits),.0001,(clip,child['part']))
 def test_mother_jaw_keeps_a_real_empty_mouth_through_motion(self):
  m=self.m;points=[author.normalized('kangaskhan',(x,y,z)) for x in (-.035,0.,.035) for y in (.902,.914) for z in (.390,.412)]
  for clip in m.clips:
   mats=m.matrices(clip,.31);parts=self.posed(mats)
   for p in points:self.assertFalse(any(occupied(part,transform(mats[2],p)) for part in parts),(clip,p))
 def test_clips_keep_source_cue_rotation_only_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertTrue(all(c['target']['path']=='rotation' and c['target']['node']!=0 for c in clip['channels']))

if __name__=='__main__':unittest.main()
