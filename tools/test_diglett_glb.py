"""Stored Diglett capsule, face fitting, independent soil and grounded skin checks."""
import math,unittest
from pathlib import Path
import diglett_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class DiglettSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'diglett.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_mole(cls.blob,'diglett')
 def test_canonical_anatomy_and_existing_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_diglett());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['diglett'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(15,10646,14064,2))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertFalse((ROOT/'diglett.mesh.json').exists())
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'diglett':self.neutral});assert_model_shading(self,{'diglett':self.neutral})
 def test_capsule_shape_and_reference_exclude_soil(self):
  vs=self.m.vertices[0]
  def width(raw_y):
   y=author.normalized((0.,raw_y,0.))[1];ring=min(vs,key=lambda p:abs(p[1]-y))[1];points=[p for p in vs if abs(p[1]-ring)<1e-7];return max(p[0] for p in points)-min(p[0] for p in points)
  self.assertAlmostEqual(width(.17)/width(.27),1.,places=2)
  center=author.normalized((0.,.34,0.));rx=author.normalized((.19,0.,0.))[0];rz=author.normalized((0.,0.,.17))[2];ry=author.normalized((0.,.56,0.))[1]-center[1]
  cap=[p for p in vs if p[1]>center[1]+1e-5];self.assertGreater(len(cap),400)
  for x,y,z in cap:self.assertAlmostEqual((x/rx)**2+((y-center[1])/ry)**2+(z/rz)**2,1.,places=5)
  span=max(p[1] for p in vs)-min(p[1] for p in vs);self.assertAlmostEqual(span,self.m.doc['extras']['capsule_reference_span'],places=6)
  self.assertGreater(min(p[1] for p in vs),0.);self.assertAlmostEqual(max(p[1] for p in vs),.82,places=6)
  self.assertEqual(sum(n.startswith('Soil /') for n in self.m.names),8)
 def test_pack_body_nose_and_soil_palette(self):
  for p,name in zip(self.m.parts,self.m.names):
   if name.startswith('Body /'):rgb555=(19,11,4)
   elif name.startswith(('Nose /','Soil /')):rgb555=(24,6,4)
   else:continue
   rgb=[((v<<3)|(v>>2))/255 for v in rgb555];expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   color=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
   for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7,msg=name)
 def test_face_and_forehead_glints_clear_their_stored_parents(self):
  parts=self.neutral['primitives'];body=parts[0]
  for part in parts:
   name=part['part']
   if name.startswith(('Eye /','Forehead glint /')):parent=body
   elif name.startswith('Eye glint /'):parent=next(p for p in parts if p['part'].startswith('Eye /') and p['part'].split()[-1]==name.split()[-1])
   else:continue
   vs=[part['positions'][i:i+3] for i in range(0,len(part['positions']),3)];checked=0
   for i in range(0,len(part['indices']),3):
    a,b,c=[vs[j] for j in part['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(p[k] for p in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits);self.assertGreater(mid[2],max(hits));checked+=1
   self.assertGreater(checked,8)
 def test_soil_clods_have_varied_footprints_and_actual_flat_facets(self):
  profiles=set()
  for p,name in enumerate(self.m.names):
   if not name.startswith('Soil /'):continue
   vs=self.m.vertices[p];profiles.add(tuple(round(max(v[k] for v in vs)-min(v[k] for v in vs),4) for k in (0,2)))
   ns=self.m.normals[p]
   for i in range(0,len(self.m.indices[p]),3):
    a,b,c=self.m.indices[p][i:i+3];self.assertEqual(ns[a],ns[b]);self.assertEqual(ns[a],ns[c])
  self.assertGreaterEqual(len(profiles),6)
 def test_soil_root_and_coherent_visible_face_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[None,0])
  for p,name in enumerate(m.names):
   if name.startswith('Body /'):continue
   owner=0 if name.startswith('Soil /') else 1
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),name)
  for ids,ws in zip(m.ids,m.weights):
   for js,weight in zip(ids,ws):self.assertAlmostEqual(sum(weight),1.,places=7);self.assertTrue(all(j<2 for j in js))
 def test_fixed_soil_floor_winding_and_strain_across_clips(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith('Soil /'):
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-12)
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_neutral_endpoints_and_source_cue_articulation(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[0],m.posed(0,m.matrices(clip,.25)))),.001)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
if __name__=='__main__':unittest.main()
