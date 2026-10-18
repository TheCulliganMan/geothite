#!/usr/bin/env python3
"""Decoded fire-horse roots, coherent layered fire, hoof support and anatomical motion."""
import copy,math,unittest
from pathlib import Path
import fire_horse_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied
from test_ponyta_sculpt import unzy
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class FireHorseTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:skin.Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def find(self,m,prefix,ending=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending))
 def test_canonical_limits_lossless_ranges_and_identity_binds(self):
  for s,m in self.models.items():
   runtime=self.runtime[s];self.assertEqual(self.blobs[s],author.export_fire_horse(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertEqual((len(m.parts),len(runtime.parts),len(m.joints)),(47,40,14) if s=='ponyta' else (54,47,15))
   self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(runtime.neutral_model()),author.PACKED_REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_fire_horse(self.blobs[s],s)),author.REST_DIGESTS[s])
   self.assertLessEqual(len(runtime.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual(m.parents[:14],[None,0,1,2,3,3,1,6,0,0,0,0,2,3])
   if s=='rapidash':self.assertEqual(m.parents[14],3)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(m.joints))))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   bad=copy.deepcopy(author.authoring_model(s));bad['primitives'][0]['positions'][0]+=.01
   with self.assertRaises(ValueError):author.export_fire_horse(s,bad)
 def test_anatomical_owners_keep_fire_layers_faces_and_hooves_coherent(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith('Neck /'):self.assertTrue(owners<={1,2,3});continue
    if n.startswith('Leg /'):
     side,row=map(int,n.split()[-2:]);owner=8+(0 if side<0 else 2)+row;self.assertIn(owner,owners);self.assertTrue(owners<={0,1,owner});continue
    if n.startswith('Hoof /'):expected={8 if s=='rapidash' and n.endswith('-1 0') else 0}
    elif n.startswith(('Body /','Shoulder /')):expected={1}
    elif n.startswith(('Ear /','Ear inset /')):expected={4 if n.endswith('-1') else 5}
    elif n.startswith('Tail stem /'):expected={6}
    elif n.startswith(('Flame /','Flame glow /')):
     label=n.split(' / ',1)[1];owner=12 if label.startswith('mane') else 7 if label.startswith('tail') else 13
     if label.startswith('ankle '):side,row=map(int,label.split()[1:3]);owner=8+(0 if side<0 else 2)+row
     expected={owner}
    elif n.startswith(('Jaw /','Tongue /')):expected={14}
    else:expected={3}
    self.assertEqual(owners,expected,(s,n))
 def test_neck_head_ears_legs_hooves_and_all_seven_fire_banks_have_posed_roots(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];find=lambda prefix,ending=None:self.find(m,prefix,ending);body=find('Body /');head=find('Head /');neck=find('Neck /');tail=find('Tail stem /');pairs=[(body,neck),(neck,head),(head,find('Muzzle /')),(body,tail),(tail,find('Flame / tail')),(neck,find('Flame / mane')),(body,find('Flame / mane')),(head,find('Flame / '+('forelock' if s=='ponyta' else 'crown')))]
   for side in ('-1','1'):
    pairs.append((head,find('Ear /',[side])));shoulder=find('Shoulder /',[side]);pairs += [(body,shoulder),(shoulder,find('Leg /',[side,'0']))]
    for row in ('0','1'):
     leg=find('Leg /',[side,row]);pairs += [(body,leg),(leg,find('Hoof /',[side,row])),(leg,find('Flame / ankle',[side,row]))]
   if s=='rapidash':pairs += [(head,find('Jaw /')),(head,find('Mouth back /')),(find('Jaw /'),find('Tongue /')),(head,find('Horn /'))]
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[find('Flame / tail')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[tail]))
 def test_floor_raised_forehoof_endpoints_winding_strain_and_independent_joints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith('Hoof /'):
       if s=='rapidash' and n.endswith('-1 0'):self.assertGreater(min(p[1] for p in points),.25)
       else:self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   rotations=m.rotations('attack',.31)
   for joint in range(1,len(m.joints)):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,joint))
 def test_fitted_gaze_and_both_fire_faces_clear_their_actual_posed_carriers(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives']
   for clip in m.clips:
    mats=m.matrices(clip,.31)
    for i,n in enumerate(m.names):
     parent=None;flame=n.startswith('Flame glow /');rear=flame and n.endswith('-1')
     if flame:parent=self.find(m,n.rsplit(' ',1)[0].replace('Flame glow /','Flame /'))
     for prefix,base in (('Ear inset /','Ear /'),('Socket /','Head /'),('Eye /','Head /' if s=='ponyta' else 'Socket /'),('Eye glint /','Eye /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Nostril /','Muzzle /'),('Mouth seam /','Muzzle /')):
      if n.startswith(prefix):parent=self.find(m,base,[n.split()[-1]] if base in ('Ear /','Socket /','Eye /','Iris /') else None)
     if parent is None:continue
     owner=m.ids[i][0][0];matrix=mats[owner]
     def local(index):
      points=[]
      for p in m.posed(index,mats):
       delta=[p[k]-matrix[12+k] for k in range(3)];points += [sum(matrix[4*j+k]*delta[k] for k in range(3)) for j in range(3)]
      result=dict(ps[index],positions=points)
      if flame:result=unzy(result)
      return reverse_front(result) if rear else result
     mark,support=local(i),local(parent);vs=[mark['positions'][j:j+3] for j in range(0,len(mark['positions']),3)]
     for k in range(0,len(mark['indices']),3):
      tri=mark['indices'][k:k+3];a,b,c=[vs[j] for j in tri]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)):
       x,y,z=[sum(w*vs[j][axis] for j,w in zip(tri,weights)) for axis in range(3)];hits=rays(support,x,y);self.assertTrue(hits,(s,n));self.assertGreater(z-max(hits),.0001,(s,clip,n))
 def test_adult_jaw_keeps_real_empty_aperture_through_motion(self):
  m=self.models['rapidash'];ps=m.neutral_model()['primitives'];samples=[author.normalized('rapidash',(x,y,z)) for x in (-.025,0.,.025) for y in (1.585,1.590) for z in (.66,.69)]
  for clip in m.clips:
   mats=m.matrices(clip,.31);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
   for point in samples:self.assertFalse(any(occupied(p,transform(mats[3],point)) for p in posed),(clip,point))
 def test_clips_retain_source_cue_rotation_only_contract(self):
  for s,m in self.models.items():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertTrue(all(c['target']['path']=='rotation' and c['target']['node']!=0 for c in clip['channels']))

if __name__=='__main__':unittest.main()
