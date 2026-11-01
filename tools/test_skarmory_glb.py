#!/usr/bin/env python3
"""Decoded metal wing strokes, planted talons and connected opening bill."""
import math,unittest
from pathlib import Path
import skarmory_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap,ray_intersections
PATH=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/skarmory.glb'
class SkarmorySkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=PATH.read_bytes();cls.m=skin.Model(cls.blob)
 def part(self,p,mats):return {'positions':[v for point in self.m.posed(p,mats) for v in point],'indices':self.m.indices[p]}
 def test_canonical_full_geometry_neutral_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_skarmory(author.decode_skarmory(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(42,3635,4288,10))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_rigid_armor_and_fixed_talons(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith('Claw /') else 3 if n.startswith(('Head /','Eye /','Crest /','Nape /')) else 9 if n.startswith('Bill /') else 6 if n.startswith('Tail /') else (3 if n.endswith(' 3') else 2) if n.startswith('Neck /') else (4 if (n.split()[-1] if n.startswith('Wing / swept') else n.split()[-2])=='-1' else 5) if n.startswith('Wing /') else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_contacts_neutral_endpoints_and_visible_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith('Claw /'):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.001:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Wing','Tail','Neck','Bill'}<=moved,moved)
 def test_attack_strokes_both_wings_and_opens_lower_bill(self):
  m=self.m;mats=m.matrices('attack',.18)
  for p,n in enumerate(m.names):
   if n.startswith('Wing / swept'):
    k=max(range(len(m.vertices[p])),key=lambda i:abs(m.vertices[p][i][0]));self.assertLess(m.point(p,k,mats)[1]-m.vertices[p][k][1],-.10,n)
  p=next(i for i,n in enumerate(m.names) if n.startswith('Bill / tapered'));k=max(range(len(m.vertices[p])),key=lambda i:m.vertices[p][i][2]);point=m.vertices[p][k]
  self.assertLess(m.point(p,k,mats)[1]-transform(mats[3],point)[1],-.008)
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12);self.assertLessEqual(m.point(p,k,mats)[1]-transform(mats[3],point)[1],1e-7)
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
 def test_actual_posed_armor_and_joint_contacts(self):
  m=self.m;pairs=[]
  def find(prefix):return next(p for p,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Body /');head=find('Head /');lower=find('Bill / tapered');pairs.append((head,body))
  for p,n in enumerate(m.names):
   if n.startswith(('Crest /','Nape /','Bill / tapered')):pairs.append((p,head))
   if n.startswith('Bill / dark'):pairs.append((p,lower))
   if n.startswith(('Wing /','Tail /','Thigh /')):pairs.append((p,body))
   if n.startswith('Leg /'):pairs.append((p,m.names.index('Thigh / articulated silver haunch '+n.split()[-1])))
   if n.startswith('Claw /'):
    side=n.split()[-1] if n.startswith('Claw / hooked rear') else n.split()[-2];pairs.append((p,m.names.index('Leg / bent dark steel hock '+side)))
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.part(a,mats),self.part(b,mats)),(clip,t,m.names[a],m.names[b]))
 def test_posed_fitted_eyes_and_neck_seams_follow_actual_supports(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     parent='Body /' if n.startswith('Neck /') else 'Head /' if n.startswith('Eye / fitted dark') else 'Eye / fitted dark angular socket '+n.split()[-1] if n.startswith('Eye / fitted yellow') else 'Eye / fitted yellow slit '+n.split()[-1] if n.startswith('Eye / dark narrow') else None
     if parent:
      q=next(i for i,name in enumerate(m.names) if name.startswith(parent));support=self.part(q,mats)
      for x,y,z in set(map(tuple,m.posed(p,mats))):
       hits=ray_intersections(support,x,y);self.assertTrue(hits,(clip,t,n));self.assertLess(abs(z-max(hits)),.008,(clip,t,n))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_skarmory(self.blob[:-1])
if __name__=='__main__':unittest.main()
