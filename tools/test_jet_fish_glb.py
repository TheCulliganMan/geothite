#!/usr/bin/env python3
"""Decoded fin strokes, grounded curling arms and an open aiming siphon."""
import math,unittest
from pathlib import Path
import jet_fish_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import ray_intersections,parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class JetFishSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def part(self,m,p,mats):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
 def test_canonical_geometry_neutral_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_jet_fish(s,author.decode_jet_fish(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'remoraid':(18,6822,7724,7),'octillery':(52,19798,21740,11)}[s])
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_facial_details_and_crown_spots(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    if n.startswith(('Eye /','Mouth /','Stripe /','Crown /')):self.assertTrue(all(ids==[1,0,0,0] for ids in m.ids[p]),(s,n))
    if s=='remoraid' and n.startswith('Tail /'):self.assertTrue(all(ids==[2,0,0,0] for ids in m.ids[p]),n)
 def test_floor_contacts_neutral_endpoints_and_individual_appendage_motion(self):
  for s,m in self.models.items():
   moved=set();floor=[(p,k) for p,vs in enumerate(m.vertices) for k,v in enumerate(vs) if v[1]<.025]
   self.assertTrue(floor)
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,n))
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.001:moved.add(n)
     for p,k in floor:
      if s=='octillery':self.assertEqual(m.point(p,k,mats),m.vertices[p][k])
      else:self.assertAlmostEqual(m.point(p,k,mats)[1],m.vertices[p][k][1],places=7)
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for p,n in enumerate(m.names):
    if n.startswith(('Tail /','Pectoral /','Arm /','Siphon /')):self.assertIn(n,moved,(s,n))
 def test_winding_and_edge_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.31,.55,.80):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,n,ratio))
 def test_actual_posed_fin_and_tentacle_roots(self):
  for s,m in self.models.items():
   body=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
   parts=[p for p,n in enumerate(m.names) if n.startswith(('Tail /','Dorsal /','Pelvic /','Pectoral /','Arm /','Siphon / real'))]
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t)
     for p in parts:self.assertTrue(parts_overlap(self.part(m,p,mats),self.part(m,body,mats)),(s,clip,t,m.names[p]))
 def test_posed_suckers_keep_open_centers_and_actual_arm_support(self):
  m=self.models['octillery']
  def find(n):return m.names.index(n)
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for i in range(8):
     arm=self.part(m,find('Arm / smooth curling tentacle '+str(i)),mats)
     arm['positions']=[v for k in range(0,len(arm['positions']),3) for v in (arm['positions'][k],arm['positions'][k+2],arm['positions'][k+1])]
     for j in range(2):
      rim=self.part(m,find(f'Sucker / raised yellow rim {i} {j}'),mats);well=self.part(m,find(f'Sucker / recessed gold well {i} {j}'),mats)
      points=list(zip(rim['positions'][::3],rim['positions'][1::3],rim['positions'][2::3]));cx=(min(v[0] for v in points)+max(v[0] for v in points))/2;cz=(min(v[2] for v in points)+max(v[2] for v in points))/2
      vertical=dict(rim);vertical['positions']=[v for x,y,z in points for v in (x,z,y)]
      for dx,dz in ((0,0),(.001,0),(-.001,0),(0,.001),(0,-.001)):self.assertFalse(ray_intersections(vertical,cx+dx,cz+dz),(clip,t,i,j))
      hits=ray_intersections(arm,cx,cz);self.assertTrue(hits);top=max(hits)
      self.assertLess(min(rim['positions'][1::3]),top+.003);self.assertLess(max(rim['positions'][1::3])-top,.035)
      self.assertLess(min(well['positions'][1::3]),top+.003);self.assertGreater(max(well['positions'][1::3]),top-.005)
 def test_aimed_siphon_retains_open_lip_and_recessed_lining(self):
  m=self.models['octillery'];outer=next(p for p,n in enumerate(m.names) if n.startswith('Siphon / real'));inner=next(p for p,n in enumerate(m.names) if n.startswith('Siphon / dark'))
  front=max(v[2] for v in m.vertices[outer]);lip=[v for v in m.vertices[outer] if front-v[2]<1e-6];cx=sum(v[0] for v in lip)/len(lip);cy=sum(v[1] for v in lip)/len(lip)
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t);a=mats[2]
    def undo(v):
     d=[v[k]-a[12+k] for k in range(3)];return [sum(a[4*i+k]*d[k] for k in range(3)) for i in range(3)]
    for p in (outer,inner):
     q={'positions':[v for point in m.posed(p,mats) for v in undo(point)],'indices':m.indices[p]}
     for dx,dy in ((0,0),(.001,0),(-.001,0),(0,.001),(0,-.001)):
      hits=ray_intersections(q,cx+dx,cy+dy);self.assertTrue(hits);self.assertLess(max(hits),front-.035,(clip,t,m.names[p]))
 def test_noncanonical_data_rejected(self):
  for s,blob in self.blobs.items():
   with self.assertRaises(ValueError):author.decode_jet_fish(blob[:-1],s)
if __name__=='__main__':unittest.main()
