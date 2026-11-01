#!/usr/bin/env python3
"""Independent stored Skiploom support, leaf-ear and flower skin checks."""
import math,unittest
from pathlib import Path
import skiploom_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SkiploomSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=Model((ROOT/'skiploom.glb').read_bytes())
 def test_canonical_hierarchy_geometry_and_budgets(self):
  m=self.model;blob=(ROOT/'skiploom.glb').read_bytes();neutral=author.decode_skiploom(blob)
  self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS['skiploom'])
  self.assertEqual(blob,author.export_skiploom(neutral));self.assertLess(len(blob),author.MAX_BYTES)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(24,5182,10028,18))
  self.assertLessEqual(len(m.doc['accessors']),256)
  self.assertEqual(m.parents[:-1],[None,0,1,1,1,1,4,5,1,8]+[9]*8)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):
    self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<18 for j in js))
 def test_splayed_ear_tip_hinges_stay_inside_the_closed_leaf_blades(self):
  m=self.model;neutral=author.decode_skiploom((ROOT/'skiploom.glb').read_bytes())
  for side,joint in ((-1,6),(1,7)):
   blade=next(p for p in neutral['primitives'] if p['part']=='Ear leaf / broad folded green paddle '+str(side))
   point=tuple(-m.inverse[joint][12+i] for i in range(3))
   hits=ray_intersections(blade,point[0],point[1]);self.assertTrue(hits)
   self.assertTrue(min(hits)-.00001<=point[2]<=max(hits)+.00001)
  body=m.vertices[m.names.index('Body / continuous rounded wind-borne torso')]
  width=max(v[0] for v in body)-min(v[0] for v in body)
  ears=[v for part,name in zip(m.vertices,m.names) if name.startswith('Ear leaf /') for v in part]
  self.assertGreater(max(v[0] for v in ears)-min(v[0] for v in ears),width*1.25)
  self.assertGreater(max(v[2] for v in ears)-min(v[2] for v in ears),width*.35)
 def test_pack_green_gold_and_distinct_leaf_flower_owners(self):
  m=self.model
  for p,name in enumerate(m.names):
   if name.startswith(('Body /','Foot /','Arm /','Ear leaf /','Flower / short')):rgb=(66,148,57)
   elif name.startswith('Flower petal /'):rgb=(255,181,41)
   else:rgb=None
   if rgb:
    expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
    actual=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
   if name.startswith('Flower petal /'):
    owner=10+int(name.rsplit(' ',1)[1]);self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]))
   if name.startswith(('Ear leaf /','Ear fold /')):
    expected={1,4,6} if name.endswith(' -1') else {1,5,7}
    owners={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w>0.}
    self.assertEqual(owners,expected)
   if name.startswith('Eye /'):self.assertTrue(all(ids==[1,0,0,0] for ids in m.ids[p]))
 def test_fixed_feet_floor_winding_strain_and_neutral_endpoints(self):
  m=self.model;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith('Foot /'):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(name.split(' /')[0])
     if step%4==0:
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
       self.assertGreater(cosine,0.,(clip,step,name,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
        self.assertTrue(.80<ratio<1.20,(clip,step,name,ratio))
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Eye','Arm','Ear leaf','Ear fold','Flower','Flower petal'}<=moved,moved)
 def test_early_body_brace_delayed_flower_and_clip_contract(self):
  m=self.model
  body=[m.rotations('attack',i/100)[1][0] for i in range(101)]
  disk=[m.rotations('attack',i/100)[9][0] for i in range(101)]
  self.assertGreater(body[10],max(body)*.4);self.assertGreater(disk.index(max(disk)),body.index(max(body)))
  self.assertEqual(len(m.doc['animations']),3)
  for a in m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
   self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
