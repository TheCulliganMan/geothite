#!/usr/bin/env python3
"""Continuous crocodile bodies, occupied limb roots, seated plates and independently hinged jaws."""
import copy,math,os,unittest
from collections import defaultdict
from pathlib import Path
import crocodile_glb as author
import test_gengar_glb as skin
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays,crossed_faces
from chikorita_sculpt import cross,sub,unit,dot
ROOT=Path(os.environ.get('CRYSTAL_CROCODILE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class CrocodileSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:skin.Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def find(self,m,prefix,suffix=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (suffix is None or n.endswith(' '+str(suffix))))
 def test_lossless_packing_limits_hierarchy_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_crocodile(s));self.assertEqual(len(m.parts),51 if s=='croconaw' else 83);self.assertEqual(len(m.joints),14);self.assertEqual(m.parents[:14],[None,0,1,2,1,4,5,1,7,8,0,0,1,12])
   self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(self.runtime[s].neutral_model()),author.PACKED_REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_crocodile(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(14)))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_matching_anatomical_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith('Foot /'):owner=0
    elif n.startswith(('Eye /','Nostril /','Jaw / broad upper','Tooth / upper','Crest / three-point cranial','Crest / broad jagged cranial','Crest / rear jagged cranial')):owner=2
    elif n.startswith(('Jaw / fitted pale lower','Tooth / lower','Mouth / rounded pink tongue')):owner=3
    elif n.startswith('Hand /'):owner=6 if n.split()[-2 if n.startswith(('Hand / curved','Hand / white')) else -1]=='-1' else 9
    if owner is not None:self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{owner},(s,n))
 def test_floor_strain_winding_and_cue_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith('Foot /'):
       for a,b in zip(points,m.vertices[i]):self.assertLess(math.dist(a,b),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_every_carrier_articulates_independently(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for j in range(1,14):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_resting_arms_are_lowered_and_adult_elbows_bend_under_the_wrists(self):
  for s,m in self.models.items():
   pivots=[]
   for i,node in enumerate(m.doc['nodes'][:14]):
    parent=m.parents[i];local=node.get('translation',[0.,0.,0.])
    pivots.append([local[k]+(pivots[parent][k] if parent is not None else 0.) for k in range(3)])
   for arm,elbow,wrist in ((4,5,6),(7,8,9)):
    self.assertGreater(pivots[arm][1]-pivots[wrist][1],.10,(s,arm))
    self.assertGreater(pivots[wrist][2]-pivots[arm][2],.12,(s,arm))
    if s=='feraligatr':self.assertLess(pivots[elbow][1],pivots[wrist][1]-.02)
 def test_elbow_and_knee_closing_faces_stay_inside_adjoining_skin(self):
  # Find actual large planar closing rings in the stored mesh. Merely having
  # overlapping limbs allowed the old exposed end disks to pass root checks.
  def caps(m,i):
   groups=defaultdict(set)
   for k in range(0,len(m.indices[i]),3):
    ids=m.indices[i][k:k+3];a,b,c=[m.vertices[i][v] for v in ids];normal=unit(cross(sub(b,a),sub(c,a)))
    groups[tuple(round(v,3) for v in normal)+(round(dot(normal,a),3),)].update(ids)
   return [sorted(ids) for ids in groups.values() if len({tuple(m.vertices[i][v]) for v in ids})>=14]
  for s,m in self.models.items():
   joins=[]
   for side in (-1,1):
    parts=[self.find(m,prefix,side) for prefix in ('Arm / shaped','Arm / broad','Leg / broad','Leg / planted')]
    for child,parent,axis,reverse in ((0,1,2,True),(1,0,2,False),(2,3,1,False),(3,2,1,True)):
     i,j=parts[child],parts[parent];rings=caps(m,i);self.assertGreaterEqual(len(rings),2,(s,m.names[i]))
     ring=sorted(rings,key=lambda ids:sum(m.vertices[i][v][axis] for v in ids)/len(ids),reverse=reverse)[0];joins.append((i,j,ring))
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(m.neutral_model()['primitives'])]
     for i,j,ring in joins:
      points=[m.point(i,v,mats) for v in ring];center=[sum(q[axis] for q in points)/len(points) for axis in range(3)]
      probes=[center]+[[center[axis]+.9*(q[axis]-center[axis]) for axis in range(3)] for q in points]
      for q in probes:
       hits=sorted(set(rays(posed[j],q[0],q[1])))
       self.assertTrue(any(a-1e-6<=q[2]<=b+1e-6 for a,b in zip(hits[::2],hits[1::2])),(s,clip,t,m.names[i],m.names[j],q))
 def test_curved_limb_surfaces_do_not_cross_themselves(self):
  for s,m in self.models.items():
   for p in m.neutral_model()['primitives']:
    if p['part'].startswith(('Arm /','Leg /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_limb_tail_and_tooth_roots_remain_occupied(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];pairs=[]
   for child,parent in (('Jaw / broad upper','Body /'),('Jaw / fitted pale lower','Body /'),('Tail /','Body /')):pairs.append((self.find(m,child),self.find(m,parent)))
   for side in (-1,1):
    for child,parent in (('Arm / shaped','Body /'),('Arm / broad','Arm / shaped'),('Hand / broad','Arm / broad'),('Leg / broad','Body /'),('Leg / planted','Leg / broad'),('Foot / raised','Leg / planted')):pairs.append((self.find(m,child,side),self.find(m,parent,None if parent=='Body /' else side)))
    for digit in range(3):
     pairs.append((self.find(m,'Hand / curved',f'{side} {digit}'),self.find(m,'Hand / broad',side)))
     pairs.append((self.find(m,'Foot / splayed',f'{side} {digit}'),self.find(m,'Foot / raised',side)))
     if s=='feraligatr':
      pairs.append((self.find(m,'Hand / white',f'{side} {digit}'),self.find(m,'Hand / curved',f'{side} {digit}')));pairs.append((self.find(m,'Foot / white',f'{side} {digit}'),self.find(m,'Foot / splayed',f'{side} {digit}')))
   for i,n in enumerate(m.names):
    if n.startswith('Tooth /'):pairs.append((i,self.find(m,'Jaw / broad upper' if n.startswith('Tooth / upper') else 'Jaw / fitted pale lower')))
    if n.startswith('Crest /'):pairs.append((i,self.find(m,'Tail /' if n.startswith('Crest / tail') else 'Body /')))
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[self.find(m,'Tail /')]);detached['positions']=[v+(2 if i%3==0 else 0) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[self.find(m,'Body /')]))
 def test_fitted_armor_keeps_its_parent_clearance_through_motion(self):
  for s,m in self.models.items():
   if s!='feraligatr':continue
   ps=m.neutral_model()['primitives']
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for i,n in enumerate(m.names):
      if not n.startswith('Armor /'):continue
      parents=[j for j,name in enumerate(m.names) if name.startswith('Body /') or not n.startswith('Armor / dorsal') and name.startswith(('Arm /','Leg /'))];back=n.startswith('Armor / dorsal')
      for k in range(0,len(ps[i]['indices']),3):
       tri=ps[i]['indices'][k:k+3];a,b,c=[m.vertices[i][j] for j in tri]
       # Check authored front cap samples, not extrusion sides/back faces.
       if ((b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]))*( -1 if back else 1)<=1e-14:continue
       for weights in ((1/3,1/3,1/3),(.5,.5,0.),(0.,.5,.5),(.5,0.,.5)):
        q=[sum(posed[i]['positions'][v*3+axis]*w for v,w in zip(tri,weights)) for axis in range(3)];hits=[z for parent in parents for z in rays(posed[parent],q[0],q[1])];self.assertTrue(hits,(clip,t,n))
        self.assertLess(abs(q[2]-(min(hits) if back else max(hits))),.030,(clip,t,n))
 def test_strict_reader_rejects_mutation(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_crocodile(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_crocodile(s,changed)
if __name__=='__main__':unittest.main()
