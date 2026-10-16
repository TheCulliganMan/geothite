#!/usr/bin/env python3
"""Decoded planted talons, physical claw strike and coherent fitted features."""
import math,unittest
from pathlib import Path
import sneasel_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
PATH=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/sneasel.glb'
class SneaselSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=PATH.read_bytes();cls.m=skin.Model(cls.blob)
 def part(self,p,mats):return {'positions':[v for point in self.m.posed(p,mats) for v in point],'indices':self.m.indices[p]}
 def test_canonical_full_geometry_neutral_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_sneasel(author.decode_sneasel(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(38,8779,9936,13))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_face_palms_claws_feathers_and_fixed_feet(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Talon / paired')) else 3 if n.startswith(('Head /','Eye /','Mouth /')) or n=='Gem / fitted gold oval forehead' else 1 if n=='Gem / fitted gold oval chest' else 4 if n.startswith('Ear /') else 5 if n.startswith('Plume /') else 6 if n.startswith('Tail /') else (8 if (n.split()[-1] if n.startswith('Hand /') else n.split()[-2])=='-1' else 10) if n.startswith(('Hand /','Talon / hooked')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_contacts_neutral_endpoints_and_visible_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith(('Foot /','Talon / paired')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.001:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Hand','Talon','Ear','Plume','Tail'}<=moved,moved)
 def test_attack_advances_raised_palm_and_both_hooked_claws(self):
  m=self.m;attack=m.matrices('attack',.18);idle=m.matrices('idle',.18)
  for p,n in enumerate(m.names):
   if n=='Hand / broad claw-bearing palm -1' or n.startswith('Talon / hooked') and n.split()[-2]=='-1':
    neutral=sum(v[2] for v in m.vertices[p])/len(m.vertices[p]);forward=sum(v[2] for v in m.posed(p,attack))/len(m.vertices[p])-neutral;guard=sum(v[2] for v in m.posed(p,idle))/len(m.vertices[p])-neutral
    self.assertGreater(forward,.03,n);self.assertGreater(forward,3*abs(guard),n)
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
 def test_actual_posed_anatomical_contacts(self):
  m=self.m;pairs=[]
  def find(prefix):return next(p for p,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Body /');head=find('Head /');stem=find('Tail / short');pairs += [(head,body),(stem,body)]
  for p,n in enumerate(m.names):
   if n.startswith(('Ear /','Plume /')):pairs.append((p,head))
   if n.startswith('Tail / swept'):pairs.append((p,stem))
   if n.startswith(('Arm /','Leg /')):pairs.append((p,body))
   if n.startswith('Hand /'):pairs.append((p,m.names.index('Arm / curved reaching forelimb '+n.split()[-1])))
   if n.startswith('Talon / hooked'):pairs.append((p,m.names.index('Hand / broad claw-bearing palm '+n.split()[-2])))
   if n.startswith('Foot /'):pairs.append((p,m.names.index('Leg / curved outward hock '+n.split()[-1])))
   if n.startswith('Talon / paired'):pairs.append((p,m.names.index('Foot / broad grounded teal pad '+n.split()[-2])))
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.part(a,mats),self.part(b,mats)),(clip,t,m.names[a],m.names[b]))
 def test_posed_fitted_features_follow_actual_supports(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     side=n.split()[-1];parent=None
     if n.startswith('Eye / fitted dark'):parent='Head / tapered'
     elif n.startswith('Eye / fitted ivory'):parent='Eye / fitted dark slanted socket '+side
     elif n.startswith('Eye / narrow'):parent='Eye / fitted ivory angular white '+side
     elif n.startswith('Eye / vertical'):parent='Eye / narrow fitted red iris '+side
     elif n.startswith('Gem /'):parent='Head / tapered' if n.endswith('forehead') else 'Body / slim'
     elif n.startswith('Mouth / fitted open'):parent='Head / tapered'
     elif n.startswith(('Mouth / fitted pink','Mouth / small')):parent='Mouth / fitted open'
     if parent:
      q=next(i for i,name in enumerate(m.names) if name.startswith(parent));support=self.part(q,mats)
      for x,y,z in set(map(tuple,m.posed(p,mats))):
       hits=ray_intersections(support,x,y);self.assertTrue(hits,(clip,t,n));self.assertLess(abs(z-max(hits)),.008,(clip,t,n))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_sneasel(self.blob[:-1])
if __name__=='__main__':unittest.main()
