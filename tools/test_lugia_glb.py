#!/usr/bin/env python3
"""Decoded five-digit palms, open jaw, fitted belly and planted sea-guardian pads."""
import math,unittest
from pathlib import Path
import lugia_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as ray_intersections,surface_distances,crossed_faces
import test_lugia_sculpt as anatomy
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class LugiaSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'lugia.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.records={};cls.last=None;cls.cached={}
  for p,metadata in enumerate(cls.m.doc['extras']['source_part_ranges']):
   sources=metadata.get('source_parts',[{'part':cls.m.names[p],'order':metadata.get('source_order'),'vertex_start':0,'vertex_count':len(cls.m.vertices[p]),'index_start':0,'index_count':len(cls.m.indices[p])}])
   for r in sources:cls.records[r['part']]=(p,r)
 def source(self,name,mats):
  if mats is not self.last:self.last=mats;self.cached={}
  p,r=self.records[name]
  if p not in self.cached:self.cached[p]=self.m.posed(p,mats)
  a=r['vertex_start'];b=a+r['vertex_count'];i=r['index_start'];j=i+r['index_count']
  return {'part':name,'positions':[v for point in self.cached[p][a:b] for v in point],'indices':[k-a for k in self.m.indices[p][i:j]]}
 def find(self,prefix):return next(n for n in self.records if n.startswith(prefix))
 def test_canonical_roundtrip_core_anatomy_and_limits(self):
  m=self.m;decoded=author.decode_lugia(self.blob);self.assertEqual(self.blob,author.export_lugia(decoded));self.assertEqual(tuple(p['part'] for p in decoded['primitives']),author.PARTS);self.assertEqual(len(decoded['primitives']),56)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(34,19823,34796,13));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  core={'primitives':[p for p in decoded['primitives'] if p['part'].startswith(('Body /','Neck /','Head /','Eye flag /','Head crest /','Jaw /','Back plate /','Tail /','Tail fin /','Leg /')) or p['part'].startswith('Wing digit /') and p['part'].endswith(' 1')]};self.assertEqual(len(core['primitives']),25);self.assertEqual(author.geometry_digest(core),'5d00415a3b36adc947a61efc4c1a8018aa616b2926e98fcf56ac3b007c525a6a')
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_palms_digits_belly_dorsal_plates_gaze_and_grounded_owners(self):
  m=self.m
  for n,(p,r) in self.records.items():
   if n.startswith(('Leg /','Haunch /','Wing arm /','Tail /')):continue
   owner=0 if n.startswith(('Foot /','Toe /')) else 10 if n.startswith('Tail fin /') else 2 if n.startswith('Neck /') else 8 if n.startswith(('Jaw / open','Tongue /')) else 3 if n.startswith(('Head /','Head crest /','Eye','Mask /','Pupil /','Jaw /','Mouth /')) else 1
   if n.startswith(('Wing palm /','Wing digit /')):owner=5 if n.split()[-2 if n.startswith('Wing digit /') else -1]=='-1' else 7
   self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p][r['vertex_start']:r['vertex_start']+r['vertex_count']]),n)
 def test_neutral_endpoints_floor_fixed_pads_and_toes_and_tail_height(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     points=m.posed(p,mats);self.assertGreaterEqual(min(q[1] for q in points),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(points,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if n.startswith(('Foot /','Toes /')):
      for a,b in zip(points,m.vertices[p]):self.assertLess(math.dist(a,b),1e-7)
     if n.startswith(('Tail /','Tail fin /')):
      for a,b in zip(points,m.vertices[p]):self.assertLess(abs(a[1]-b[1]),1e-7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Wing arm','Wing palm','Wing digits','Jaw','Tail','Tail fin'}<=moved,moved)
 def test_winding_and_physical_edge_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,t,n,ratio))
 def test_actual_posed_roots_all_ten_digits_dorsal_plates_and_supported_legs(self):
  m=self.m;f=self.find;body=f('Body /');neck=f('Neck /');head=f('Head /');pairs=[(body,neck),(neck,head),(body,f('Tail /')),(f('Tongue /'),f('Jaw / open'))]
  pairs += [(n,head) for n in self.records if n.startswith(('Eye flag /','Head crest /','Jaw /','Mouth /'))]
  pairs += [(n,body) for n in self.records if n.startswith('Back plate /')]
  pairs += [(n,f('Tail /')) for n in self.records if n.startswith('Tail fin /')]
  for side in ('-1','1'):
   arm=f('Wing arm / broad bent white shoulder '+side);palm=f('Wing palm / rounded five-finger white fan '+side);leg=f('Leg / broad white guardian ankle '+side);haunch=f('Haunch / powerful rooted white guardian thigh '+side);foot=f('Foot / grounded broad white guardian pad '+side)
   pairs += [(arm,body),(arm,palm),(haunch,body),(haunch,leg),(leg,foot)]
   pairs += [(n,palm) for n in self.records if n.startswith('Wing digit /') and n.split()[-2]==side]
   pairs += [(n,foot) for n in self.records if n.startswith('Toe /') and n.split()[-2]==side]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.source(a,mats),self.source(b,mats)),(clip,t,a,b))
 def test_posed_belly_and_gaze_support_and_triangle_clearance(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for n in self.records:
     side=n.split()[-1];parent=None
     if n.startswith('Belly /'):parent=self.find('Body /')
     elif n.startswith('Mask /'):parent=self.find('Head /')
     elif n.startswith('Eye /'):parent=self.find('Mask / fitted navy pointed eye shield '+side)
     elif n.startswith('Pupil /'):parent=self.find('Eye / fitted ivory angular sea-guardian lens '+side)
     if parent is None:continue
     p,r=self.records[n];carrier=mats[m.ids[p][r['vertex_start']][0]]
     def local(name):
      q=self.source(name,mats);points=[]
      for i in range(0,len(q['positions']),3):
       d=[q['positions'][i+k]-carrier[12+k] for k in range(3)];points.extend(sum(carrier[4*j+k]*d[k] for k in range(3)) for j in range(3))
      return dict(q,positions=points)
     mark,support=local(n),local(parent)
     for i in range(0,len(mark['positions']),3):
      x,y,z=mark['positions'][i:i+3];hits=ray_intersections(support,x,y);self.assertTrue(hits,(clip,n));self.assertLess(abs(z-max(hits)),.018,(clip,n))
     for k in range(0,len(mark['indices']),3):
      a,b,c=[mark['positions'][3*i:3*i+3] for i in mark['indices'][k:k+3]]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
      for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
       hits=ray_intersections(support,q[0],q[1]);self.assertTrue(hits,(clip,n));self.assertGreater(q[2]-max(hits),.0001,(clip,n))
 def test_posed_open_roar_keeps_a_real_void_above_the_supported_tongue(self):
  m=self.m;probe=anatomy.LugiaTests()
  for clip in m.clips:
   for t in (0.,.18,.31,.55):
    mats=m.matrices(clip,t);carrier=mats[3];parts=[]
    for n in self.records:
     q=self.source(n,mats);points=[]
     for i in range(0,len(q['positions']),3):
      d=[q['positions'][i+k]-carrier[12+k] for k in range(3)];points.extend(sum(carrier[4*j+k]*d[k] for k in range(3)) for j in range(3))
     parts.append(dict(q,positions=points))
    self.assertTrue(probe.jaw_gap(parts),(clip,t))
 def test_attack_wing_strokes_independent_palms_mandible_and_tail(self):
  m=self.m;mats=m.matrices('attack',.18)
  for side,parent in (('-1',4),('1',6)):
   p,r=self.records[self.find('Wing palm / rounded five-finger white fan '+side)]
   self.assertGreater(max(a[1]-b[1] for a,b in zip(m.vertices[p],m.posed(p,mats))),.08)
   self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[parent],v))>.003 for k,v in enumerate(m.vertices[p])))
  for prefix,parent in (('Jaw / open',3),('Tail fin /',9)):
   p,r=self.records[self.find(prefix)];self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[parent],v))>.004 for k,v in enumerate(m.vertices[p])),prefix)
 def test_rejects_noncanonical_source_and_blob(self):
  with self.assertRaises(ValueError):author.decode_lugia(self.blob+b'x')
  with self.assertRaises(ValueError):author.decode_lugia(self.blob[:-1])
  with self.assertRaises(ValueError):author.export_lugia(dict(author.SOURCE,name='ho_oh'))
if __name__=='__main__':unittest.main()
