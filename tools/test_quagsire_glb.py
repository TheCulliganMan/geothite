#!/usr/bin/env python3
"""Independent stored Quagsire skin, grounded support and dorsal-field checks."""
import math,unittest
from pathlib import Path
import quagsire_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class QuagsireSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=Model((ROOT/'quagsire.glb').read_bytes())
 def test_canonical_bytes_geometry_hierarchy_and_budgets(self):
  m=self.model;blob=(ROOT/'quagsire.glb').read_bytes();neutral=author.decode_quagsire(blob)
  self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS['quagsire'])
  self.assertEqual(blob,author.export_quagsire(neutral));self.assertLess(len(blob),author.MAX_BYTES)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints),len(m.doc['accessors'])),(22,4380,6836,13,127))
  self.assertEqual(m.parents[:-1],[None,0,1,2,2,2,4,5,1,1,1,10,11])
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):
    self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<13 for j in js))
 def test_pack_palette_face_owners_and_continuous_dorsal_field(self):
  m=self.model
  for prefix,rgb in [('Body /',(115,173,231)),('Dorsal fin /',(115,24,165))]:
   part=next(p for p,n in zip(m.parts,m.names) if n.startswith(prefix))
   expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
   actual=m.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']
   for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
  for p,name in enumerate(m.names):
   if name.startswith(('Eye /','Mouth /','Nostril /')):self.assertTrue(all(ids==[3,0,0,0] for ids in m.ids[p]))
   if name.startswith(('Foot /','Toe crease /')):self.assertTrue(all(ids==[0,0,0,0] for ids in m.ids[p]))
   if name.startswith('Finger crease /'):
    owners={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w>0.}
    self.assertEqual(owners,{4,6} if sum(v[0] for v in m.vertices[p])<0 else {5,7})
  fin=m.names.index('Dorsal fin / continuous lavender back-to-tail ridge')
  owners={j for ids,ws in zip(m.ids[fin],m.weights[fin]) for j,w in zip(ids,ws) if w>0.}
  self.assertEqual(owners,{1,2,3,10,11,12})
  self.assertFalse(any(n.startswith(('Gill /','Gill branch /','Belly marking /')) for n in m.names))
 def test_floor_fixed_digits_winding_strain_and_neutral_endpoints(self):
  m=self.model;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith(('Foot /','Toe crease /')):self.assertEqual(posed,m.vertices[p],(clip,name))
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(name.split(' /')[0])
     if step%4==0:
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
       self.assertGreater(cosine,0.,(clip,step,name,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
        self.assertTrue(.70<ratio<1.30,(clip,step,name,ratio))
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Eye','Arm','Finger crease','Tail','Dorsal fin'}<=moved,moved)
 def test_early_head_brace_delayed_paddle_and_clip_contract(self):
  m=self.model
  head=[m.rotations('attack',i/100)[3][0] for i in range(101)]
  tail=[abs(m.rotations('attack',i/100)[12][0]) for i in range(101)]
  self.assertGreater(head[10],max(head)*.4);self.assertGreater(head[20],max(head)*.9)
  self.assertGreater(tail.index(max(tail)),head.index(max(head)))
  for a in m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
   self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
