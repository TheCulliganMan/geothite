#!/usr/bin/env python3
"""Decoded Girafarig skin: grounded hooves, coherent color seam and independent rear-facing second head."""
import math,unittest
from pathlib import Path
import girafarig_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/girafarig.glb'
class GirafarigSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.m=Model(cls.blob)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_girafarig(author.decode_girafarig(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(57,14025,22432,13))
  self.assertEqual(author.geometry_digest(author.decode_girafarig(self.blob)),author.REST_DIGEST)
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(255,255,255) if n.startswith(('Eye / fitted ivory','Eye / small','Horn /','Horn tip /','Tail mouth /')) else (231,82,132) if n.startswith(('Muzzle /','Ear / fitted','Mane /','Hoof /','Mouth / pink','Spot / fitted brown')) else (0,0,0) if n.startswith(('Spot center /','Eye / fitted dark','Eye / dark','Hoof groove /','Body / continuous brown','Tail /','Tail head /','Tail tooth /','Tail eye / dark','Mouth / fitted')) or (n.startswith('Leg /') and n.endswith('1')) else (255,181,33)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_fitted_main_and_tail_faces_share_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Hoof /','Hoof groove /')) else 3 if n.startswith(('Head /','Eye /','Mouth /','Muzzle /','Horn /','Horn tip /')) else (4 if n.endswith('-1') else 5) if n.startswith('Ear /') else 12 if n.startswith(('Tail head /','Tail eye /','Tail mouth /','Tail tooth /')) else 1 if n.startswith(('Body /','Spot / fitted gold','Mane / pink back')) or n=='Spot / fitted brown shoulder marking group 0' else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_packed_spots_restore_every_source_part_and_order(self):
  m=self.m;rest=author.decode_girafarig(self.blob)
  self.assertEqual(tuple(p['part'] for p in rest['primitives']),author.PARTS)
  groups=[x for x in m.doc['extras']['source_part_ranges'] if 'source_parts' in x]
  self.assertEqual(sorted(len(x['source_parts']) for x in groups),[2,2,4,6,6])
  self.assertEqual(len(rest['primitives']),72)
 def test_two_color_body_seam_has_identical_animated_positions(self):
  m=self.m;front={tuple(v):i for i,v in enumerate(m.vertices[0])};rear={tuple(v):i for i,v in enumerate(m.vertices[1])};shared=front.keys()&rear.keys()
  self.assertEqual(len(shared),32)
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80,1.):
    mats=m.matrices(clip,t);a=m.posed(0,mats);b=m.posed(1,mats)
    for key in shared:self.assertEqual(a[front[key]],b[rear[key]])
 def test_floor_fixed_hooves_endpoints_and_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Hoof /','Hoof groove /')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Eye','Muzzle','Horn','Ear','Tail','Tail head','Tail eye','Tail mouth','Leg','Mane','Spot'}<=moved,moved)
 def test_winding_and_edge_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.31,.55,.80,1.):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,t,n,ratio))
 def test_rotation_only_root_and_delayed_tail_curl(self):
  m=self.m
  for a in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  torso=[m.rotations('attack',i/100)[1][1] for i in range(101)];held=[m.rotations('attack',i/100)[12][0] for i in range(101)]
  self.assertGreater(held.index(max(held)),torso.index(max(torso)))
if __name__=='__main__':unittest.main()
