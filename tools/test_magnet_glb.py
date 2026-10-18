#!/usr/bin/env python3
"""Stored magnetic skins: coherent screws, open hinged magnets and joined orbs."""
import copy,math,unittest
from pathlib import Path
import magnet_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MagneticSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:skin.Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def find(self,m,prefix,unit,ending=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and n.endswith(' / unit '+str(unit)) and (ending is None or n.split(' / unit ')[0].endswith(' '+str(ending))))
 def test_canonical_limits_lossless_ranges_and_identity_binds(self):
  for s,m in self.models.items():
   r=self.runtime[s];count=1 if s=='magnemite' else 3;self.assertEqual(self.blobs[s],author.export_magnet(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertEqual(len(m.parts),18*count);self.assertEqual(len(r.parts),9*count);self.assertEqual(len(m.joints),1+6*count)
   self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(r.neutral_model()),author.PACKED_REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_magnet(self.blobs[s],s)),author.REST_DIGESTS[s])
   parents=[None]
   for unit in range(count):parents += [0]+[1+6*unit]*5
   self.assertEqual(m.parents[:len(m.joints)],parents)
   self.assertLessEqual(sum(map(len,r.vertices)),20000);self.assertLessEqual(sum(map(len,r.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(1,len(m.joints))))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertEqual(ws,[1.,0.,0.,0.]);self.assertEqual(js[1:],[0,0,0])
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_units_crowns_poles_and_bolt_slots(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    name,unit=n.rsplit(' / unit ',1);base=1+6*int(unit)
    if name.startswith(('Crown shaft /','Thread /','Crown head /','Crown slot /')):owner=base+3
    elif name.startswith(('Face bolt /','Face slot /')):owner=base+4+int(name.split()[-1])
    elif name.startswith(('Magnet /','Pole /')):owner=base+(1 if int(name.split()[-2] if name.startswith('Pole /') else name.split()[-1])<0 else 2)
    else:owner=base
    self.assertEqual({js[0] for js in m.ids[i]},{owner},(s,n))
 def test_actual_magnet_screw_pole_and_cluster_roots_survive_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];count=1 if s=='magnemite' else 3;find=lambda prefix,u,ending=None:self.find(m,prefix,u,ending);pairs=[]
   for u in range(count):
    body=find('Sphere /',u);shaft=find('Crown shaft /',u);pairs += [(body,shaft),(shaft,find('Crown head /',u)),(shaft,find('Thread /',u))]
    pairs += [(body,find('Face bolt /',u,k)) for k in range(2)]
    for sign in (-1,1):
     magnet=find('Magnet /',u,sign);pairs.append((body,magnet));pairs += [(magnet,i) for i,n in enumerate(m.names) if n.startswith('Pole /') and n.endswith(' / unit '+str(u)) and n.split(' / unit ')[0].split()[-2]==str(sign)]
   if count==3:pairs += [(find('Sphere /',i),find('Sphere /',j)) for i,j in ((0,1),(0,2),(1,2))]
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[find('Magnet /',0,-1)]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[find('Sphere /',0)]))
 def test_open_horseshoe_notches_stay_empty_and_arms_remain_solid(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives']
   for clip in m.clips:
    mats=m.matrices(clip,.31)
    for u in range(1 if s=='magnemite' else 3):
     for sign in (-1,1):
      i=self.find(m,'Magnet /',u,sign);owner=1+6*u+(1 if sign<0 else 2);posed=dict(ps[i],positions=sum(m.posed(i,mats),[]))
      def point(x,y):
       root=sign*.17;a=sign*.28;c,t=math.cos(a),math.sin(a);q=(root+(sign*x-root)*c-y*t,(sign*x-root)*t+y*c,0.);q=author.normalized(s,author.unit_point(s,u,q));return skin.transform(mats[owner],q)
      for x in (.32,.38,.44,.50,.56):self.assertFalse(occupied(posed,point(x,0.)),(s,clip,u,sign,x))
      for y in (-.105,.105):self.assertTrue(occupied(posed,point(.40,y)),(s,clip,u,sign,y))
 def test_floor_winding_rigidity_and_neutral_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertAlmostEqual(math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b]),1.,places=5)
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_independent_carriers_and_original_crown_axes(self):
  for s,m in self.models.items():
   for joint in range(1,len(m.joints)):
    distance=0.
    for t in (.13,.31,.57,.83):
     isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=m.rotations('idle',t)[joint];mats=m.matrices(rotations=isolate);distance=max(distance,max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))))
    self.assertGreater(distance,.0005,(s,joint))
   for clip in m.clips:
    for t in (.13,.31,.57,.83):
     mats=m.matrices(clip,t)
     for u in range(1 if s=='magnemite' else 3):
      base=1+6*u
      for y in (.195,.24,.30):
       point=author.normalized(s,author.unit_point(s,u,(0.,y,0.)));self.assertLess(math.dist(skin.transform(mats[base],point),skin.transform(mats[base+3],point)),1e-7,(s,clip,u,y))
 def test_strict_reader_rejects_geometry_skin_and_clip_mutations(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_magnet(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_magnet(s,changed)
if __name__=='__main__':unittest.main()
