#!/usr/bin/env python3
"""Stored steel-mantis anatomy, actual open jaws, fitted marks and planted armor."""
import copy,math,unittest
from pathlib import Path
import scizor_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays,crossed_faces
from sculpt_geometry import ellipsoid
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ScizorTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'scizor.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.parts=author.decode_scizor(cls.blob)['primitives'];cls.links={}
  for i,(p,extra) in enumerate(zip(cls.m.parts,cls.m.doc['extras']['source_part_ranges'])):
   sources=extra.get('source_parts',[{'part':cls.m.names[i],'vertex_start':0,'vertex_count':len(cls.m.vertices[i])}])
   for q in sources:cls.links[q['part']]=(i,q['vertex_start'],q['vertex_start']+q['vertex_count'])
 def find(self,prefix,side=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (side is None or p['part'].split()[-1]==str(side)))
 def posed_parts(self,mats):
  runtime=[self.m.posed(i,mats) for i in range(len(self.m.parts))];parts={}
  for p in self.parts:
   i,a,b=self.links[p['part']];parts[p['part']]=dict(p,positions=sum(runtime[i][a:b],[]))
  return parts
 def test_schema_recoverable_anatomy_and_floor(self):
  m=self.m;self.assertEqual(self.blob,author.export_scizor());self.assertEqual(len(self.parts),55);self.assertEqual(len(m.parts),47);self.assertEqual(len(m.joints),17);self.assertFalse((ROOT/'scizor.mesh.json').exists())
  self.assertEqual(author.geometry_digest({'primitives':self.parts}),author.REST_DIGEST);self.assertEqual(author.geometry_digest(m.neutral_model()),author.PACKED_REST_DIGEST)
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.28,places=6)
  for prefix,count in [('Crest /',3),('Pincer upper /',2),('Pincer lower /',2),('Pincer eye /',2),('Wing /',4),('Wing vein /',4),('Greave /',2),('Toe cap /',2),('Abdominal plate /',3)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.parts),count)
 def test_closed_outward_solids(self):assert_closed_models(self,{'scizor':{'primitives':self.parts}})
 def test_actual_face_normals(self):assert_model_shading(self,{'scizor':{'primitives':self.parts}})
 def test_fitted_gaze_armor_false_eyes_and_crushing_lips_clear_parents(self):
  for mark in self.parts:
   n=mark['part'];side=n.split()[-1]
   if n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',side)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',side)
   elif n.startswith('Chest plate /'):parent=self.find('Body /')
   elif n.startswith('Abdominal plate /'):parent=self.find('Abdomen /')
   elif n.startswith(('Pincer upper lining /','Pincer eye rim /')):parent=self.find('Pincer upper /',side)
   elif n.startswith('Pincer lower lining /'):parent=self.find('Pincer lower /',side)
   elif n.startswith('Pincer eye /'):parent=self.find('Pincer eye rim /',side)
   elif n.startswith('Pincer pupil /'):parent=self.find('Pincer eye /',side)
   else:continue
   for k in range(0,len(mark['indices']),3):
    a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    samples=[tuple((a[j]+b[j]+c[j])/3 for j in range(3)),*[tuple((u[j]+v[j])/2 for j in range(3)) for u,v in [(a,b),(b,c),(c,a)]]]
    for q in samples:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.00005,(n,q))
 def test_pincer_apertures_are_actual_empty_space(self):
  for side in (-1,1):
   jaws=[self.find('Pincer upper /',side),self.find('Pincer lower /',side)]
   for dx in (.04,.08,.12):
    x,y,z=author.pivot((side*(.48+dx),.88-.042,.13))
    for p in jaws:self.assertFalse(rays(p,x,y),p['part'])
   x,y,z=author.pivot((side*.56,.838,.13));fill=ellipsoid('negative filled aperture',(.1,.1,.1),(x,y,z),(.02,.02,.06),16,10)
   self.assertTrue(rays(fill,x,y),'filled aperture negative control must be detected')
 def test_occupied_axial_arm_jaw_leg_and_wing_roots_survive_motion(self):
  find=lambda p,s=None:self.find(p,s)['part'];pairs=[(find('Abdomen /'),find('Body /')),(find('Neck /'),find('Body /')),(find('Head /'),find('Neck /'))]
  for crest in [p for p in self.parts if p['part'].startswith('Crest /')]:pairs.append((crest['part'],find('Head /')))
  for side in (-1,1):
   pairs += [(find('Shoulder /',side),find('Body /')),(find('Upper arm /',side),find('Shoulder /',side)),(find('Forearm /',side),find('Upper arm /',side)),(find('Pincer upper /',side),find('Forearm /',side)),(find('Pincer lower /',side),find('Pincer upper /',side)),(find('Hip armor /',side),find('Abdomen /')),(find('Thigh /',side),find('Hip armor /',side)),(find('Greave /',side),find('Thigh /',side)),(find('Foot /',side),find('Greave /',side)),(find('Toe cap /',side),find('Foot /',side))]
   for j in range(2):
    wing=next(p for p in self.parts if p['part'].startswith('Wing /') and p['part'].split()[-2:]==[str(side),str(j)])
    vein=next(p for p in self.parts if p['part'].startswith('Wing vein /') and p['part'].split()[-2:]==[str(side),str(j)])
    pairs += [(wing['part'],find('Body /')),(vein['part'],wing['part'])]
  for clip in self.m.clips:
   for t in (0.,.18,.55):
    parts=self.posed_parts(self.m.matrices(clip,t))
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  jaw=copy.deepcopy(self.find('Pincer lower /'));jaw['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(jaw['positions'])];self.assertFalse(parts_overlap(jaw,self.find('Pincer upper /')))
 def test_coherent_marking_and_ground_owners(self):
  for n,(i,a,b) in self.links.items():
   side=n.split()[-2] if n.startswith(('Wing /','Wing vein /')) else n.split()[-1]
   if n.startswith(('Greave /','Foot /','Toe cap /')):owner=0
   elif n.startswith(('Head /','Crest /','Eye','Pupil /')):owner=4
   elif n.startswith(('Pincer lower /','Pincer lower lining /')):owner=8 if side=='-1' else 12
   elif n.startswith('Pincer '):owner=7 if side=='-1' else 11
   elif n.startswith(('Wing /','Wing vein /')):owner=13 if side=='-1' else 14
   elif n.startswith(('Abdomen /','Abdominal plate /')):owner=2
   else:continue
   self.assertTrue(all(js==[owner,0,0,0] for js in self.m.ids[i][a:b]),n)
 def test_planted_armor_neutral_endpoints_and_independent_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(17):
    mats=m.matrices(clip,step/16)
    for i,n in enumerate(m.names):
     points=m.posed(i,mats);self.assertGreaterEqual(min(q[1] for q in points),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(points,m.vertices[i]))>.002:moved.add(n.split(' /')[0])
     if n.startswith(('Greave /','Toe cap /')):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
    if step in (0,16):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Abdomen','Head','Pincer upper','Pincer lower','Upper arm','Forearm','Wing'}<=moved,moved)
 def test_winding_strain_and_nonintersecting_anatomy(self):
  m=self.m
  for p in self.parts:
   if p['part'].startswith(('Body /','Head /','Pincer upper /','Pincer lower /','Greave /','Wing /','Thigh /')):self.assertIsNone(crossed_faces(p),p['part'])
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.30,(clip,n))
 def test_attack_reduces_actual_jaw_tip_separation(self):
  from test_gengar_glb import transform
  for side in (-1,1):
   def nearest(part,xy):
    target=author.pivot((side*(.48+xy[0]),.88+xy[1],.17))
    return min([part['positions'][i:i+3] for i in range(0,len(part['positions']),3)],key=lambda p:math.dist(p,target))
   upper=nearest(self.find('Pincer upper /',side),(.12,.015));lower=nearest(self.find('Pincer lower /',side),(.12,-.075))
   neutral=math.dist(upper,lower);mats=self.m.matrices('attack',.31);j=7 if side<0 else 11
   closed=math.dist(transform(mats[j],upper,1.),transform(mats[j+1],lower,1.))
   self.assertLess(closed,neutral*.75,'actual stored jaw tips must close during attack')
   self.assertGreater(closed,.01,'jaw tips must retain clearance')
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
