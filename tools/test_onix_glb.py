"""Stored Onix rock-chain ownership, rooted supports, attachments and clip strain."""
import math,unittest
from pathlib import Path
import onix_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class OnixSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'onix.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_rock_chain(cls.blob,'onix')
 def test_canonical_anatomy_and_existing_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_onix());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['onix'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(36,12083,9848,12))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertFalse((ROOT/'onix.mesh.json').exists())
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'onix':self.neutral});assert_model_shading(self,{'onix':self.neutral})
 def test_head_and_rigid_rocks_have_correct_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['onix']])
  for p,name in enumerate(m.names):
   if name.startswith('Rock segment /'):owner=max(0,int(name.split()[-1])-2)
   elif name.startswith('Joint /'):continue
   else:owner=11
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),name)
  for ids,weights in zip(m.ids,m.weights):
   for js,ws in zip(ids,weights):self.assertTrue(all(j<12 for j in js));self.assertAlmostEqual(sum(ws),1.,places=7)
 def test_source_stone_and_shadow_palette(self):
  for prefix,rgb555 in [('Head /',(17,15,20)),('Rock segment /',(17,15,20)),('Joint /',(9,6,11)),('Jaw /',(9,6,11))]:
   rgb=[((v<<3)|(v>>2))/255 for v in rgb555];expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for p,name in zip(self.m.parts,self.m.names):
    if name.startswith(prefix):
     color=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7)
 def test_reference_length_uses_stored_bind_centers_and_body_datums(self):
  centers=[];datums=self.m.doc['extras']['body_centerline_points'];self.assertEqual(len(datums),14)
  for j in range(13):
   p=self.m.names.index('Rock segment / original irregular Onix boulder '+str(j));vs=self.m.vertices[p]
   if j>=2:
    center=[-v for v in self.m.inverse[j-1][12:15]];self.assertLess(math.dist(center,datums[j]),1e-7)
   else:
    center=datums[j];self.assertAlmostEqual(center[1],(min(v[1] for v in vs)+max(v[1] for v in vs))*.5,places=7)
   for k in range(3):
    lo=min(v[k] for v in vs);hi=max(v[k] for v in vs);self.assertTrue(lo+(hi-lo)*.10<center[k]<hi-(hi-lo)*.10)
   centers.append(center)
  head=next(p for p,n in enumerate(self.m.names) if n.startswith('Head /'));end=datums[-1]
  for k in range(3):self.assertTrue(min(v[k] for v in self.m.vertices[head])<=end[k]<=max(v[k] for v in self.m.vertices[head]))
  centers.append(end);length=sum(math.dist(a,b) for a,b in zip(centers,centers[1:]))
  self.assertAlmostEqual(length,self.m.doc['extras']['body_centerline_length'],places=6)
 def test_pinned_supports_floor_winding_and_strain(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith('Rock segment /') and int(name.split()[-1])<3:
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-12)
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_buried_connections_remain_rooted_during_motion(self):
  m=self.m;rocks=[m.names.index('Rock segment / original irregular Onix boulder '+str(j)) for j in range(13)];joints=[m.names.index('Joint / buried dark Onix rock connection '+str(j)) for j in range(12)]
  def overlap(a,b):
   return all(min(max(p[k] for p in a),max(p[k] for p in b))-max(min(p[k] for p in a),min(p[k] for p in b))>0 for k in range(3))
  for clip in m.clips:
   for t in (0.,.125,.25,.5,.75,.875,1.):
    mats=m.matrices(clip,t);posed=[m.posed(p,mats) for p in range(len(m.parts))]
    for j in range(12):
     for a,b in [(rocks[j],rocks[j+1]),(joints[j],rocks[j]),(joints[j],rocks[j+1])]:self.assertTrue(overlap(posed[a],posed[b]),(clip,t,j,m.names[a],m.names[b]))
    head=next(p for p,n in enumerate(m.names) if n.startswith('Head /'));self.assertTrue(overlap(posed[head],posed[rocks[12]]))
 def test_neutral_endpoints_articulation_and_source_cue(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   head=next(p for p,n in enumerate(m.names) if n.startswith('Head /'));self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[head],m.posed(head,m.matrices(clip,.25)))),.001)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
if __name__=='__main__':unittest.main()
