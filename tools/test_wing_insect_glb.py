#!/usr/bin/env python3
"""Independent stored wing insect skins, anatomical ownership and deformation."""
import math,unittest
from pathlib import Path
import wing_insect_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
COUNTS={'butterfree':(59,5428,9420,18),'beedrill':(42,4304,7304,20)}
class WingInsectSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={n:Model((ROOT/(n+'.glb')).read_bytes()) for n in author.SPECIES}
 def test_exact_neutral_anatomy_canonical_rebuild_and_limits(self):
  for n,m in self.models.items():
   blob=(ROOT/(n+'.glb')).read_bytes();neutral=author.decode_insect(blob,n)
   self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS[n])
   self.assertEqual((len(m.parts),sum(len(p) for p in m.vertices),sum(len(p)//3 for p in m.indices),len(m.joints)),COUNTS[n])
   self.assertLess(len(blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertFalse((ROOT/(n+'.mesh.json')).exists())
 def test_hierarchy_rest_binds_and_normalized_weights(self):
  for n,m in self.models.items():
   self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS[n]])
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):
     self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<len(m.joints) for j in js))
 def test_compound_eyes_drills_and_seated_abdomen_marks_keep_owners(self):
  for n,m in self.models.items():
   for p,name in enumerate(m.names):
    if name.startswith(('Head /','Eye /','Mouth /','Muzzle /')):self.assertTrue(all(js==[2,0,0,0] for js in m.ids[p]))
    if name.startswith('Abdomen'):self.assertTrue(all(js==[3,0,0,0] for js in m.ids[p]))
    if name.startswith('Drill /'):
     owner=18 if sum(v[0] for v in m.vertices[p])<0. else 19
     self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]))
 def test_four_wing_fields_and_two_antenna_chains(self):
  for n,m in self.models.items():
   for p,name in enumerate(m.names):
    used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w>0};left=sum(v[0] for v in m.vertices[p])<0.
    if name.startswith('Wing /'):
     expected=({1,8,9} if left else {1,10,11}) if 'lower' in name else ({1,4,5} if left else {1,6,7})
     self.assertEqual(used,expected,(n,name))
    if name.startswith('Antenna /'):self.assertEqual(used,{12,13} if left else {14,15})
 def test_floor_contacts_winding_and_deformation_strain(self):
  for n,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8)
     for p,name in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(n,clip,step,name))
      if n=='butterfree' and name.startswith(('Leg /','Foot /')):self.assertEqual(posed,m.vertices[p])
      for i in range(0,len(m.indices[p]),3):
       tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
       self.assertGreater(cosine,0.,(n,clip,step,name,i));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
        self.assertTrue(.7<ratio<1.3,(n,clip,step,name,ratio))
 def test_rotation_only_identity_endpoints_and_early_thorax_brace(self):
  for n,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   values=[m.rotations('attack',i/100)[1][0] for i in range(101)];self.assertGreater(values[20],max(values)*.9)
   for a in m.doc['animations']:
    idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle);self.assertEqual(a['extras']['cueRelative'],not idle)
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
