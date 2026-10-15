#!/usr/bin/env python3
"""Independent decoded Hoppip hierarchy, support and botanical skin checks."""
import math,unittest
from pathlib import Path
import hoppip_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class HoppipSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=Model((ROOT/'hoppip.glb').read_bytes())
 def test_canonical_neutral_hierarchy_and_budgets(self):
  m=self.model;blob=(ROOT/'hoppip.glb').read_bytes();neutral=author.decode_hoppip(blob)
  self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS['hoppip'])
  self.assertEqual(blob,author.export_hoppip(neutral));self.assertLess(len(blob),author.MAX_BYTES)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(21,5963,8380,11))
  self.assertLessEqual(len(m.doc['accessors']),256)
  self.assertEqual(m.parents[:-1],[None,0,1,1,1,1,1,6,1,8,1])
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):
    self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<11 for j in js))
 def test_bowed_closed_blades_keep_veins_close_to_the_surface(self):
  neutral=author.decode_hoppip((ROOT/'hoppip.glb').read_bytes())
  for side in (-1,1):
   blade=next(p for p in neutral['primitives'] if p['part']=='Leaf / continuous serrated crown blade '+str(side))
   vein=next(p for p in neutral['primitives'] if p['part']=='Leaf vein / tapered midrib '+str(side))
   zs=blade['positions'][2::3]
   self.assertGreater(max(zs)-min(zs),.08)
   for k in range(0,len(vein['positions']),3):
    x,y,z=vein['positions'][k:k+3];hits=ray_intersections(blade,x,y)
    self.assertTrue(hits,(side,k))
    self.assertLess(abs(z-max(hits)),.025,(side,k,z,hits))
 def test_leaf_tip_pivots_follow_the_fitted_mid_ribs_and_separate_in_depth(self):
  m=self.model;tips=[]
  for side,joint in ((-1,7),(1,9)):
   point=tuple(-m.inverse[joint][12+i] for i in range(3));tips.append(point)
   part=m.names.index('Leaf vein / tapered midrib '+str(side))
   self.assertLess(min(math.dist(point,v) for v in m.vertices[part]),.004)
  body=m.vertices[m.names.index('Body / continuous rounded wind-borne torso')]
  body_width=max(v[0] for v in body)-min(v[0] for v in body)
  blades=[v for part,name in zip(m.vertices,m.names) if name.startswith('Leaf /') for v in part]
  for axis in (0,2):
   self.assertGreater(max(v[axis] for v in blades)-min(v[axis] for v in blades),body_width*.75)
 def test_verified_pack_body_and_leaf_materials(self):
  m=self.model
  for p,name in zip(m.parts,m.names):
   if name.startswith(('Body /','Foot /','Arm /','Ear / pointed','Tail / curved')):rgb=(173,49,57)
   elif name.startswith('Leaf /'):rgb=(90,189,49)
   else:continue
   expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
   actual=m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
   for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
 def test_leaf_veins_share_blade_field_and_ears_keep_fitted_insets(self):
  m=self.model
  for side,ear,leaf,tip in [(-1,4,6,7),(1,5,8,9)]:
   for p,name in enumerate(m.names):
    if name.startswith('Ear /') and name.endswith(' '+str(side)):self.assertTrue(all(ids==[ear,0,0,0] for ids in m.ids[p]))
    if name.startswith(('Leaf /','Leaf vein /')) and name.endswith(' '+str(side)):
     owners={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w>0.}
     self.assertEqual(owners,{leaf,tip})
     for v,ids,weights in zip(m.vertices[p],m.ids[p],m.weights[p]):
      expected_ids,expected_weights=author.skin_weights('hoppip','Leaf /',v)
      q=author.quantized_weights(expected_weights)
      self.assertEqual(ids,[j if w else 0 for j,w in zip(expected_ids,q)])
      self.assertEqual(weights,[w/255 for w in q])
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
  self.assertTrue({'Body','Eye','Ear','Arm','Leaf','Leaf vein','Tail'}<=moved,moved)
 def test_early_body_brace_delayed_leaf_tip_and_clip_contract(self):
  m=self.model
  body=[m.rotations('attack',i/100)[1][0] for i in range(101)]
  tip=[m.rotations('attack',i/100)[7][0] for i in range(101)]
  self.assertGreater(body[10],max(body)*.4)
  self.assertGreater(tip.index(max(tip)),body.index(max(body)))
  for a in m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
   self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
