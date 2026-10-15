#!/usr/bin/env python3
"""Decode the stored Aipom rig independently of the authoring skin fields."""
import math,unittest
from pathlib import Path
import aipom_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class AipomSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'aipom.glb').read_bytes();cls.model=Model(cls.blob)
 def test_canonical_geometry_and_neutral_bind(self):
  m=self.model;decoded=author.decode_aipom(self.blob)
  self.assertEqual(self.blob,author.export_aipom(decoded));self.assertLess(len(self.blob),1024*1024)
  self.assertEqual(author.geometry_digest(decoded),author.REST_DIGESTS['aipom'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(50,17804,30016,20))
  self.assertLessEqual(len(m.doc['accessors']),256)
  self.assertEqual(m.parents[:-1],[None,0,1,2,3,3,1,6,1,8,1,10,11,12,13,13,13,3,3,3])
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_face_supports_and_tail_fingertips_share_owners(self):
  m=self.model
  for p,n in enumerate(m.names):
   owner=3 if n.startswith(('Head /','Face /','Eye /','Pupil /','Grin /')) else 13 if n.startswith('Tail hand /') else 14+int(n.rsplit(' ',1)[1]) if n.startswith('Tail fingertip /') else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
  tail=m.names.index('Tail / long curled prehensile tail')
  self.assertTrue({1,10,11,12,13}<={j for ids,ws in zip(m.ids[tail],m.weights[tail]) for j,w in zip(ids,ws) if w})
 def test_verified_pack_purple_and_tan_materials(self):
  m=self.model
  for prefix,rgb in (('Body /',(99,33,115)),('Tail hand /',(239,189,99))):
   p=next(i for i,n in enumerate(m.names) if n.startswith(prefix))
   actual=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
   for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
 def test_tail_finger_pivots_and_curl_axes_follow_the_palm_turn(self):
  m=self.model;root=[-v for v in m.inverse[13][12:15]]
  for joint in (14,15,16):
   finger=[-v for v in m.inverse[joint][12:15]]
   self.assertAlmostEqual((finger[2]-root[2])/(finger[0]-root[0]),math.tan(math.radians(35)),places=5)
  q=m.rotations('attack',.31)[16]
  self.assertAlmostEqual(q[2]/q[0],math.tan(math.radians(35)),places=6)
  self.assertAlmostEqual(q[1],0.,places=7)
 def test_fixed_feet_floor_winding_strain_and_endpoints(self):
  m=self.model;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,n))
     if n.startswith(('Foot /','Toe fold /')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step%6==0:
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
       self.assertGreater(cosine,0.,(clip,step,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
        self.assertTrue(.80<ratio<1.20,(clip,step,n,ratio))
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Ear','Arm','Hand','Tail','Tail hand','Tail finger','Tail fingertip','Tuft'}<=moved,moved)
 def test_delayed_tail_palm_and_rotation_only_contract(self):
  m=self.model
  torso=[m.rotations('attack',i/100)[1][0] for i in range(101)]
  palm=[m.rotations('attack',i/100)[13][0] for i in range(101)]
  self.assertGreater(torso[10],max(torso)*.4);self.assertGreater(palm.index(max(palm)),torso.index(max(torso)))
  for a in m.doc['animations']:
   self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'))
   self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
