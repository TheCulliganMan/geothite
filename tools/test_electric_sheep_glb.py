#!/usr/bin/env python3
"""Decoded electric sheep: planted supports, continuous band seams and fitted beacons."""
import math,unittest
from pathlib import Path
import electric_sheep_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ElectricSheepSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()}
 def test_canonical_anatomy_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_electric_sheep(s,author.decode_electric_sheep(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'mareep':(26,6382,11840,13),'flaaffy':(33,10302,16636,13),'ampharos':(26,2402,4700,9)}[s])
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palettes(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    if s=='ampharos':rgb=(0,0,0) if n.startswith(('Horn / rounded','Eye / black','Nostril /','Neck /','Muzzle /','Tail / fitted')) else (255,255,255) if n.startswith(('Belly /','Eye / catchlight','Forehead / restrained','Tail / quiet')) else (173,82,8) if n.startswith(('Forehead / inset','Tail / ruby')) else (255,255,16)
    else:
     mid=s=='flaaffy';skin=(255,107,156) if mid else (82,41,156);wool=(255,255,255) if mid else (198,165,57);shade=(82,24,115) if mid else skin
     if n.startswith(('Ear /','Tail / curved')):rgb=(0,0,0) if int(n.split()[-1])%2==0 else skin if mid else (198,165,57)
     elif n.startswith(('Fleece /','Crown /')):rgb=wool
     elif n.startswith('Crown groove /'):rgb=shade
     elif n.startswith(('Eye / outer','Toe /','Tail / ivory')):rgb=(255,255,255)
     elif n.startswith(('Eye / black','Nostril /')):rgb=(0,0,0)
     elif n.startswith('Tail / round'):rgb=shade if mid else (198,165,57)
     else:rgb=skin
    color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for v,a in zip(rgb,color):
     x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_fitted_crowns_faces_horns_feet_and_beacon_owners(self):
  for s,m in self.models.items():
   beacon=10 if s=='flaaffy' else 8
   for p,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Toe /')) else 3 if n.startswith(('Crown /','Crown groove /','Eye /','Nostril /','Muzzle /','Forehead /','Horn /')) else beacon if n.startswith(('Tail / round','Tail / ivory','Tail / ruby','Tail / quiet')) else 1 if s=='mareep' and n.startswith('Fleece /') else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_shared_ear_and_tail_band_seams_survive_all_clips(self):
  for s in ['mareep','flaaffy']:
   m=self.models[s]
   for prefix,count in [('Tail / curved alternating segment',4),('Ear / -1',3),('Ear / 1',3)]:
    parts=[m.names.index(prefix+' band '+str(k)) for k in range(count)]
    for p,q in zip(parts,parts[1:]):
     a={tuple(v):i for i,v in enumerate(m.vertices[p])};b={tuple(v):i for i,v in enumerate(m.vertices[q])};shared=a.keys()&b.keys();self.assertEqual(len(shared),14)
     for clip in m.clips:
      for t in (0.,.18,.31,.55,.80,1.):
       mats=m.matrices(clip,t);ap=m.posed(p,mats);bp=m.posed(q,mats)
       for key in shared:self.assertEqual(ap[a[key]],bp[b[key]],(s,clip,p,q))
 def test_floor_fixed_supports_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      if n.startswith(('Foot /','Toe /')):self.assertEqual(posed,m.vertices[p])
      for i,v in enumerate(m.vertices[p]):
       if v[1]==0.:self.assertEqual(posed[i],v,(s,n))
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   expected={'Head','Eye','Ear','Crown','Crown groove','Tail','Leg','Fleece'} if s=='mareep' else {'Body','Fleece','Arm','Head','Eye','Ear','Crown','Tail','Leg'} if s=='flaaffy' else {'Body','Belly','Flipper','Horn','Eye','Neck','Forehead','Tail'}
   self.assertTrue(expected<=moved,(s,moved))
 def test_winding_and_edge_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,.18,.31,.55,.80,1.):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,n,ratio))
 def test_ampharos_actual_foot_body_and_ring_contacts(self):
  m=self.models['ampharos'];body=next(p for p,n in enumerate(m.names) if n.startswith('Body /'));tail=next(p for p,n in enumerate(m.names) if n.startswith('Tail / continuous'))
  def part(p,mats):return {'positions':[c for v in m.posed(p,mats) for c in v],'indices':m.indices[p]}
  def overlap(a,b):
   xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
   if xh<=xl or yh<=yl:return False
   for i in range(1,19):
    for j in range(1,23):
     aa=sorted(ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23));bb=sorted(ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23))
     for al,ah in zip(aa[::2],aa[1::2]):
      for bl,bh in zip(bb[::2],bb[1::2]):
       if min(ah,bh)-max(al,bl)>.0001:return True
   return False
  for clip,t in [('idle',.31),('attack',.31),('hit',.55)]:
   mats=m.matrices(clip,t)
   for p,n in enumerate(m.names):
    if n.startswith(('Foot /','Neck /')):support=body
    elif n.startswith('Tail / fitted'):support=tail
    else:continue
    self.assertTrue(overlap(part(p,mats),part(support,mats)),(clip,n))
 def test_identity_root_rotation_only_and_delayed_beacon(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
   base,tip=(8,10) if s=='flaaffy' else (6,8)
   first=[m.rotations('attack',i/100)[base][1] for i in range(101)];last=[m.rotations('attack',i/100)[tip][0] for i in range(101)]
   self.assertGreater(last.index(max(last)),first.index(max(first)))
if __name__=='__main__':unittest.main()
