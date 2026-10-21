#!/usr/bin/env python3
"""Independent stored Sentret topology, annulus, shared tail bands and support rig."""
import math,os,unittest
from pathlib import Path
import sentret_glb as author
from sentret_sculpt import TAIL,TAIL_FRAMES,RADII,LIGHT,DARK
from sculpt_geometry import cross
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=Path(os.environ.get('SENTRET_GLB',ROOT/'sentret.glb'))
class SentretSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_lookout(cls.blob,'sentret')
 def test_canonical_anatomy_counts_and_reader_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_sentret());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['sentret'])
  self.assertEqual((len(m.parts),sum(len(p) for p in m.vertices),sum(len(p)//3 for p in m.indices),len(m.joints)),(40,14430,13752,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertFalse((ROOT/'sentret.mesh.json').exists())
 def test_closed_outward_solids_and_stored_shading(self):
  assert_closed_models(self,{'sentret':self.neutral});assert_model_shading(self,{'sentret':self.neutral})
 def test_belly_mark_is_an_annulus_on_the_real_body(self):
  ring=next(p for p in self.neutral['primitives'] if p['part'].startswith('Belly ring /'));body=self.neutral['primitives'][0]
  x=0.;y=author.normalized((0.,.598,0.))[1]
  self.assertFalse(ray_intersections(ring,x,y));self.assertTrue(ray_intersections(body,x,y))
  x=author.normalized((.085,.598,0.))[0];hits=ray_intersections(ring,x,y);parent=ray_intersections(body,x,y)
  self.assertTrue(hits);self.assertTrue(parent);self.assertGreater(max(hits),max(parent))
 def test_body_and_bands_use_inspected_full_colour_game_pigments(self):
  for prefix,rgb in [('Body /',LIGHT),('Tail band /',DARK)]:
   expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for part,name in zip(self.m.parts,self.m.names):
    if name.startswith(prefix):
     color=self.m.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7)
 def test_stored_tail_bands_share_the_parent_cross_sections_and_normals(self):
  ps=[p for p in self.neutral['primitives'] if p['part'].startswith(('Tail /','Tail band /'))]
  self.assertEqual(len(ps),7)
  def points(p):
   result={}
   for i in range(0,len(p['positions']),3):result.setdefault(tuple(p['positions'][i:i+3]),set()).add(tuple(p['normals'][i:i+3]))
   return result
  for left,right in zip(ps,ps[1:]):
   a,b=points(left),points(right);shared=a.keys()&b.keys();self.assertGreaterEqual(len(shared),24)
   for point in shared:self.assertTrue(a[point]&b[point],point)

 def test_hierarchy_binds_face_and_limb_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['sentret']])
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  for p,name in enumerate(m.names):
   for js,ws in zip(m.ids[p],m.weights[p]):self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<11 for j in js))
   if name.startswith(('Body /','Belly ring /','Eye /','Glint /','Muzzle /','Nose /','Mouth /')):self.assertTrue(all(js==[4,0,0,0] for js in m.ids[p]),name)
   if name.startswith(('Ear /','Ear inset /')):self.assertTrue(all(js==[5 if sum(v[0] for v in m.vertices[p])<0 else 6,0,0,0] for js in m.ids[p]))
 def test_pinned_tail_contacts_floor_clearance_winding_and_strain(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     for index,v in enumerate(m.vertices[p]):
      if v[1]<.002:self.assertLess(math.dist(posed[index],v),1e-12,(clip,name,index))
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_neutral_endpoints_and_source_cue_body_brace(self):
  m=self.m
  for clip in m.clips:
   for t in [0.,1.]:
    for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
  values=[m.rotations('attack',i/100)[4][0] for i in range(101)];self.assertGreater(values[20],max(values)*.9)
  for a in m.doc['animations']:
   idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle);self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
