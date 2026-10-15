"""Stored Voltorb shell/face ownership, equator coherence and grounded rocking."""
import math,unittest
from pathlib import Path
import voltorb_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class VoltorbSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'voltorb.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_orb(cls.blob,'voltorb')
 def test_canonical_neutral_and_existing_reader_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_voltorb());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['voltorb'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(8,4660,8248,2))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertFalse((ROOT/'voltorb.mesh.json').exists())
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'voltorb':self.neutral});assert_model_shading(self,{'voltorb':self.neutral})
 def test_face_and_equator_share_the_shell_owner(self):
  m=self.m;self.assertEqual(m.parents[:-1],[None,0]);self.assertEqual(len(m.names),8)
  for p,name in enumerate(m.names):
   if not name.startswith('Lower shell /'):self.assertTrue(all(js==[1,0,0,0] for js in m.ids[p]),name)
   for js,ws in zip(m.ids[p],m.weights[p]):self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<2 for j in js))
  seam=author.HEIGHT/2
  rim=[]
  for p in (0,1):
   rim.append({tuple(v):i for i,v in enumerate(m.vertices[p]) if abs(v[1]-seam)<1e-7})
  self.assertEqual(set(rim[0]),set(rim[1]));self.assertEqual(len(rim[0]),64)
  for point in rim[0]:self.assertEqual(m.ids[0][rim[0][point]],m.ids[1][rim[1][point]]);self.assertEqual(m.weights[0][rim[0][point]],m.weights[1][rim[1][point]])
  for clip in m.clips:
   for t in (.125,.25,.5,.875):
    mats=m.matrices(clip,t);a=m.posed(0,mats);b=m.posed(1,mats)
    for point in rim[0]:self.assertLess(math.dist(a[rim[0][point]],b[rim[1][point]]),1e-12)
 def test_normal_body_and_face_palette_matches_pack(self):
  red=[((v<<3)|(v>>2))/255 for v in (24,5,5)];expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in red]
  for a,b in zip(self.neutral['primitives'][0]['base_color'],expected):self.assertAlmostEqual(a,b,places=7)
  for part in self.neutral['primitives'][1:]:
   expected=[1.,1.,1.] if part['part'].startswith(('Lower shell /','Eye white /')) else [0.,0.,0.]
   self.assertEqual(part['base_color'][:3],expected)
 def test_support_floor_clearance_winding_and_strain(self):
  m=self.m;contacts=[i for i,v in enumerate(m.vertices[1]) if v[1]<.001];self.assertGreater(len(contacts),64)
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if p==1:
      for i in contacts:self.assertLess(math.dist(posed[i],m.vertices[p][i]),1e-12,(clip,step,i))
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_neutral_endpoints_articulation_and_early_source_cue(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   moved=max(math.dist(a,b) for a,b in zip(m.vertices[0],m.posed(0,m.matrices(clip,.25))));self.assertGreater(moved,.001)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
  early=author.pose_angles('voltorb','attack',.18)[1][0];late=author.pose_angles('voltorb','attack',.75)[1][0];self.assertGreater(abs(early),abs(late))
if __name__=='__main__':unittest.main()
