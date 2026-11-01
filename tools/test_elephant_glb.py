#!/usr/bin/env python3
"""Decoded elephant skins: planted pads, fitted ear units, lifted foot and tire tread."""
import math,unittest
from pathlib import Path
import elephant_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
import test_donphan_sculpt as donphan_checks
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ElephantSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()}
 def test_canonical_anatomy_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_elephant(s,author.decode_elephant(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(32,18472,31284,13) if s=='phanpy' else (49,17395,29596,13))
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palettes(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    if s=='phanpy':rgb=(189,74,49) if n.startswith(('Forehead /','Ear tip /','Tongue /')) else (255,255,255) if n.startswith(('Eye glint /','Toenail /')) else (0,0,0) if n.startswith(('Eye /','Nostril /','Mouth /')) else (140,165,214)
    else:rgb=(255,255,255) if n.startswith(('Tusk /','Toenail /','Eye white /')) else (0,0,0) if n.startswith(('Eye /','Pupil /')) else (41,90,66) if n.startswith(('Tire /','Ear /','Ear lining /','Ankle armor /','Tail /')) else (148,165,156)
    color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for v,a in zip(rgb,color):
     x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_fitted_faces_ear_units_pads_and_toes_share_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=1 if n.startswith('Body /') else 4 if n.startswith(('Nose /','Nostril /')) else 2 if n.startswith(('Head /','Eye /','Eye glint /','Eye white /','Pupil /','Forehead /','Mouth /','Tongue /','Tusk /')) else None
    if n.startswith(('Foot /','Toenail /','Ankle armor /')):owner=9 if s=='phanpy' and n.endswith('-1 front') else 0
    if n.startswith(('Ear /','Ear tip /','Ear lining /')):
     side=n.split()[-2] if n.startswith('Ear tip /') else n.split()[-1];owner=5 if side=='-1' else 6
    if n.startswith('Tread /'):
     plate=int(n.split()[-1]);owner=1 if plate<3 else 2 if plate<5 else 3 if plate<7 else 4
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_floor_fixed_supports_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      fixed=n.startswith(('Foot /','Toenail /','Ankle armor /')) and not (s=='phanpy' and n.endswith('-1 front'))
      if fixed:self.assertEqual(posed,m.vertices[p],(s,n))
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue(({'Body','Head','Trunk','Nose','Forehead','Ear','Ear tip','Tail','Leg','Foot'} if s=='phanpy' else {'Body','Head','Tire','Tread','Ear','Ear lining','Tusk','Tail','Leg'})<=moved,(s,moved))
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
 def test_all_tread_plates_retain_actual_tire_volume_contact_during_motion(self):
  m=self.models['donphan'];tire=next(p for p,n in enumerate(m.names) if n.startswith('Tire /'))
  def posed_part(p,mats):return {'part':m.names[p],'positions':[c for v in m.posed(p,mats) for c in v],'indices':m.indices[p]}
  for clip,t in [('attack',.31),('hit',.55),('idle',.31)]:
   mats=m.matrices(clip,t);support=posed_part(tire,mats)
   for p,n in enumerate(m.names):
    if n.startswith('Tread /'):self.assertTrue(donphan_checks.DonphanTests.overlap(self,support,posed_part(p,mats)),(clip,t,n))
 def test_identity_root_rotation_only_and_delayed_trunk_tip(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
   base=[m.rotations('attack',i/100)[3][0] for i in range(101)];tip=[m.rotations('attack',i/100)[4][0] for i in range(101)]
   self.assertGreater(tip.index(max(tip)),base.index(max(base)))
if __name__=='__main__':unittest.main()
