#!/usr/bin/env python3
"""Independent stored spider skin, material-seam and grounded-pose checks."""
import math,unittest
from pathlib import Path
import spider_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class SpiderSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={n:Model((ROOT/(n+'.glb')).read_bytes()) for n in author.SPECIES}
 def test_canonical_geometry_hierarchy_and_byte_budget(self):
  for n,m in self.models.items():
   blob=(ROOT/(n+'.glb')).read_bytes();neutral=author.decode_spider(blob,n)
   self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS[n])
   self.assertEqual(blob,author.export_spider(n,neutral));self.assertLess(len(blob),author.MAX_BYTES)
   self.assertEqual(len(m.parts),43 if n=='spinarak' else 41);self.assertEqual(len(m.joints),18)
   self.assertLessEqual(len(m.doc['accessors']),256)
   self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS[n]])
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):
     self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<18 for j in js))
 def test_palette_and_fitted_features_keep_anatomical_owners(self):
  for n,m in self.models.items():
   rgb=(74,165,41) if n=='spinarak' else (222,82,66)
   expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
   actual=m.doc['materials'][m.parts[0]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
   for p,name in enumerate(m.names):
    if name.startswith(('Eye socket /','Eye pupil /','Eye glint /','Head /')):owner=2
    elif name.startswith(('Abdomen /','Abdomen stripe /','False eye /','False mouth /')):owner=1
    elif name.startswith('Horn /'):owner=3
    elif name.startswith('Mandible /'):owner=4 if sum(v[0] for v in m.vertices[p])<0 else 5
    else:continue
    self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),(n,name))
 def test_band_seams_share_positions_influences_and_stay_closed_in_pose(self):
  for n,m in self.models.items():
   for p,name in enumerate(m.names):
    if not name.startswith(('Walking leg /','Raised leg /')) or name.endswith('band 0'):continue
    previous=m.names.index(name.rsplit(' ',1)[0]+' '+str(int(name.rsplit(' ',1)[1])-1))
    here={tuple(v):i for i,v in enumerate(m.vertices[p])};there={tuple(v):i for i,v in enumerate(m.vertices[previous])}
    shared=set(here)&set(there);self.assertEqual(len(shared),14,(n,name))
    for point in shared:
     a,b=here[point],there[point]
     self.assertEqual(m.ids[p][a],m.ids[previous][b]);self.assertEqual(m.weights[p][a],m.weights[previous][b])
    for clip in m.clips:
     mats=m.matrices(clip,.28);a=m.posed(p,mats);b=m.posed(previous,mats)
     for point in shared:self.assertLess(math.dist(a[here[point]],b[there[point]]),1e-8)
 def test_grounded_tips_winding_strain_and_return_to_neutral(self):
  for n,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,name in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(n,clip,step,name))
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(name.split(' /')[0])
      if name.startswith('Walking leg /'):
       for a,b,ids,ws in zip(posed,m.vertices[p],m.ids[p],m.weights[p]):
        if ids==[0,0,0,0]:self.assertEqual(a,b,(n,clip,name))
      if step%4==0:
       for k in range(0,len(m.indices[p]),3):
        tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
        self.assertGreater(cosine,0.,(n,clip,step,name,k));self.assertGreater(area,1e-12)
        for a,b in zip(tri,tri[1:]+tri[:1]):
         ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
         self.assertTrue(.70<ratio<1.30,(n,clip,step,name,ratio))
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Abdomen','Head','Mandible','Horn','Walking leg'}<=moved,(n,moved))
   if n=='ariados':self.assertIn('Raised leg',moved)
   for a in m.doc['animations']:
    idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
    self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
