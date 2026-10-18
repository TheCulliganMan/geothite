#!/usr/bin/env python3
"""Decoded soft bodies, broad fixed puddles and independent palms, digits and lobes."""
import copy,math,unittest
from pathlib import Path
import slime_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SlimeSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def test_canonical_bounded_containers_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_slime(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());count=5 if s=='ditto' else 13;self.assertEqual(len(m.joints),count)
   self.assertEqual(m.parents[:count],[None,0,1,2,2] if s=='ditto' else [None,0,1,1,3,1,5,4,4,4,6,6,6])
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(count)))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_anatomical_owners_keep_faces_coherent_and_sole_fixed(self):
  for s,m in self.models.items():
   ground=set()
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith(('Body /','Body fold /','Body sheen /')):
     self.assertTrue(owners<=(set(range(5)) if s=='ditto' else {0,1,2}))
     for v,p in enumerate(m.vertices[i]):
      if p[1]<1e-7:self.assertEqual(m.ids[i][v],[0,0,0,0]);self.assertEqual(m.weights[i][v],[1.,0.,0.,0.]);ground.add(tuple(p))
    elif n.startswith('Arm /'):
     side=int(n.split()[-1]);self.assertTrue(owners<={1,2,3 if side<0 else 5,4 if side<0 else 6})
    elif n.startswith('Hand /'):self.assertEqual(owners,{4 if n.endswith('-1') else 6})
    elif n.startswith('Finger /'):
     side,j=map(int,n.split()[-2:]);self.assertEqual(owners,{4 if side<0 else 6,7+(0 if side<0 else 3)+j})
    else:self.assertEqual(owners,{2},(s,n))
   self.assertGreaterEqual(len(ground),48)
 def test_arms_palms_and_six_digits_remain_physically_rooted(self):
  for s in ('grimer','muk'):
   m=self.models[s];parts=m.neutral_model()['primitives'];find=lambda prefix,ending=None:next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending));body=find('Body /');pairs=[]
   for side in ('-1','1'):
    arm=find('Arm /',[side]);hand=find('Hand /',[side]);pairs += [(body,arm),(arm,hand)]
    pairs += [(hand,find('Finger /',[side,str(j)])) for j in range(3)]
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(parts[find('Hand /')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,parts[find('Arm /')]))
 def test_floor_endpoints_winding_strain_and_local_articulation(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      for v,p in enumerate(m.vertices[i]):
       if p[1]<1e-7:self.assertLess(math.dist(p,points[v]),1e-7)
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
 def test_ditto_sheen_and_all_faces_stay_on_their_posed_surfaces(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives'];find=lambda prefix,suffix=None:next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (suffix is None or n.split()[-1]==suffix));pairs=[]
   for i,n in enumerate(m.names):
    if n.startswith(('Eye /','Eye border /','Mouth /','Body sheen /')):parent=find('Body /')
    elif n.startswith('Eye white /'):parent=find('Eye border /',n.split()[-1])
    elif n.startswith('Pupil /'):parent=find('Eye white /',n.split()[-1])
    elif n.startswith(('Tongue /','Smile interior /')):parent=find('Mouth /')
    else:continue
    pairs.append((i,parent))
   for clip in m.clips:
    mats=m.matrices(clip,.31);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)]
    for mark,parent in pairs:
     p=posed[mark];vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
     for k in range(0,len(p['indices']),3):
      ids=p['indices'][k:k+3];a,b,c=[m.vertices[mark][j] for j in ids]
      # Closed markings have embedded back and edge walls. A head turn can
      # make an edge wall face the camera; only the authored front coats it.
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      a,b,c=[vs[j] for j in ids]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=rays(posed[parent],x,y);self.assertTrue(h,(s,m.names[mark]));self.assertGreater(z-max(h),.0001,(s,clip,m.names[mark]))
 def test_source_cues_own_attack_and_hit_clocks(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
