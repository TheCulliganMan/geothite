"""Stored Slowpoke anatomy, fitted face, planted feet and tail ownership checks."""
import math,unittest
from pathlib import Path
import slowpoke_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SlowpokeSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'slowpoke.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_sleepy_quadruped(cls.blob,'slowpoke')
 def test_canonical_anatomy_and_existing_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_slowpoke());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['slowpoke'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(40,19574,32984,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256);self.assertFalse((ROOT/'slowpoke.mesh.json').exists())
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'slowpoke':self.neutral});assert_model_shading(self,{'slowpoke':self.neutral})
 def test_low_body_four_feet_and_raised_tail(self):
  m=self.m;self.assertEqual(sum(n.startswith('Foot /') for n in m.names),4);self.assertEqual(sum(n.startswith('Toe /') for n in m.names),12)
  body=m.vertices[0];self.assertGreater(max(p[2] for p in body)-min(p[2] for p in body),max(p[1] for p in body)-min(p[1] for p in body))
  tip=next(p for p,n in enumerate(m.names) if n.startswith('Tail tip /'));self.assertGreater(min(v[1] for v in m.vertices[tip]),max(v[1] for v in body))
 def test_fitted_eyes_pupils_ears_and_nostrils_clear_parents(self):
  parts=self.neutral['primitives'];head=parts[1];muzzle=parts[2]
  for part in parts:
   name=part['part'];side=name.split()[-1]
   if name.startswith('Eye /'):parent=head
   elif name.startswith('Pupil /'):parent=next(p for p in parts if p['part'].startswith('Eye /') and p['part'].split()[-1]==side)
   elif name.startswith('Ear inset /'):parent=next(p for p in parts if p['part'].startswith('Ear /') and p['part'].split()[-1]==side)
   elif name.startswith('Nostril /'):parent=muzzle
   else:continue
   vs=[part['positions'][i:i+3] for i in range(0,len(part['positions']),3)];checked=0
   for i in range(0,len(part['indices']),3):
    a,b,c=[vs[j] for j in part['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(p[k] for p in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits);self.assertGreater(mid[2],max(hits));checked+=1
   self.assertGreater(checked,8)
 def test_source_pink_and_shadow_materials(self):
  for prefix,rgb555 in [('Body /',(31,10,17)),('Ear inset /',(21,5,8)),('Tongue /',(21,5,8))]:
   rgb=[((v<<3)|(v>>2))/255 for v in rgb555];expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for p,name in zip(self.m.parts,self.m.names):
    if name.startswith(prefix):
     color=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7)
 def test_head_ears_and_tail_tip_share_correct_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['slowpoke']])
  for p,name in enumerate(m.names):
   if name.startswith(('Body /','Leg /','Tail /')):continue
   owner=0 if name.startswith(('Foot /','Toe /')) else 4 if name.startswith('Tail tip /') else (5 if name.endswith('-1') else 6) if name.startswith(('Ear /','Ear inset /')) else 2
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),name)
  for ids,ws in zip(m.ids,m.weights):
   for js,weight in zip(ids,ws):self.assertAlmostEqual(sum(weight),1.,places=7);self.assertTrue(all(j<11 for j in js))
 def test_planted_feet_floor_winding_and_strain(self):
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
 def test_neutral_endpoints_and_source_cue_articulation(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[1],m.posed(1,m.matrices(clip,.25)))),.001)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
if __name__=='__main__':unittest.main()
