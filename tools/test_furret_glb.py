"""Stored Furret skin, grounded contacts and continuous colored seam playback."""
import math,os
from pathlib import Path
import furret_glb as author
import test_furret_sculpt as geometry_checks
from test_gengar_glb import Model,winding_alignment
ASSET=Path(os.environ.get('FURRET_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/furret.glb'))

class FurretSkinTests(geometry_checks.FurretSculptTests):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob);cls.model=author.decode_banded_ferret(cls.blob,'furret');cls.parts=cls.model['primitives']
 def test_canonical_anatomy_and_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_furret());self.assertEqual(author.geometry_digest(self.model),author.REST_DIGESTS['furret'])
  self.assertEqual(len(m.parts),38);self.assertEqual(sum(map(len,m.vertices)),17037);self.assertEqual(len(m.joints),14)
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
 def test_hierarchy_and_support_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[None,0,1,2,3,4,4,2,2,1,1,1,11,12])
  for p,name in enumerate(m.names):
   if name.startswith('Foot /'):self.assertTrue(all(ids==[0,0,0,0] for ids in m.ids[p]))
   if name.startswith(('Ear /','Ear tip /')):
    owner=5 if name.endswith('-1') else 6;self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]))
   for ids,ws in zip(m.ids[p],m.weights[p]):self.assertTrue(all(i<14 for i in ids));self.assertAlmostEqual(sum(ws),1.,places=7)
 def test_colored_seams_keep_identical_skin_influences(self):
  m=self.m
  for prefix in ('Body /','Tail /'):
   parts=[[p] for p,n in enumerate(m.names) if n.startswith(prefix)]
   if prefix=='Body /':
    next(g for g in parts if m.names[g[0]]=='Body / joined cream neck front').append(next(i for i,n in enumerate(m.names) if n.startswith('Back coat /')))
   for left,right in zip(parts,parts[1:]):
    def field(group):return {tuple(v):(tuple(ids),tuple(ws)) for p in group for v,ids,ws in zip(m.vertices[p],m.ids[p],m.weights[p])}
    a,b=field(left),field(right);shared=a.keys()&b.keys();self.assertGreaterEqual(len(shared),24)
    for point in shared:self.assertEqual(a[point],b[point])
 def test_floor_fixed_paws_winding_and_strain(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith('Foot /'):
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-12)
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_neutral_endpoints_and_body_ear_arm_tail_motion(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   mats=m.matrices(clip,.28)
   for prefix in ('Eye /','Ear /','Arm /','Tail /'):
    p=next(i for i,n in enumerate(m.names) if n.startswith(prefix));self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,mats))),.001,prefix)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)

if __name__=='__main__':
 import unittest
 unittest.main()
