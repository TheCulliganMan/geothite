#!/usr/bin/env python3
"""Decoded planted flame bird, coherent warm/hot banks and delayed fire motion."""
import math,unittest
from pathlib import Path
import moltres_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as ray_intersections
from bovine_sculpt import reverse_front
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MoltresSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'moltres.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.records={};cls.last=None;cls.cached={}
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
 def test_canonical_source_roundtrip_unaffected_anatomy_and_budgets(self):
  m=self.m;decoded=author.decode_moltres(self.blob);self.assertEqual(self.blob,author.export_moltres(decoded));self.assertEqual(tuple(p['part'] for p in decoded['primitives']),author.PARTS);self.assertEqual(len(decoded['primitives']),62)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(46,19671,26024,12));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  core={'primitives':[p for p in decoded['primitives'] if p['part'].startswith(('Body /','Neck /','Head /','Bill','Leg /','Foot /','Toe /','Talon /'))]};self.assertEqual(len(core['primitives']),26);self.assertEqual(author.geometry_digest(core),'9e70e679684ecaa0712dff6eff9eee706bc3423f156fcde34d2117a657f4a2dc')
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_source_flame_gaze_and_grounded_digit_owners(self):
  m=self.m
  for n,(p,r) in self.records.items():
   if n.startswith('Leg /'):continue
   owner=0 if n.startswith(('Foot /','Toe /','Talon /')) else 4 if n.startswith('Wing /') and n.endswith('-1') else 5 if n.startswith('Wing /') else 8 if n.startswith('Crown') else 9 if n.startswith('Tail') else 2 if n.startswith('Neck /') else 3 if n.startswith(('Head /','Eye','Pupil /','Bill')) else 1
   if n.startswith(('Wing fire /','Wing glow /','Wing heart /')):
    side=n.split()[-3 if n.startswith('Wing heart /') else -2 if n.startswith('Wing glow /') else -1];owner=6 if side=='-1' else 7
   self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p][r['vertex_start']:r['vertex_start']+r['vertex_count']]),n)
 def test_neutral_endpoints_fixed_feet_digits_and_talons_floor_and_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if n.startswith(('Foot /','Toe /','Talon /')):
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Wing','Wing fire','Wing glow','Crown','Tail'}<=moved,moved)
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
 def test_actual_posed_neck_head_gold_wing_fire_crown_tail_and_shin_roots(self):
  m=self.m;f=self.find;body=f('Body /');neck=f('Neck /');head=f('Head /');pairs=[(body,neck),(neck,head),(f('Crown /'),head),(f('Tail /'),body)]
  pairs += [(n,head) for n in self.records if n.startswith(('Bill /','Bill seam /'))]
  for side in ('-1','1'):
   wing=f('Wing / slim gold leading arm '+side);fire=f('Wing fire / flowing red-orange flame bank '+side);leg=f('Leg / rooted pink shin '+side);foot=f('Foot / grounded pink knuckle '+side)
   pairs += [(wing,body),(wing,fire),(leg,body),(leg,foot)]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.source(a,mats),self.source(b,mats)),(clip,t,a,b))
 def test_all_posed_front_and_rear_flame_color_layers_remain_seated(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for n in self.records:
     words=n.split();last=words[-1];parent=None;rear=False
     if n.startswith('Eye socket /'):parent=self.find('Head /')
     elif n.startswith('Eye /'):parent=self.find('Eye socket / fitted alert black lens '+last)
     elif n.startswith('Pupil /'):parent=self.find('Eye / narrow ivory fire-bird gaze '+last)
     elif n.startswith('Wing glow /'):parent=self.find('Wing fire / flowing red-orange flame bank '+words[-2]);rear=last=='-1'
     elif n.startswith('Wing heart /'):parent=self.find('Wing glow / fitted broad orange fire core '+words[-3]+' '+last);rear=last=='-1'
     elif n.startswith('Crown glow /'):parent=self.find('Crown /');rear=last=='-1'
     elif n.startswith('Crown heart /'):parent=self.find('Crown glow / supported orange flame heart '+last);rear=last=='-1'
     elif n.startswith('Tail glow /'):parent=self.find('Tail /');rear=last=='-1'
     elif n.startswith('Tail heart /'):parent=self.find('Tail glow / supported flowing orange flame '+last);rear=last=='-1'
     if parent is None:continue
     # Undo the rigid carrier to make the support projection exact for both faces.
     p,r=self.records[n];owner=m.ids[p][r['vertex_start']][0];a=mats[owner]
     def local(name):
      q=self.source(name,mats);points=[]
      for i in range(0,len(q['positions']),3):
       d=[q['positions'][i+k]-a[12+k] for k in range(3)];points.extend(sum(a[4*j+k]*d[k] for k in range(3)) for j in range(3))
      q['positions']=points;return q
     mark,support=local(n),local(parent)
     if rear:
      def reverse(q):return dict(q,positions=[(-v if i%3!=1 else v) for i,v in enumerate(q['positions'])])
      mark,support=reverse(mark),reverse(support)
     for i in range(0,len(mark['positions']),3):
      x,y,z=mark['positions'][i:i+3];hits=ray_intersections(support,x,y);self.assertTrue(hits,(clip,n));self.assertLess(abs(z-max(hits)),.025,(clip,n))
 def test_gold_leading_arms_clear_the_posed_warm_and_hot_fire_layers(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for side,owner in (('-1',4),('1',5)):
     a=mats[owner]
     def local(name):
      q=self.source(name,mats);points=[]
      for i in range(0,len(q['positions']),3):
       d=[q['positions'][i+k]-a[12+k] for k in range(3)];points.extend(sum(a[4*j+k]*d[k] for k in range(3)) for j in range(3))
      q['positions']=points;return q
     arm=local(self.find('Wing / slim gold leading arm '+side));colors=[local(n) for n in self.records if n.startswith(('Wing glow /','Wing heart /')) and n.split()[-1]=='1' and n.split()[-2 if n.startswith('Wing glow /') else -3]==side];covered=0
     for i in range(24):
      for j in range(22):
       x=int(side)*(.13+i*.04)*author.SCALE;y=(.60+j*.025-author.LOW)*author.SCALE;h=ray_intersections(arm,x,y)
       if not h:continue
       for q in colors:
        hits=ray_intersections(q,x,y)
        if hits:covered+=1;self.assertGreater(max(h)-max(hits),.001,(clip,t,q['part'],x,y))
     self.assertGreater(covered,20)
 def test_attack_strokes_both_wings_and_independently_articulates_flames(self):
  m=self.m;mats=m.matrices('attack',.18)
  for side in ('-1','1'):
   n=self.find('Wing / slim gold leading arm '+side);p,r=self.records[n];tips=[k for k,v in enumerate(m.vertices[p]) if abs(v[0])>.85*author.SCALE];self.assertTrue(tips);self.assertLess(sum(m.point(p,k,mats)[1]-m.vertices[p][k][1] for k in tips)/len(tips),-.10)
  for prefix,parent in (('Wing fire / flowing red-orange flame bank -1',4),('Wing fire / flowing red-orange flame bank 1',5),('Crown /',3),('Tail /',1)):
   p,r=self.records[self.find(prefix)];self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[parent],v))>.005 for k,v in enumerate(m.vertices[p])),prefix)
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_moltres(self.blob[:-1])
if __name__=='__main__':unittest.main()
