#!/usr/bin/env python3
"""Decoded Smoochum skin preserves grounded toes, fitted gaze and recessed kiss rim."""
import math,unittest
from pathlib import Path
import smoochum_glb as author
import test_gengar_glb as skin
import test_donphan_sculpt as joins
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/smoochum.glb'
class SmoochumSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.m=skin.Model(cls.blob)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_smoochum(author.decode_smoochum(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(38,17939,30344,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(0,0,0) if n.startswith(('Eye socket /','Pupil /','Mouth /')) else (255,255,255) if n.startswith(('Eye /','Glint /','Neck /','Chest /','Chest bib /','Arm /','Hand /','Finger /','Leg /','Foot /')) else (239,198,33) if n.startswith(('Hair','Bangs /','Lips /')) else (165,8,156)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_coherent_bob_gaze_lips_and_small_digit_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith('Foot /') else (4 if n.endswith('-1') else 5) if n.startswith('Hair side /') else 6 if n.startswith('Hair tuft /') else 3 if n.startswith(('Head /','Eye socket /','Eye /','Pupil /','Iris /','Glint /','Lips /','Mouth /','Hair','Bangs /')) else None
   if n.startswith(('Arm /','Hand /','Finger /')):
    side=n.split()[-2] if n.startswith('Finger /') else n.split()[-1];owner=7 if side=='-1' else 8
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_fixed_toes_endpoints_and_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith('Foot /'):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Eye','Lips','Hair side','Hair tuft','Arm','Hand','Finger','Leg'}<=moved,moved)
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
 def test_actual_posed_hair_neck_lips_limbs_and_digits_stay_rooted(self):
  m=self.m
  def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Body /');head=find('Head /');neck=find('Neck /');back=find('Hair back /');cap=find('Hair crown /');chest=find('Chest /')
  pairs=[(neck,body),(head,neck),(back,head),(cap,back),(find('Bangs /'),cap),(chest,body),(find('Lips /'),head)]
  pairs += [(find('Hair tuft / rooted golden baby leaf '+str(j)),cap) for j in range(4)]
  for side in ('-1','1'):
   arm=find('Arm / rooted short curved cream baby forelimb '+side);hand=find('Hand / small cream three-finger baby palm '+side);leg=find('Leg / rooted short cream baby support '+side)
   pairs += [(find('Hair side / rooted short golden baby side lock '+side),back),(arm,chest),(hand,arm),(leg,body),(find('Foot / grounded rounded cream baby toe '+side),leg)]
   pairs += [(find('Finger / rooted small cream baby digit '+side+' '+str(j)),hand) for j in range(3)]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:
     def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
     self.assertTrue(joins.DonphanTests.overlap(self,part(a),part(b)),(clip,t,m.names[a],m.names[b]))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_smoochum(self.blob[:-1])
if __name__=='__main__':unittest.main()
