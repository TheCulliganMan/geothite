#!/usr/bin/env python3
"""Stored egg containment, rooted appendages, fitted faces and independent skin carriers."""
import copy,math,os,unittest
from pathlib import Path
from collections import defaultdict
from chikorita_sculpt import cross,sub,unit,dot
import egg_nurse_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import crossed_faces,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied
ROOT=Path(os.environ.get('CRYSTAL_EGG_NURSE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def find(m,prefix,side=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (side is None or n.endswith(' '+str(side))))
def undo(matrix,point):
 d=[point[i]-matrix[12+i] for i in range(3)];return [sum(matrix[4*i+k]*d[k] for k in range(3)) for i in range(3)]
def parts_overlap(a,b):
 # Preserve the shared occupied-interval grid and tolerance; cull ray
 # candidates with the independent stored-triangle index for dense bodies.
 xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
 if xh<=xl or yh<=yl:return False
 for i in range(1,19):
  for j in range(1,23):
   x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23;aa=sorted(rays(a,x,y));bb=sorted(rays(b,x,y))
   for al,ah in zip(aa[::2],aa[1::2]):
    for bl,bh in zip(bb[::2],bb[1::2]):
     if min(ah,bh)-max(al,bl)>.00025:return True
 return False
class EggNurseSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def test_exact_geometry_hierarchy_inverse_binds_and_budgets(self):
  for s,m in self.models.items():
   with self.subTest(s=s):
    self.assertEqual(self.blobs[s],author.export_egg_nurse(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(self.runtime[s].neutral_model()),author.PACKED_REST_DIGESTS[s])
    expected=(24, 16, 10390, 18704, 17) if s=='chansey' else (38, 19, 19379, 34472, 25)
    self.assertEqual((len(m.parts),len(self.runtime[s].parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),expected);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
    parents=[None,0,1,1,3,1,5,0,0]+([2]*6+[1,15] if s=='chansey' else [2,9,2,11,3,5]+[1]*10);self.assertEqual(m.parents[:len(m.joints)],parents)
    self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(m.joints))))
    for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_egg_pouch_gathered_hands_face_and_floor_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith('Foot /') else 1 if n.startswith(('Egg /','Pouch /','Lower body /')) else 2 if n.startswith(('Eye /','Eye glint /','Smile /','Chin /','Mouth lining /','Tongue /')) else None
    if owner is not None:self.assertEqual({j for ids,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(ids,ws) if w},{owner},(s,n))
 def test_floor_strain_winding_and_neutral_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=1e9
     for i,n in enumerate(m.names):
      vs=m.posed(i,mats);ns=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(s,clip,n))
      if n.startswith('Foot /'):
       for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,ns,tri);self.assertGreater(cosine,0.,(s,clip,n,step));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n,step))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_every_carrier_has_independent_motion(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for j in range(1,len(m.joints)):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_actual_roots_and_uncrossed_surfaces_survive_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];body='Body /' if s=='chansey' else 'Upper body /';lower=body if s=='chansey' else 'Lower body /';pairs=[]
   def add(a,b,side=None,parent_side=None):pairs.append((find(m,a,side),find(m,b,parent_side)))
   add('Pouch /',lower);add('Egg /','Pouch /')
   if s=='chansey':add('Tail /',body)
   else:add(body,lower)
   for side in (-1,1):
    for a,b in (('Shoulder /',body),('Arm /','Shoulder /'),('Ankle /' if s=='chansey' else 'Leg /',lower),('Foot /','Ankle /' if s=='chansey' else 'Leg /')):add(a,b,side,None if b in (body,lower) else side)
    if s=='chansey':
     add('Arm /','Egg /',side)
     for j in range(3):add('Ribbon /',body,f'{side} {j}')
    else:
     add('Crown /',body,side);add('Curl /','Crown /',side,side)
     for j in range(3):add('Arm frill /','Shoulder /',f'{side} {j}',side)
   if s=='blissey':
    for j in range(10):add('Skirt /',lower,j)
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
     for i,n in enumerate(m.names):
      if n.startswith(('Arm /','Ribbon /','Tail /','Curl /','Arm frill /','Skirt /')):self.assertIsNone(crossed_faces(posed[i]),(s,clip,t,n))
 def test_fitted_caps_follow_actual_posed_parent(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];pairs=[];body='Body /' if s=='chansey' else 'Upper body /'
   for i,n in enumerate(m.names):
    for child,parent in (('Eye /',body),('Eye glint /','Eye /'),('Smile /',body),('Chin /',body),('Mouth lining /',body),('Tongue /','Mouth lining /')):
     if n.startswith(child):pairs.append((i,find(m,parent,n.split()[-1] if parent=='Eye /' else None)))
   for clip in m.clips:
    for t in (0.,.31,.67):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for child,parent in pairs:
      points=m.posed(child,mats)
      for k in range(0,len(m.indices[child]),3):
       ids=m.indices[child][k:k+3];a,b,c=[m.vertices[child][i] for i in ids]
       if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
       tri=[points[i] for i in ids]
       for ws in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
        x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(posed[parent],x,y);self.assertTrue(hits,(s,clip,t,m.names[child]));gap=z-max(hits);self.assertGreater(gap,.0001,(s,clip,t,m.names[child],gap));self.assertLess(gap,.02,(s,clip,t,m.names[child],gap))
        if m.names[child].startswith('Mouth lining /'):self.assertLess(gap,author.HEIGHTS[s]*.001,(s,clip,t,'raised mouth lining',gap))
 def test_egg_containment_and_empty_pouch_and_mouth_survive_animation(self):
  from test_chansey_sculpt import ChanseyTests
  from test_blissey_sculpt import BlisseyTests
  for s,m in self.models.items():
   context=ChanseyTests() if s=='chansey' else BlisseyTests();raw=author.SOURCES[s].anatomy();context.floor=min(v for p in raw for v in p['positions'][1::3]);context.scale=author.HEIGHTS[s]/(max(v for p in raw for v in p['positions'][1::3])-context.floor);ps=m.neutral_model()['primitives']
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);in_body=[dict(p,positions=sum([undo(mats[1],v) for v in m.posed(i,mats)],[])) for i,p in enumerate(ps)];context.ps=in_body
     self.assertTrue(context.contained(in_body[find(m,'Egg /')]),(s,clip,t))
     for point in context.opening():self.assertFalse(any(occupied(p,point) for p in in_body),(s,clip,t,'pouch',point))
     if s=='blissey':
      in_head=[dict(p,positions=sum([undo(mats[2],v) for v in m.posed(i,mats)],[])) for i,p in enumerate(ps)]
      for point in context.mouth_space():self.assertFalse(any(occupied(p,point) for p in in_head),(s,clip,t,'mouth',point))
 def test_continuous_palms_have_no_large_wrist_closing_faces(self):
  for s,m in self.models.items():
   self.assertFalse(any(n.startswith('Hand /') for n in m.names))
   for side,owner in ((-1,3),(1,5)):
    i=find(m,'Arm /',side);groups=defaultdict(list);root=[-v for v in m.inverse[owner][12:15]]
    for k in range(0,len(m.indices[i]),3):
     tri=[m.vertices[i][v] for v in m.indices[i][k:k+3]];a,b,c=tri;normal=unit(cross(sub(b,a),sub(c,a)));groups[tuple(round(v,4) for v in normal)+(round(dot(normal,a),4),)].append(tri)
    for triangles in groups.values():
     points={tuple(q) for tri in triangles for q in tri}
     if len(points)<14:continue
     common=set.intersection(*({tuple(q) for q in tri} for tri in triangles))
     if not common:continue
     area=sum(math.sqrt(dot(n,n))*.5 for a,b,c in triangles for n in [cross(sub(b,a),sub(c,a))])
     if area<.001:continue
     a,b,c=triangles[0];normal=unit(cross(sub(b,a),sub(c,a)));self.assertLess(abs(dot(normal,sub(root,a))),.00001,(s,side,'large closing plane outside shoulder'))
 def test_strict_reader_rejects_geometry_and_byte_mutations(self):
  for s in author.SPECIES:
   changed=bytearray(self.blobs[s]);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_egg_nurse(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_egg_nurse(s,changed)
if __name__=='__main__':unittest.main()
