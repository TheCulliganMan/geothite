"""Stored Suicune anatomy, planted supports and independent ribbon skin playback."""
import math,os
from pathlib import Path
import suicune_glb as author
import test_suicune_sculpt as geometry_checks
from test_gengar_glb import Model,winding_alignment
ASSET=Path(os.environ.get('SUICUNE_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/suicune.glb'))

class SuicuneSkinTests(geometry_checks.SuicuneSculptTests):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob);cls.model=author.decode_aurora_beast(cls.blob,'suicune');cls.parts=cls.model['primitives']
 def test_canonical_stored_anatomy_and_existing_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_suicune());self.assertEqual(author.geometry_digest(self.model),author.REST_DIGESTS['suicune'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(47,10500,17216,18))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
 def test_hierarchy_and_support_ownership(self):
  m=self.m;self.assertEqual(m.parents[:-1],[None,0,1,2,3,1,5,6,1,1,1,1,1,12,13,1,15,16])
  for p,name in enumerate(m.names):
   owner=0 if name.startswith(('Foot /','Toe /')) else 1 if name.startswith(('Body /','Body diamond /')) else 2 if name.startswith(('Neck /','Bib /')) else 4 if name.startswith('Crest /') else 3 if name.startswith(('Head /','Muzzle /','Nose /','Crest base /','Eye socket /','Eye /','Iris /','Pupil /')) else None
   if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),name)
   if name.startswith('Ribbon /'):
    allowed={12,13,14} if name.endswith('-1') else {15,16,17}
    self.assertEqual({j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w},allowed)
  for ids,ws in zip(m.ids,m.weights):
   for js,weight in zip(ids,ws):self.assertAlmostEqual(sum(weight),1.,places=7);self.assertTrue(all(j<18 for j in js))
 def test_floor_fixed_paws_winding_and_strain(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith(('Foot /','Toe /')):
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-12)
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_neutral_endpoints_and_head_crest_mane_ribbon_motion(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   mats=m.matrices(clip,.28)
   for p,name in enumerate(m.names):
    if name.startswith(('Head /','Crest /','Mane /','Ribbon /')):self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,mats))),.001,name)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)

if __name__=='__main__':
 import unittest
 unittest.main()
