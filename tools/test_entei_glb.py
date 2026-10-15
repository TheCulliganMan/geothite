"""Stored Entei mask, mane, blades, smoke and planted-paw regressions."""
import math,unittest
from pathlib import Path
import entei_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class EnteiSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'entei.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_volcano_beast(cls.blob,'entei')
 def test_canonical_anatomy_and_existing_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_entei());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['entei'])
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(59,15542,26368,16))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'entei':self.neutral});assert_model_shading(self,{'entei':self.neutral})
 def test_distinctive_mask_mane_blades_smoke_and_paws(self):
  m=self.m
  for prefix,count in [('Mane /',2),('Back blade /',6),('Smoke /',4),('Foot /',4),('Toe /',12),('Cuff /',4),('Face plate /',2),('Mask /',1),('Crown /',1)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  crown=next(p for p,n in enumerate(m.names) if n.startswith('Crown /'));self.assertGreater(max(v[1] for v in m.vertices[crown]),max(v[1] for v in m.vertices[2]))
 def test_fitted_eye_layers_clear_stored_parents(self):
  parts=self.neutral['primitives']
  for part in parts:
   name=part['part'];side=name.split()[-1]
   prefixes={'Eye socket /':'Face plate /','Eye /':'Eye socket /','Iris /':'Eye /','Pupil /':'Iris /'}
   parent_prefix=next((b for a,b in prefixes.items() if name.startswith(a)),None)
   if parent_prefix is None:continue
   parent=next(p for p in parts if p['part'].startswith(parent_prefix) and p['part'].split()[-1]==side)
   vs=[part['positions'][i:i+3] for i in range(0,len(part['positions']),3)];checked=0
   for i in range(0,len(part['indices']),3):
    a,b,c=[vs[j] for j in part['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(p[k] for p in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits);self.assertGreater(mid[2],max(hits));checked+=1
   self.assertGreater(checked,8)
 def test_source_amber_body_and_red_face_materials(self):
  for prefix,rgb555 in [('Body /',(24,14,0)),('Face plate /',(21,0,0)),('Crown /',(24,14,0))]:
   rgb=[((v<<3)|(v>>2))/255 for v in rgb555];expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for p,name in zip(self.m.parts,self.m.names):
    if name.startswith(prefix):
     color=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7)
 def test_coherent_head_supports_and_correct_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['entei']])
  for p,name in enumerate(m.names):
   if name.startswith(('Leg /','Tail /','Tail cloud /','Smoke /')):continue
   owner=0 if name.startswith(('Foot /','Toe /','Cuff /')) else 1 if name.startswith('Body /') else 2 if name.startswith(('Neck /','Bib /','Beard /')) else (4 if name.endswith('-1') else 5) if name.startswith('Mane /') else (8 if ' -1 ' in name else 9) if name.startswith('Back blade /') else 3
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),name)
  for ids,ws in zip(m.ids,m.weights):
   for js,weight in zip(ids,ws):self.assertAlmostEqual(sum(weight),1.,places=7);self.assertTrue(all(j<16 for j in js))
 def test_paws_cuffs_floor_winding_and_strain(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     if name.startswith(('Foot /','Toe /','Cuff /')):
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-12)
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_closed_endpoints_and_head_mane_smoke_articulation(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   mats=m.matrices(clip,.28)
   for prefix in ['Head /','Mane /','Smoke /','Tail /']:
    p=next(i for i,n in enumerate(m.names) if n.startswith(prefix));self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,mats))),.001)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
if __name__=='__main__':unittest.main()
