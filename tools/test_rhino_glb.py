#!/usr/bin/env python3
"""Decoded planted armored bodies, lossless packing, open jaw and independent appendages."""
import copy,math,unittest
from pathlib import Path
import rhino_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class RhinoSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:skin.Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def find(self,m,prefix,ending=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending))
 def test_canonical_limits_lossless_ranges_hierarchy_and_identity(self):
  for s,m in self.models.items():
   r=self.runtime[s];self.assertEqual(self.blobs[s],author.export_rhino(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertEqual(len(m.joints),11 if s=='rhyhorn' else 15)
   self.assertEqual(len(r.parts),46 if s=='rhyhorn' else 48);self.assertEqual(len(m.parts),46 if s=='rhyhorn' else 64)
   self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(r.neutral_model()),author.PACKED_REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_rhino(self.blobs[s],s)),author.REST_DIGESTS[s])
   self.assertLessEqual(len(r.parts),64);self.assertLessEqual(sum(map(len,r.vertices)),20000);self.assertLessEqual(sum(map(len,r.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual(m.parents[:len(m.joints)],[None,0,1,1,0,0,0,0,2,2,2] if s=='rhyhorn' else [None,0,1,2,1,4,1,6,0,0,1,10,2,2,2])
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(m.joints))))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_anatomical_owners_preserve_planted_pads_and_coherent_armor(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith(('Foot /','Foot claw /')) or s=='rhyhorn' and n.startswith('Claw /'):expected={0}
    elif n.startswith(('Leg /','Leg armor /','Thigh /')):
     if s=='rhyhorn':side=int(n.split()[-2]);own=4+(0 if side<0 else 2)+int(n.endswith('front'))
     else:own=8 if n.endswith('-1') else 9
     self.assertTrue(owners<={0,1,own});self.assertIn(own,owners);continue
    elif s=='rhyhorn':
     if n.startswith(('Body /','Back armor /')):expected={1}
     elif n.startswith('Tail /'):expected={3}
     elif n.startswith('Jaw /'):expected={8}
     elif n.startswith('Ear /'):expected={9 if n.endswith('-1') else 10}
     else:expected={2}
    else:
     if n.startswith(('Body /','Belly plate /','Collar /','Back armor /')):expected={1}
     elif n.startswith(('Jaw /','Tongue /')):expected={3}
     elif n.startswith(('Ear /','Ear inset /')):expected={12 if n.endswith('-1') else 13}
     elif n.startswith(('Horn /','Horn spiral /')):expected={14}
     elif n.startswith(('Tail /','Tail armor /')):self.assertTrue(owners<={1,10,11});continue
     elif n.startswith(('Hand /','Hand claw /')):
      side=n.split()[-2] if n.startswith('Hand claw /') else n.split()[-1];expected={5 if side=='-1' else 7}
     elif n.startswith('Shoulder /'):expected={4 if n.endswith('-1') else 6}
     elif n.startswith('Arm /'):self.assertTrue(owners<={1,4 if n.endswith('-1') else 6,5 if n.endswith('-1') else 7});continue
     else:expected={2}
    self.assertEqual(owners,expected,(s,n))
 def test_armor_limb_jaw_drill_and_tail_roots_survive_motion(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];find=lambda prefix,ending=None:self.find(m,prefix,ending);body=find('Body /');head=find('Head /');pairs=[(body,head),(body,find('Tail /'))]
   if s=='rhyhorn':
    pairs += [(head,find(prefix)) for prefix in ('Jaw /','Horn /','Brow armor /')]
    pairs += [(body,i) for i,n in enumerate(m.names) if n.startswith('Back armor /')]
    pairs += [(head,i) for i,n in enumerate(m.names) if n.startswith('Ear /')]
    for side in ('-1','1'):
     for row in ('front','rear'):
      leg=find('Leg /',[side,row]);pairs += [(body,leg),(leg,find('Leg armor /',[side,row])),(leg,find('Foot /',[side,row]))]
   else:
    muzzle=find('Muzzle /');pairs += [(muzzle,head),(body,find('Jaw /')),(find('Jaw /'),find('Tongue /')),(muzzle,find('Mouth back /')),(muzzle,find('Horn /')),(find('Horn /'),find('Horn spiral /'))]
    pairs += [(head,i) for i,n in enumerate(m.names) if n.startswith(('Ear /','Cheek armor /'))]
    pairs += [(body,i) for i,n in enumerate(m.names) if n.startswith('Back armor /')]
    pairs += [(find('Tail /'),i) for i,n in enumerate(m.names) if n.startswith('Tail armor /')]
    for side in ('-1','1'):
     shoulder=find('Shoulder /',[side]);arm=find('Arm /',[side]);hand=find('Hand /',[side]);thigh=find('Thigh /',[side]);leg=find('Leg /',[side]);foot=find('Foot /',[side])
     pairs += [(body,shoulder),(shoulder,arm),(arm,hand),(body,thigh),(thigh,leg),(leg,foot),(muzzle,find('Fang /',[side]))]
     pairs += [(hand,find('Hand claw /',[side,str(j)])) for j in range(3)];pairs += [(foot,find('Foot claw /',[side,str(j)])) for j in range(2)]
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[find('Tail /')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[body]))
 def test_floor_endpoints_winding_strain_and_local_articulation(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Foot claw /')) or s=='rhyhorn' and n.startswith('Claw /'):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
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
 def test_rhydon_collar_layers_remain_seated_and_clear_during_motion(self):
  m=self.models['rhydon'];ps=m.neutral_model()['primitives'];tail=self.find(m,'Tail /')
  for clip in m.clips:
   mats=m.matrices(clip,.31);body=dict(ps[tail],positions=sum(m.posed(tail,mats),[]))
   for i,n in enumerate(m.names):
    if not n.startswith('Tail armor /'):continue
    vs=m.posed(i,mats);indices=m.indices[i]
    for offset in range(0,2*32*12,12):
     for layer,inside in ((0,False),(6,True)):
      for t in (0,3):
       tri=[vs[k] for k in indices[offset+layer+t:offset+layer+t+3]];p=tuple(sum(v[k] for v in tri)/3 for k in range(3));self.assertEqual(occupied(body,p),inside,(clip,n,offset,layer,t))
 def test_rhydon_open_jaw_keeps_the_actual_aperture_clear(self):
  m=self.models['rhydon'];ps=m.neutral_model()['primitives'];points=[author.normalized('rhydon',(x,y,z)) for x in (-.04,0.,.04) for y in (1.,1.02) for z in (.43,.46)]
  for clip in m.clips:
   mats=m.matrices(clip,.31);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
   for point in points:self.assertFalse(any(occupied(p,transform(mats[2],point)) for p in posed),(clip,point))
 def test_source_cues_own_attack_and_hit_clocks(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
