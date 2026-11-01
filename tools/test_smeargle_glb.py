#!/usr/bin/env python3
"""Decoded painter grip, shared band/paint seams and planted paw contacts."""
import math,unittest
from pathlib import Path
import smeargle_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
PATH=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/smeargle.glb'
class SmeargleSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=PATH.read_bytes();cls.m=skin.Model(cls.blob)
 def sources(self,mats):
  m=self.m;parts={}
  for p,meta in enumerate(m.doc['extras']['source_part_ranges']):
   positions=m.posed(p,mats);indices=m.indices[p]
   ranges=meta.get('source_parts',[{'part':m.names[p],'vertex_start':0,'vertex_count':len(positions),'index_start':0,'index_count':len(indices)}])
   for src in ranges:
    a=src['vertex_start'];b=a+src['vertex_count'];i=src['index_start'];j=i+src['index_count'];n=src['part']
    parts[n]={'part':n,'positions':[v for point in positions[a:b] for v in point],'indices':[k-a for k in indices[i:j]]}
  return parts
 def test_canonical_geometry_packed_ranges_bind_and_budgets(self):
  m=self.m;rest=author.decode_smeargle(self.blob);self.assertEqual(self.blob,author.export_smeargle(rest))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(43,16875,21004,13))
  self.assertEqual(author.geometry_digest(rest),author.REST_DIGEST);self.assertEqual(tuple(p['part'] for p in rest['primitives']),author.PARTS);self.assertEqual(len(rest['primitives']),65)
  self.assertEqual(sorted(len(x['source_parts']) for x in m.doc['extras']['source_part_ranges'] if 'source_parts' in x),[4,4,9,9])
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_face_grip_brush_and_fixed_paws(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Toe groove /','Cuff /')) else 3 if n.startswith(('Head /','Eye /','Muzzle /','Nostril /','Mouth /','Tongue /')) else 6 if n.startswith('Beret /') else (4 if n.endswith('-1') else 5) if n.startswith('Ear /') else 10 if n.startswith('Brush /') else (8 if (n.split()[-1] if n.startswith('Hand /') else n.split()[-2])=='-1' else 10) if n.startswith(('Hand /','Finger /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_contacts_neutral_endpoints_and_visible_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith(('Foot /','Toe groove /','Cuff /')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.001:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Hand','Finger','Ear','Beret','Tail','Brush'}<=moved,moved)
 def test_attack_advances_paint_tip_and_retains_held_tail_contact(self):
  m=self.m;p=next(i for i,n in enumerate(m.names) if n.startswith('Brush / tapered'));neutral=sum(v[2] for v in m.vertices[p])/len(m.vertices[p]);attack=m.matrices('attack',.18);idle=m.matrices('idle',.18)
  forward=sum(v[2] for v in m.posed(p,attack))/len(m.vertices[p])-neutral;guard=sum(v[2] for v in m.posed(p,idle))/len(m.vertices[p])-neutral
  self.assertGreater(forward,.035);self.assertGreater(forward,2*abs(guard))
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80,1.):
    ps=self.sources(m.matrices(clip,t));tail=ps['Tail / continuous looping cream paintbrush handle'];hand=ps['Hand / cream open palm 1'];self.assertTrue(parts_overlap(tail,hand),(clip,t))
 def test_banded_arm_rings_and_scalloped_paint_boundary_stay_exact(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80,1.):
    ps=self.sources(m.matrices(clip,t))
    def points(p):return {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}
    for side in (-1,1):
     bands=[ps[f'Arm / cream forelimb and brown wrist bands {side} band {j}'] for j in range(13)]
     for a,b in zip(bands,bands[1:]):self.assertEqual(len(points(a)&points(b)),14,(clip,t,side))
    self.assertEqual(len(points(ps['Brush / cream bristles beneath scalloped paint'])&points(ps['Brush / tapered green paint tuft'])),36,(clip,t))
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
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.55):
    ps=self.sources(m.matrices(clip,t));pairs=[]
    def name(prefix):return next(n for n in ps if n.startswith(prefix))
    body=name('Body /');neck=name('Neck /');head=name('Head /');tail=name('Tail /');cap=name('Beret / continuous')
    pairs += [(neck,body),(head,neck),(name('Muzzle /'),head),(name('Tongue /'),head),(cap,head),(name('Beret / bent'),cap),(tail,body),(tail,name('Brush / cream'))]
    for side in (-1,1):
     leg=f'Leg / slim cream canine shin {side}';foot=f'Foot / broad grounded cream three-toe paw {side}';hand=f'Hand / cream open palm {side}'
     pairs += [(leg,body),(foot,leg),(f'Cuff / brown ankle marking {side}',leg),(f'Ear / short floppy brown dog ear {side}',head),(f'Arm / cream forelimb and brown wrist bands {side} band 0',body),(hand,f'Arm / cream forelimb and brown wrist bands {side} band 12')]
     pairs += [(f'Finger / rounded canine digit {side} {j}',hand) for j in range(3)]
    for a,b in pairs:self.assertTrue(parts_overlap(ps[a],ps[b]),(clip,t,a,b))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_smeargle(self.blob[:-1])
if __name__=='__main__':unittest.main()
