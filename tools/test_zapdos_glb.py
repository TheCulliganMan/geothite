#!/usr/bin/env python3
"""Decoded thunder fans, occupied anatomy joins and fixed floor contact."""
import math,unittest
from pathlib import Path
import zapdos_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ZapdosSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'zapdos.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.last=None;cls.cache={}
 def source(self,p,mats):
  if mats is not self.last:self.last=mats;self.cache={}
  if p not in self.cache:self.cache[p]={'part':self.m.names[p],'positions':[v for q in self.m.posed(p,mats) for v in q],'indices':self.m.indices[p]}
  return self.cache[p]
 def find(self,prefix,suffix=None):return next(p for p,n in enumerate(self.m.names) if n.startswith(prefix) and (suffix is None or n.endswith(' '+str(suffix))))
 def test_canonical_roundtrip_original_anatomy_and_limits(self):
  m=self.m;decoded=author.decode_zapdos(self.blob);self.assertEqual(self.blob,author.export_zapdos(decoded));self.assertEqual(tuple(p['part'] for p in decoded['primitives']),author.PARTS)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),(54,18495,28584,9));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  untouched={'primitives':[p for p in decoded['primitives'] if not p['part'].startswith('Pupil /')]};self.assertEqual(len(untouched['primitives']),52);self.assertEqual(author.geometry_digest(untouched),'f127297ad8e2e71188ad235f4d43678895b7eeca66157130d14a1f82f6e1b00d')
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_actual_coherent_gold_black_fans_gaze_and_planted_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   if n.startswith(('Leg /','Thigh /')):continue
   owner=0 if n.startswith(('Foot /','Toe /','Talon /')) else 6 if n.startswith('Tail /') else 2 if n.startswith(('Neck /','Throat /')) else 3 if n.startswith(('Head /','Eye','Pupil /','Bill','Crown /','Cheek /')) else 1
   if n.startswith(('Wing /','Underfeather /','Covert /')):owner=4 if n.split()[-2 if n.startswith('Underfeather /') else -1]=='-1' else 5
   self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
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
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Wing','Underfeather','Tail'}<=moved,moved)
 def test_winding_and_physical_edge_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     points=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.80<math.dist(points[a],points[b])/math.dist(m.vertices[p][a],m.vertices[p][b])<1.20,(clip,t,n,k))
 def test_actual_posed_neck_head_bill_crown_cheek_wing_tail_and_leg_roots(self):
  m=self.m;f=self.find;body=f('Body /');neck=f('Neck /');head=f('Head /');pairs=[(body,neck),(neck,head)]
  for p,n in enumerate(m.names):
   if n.startswith(('Bill','Crown /','Cheek /')):pairs.append((p,head))
   elif n.startswith(('Tail /','Breast feather /')):pairs.append((p,body))
  for side in ('-1','1'):
   wing=f('Wing /',side);leg=f('Leg /',side);thigh=f('Thigh /',side);foot=f('Foot /',side)
   pairs += [(wing,body),(thigh,body),(leg,thigh),(leg,foot)]+[(f('Underfeather /',side+' '+str(j)),wing) for j in range(3)]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.source(a,mats),self.source(b,mats)),(clip,t,m.names[a],m.names[b]))
 def test_all_posed_fitted_gaze_throat_and_shoulders_stay_seated_and_clear(self):
  m=self.m;f=self.find
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     side=n.split()[-1];parent=None
     if n.startswith('Throat /'):parent=f('Neck /')
     elif n.startswith('Covert /'):parent=f('Wing /',side)
     elif n.startswith('Eye socket /'):parent=f('Head /')
     elif n.startswith('Eye /'):parent=f('Eye socket /',side)
     elif n.startswith('Pupil /'):parent=f('Eye /',side)
     if parent is None:continue
     carrier=mats[m.ids[p][0][0]]
     def local(index):
      q=self.source(index,mats);points=[]
      for i in range(0,len(q['positions']),3):
       d=[q['positions'][i+k]-carrier[12+k] for k in range(3)];points.extend(sum(carrier[4*j+k]*d[k] for k in range(3)) for j in range(3))
      return dict(q,positions=points)
     mark,support=local(p),local(parent)
     for i in range(0,len(mark['positions']),3):
      x,y,z=mark['positions'][i:i+3];hits=rays(support,x,y);self.assertTrue(hits,(clip,n));self.assertLess(abs(z-max(hits)),.018,(clip,n))
     for k in range(0,len(mark['indices']),3):
      a,b,c=[mark['positions'][3*i:3*i+3] for i in mark['indices'][k:k+3]]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
      for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
       hits=rays(support,q[0],q[1]);self.assertTrue(hits,(clip,n));self.assertGreater(q[2]-max(hits),.0001,(clip,n))
 def test_attack_strokes_both_fans_and_articulates_tail(self):
  m=self.m;mats=m.matrices('attack',.18)
  for side in ('-1','1'):
   p=self.find('Wing /',side);self.assertGreater(max(a[1]-b[1] for a,b in zip(m.vertices[p],m.posed(p,mats))),.10)
  p=self.find('Tail /');self.assertGreater(max(math.dist(a,skin.transform(mats[1],b)) for a,b in zip(m.posed(p,mats),m.vertices[p])),.005)
 def test_rejects_noncanonical_source_and_blob(self):
  with self.assertRaises(ValueError):author.decode_zapdos(self.blob+b'x')
  with self.assertRaises(ValueError):author.decode_zapdos(self.blob[:-1])
  with self.assertRaises(ValueError):author.export_zapdos(dict(author.SOURCE,name='moltres'))
if __name__=='__main__':unittest.main()
