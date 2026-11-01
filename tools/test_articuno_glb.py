#!/usr/bin/env python3
"""Decoded ice-bird feather banks, planted talons and continuous streamer skin."""
import math,unittest
from pathlib import Path
import articuno_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as ray_intersections,surface_distances,crossed_faces
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ArticunoSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'articuno.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.records={};cls.last=None;cls.cached={}
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
 def test_canonical_roundtrip_original_anatomy_and_limits(self):
  m=self.m;decoded=author.decode_articuno(self.blob);self.assertEqual(self.blob,author.export_articuno(decoded));self.assertEqual(tuple(p['part'] for p in decoded['primitives']),author.PARTS);self.assertEqual(len(decoded['primitives']),109)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(49,19074,26704,11));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  untouched={'primitives':[p for p in decoded['primitives'] if p['part'].startswith(('Body /','Neck /','Head /','Bill','Crown /','Wing /','Leg /','Foot /','Toe /','Talon /','Tail /'))]};self.assertEqual(len(untouched['primitives']),32);self.assertEqual(author.geometry_digest(untouched),'3e1053c83be263e340ea67b0dd98bae4a1ade7ad9f9a4eaa6ce40cdc76fef350')
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_layered_wings_gaze_and_fixed_floor_owners(self):
  m=self.m
  for n,(p,r) in self.records.items():
   if n.startswith(('Leg /','Tail /','Tail fold /')):continue
   owner=0 if n.startswith(('Foot /','Toe /','Talon /')) else 2 if n.startswith(('Neck /','Throat /')) else 3 if n.startswith(('Head /','Eye','Pupil /','Glint /','Bill','Crown /')) else 1
   if n.startswith(('Wing /','Primary /','Covert /')):owner=4 if n.split()[-3 if n.startswith(('Primary /','Covert /')) else -1]=='-1' else 5
   self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p][r['vertex_start']:r['vertex_start']+r['vertex_count']]),n)
  owners=set()
  for n,(p,r) in self.records.items():
   if n.startswith(('Tail /','Tail fold /')):
    for ids,ws in zip(m.ids[p],m.weights[p]):owners.update(j for j,w in zip(ids,ws) if w)
  self.assertEqual(owners,{6,7,8})
 def test_neutral_endpoints_floor_contacts_and_moving_anatomy(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     points=m.posed(p,mats);self.assertGreaterEqual(min(q[1] for q in points),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(points,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if n.startswith(('Foot /','Toe /','Talon /')):
      for a,b in zip(points,m.vertices[p]):self.assertLess(math.dist(a,b),1e-7)
     if n.startswith(('Tail /','Tail fold /')):
      for a,b in zip(points,m.vertices[p]):self.assertLess(abs(a[1]-b[1]),1e-7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Wing','Flight feathers','Tail','Tail fold'}<=moved,moved)
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
 def test_actual_posed_neck_head_crown_bill_wing_streamer_and_shin_roots(self):
  m=self.m;f=self.find;body=f('Body /');neck=f('Neck /');head=f('Head /');pairs=[(body,neck),(neck,head),(body,f('Tail /'))]
  pairs += [(n,head) for n in self.records if n.startswith(('Bill /','Crown /'))]
  for side in ('-1','1'):
   wing=f('Wing / bent ice-blue upper arm '+side);leg=f('Leg / rooted pink shin '+side);foot=f('Foot / grounded pink knuckle '+side)
   pairs += [(wing,body),(leg,body),(leg,foot)]
   pairs += [(n,wing) for n in self.records if n.startswith(('Primary /','Covert /')) and n.split()[-3]==side]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.source(a,mats),self.source(b,mats)),(clip,t,a,b))
 def test_all_posed_pale_bib_throat_and_gaze_stay_supported_and_clear(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for n in self.records:
     side=n.split()[-1];parent=None
     if n.startswith('Breast /'):parent=self.find('Body /')
     elif n.startswith('Throat /'):parent=self.find('Neck /')
     elif n.startswith('Eye socket /'):parent=self.find('Head /')
     elif n.startswith('Eye /'):parent=self.find('Eye socket / calm fitted ice-bird lens '+side)
     elif n.startswith('Pupil /'):parent=self.find('Eye / fitted ivory almond '+side)
     elif n.startswith('Glint /'):parent=self.find('Pupil / quiet dark ice-bird gaze '+side)
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
 def test_posed_channels_follow_the_actual_articulated_streamer(self):
  m=self.m;tail=self.find('Tail /')
  for clip in m.clips:
   for t in (.18,.31,.55):
    mats=m.matrices(clip,t);support=self.source(tail,mats)
    for n in self.records:
     if n.startswith('Tail fold /'):
      q=self.source(n,mats);points=[q['positions'][i:i+3] for i in range(0,len(q['positions']),3)];self.assertLess(max(surface_distances(support,points,.006)),.006,(clip,t,n))
 def test_attack_strokes_both_wings_and_articulates_streamer_curve(self):
  m=self.m;mats=m.matrices('attack',.18)
  for side in ('-1','1'):
   p,r=self.records[self.find('Wing / bent ice-blue upper arm '+side)];self.assertGreater(max(a[1]-b[1] for a,b in zip(m.vertices[p],m.posed(p,mats))),.10)
  p,r=self.records[self.find('Tail /')]
  self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[6],v))>.005 for k,v in enumerate(m.vertices[p])))
  for clip,t in (('attack',.18),('hit',.31)):self.assertIsNone(crossed_faces(self.source(self.find('Tail /'),m.matrices(clip,t))))
 def test_rejects_noncanonical_blob_and_source(self):
  with self.assertRaises(ValueError):author.decode_articuno(self.blob+b'x')
  with self.assertRaises(ValueError):author.decode_articuno(self.blob[:-1])
  with self.assertRaises(ValueError):author.export_articuno(dict(author.SOURCE,name='zapdos'))
if __name__=='__main__':unittest.main()
