#!/usr/bin/env python3
"""Stored aquatic anatomy, seated fan rays and articulated mermaid-tail checks."""
import copy,math,unittest
from pathlib import Path
import vaporeon_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'vaporeon.glb'
def flipped(p):return dict(p,positions=[-v if i%3!=1 else v for i,v in enumerate(p['positions'])])
def embedded(child,parent):
 count=0
 for i in range(0,len(child['positions']),3):
  x,y,z=child['positions'][i:i+3];hits=sorted(rays(parent,x,y))
  if any(a+.00005<z<b-.00005 for a,b in zip(hits[::2],hits[1::2])):count+=1
 return count
class VaporeonTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_aquatic_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_vaporeon());self.assertEqual(len(m.parts),57);self.assertEqual(len(m.joints),16)
  self.assertFalse((ROOT/'vaporeon.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.0,places=6)
  for prefix,count in [('Head fin /',2),('Head fin membrane /',4),('Head fin ray /',8),('Head cap /',2),('Crest /',1),('Frill /',1),('Fluke /',2),('Tail /',1),('Tail ridge /',1),('Paw /',4)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'vaporeon':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'vaporeon':self.m.neutral_model()})
 def test_fitted_face_caps_and_both_sides_of_head_fans(self):
  for mark in self.parts:
   n=mark['part'];flip=False;side=[n.split()[-1]]
   if n.startswith(('Eye /','Head cap /')):parent=self.find('Head /');flip=n.startswith('Head cap /') and n.endswith('-1')
   elif n.startswith(('Iris /','Pupil /')):parent=self.find('Eye /',side)
   elif n.startswith('Glint /'):parent=self.find('Pupil /',side)
   elif n.startswith('Head fin membrane /'):
    side,face=n.split()[-2:];parent=self.find('Head fin /',[side]);flip=face=='-1'
   else:continue
   if flip:mark=flipped(mark);parent=flipped(parent)
   vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
   for k in range(0,len(mark['indices']),3):
    a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.0001,(n,q))
 def test_actual_fan_rays_are_seated_inside_cream_membranes(self):
  for p in self.parts:
   if not p['part'].startswith('Head fin ray /'):continue
   side,face,j=p['part'].split()[-3:];parent=self.find('Head fin membrane /',[side,face]);self.assertGreater(embedded(p,parent),5,p['part'])
  p=copy.deepcopy(next(p for p in self.parts if p['part'].startswith('Head fin ray /')));p['positions']=[v+(.03 if i%3==2 else 0.) for i,v in enumerate(p['positions'])]
  self.assertEqual(embedded(p,self.find('Head fin membrane /',['-1','1'])),0)
 def test_anatomical_owners_and_four_planted_paws(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith(('Paw /','Toe crease /')):self.assertEqual(owners,{0},n)
   elif n.startswith(('Body /','Shoulder /','Haunch /','Back ridge /')):self.assertEqual(owners,{1},n)
   elif n.startswith('Frill /'):self.assertEqual(owners,{15})
   elif n.startswith('Head fin'):self.assertEqual(owners,{4 if m.vertices[i][0][0]<0. else 5},n)
   elif n.startswith('Fluke /'):self.assertEqual(owners,{9 if n.endswith('-1') else 10})
   elif n.startswith('Leg /'):
    side,row=map(int,n.split()[-2:]);self.assertEqual(owners,{0,1,11+(0 if side<0 else 2)+row})
   elif n.startswith('Neck /'):self.assertEqual(owners,{1,2,3})
   elif not n.startswith(('Tail /','Tail ridge /')):self.assertEqual(owners,{3},n)
 def test_occupied_neck_frill_fan_limb_and_fluke_roots_survive_motion(self):
  m=self.m;body=self.find('Body /')['part'];head=self.find('Head /')['part'];neck=self.find('Neck /')['part'];tail=self.find('Tail /')['part']
  pairs=[(neck,body),(neck,head),(self.find('Frill /')['part'],neck),(self.find('Muzzle /')['part'],head),(tail,body)]
  for p in self.parts:
   n=p['part']
   if n.startswith(('Head fin /','Crest /')):pairs.append((n,head))
   elif n.startswith(('Shoulder /','Haunch /')):pairs.append((n,body))
   elif n.startswith('Fluke /'):pairs.append((n,tail))
   elif n.startswith('Leg /'):
    side,row=n.split()[-2:];pairs.append((n,self.find('Paw /',[side,row])['part']));pairs.append((n,self.find('Shoulder /' if row=='0' else 'Haunch /',[side])['part']))
  for clip in m.clips:
   for t in (0.,.31):
    mats=m.matrices(clip,t);parts={p['part']:dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)}
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
    for prefix,parent in [('Back ridge /',body),('Tail ridge /',tail)]:self.assertGreater(embedded(parts[self.find(prefix)['part']],parts[parent]),3,(clip,t,prefix))
  detached=copy.deepcopy(self.find('Fluke /'));detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,self.find('Tail /')))
 def test_floor_neutral_endpoints_winding_strain_and_independent_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(q[1] for q in posed),-1e-7,(clip,n))
     displacement=max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))
     if n.startswith(('Paw /','Toe crease /')):self.assertLess(displacement,1e-7,n)
     elif displacement>.002:moved.add(n.split(' /')[0])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    if step in (0,8):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Head','Head fin','Frill','Leg','Tail','Fluke'}<=moved,moved)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
