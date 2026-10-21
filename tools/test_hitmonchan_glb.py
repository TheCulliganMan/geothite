#!/usr/bin/env python3
"""Decoded Hitmonchan skin: fixed boots, fitted crown and coherent boxing gloves."""
import math,os,unittest
from pathlib import Path
import hitmonchan_glb as author
import test_gengar_glb as skin
import test_donphan_sculpt as joins
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_HITMONCHAN_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))/'hitmonchan.glb'
class HitmonchanSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.m=skin.Model(cls.blob)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_hitmonchan(author.decode_hitmonchan(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(34,10981,16084,12))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(0,0,0) if n.startswith(('Eye socket /','Iris /','Pupil /','Mouth /','Belt /','Glove seam /')) else (255,255,255) if n.startswith(('Eye /','Glint /')) else (222,24,148) if n.startswith(('Tunic /','Skirt /','Boot /','Boot cuff /','Boot seam /','Glove')) else (173,123,99)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_coherent_crown_gaze_shoulders_and_complete_glove_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Boot /','Boot cuff /','Boot seam /')) else 3 if n.startswith(('Head /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Mouth /')) else (4 if n.endswith('-1') else 5) if n.startswith('Shoulder guard /') else (7 if n.endswith('-1') else 9) if n.startswith(('Glove cuff /','Glove /','Glove thumb /','Glove seam /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_fixed_boots_endpoints_and_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith(('Boot /','Boot cuff /','Boot seam /')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Tunic','Skirt','Head','Eye','Shoulder guard','Arm','Glove','Glove thumb','Leg'}<=moved,moved)
 def test_both_boxing_mitts_advance_during_attack(self):
  m=self.m;mats=m.matrices('attack',.18)
  for p,n in enumerate(m.names):
   if n.startswith('Glove /'):
    original=sum(v[2] for v in m.vertices[p])/len(m.vertices[p]);posed=sum(v[2] for v in m.posed(p,mats))/len(m.vertices[p]);self.assertGreater(posed-original,.01,n)
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
 def test_actual_posed_neck_costume_arms_gloves_thumbs_and_boot_roots(self):
  m=self.m
  def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Tunic /');head=find('Head /');neck=find('Neck /');skirt=find('Skirt /');belt=find('Belt /')
  pairs=[(neck,body),(head,neck),(skirt,body),(belt,body),(belt,skirt)]
  for side in ('-1','1'):
   shoulder=find('Shoulder guard / rooted pointed tan boxing armor '+side);arm=find('Arm / rooted bent tan boxing forelimb '+side);cuff=find('Glove cuff / rooted dark red wrist collar '+side);glove=find('Glove / rounded padded boxing mitt '+side);leg=find('Leg / rooted long tan boxing support '+side);boot=find('Boot / grounded broad lavender boxing shoe '+side);ankle=find('Boot cuff / rooted lavender ankle collar '+side)
   pairs += [(shoulder,body),(arm,shoulder),(arm,body),(cuff,arm),(glove,cuff),(find('Glove thumb / rooted curled boxing fold '+side),glove),(leg,body),(boot,leg),(ankle,leg),(ankle,boot)]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:
     def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
     self.assertTrue(joins.DonphanTests.overlap(self,part(a),part(b)),(clip,t,m.names[a],m.names[b]))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_hitmonchan(self.blob[:-1])
if __name__=='__main__':unittest.main()
