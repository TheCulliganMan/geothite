"""Stored Raikou surface fitting, topology, palette and planted support checks."""
import math,unittest
from copy import deepcopy
from pathlib import Path
import raikou_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class RaikouSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'raikou.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_thunder_beast(cls.blob,'raikou')
 def test_canonical_limits_and_anatomy(self):
  self.assertEqual(self.blob,author.export_raikou());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['raikou'])
  self.assertEqual((len(self.m.parts),sum(map(len,self.m.vertices)),sum(len(p)//3 for p in self.m.indices),len(self.m.joints)),(62,18482,28632,16))
  self.assertLess(sum(map(len,self.m.vertices)),20000)
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(self.m.doc['accessors']),256)
  for prefix,count in [('Body stripe /',6),('Leg stripe /',4),('Foot /',4),('Toe /',12),('Fang /',2),('Helmet /',1),('Whisker mask /',1),('Storm mane /',1),('Storm curl /',3),('Tail /',1),('Tail star /',1)]:
   self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'raikou':self.neutral});assert_model_shading(self,{'raikou':self.neutral})
 def test_fitted_stripes_and_face_layers_clear_actual_parent_triangles(self):
  parts=self.neutral['primitives']
  def side_view(part,side):
   part=deepcopy(part);vs=part['positions'];part['positions']=[v for i in range(0,len(vs),3) for v in (-side*vs[i+2],vs[i+1],side*vs[i])];return part
  for part in parts:
   name=part['part'];side=name.split()[-1];child=part;parent=None
   if name.startswith('Body stripe /'):
    sign=int(name.split()[-2]);child=side_view(part,sign);parent=side_view(next(p for p in parts if p['part'].startswith('Body /')),sign)
   elif name.startswith('Leg stripe /'):
    sign=int(side);label='rear' if 'rear' in name else 'fore';child=side_view(part,sign)
    parent=side_view(next(p for p in parts if p['part'].startswith('Leg /') and label in p['part'] and p['part'].endswith(' '+side)),sign)
   else:
    prefixes={'Eye socket /':'Helmet /','Eye /':'Eye socket /','Iris /':'Eye /','Pupil /':'Iris /','Ear inset /':'Ear /','Armor ridge /':'Helmet /'}
    target=next((b for a,b in prefixes.items() if name.startswith(a)),None)
    if target:parent=next(p for p in parts if p['part'].startswith(target) and (target=='Helmet /' or p['part'].endswith(' '+side)))
   if parent is None:continue
   vs=[child['positions'][i:i+3] for i in range(0,len(child['positions']),3)];checked=0
   for i in range(0,len(child['indices']),3):
    a,b,c=[vs[j] for j in child['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(p[k] for p in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits,name);self.assertGreater(mid[2],max(hits),name);checked+=1
   self.assertGreater(checked,8,name)
 def test_pack_yellow_and_purple_materials(self):
  for prefix,rgb555 in [('Body /',(31,31,0)),('Storm mane /',(19,6,18)),('Storm curl /',(19,6,18))]:
   rgb=[((v<<3)|(v>>2))/255 for v in rgb555];expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for p,name in zip(self.m.parts,self.m.names):
    if name.startswith(prefix):
     color=self.m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7)
 def test_joint_hierarchy_and_support_owners(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['raikou']])
  for p,name in enumerate(m.names):
   owner=0 if name.startswith(('Foot /','Toe /')) else 1 if name.startswith(('Body /','Body stripe /')) else 2 if name.startswith(('Neck /','Bib /')) else None
   if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),name)
  for ids,ws in zip(m.ids,m.weights):
   for js,weight in zip(ids,ws):self.assertAlmostEqual(sum(weight),1.,places=7);self.assertTrue(all(j<16 for j in js))
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
 def test_neutral_endpoints_and_head_mantle_tail_motion(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   mats=m.matrices(clip,.28)
   for prefix in ['Head /','Storm mane /','Storm curl /','Tail /']:
    p=next(i for i,n in enumerate(m.names) if n.startswith(prefix));self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,mats))),.001)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
if __name__=='__main__':unittest.main()
