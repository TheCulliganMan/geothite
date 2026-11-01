#!/usr/bin/env python3
"""Independent stored Jumpluff grouping, support and three seed-head checks."""
import math,unittest
from pathlib import Path
import jumpluff_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
from wind_plant_sculpt import sculpture as source_sculpture
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class JumpluffSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=Model((ROOT/'jumpluff.glb').read_bytes())
 def test_canonical_grouping_expands_all_original_components_without_geometry_loss(self):
  blob=(ROOT/'jumpluff.glb').read_bytes();m=self.model;neutral=author.decode_jumpluff(blob,expanded=False)
  self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS['jumpluff'])
  self.assertEqual(blob,author.export_jumpluff(neutral));self.assertLess(len(blob),author.MAX_BYTES)
  source=author.decode_jumpluff(blob);self.assertEqual(len(source['primitives']),106)
  self.assertEqual(author.geometry_digest(source),author.geometry_digest(source_sculpture('jumpluff')))
  self.assertEqual(sum('fine seed bristle' in p['part'] for p in source['primitives']),84)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints),len(m.doc['accessors'])),(25,6247,10560,13,136))
  self.assertEqual(m.parents[:-1],[None,0,1,1,1,1,4,5,1,8,9,8,8])
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):
    self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<13 for j in js))
 def test_pack_blue_tan_and_bristles_move_with_their_head(self):
  m=self.model
  for p,name in enumerate(m.names):
   if name.startswith(('Body /','Foot /','Arm /')):rgb=(74,66,198)
   elif 'continuous seed sphere' in name:rgb=(255,255,255)
   elif name.startswith('Seed head /'):rgb=(231,189,107)
   else:rgb=None
   if rgb:
    expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
    actual=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
    for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
   if name.startswith('Seed head /'):
    owner=6 if 'lower -1' in name else 7 if 'lower 1' in name else 10
    self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]))
  self.assertEqual(sum('fitted bristle group' in n for n in m.names),3)
  for p,name in enumerate(m.names):
   if 'fitted bristle group' in name:self.assertEqual(len(m.doc['extras']['source_components'][p]),28)
 def test_splayed_lower_heads_keep_centered_pivots_and_depth_separation(self):
  m=self.model;centers=[]
  for side,joint in ((-1,6),(1,7)):
   p=m.names.index('Seed head / lower '+str(side)+' / continuous seed sphere')
   q=m.names.index('Seed head / lower '+str(side)+' / scalloped underside shade')
   surface=m.vertices[p]+m.vertices[q]
   center=[(min(v[k] for v in surface)+max(v[k] for v in surface))/2 for k in range(3)]
   pivot=[-v for v in m.inverse[joint][12:15]]
   self.assertLess(math.dist(center,pivot),1e-5)
   centers.append(center)
  body=next(m.vertices[p] for p,n in enumerate(m.names) if n.startswith('Body /'))
  width=max(v[0] for v in body)-min(v[0] for v in body)
  self.assertGreater(centers[1][2]-centers[0][2],width*.25)
 def test_white_and_shade_regions_share_a_scalloped_seam(self):
  m=self.model
  for suffix in ('lower -1','lower 1','crown'):
   upper=m.names.index('Seed head / '+suffix+' / continuous seed sphere')
   lower=m.names.index('Seed head / '+suffix+' / scalloped underside shade')
   seam=set(map(tuple,m.vertices[upper])) & set(map(tuple,m.vertices[lower]))
   self.assertEqual(len(seam),32)
   # A deliberately irregular latitude reproduces the seed-head shade edge.
   self.assertGreater(max(p[1] for p in seam)-min(p[1] for p in seam),.015)
   self.assertEqual(m.ids[upper][0],m.ids[lower][0])
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
  self.assertTrue({'Body','Eye','Arm','Leaf','Seed stem','Seed head'}<=moved,moved)
 def test_early_body_brace_delayed_crown_and_clip_contract(self):
  m=self.model
  body=[m.rotations('attack',i/100)[1][0] for i in range(101)]
  crown=[m.rotations('attack',i/100)[10][0] for i in range(101)]
  self.assertGreater(body[10],max(body)*.4);self.assertGreater(crown.index(max(crown)),body.index(max(body)))
  for a in m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
   self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
