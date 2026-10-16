#!/usr/bin/env python3
"""Decoded psychic birds: planted talons, rigid wing coatings and fitted symbolism."""
import math,unittest
from pathlib import Path
import psychic_bird_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class PsychicBirdSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()}
 def test_canonical_anatomy_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_psychic_bird(s,author.decode_psychic_bird(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(46,18853,28820,10) if s=='natu' else (49,15289,24060,11))
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    rgb=(82,189,107) if n.startswith(('Body /','Head /')) else (0,0,0) if n.startswith(('Pupil /','Bill seam /','Wing / dark','Collar /','Chest pupil /','Wing hem /')) else (165,41,49) if n.startswith(('Leg /','Toe /','Crest /','Wing red /','Tail /','Chest symbol /','Wing badge /','Wing hem red /','Streamer /')) or (s=='natu' and n.startswith('Beak /')) else (255,255,255)
    color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for v,a in zip(rgb,color):
     x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_gaze_bill_and_all_wing_layers_share_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=0 if n.startswith(('Toe /','Claw /')) else 2 if n.startswith(('Head /','Eye /','Pupil /','Eye glint /','Beak /','Bill seam /')) else None
    if n.startswith(('Wing /','Wing gold /','Wing red /','Wing badge','Wing hem')):
     core=n.removesuffix(' rear');side=core.split()[-2] if n.startswith(('Wing red /','Wing hem')) else core.split()[-1];owner=3 if side=='-1' else 4
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_floor_fixed_toes_talons_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      if n.startswith(('Toe /','Claw /')):self.assertEqual(posed,m.vertices[p])
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   expected={'Body','Eye','Beak','Wing','Leg'}|({'Crest','Wing gold','Wing red','Tail'} if s=='natu' else {'Head','Collar','Chest symbol','Chest pupil','Wing badge','Wing badge gold','Wing hem','Wing hem red','Back stripe','Streamer'})
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
 def test_xatu_symbols_and_collar_remain_fitted_during_motion(self):
  from test_axolotl_sculpt import ray_intersections
  m=self.models['xatu'];body=next(p for p,n in enumerate(m.names) if n.startswith('Body /'))
  def part(p,mats):return {'positions':[c for v in m.posed(p,mats) for c in v],'indices':m.indices[p]}
  for clip,t in [('idle',.31),('attack',.31),('hit',.55)]:
   mats=m.matrices(clip,t);support=part(body,mats)
   for p,n in enumerate(m.names):
    if not n.startswith(('Collar /','Chest symbol /','Back stripe /')):continue
    for x,y,z in {tuple(v) for v in m.posed(p,mats)}:
     hits=ray_intersections(support,x,y);self.assertTrue(hits,(clip,n))
     self.assertLess(abs(z-(min(hits) if n.startswith('Back stripe /') else max(hits))),.026,(clip,n))
 def test_identity_root_rotation_only_and_delayed_streamer_tip(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
  m=self.models['xatu'];base=[m.rotations('attack',i/100)[5][0] for i in range(101)];tip=[m.rotations('attack',i/100)[6][0] for i in range(101)]
  self.assertGreater(tip.index(max(tip)),base.index(max(base)))
if __name__=='__main__':unittest.main()
