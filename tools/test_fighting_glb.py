#!/usr/bin/env python3
"""Decoded fighting-family support, arm-chain, fitted-paint and pose regressions."""
import copy,math,unittest
from pathlib import Path
import fighting_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class FightingSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,m,prefix,ending=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending))
 def test_canonical_limits_weights_hierarchy_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_fighting(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertEqual(len(m.joints),18 if s=='machamp' else 13 if s=='machop' else 12)
   self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_fighting(self.blobs[s],s)),author.REST_DIGESTS[s])
   self.assertEqual(m.parents[:6],[None,0,1,2,0,0])
   for i in range(6,12 if s!='machamp' else 18,3):self.assertEqual(m.parents[i:i+3],[1,i,i+1])
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(m.joints))))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   bad=copy.deepcopy(author.authoring_model(s));bad['primitives'][0]['positions'][0]+=.01
   with self.assertRaises(ValueError):author.export_fighting(s,bad)
 def test_anatomical_owners_keep_toes_planted_and_each_fist_independent(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith(('Foot /','Toe /')):self.assertEqual(owners,{0})
    elif n.startswith(('Eye /','Mouth /','Brow /','Crest /')):self.assertEqual(owners,{3})
    elif n.startswith(('Hand /','Finger /','Thumb /')):
     side=n.split()[-2] if n.startswith('Finger /') else n.split()[-1];row=2 if ' / lower ' in n else 0;self.assertEqual(owners,{8+3*(row+int(side=='1'))})
    elif n.startswith('Leg /'):self.assertTrue(owners<={0,1,4 if n.endswith('-1') else 5})
    elif n.startswith('Tail /'):self.assertEqual(owners,{12})
 def test_limb_finger_crest_tail_and_foot_roots_remain_occupied(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];body=self.find(m,'Body /');pairs=[]
   for i,n in enumerate(m.names):
    if n.startswith(('Arm /','Leg /','Crest /','Tail /')):pairs.append((body,i))
    elif n.startswith(('Hand /','Finger /','Thumb /')):
     row='lower' if ' / lower ' in n else 'upper';side=n.split()[-2] if n.startswith('Finger /') else n.split()[-1]
     parent=self.find(m,'Arm /',[side]) if row=='upper' else self.find(m,'Arm / lower',[side])
     if not n.startswith('Hand /'):parent=self.find(m,'Hand / '+row,[side])
     pairs.append((parent,i))
    elif n.startswith('Foot /'):pairs.append((self.find(m,'Leg /',[n.split()[-1]]),i))
    elif n.startswith('Toe /'):pairs.append((self.find(m,'Foot /',[n.split()[-2]]),i))
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[self.find(m,'Arm /')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[body]))
 def test_floor_winding_strain_endpoints_and_isolated_joint_motion(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Toe /')):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.72<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.35,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   rotations=m.rotations('attack',.31)
   for joint in range(1,len(m.joints)):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,joint))
 def test_complete_shoulder_and_hip_caps_stay_buried_in_the_torso(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];body=self.find(m,'Body /')
   for clip in m.clips:
    mats=m.matrices(clip,.31);parent=dict(ps[body],positions=sum(m.posed(body,mats),[]))
    for i,n in enumerate(m.names):
     if not n.startswith(('Arm /','Leg /')):continue
     count=24 if n.startswith('Arm /') else 14;points=m.posed(i,mats)
     for j in set(m.indices[i][:(count-2)*3]):
      x,y,z=points[j];hits=rays(parent,x,y);self.assertTrue(hits,(s,clip,n));self.assertTrue(min(hits)<z<max(hits),(s,clip,n))
 def test_fitted_gaze_and_machoke_bands_clear_actual_parent_during_motion(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives']
   for i,n in enumerate(m.names):
    if n.startswith('Eye / fitted'):parent=self.find(m,'Body /')
    elif n.startswith('Eye / narrow'):parent=self.find(m,'Eye / fitted',[n.split()[-1]])
    elif n.startswith('Arm marking /'):parent=self.find(m,'Arm /',[n.split()[-2]])
    else:continue
    for clip in m.clips:
     mats=m.matrices(clip,.31);posed_parent=dict(ps[parent],positions=sum(m.posed(parent,mats),[]));vs=m.posed(i,mats)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];rest=[m.vertices[i][j] for j in tri];a,b,c=rest
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
       x,y,z=[sum(w*vs[j][axis] for j,w in zip(tri,weights)) for axis in range(3)];hits=rays(posed_parent,x,y)
       self.assertTrue(hits,(s,n));self.assertGreater(z-max(hits),0.,(s,clip,n))
 def test_trunks_and_waist_bands_cover_the_torso_front_and_back_in_motion(self):
  for s,m in self.models.items():
   if s=='machop':continue
   ps=m.neutral_model()['primitives'];body=self.find(m,'Body /')
   for clip in m.clips:
    mats=m.matrices(clip,.31);parent=dict(ps[body],positions=sum(m.posed(body,mats),[]))
    for i,n in enumerate(m.names):
     if not n.startswith(('Trunks /','Belt / fitted')):continue
     points=m.posed(i,mats)
     for k in range(0,len(m.indices[i]),3):
      ids=m.indices[i][k:k+3];a,b,c=[m.vertices[i][j] for j in ids];area=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
      u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],area)
      if abs(area)<.65*math.sqrt(sum(q*q for q in normal)):continue
      x,y,z=[sum(points[j][axis] for j in ids)/3 for axis in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,n))
      self.assertGreater(z-max(hits) if area>0 else min(hits)-z,0.,(s,clip,n))
 def test_clips_keep_source_cue_timing_and_rotation_only_contract(self):
  for s,m in self.models.items():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle)
    self.assertTrue(all(c['target']['path']=='rotation' and c['target']['node']!=0 for c in clip['channels']))

if __name__=='__main__':unittest.main()
