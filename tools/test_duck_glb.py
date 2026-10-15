#!/usr/bin/env python3
"""Independent stored geometry and articulation checks for both standard duck skins."""
import math,unittest
from pathlib import Path
import duck_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
COUNTS={'psyduck':(28,3754,3828,10,131),'golduck':(45,4974,4460,11,186)}

class DuckSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={n:Model((ROOT/(n+'.glb')).read_bytes()) for n in author.SPECIES}
 def test_preserved_anatomy_and_bounded_canonical_bytes(self):
  for n,m in self.models.items():
   with self.subTest(species=n):
    blob=(ROOT/(n+'.glb')).read_bytes();neutral=author.decode_duck(blob,n);parts,v,tri,joints,accessors=COUNTS[n]
    self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS[n]);self.assertEqual(blob,author.export_duck(n,neutral))
    self.assertEqual(len(m.parts),parts);self.assertEqual(sum(len(p) for p in m.vertices),v)
    self.assertEqual(sum(len(p)//3 for p in m.indices),tri);self.assertEqual(len(m.doc['accessors']),accessors)
    self.assertLess(len(blob),1024*1024);self.assertLessEqual(accessors,256)
    self.assertEqual(len(m.joints),joints);self.assertFalse((ROOT/(n+'.mesh.json')).exists())
 def test_body_material_matches_verified_normal_rgb555_anchor(self):
  references={'psyduck':(31,31,0),'golduck':(8,11,24)}
  for n,m in self.models.items():
   rgb=[((c<<3)|(c>>2))/255 for c in references[n]]
   expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   body=next(p for p,name in zip(m.parts,m.names) if name.startswith('Body /'))
   actual=m.doc['materials'][body['material']]['pbrMetallicRoughness']['baseColorFactor']
   for a,b in zip(actual[:3],expected):self.assertAlmostEqual(a,b,places=7)
   self.assertEqual(actual[3],1.)
   for p,name in zip(m.parts,m.names):
    color=m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
    if name.startswith('Forehead / fitted ruby'):self.assertGreater(color[0],color[2]*5)
    if name.startswith('Bill /') and 'shell' in name:self.assertGreater(color[0],color[2]*2)
    if name.startswith('Hair /'):self.assertLess(max(color[:3]),.01)

 def test_hierarchy_inverse_binds_and_normalized_weights(self):
  for n,m in self.models.items():
   self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS[n]])
   self.assertEqual([p['name'] for p in m.doc['nodes'][:-1]],[n+'/'+name for name,_,_ in author.JOINTS[n]])
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):
     self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<len(author.JOINTS[n]) for j in js))
 def test_fitted_faces_webbing_and_claws_keep_owners(self):
  for n,m in self.models.items():
   for p,name in enumerate(m.names):
    if name.startswith(('Eye /','Forehead /','Nostril /','Bill / broad upper','Bill / narrow streamlined')):
     self.assertTrue(all(ids==[2,0,0,0] for ids in m.ids[p]),(n,name))
    if name.startswith(('Bill / fitted lower','Bill / subtle curved seam')):
     self.assertTrue(all(ids==[9,0,0,0] for ids in m.ids[p]))
    if name.startswith(('Hair /','Crown /')):self.assertTrue(all(ids==[8,0,0,0] for ids in m.ids[p]))
    if name.startswith(('Hand /','Finger /','Hand membrane /','Hand claw /')):
     owner=4 if sum(v[0] for v in m.vertices[p])<0. else 6
     self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(n,name))
 def test_planted_webs_floor_winding_and_animation_strain(self):
  for n,m in self.models.items():
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,name in enumerate(m.names):
      posed=m.posed(p,mats)
      if name.startswith(('Leg /','Foot /','Foot membrane /','Foot claw /')):self.assertEqual(posed,m.vertices[p],(n,clip,name))
      self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(n,clip,step,name))
      if step%4==0:
       normals=m.posed(p,mats,True)
       for i in range(0,len(m.indices[p]),3):
        tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
        self.assertGreater(cosine,0.,(n,clip,step,name,i));self.assertGreater(area,1e-12)
        for a,b in zip(tri,tri[1:]+tri[:1]):
         ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
         self.assertTrue(.7<ratio<1.3,(n,clip,step,name,ratio))
 def test_fused_head_and_anatomically_distinct_tail_fields(self):
  for n,m in self.models.items():
   body=next(i for i,name in enumerate(m.names) if name.startswith('Body /'))
   self.assertEqual({j for ids,ws in zip(m.ids[body],m.weights[body]) for j,w in zip(ids,ws) if w>0.},{1,2})
   tail=next(i for i,name in enumerate(m.names) if name.startswith('Tail /'))
   used={j for ids,ws in zip(m.ids[tail],m.weights[tail]) for j,w in zip(ids,ws) if w>0.}
   self.assertEqual(used,{7} if n=='psyduck' else {1,7,10})
   self.assertGreater(max(v[1] for p in m.vertices for v in p),1.)
 def test_neutral_endpoints_and_early_head_brace(self):
  for n,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for joint in (1,2,3,5):
    values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
    self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
   for a in m.doc['animations']:
    idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
    self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
