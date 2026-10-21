#!/usr/bin/env python3
"""Decoded Hitmonlee skin: rooted coils, one planted support and a real lifted kick."""
import math,os,unittest
from pathlib import Path
import hitmonlee_glb as author
import test_gengar_glb as skin
import test_donphan_sculpt as joins
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_HITMONLEE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))/'hitmonlee.glb'
class HitmonleeSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.m=skin.Model(cls.blob)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_hitmonlee(author.decode_hitmonlee(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(33,15445,25744,10))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(0,0,0) if n.startswith(('Eye /','Pupil core /')) else (255,255,255) if n.startswith(('Pupil /','Toe claw /')) else (123,132,123) if n.startswith(('Leg spring /','Wrist spring /','Foot mark /')) else (181,115,41)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_fitted_gaze_complete_feet_and_curled_hand_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=5 if n.startswith(('Eye /','Pupil /','Pupil core /')) else None
   if n.startswith(('Foot /','Toe claw /','Foot mark /')):
    side=n.split()[-2] if n.startswith('Toe claw /') else n.split()[-1];owner=3 if side=='-1' else 0
   if n.startswith(('Hand /','Finger /')):
    side=n.split()[-2] if n.startswith('Finger /') else n.split()[-1];owner=7 if side=='-1' else 9
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_support_stays_fixed_kick_lifts_and_returns_without_floor_penetration(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(25):
    mats=m.matrices(clip,step/24)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Foot /','Toe claw /','Foot mark /')):
      side=n.split()[-2] if n.startswith('Toe claw /') else n.split()[-1]
      if side=='1':self.assertEqual(posed,m.vertices[p])
      elif clip!='attack' or step in (0,24):self.assertLess(max(math.dist(a,b) for a,b in zip(posed,m.vertices[p])),1e-7)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,24):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  p=next(i for i,n in enumerate(m.names) if n.startswith('Foot /') and n.endswith('-1'));posed=m.posed(p,m.matrices('attack',.18));self.assertGreater(min(v[1] for v in posed),.10);self.assertGreater(sum(v[2] for v in posed)/len(posed)-sum(v[2] for v in m.vertices[p])/len(posed),.20)
  self.assertTrue({'Body','Eye','Hip','Leg spring','Foot','Toe claw','Foot mark','Arm','Wrist spring','Hand','Finger'}<=moved,moved)
 def test_winding_and_edge_strain(self):
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
 def test_actual_posed_hip_coil_foot_toes_arm_wrist_hand_and_digit_roots(self):
  m=self.m
  def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Body /');pairs=[]
  for side in ('-1','1'):
   hip=find('Hip / rooted tapered tan kicking thigh '+side);leg=find('Leg spring / continuous cream nine-rib shin '+side);foot=find('Foot / grounded broad tan three-toed pad '+side);arm=find('Arm / rooted bent tan upper forelimb '+side);wrist=find('Wrist spring / continuous cream five-rib forearm '+side);hand=find('Hand / tan three-finger martial palm '+side)
   pairs += [(hip,body),(leg,hip),(foot,leg),(arm,body),(wrist,arm),(hand,wrist)]
   pairs += [(find('Toe claw / three rooted ivory kicking points '+side+' '+str(j)),foot) for j in range(3)]+[(find('Finger / three rooted curled tan martial digits '+side+' '+str(j)),hand) for j in range(3)]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:
     def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
     self.assertTrue(joins.DonphanTests.overlap(self,part(a),part(b)),(clip,t,m.names[a],m.names[b]))
 def test_neutral_coils_keep_original_ring_density(self):
  m=self.m
  for p,n in enumerate(m.names):
   if n.startswith(('Leg spring /','Wrist spring /')):
    self.assertEqual(len(set(map(tuple,m.vertices[p]))),1568,n)
    self.assertEqual(len(m.indices[p])//3,3132,n)
    self.assertEqual(len(m.vertices[p]),1856 if n.startswith('Leg spring /') else 1753 if n.endswith('-1') else 1698,n)
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_hitmonlee(self.blob[:-1])
if __name__=='__main__':unittest.main()
