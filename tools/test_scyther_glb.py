#!/usr/bin/env python3
"""Stored green-mantis anatomy, fitted blade spines and membranes, and planted claws."""
import os,copy,math,unittest
from pathlib import Path
import scyther_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays,crossed_faces
from sculpt_geometry import ellipsoid
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class ScytherTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=Path(os.environ.get('SCYTHER_GLB',str(ROOT/'scyther.glb'))).read_bytes();cls.m=skin.Model(cls.blob);cls.parts=author.decode_scyther(cls.blob)['primitives'];cls.links={}
  for i,(p,extra) in enumerate(zip(cls.m.parts,cls.m.doc['extras']['source_part_ranges'])):
   sources=extra.get('source_parts',[{'part':cls.m.names[i],'vertex_start':0,'vertex_count':len(cls.m.vertices[i])}])
   for q in sources:cls.links[q['part']]=(i,q['vertex_start'],q['vertex_start']+q['vertex_count'])
 def find(self,prefix,side=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (side is None or p['part'].split()[-1]==str(side)))
 def posed_parts(self,mats):
  runtime=[self.m.posed(i,mats) for i in range(len(self.m.parts))];parts={}
  for p in self.parts:
   i,a,b=self.links[p['part']];parts[p['part']]=dict(p,positions=sum(runtime[i][a:b],[]))
  return parts
 def local_feature(self,p,side,j=None):
  if j is None:
   origin=author.pivot((side*.20,.81,.27));a=math.radians(20.);u=(side*math.sin(a),0.,math.cos(a));v=(0.,1.,0.);w=(-math.cos(a),0.,side*math.sin(a))
  else:
   root,tip=(((side*.071,.861,-.055),(side*.29,1.27,-.82)),((side*.072,.831,-.055),(side*.26,1.035,-.76)))[j]
   origin=author.pivot(root);length=math.dist(root,tip);v=tuple((b-a)/length for a,b in zip(root,tip));seed=(0.,1.,0.);d=sum(a*b for a,b in zip(seed,v));u=tuple(a-d*b for a,b in zip(seed,v));length=math.sqrt(sum(a*a for a in u));u=tuple(a/length for a in u);w=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
  positions=[]
  for i in range(0,len(p['positions']),3):
   delta=[p['positions'][i+k]-origin[k] for k in range(3)];positions.extend(sum(a*b for a,b in zip(delta,axis)) for axis in (u,v,w))
  return dict(p,positions=positions)
 def test_schema_recoverable_anatomy_and_floor(self):
  m=self.m;self.assertEqual(self.blob,author.export_scyther());self.assertEqual(len(self.parts),57);self.assertEqual(len(m.parts),39);self.assertEqual(len(m.joints),13);self.assertFalse((ROOT/'scyther.mesh.json').exists())
  self.assertEqual(author.geometry_digest({'primitives':self.parts}),author.REST_DIGEST);self.assertEqual(author.geometry_digest(m.neutral_model()),author.PACKED_REST_DIGEST)
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.18,places=6)
  for prefix,count in [('Crest /',3),('Scythe /',2),('Scythe rim /',2),('Wing case /',4),('Wing /',4),('Wing vein /',4),('Greave /',2),('Foot claw /',6),('Abdomen seam /',3)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in self.parts),count)
 def test_closed_outward_solids(self):assert_closed_models(self,{'scyther':{'primitives':self.parts}})
 def test_actual_face_normals(self):assert_model_shading(self,{'scyther':{'primitives':self.parts}})
 def test_fitted_gaze_blade_spines_and_wing_membranes_clear_parents(self):
  for mark in self.parts:
   n=mark['part'];side=n.split()[-1]
   if n.startswith(('Eye socket /','Nostril /')):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',side)
   elif n.startswith('Pupil /'):parent=self.find('Eye /',side)
   elif n.startswith('Breast plate /'):parent=self.find('Thorax /')
   elif n.startswith('Scythe rim /'):parent=self.find('Scythe /',side)
   elif n.startswith('Wing /'):
    parent=next(p for p in self.parts if p['part'].startswith('Wing case /') and p['part'].split()[-2:]==n.split()[-2:])
   else:continue
   if n.startswith('Scythe rim /'):mark=self.local_feature(mark,int(side));parent=self.local_feature(parent,int(side))
   elif n.startswith('Wing /'):
    wing_side,j=map(int,n.split()[-2:]);mark=self.local_feature(mark,wing_side,j);parent=self.local_feature(parent,wing_side,j)
   for k in range(0,len(mark['indices']),3):
    a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][k:k+3]]
    u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
    if normal[2]<=.50*math.sqrt(sum(q*q for q in normal)):continue
    samples=[tuple((a[j]+b[j]+c[j])/3 for j in range(3)),*[tuple((u[j]+v[j])/2 for j in range(3)) for u,v in [(a,b),(b,c),(c,a)]]]
    for q in samples:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.00005,(n,q))
 def test_pale_membranes_clear_actual_rear_wing_faces(self):
  from bovine_sculpt import reverse_front
  for pale in [p for p in self.parts if p['part'].startswith('Wing /')]:
   case=next(p for p in self.parts if p['part'].startswith('Wing case /') and p['part'].split()[-2:]==pale['part'].split()[-2:])
   side,j=map(int,pale['part'].split()[-2:]);mark=reverse_front(self.local_feature(pale,side,j));parent=reverse_front(self.local_feature(case,side,j))
   for k in range(0,len(mark['indices']),3):
    a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][k:k+3]]
    u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
    if normal[2]<=.50*math.sqrt(sum(q*q for q in normal)):continue
    samples=[tuple((a[j]+b[j]+c[j])/3 for j in range(3)),*[tuple((u[j]+v[j])/2 for j in range(3)) for u,v in [(a,b),(b,c),(c,a)]]]
    for q in samples:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,pale['part']);self.assertGreater(q[2]-max(hits),.00005,'pale membrane must also cover the rear case')
 def test_scythe_inner_cutouts_are_unfilled(self):
  for side in (-1,1):
   blade=self.local_feature(self.find('Scythe /',side),side)
   self.assertFalse(rays(blade,.15*1.75*author.SCALE,-.28*.65*author.SCALE),'actual curved blade must leave the inner cutout empty')
   self.assertTrue(rays(blade,.35*1.75*author.SCALE,-.22*.65*author.SCALE),'outer cutting blade must have real volume')
 def test_occupied_axial_blade_leg_and_wing_roots_survive_motion(self):
  find=lambda p,s=None:self.find(p,s)['part'];pairs=[(find('Waist /'),find('Thorax /')),(find('Abdomen /'),find('Waist /')),(find('Neck /'),find('Thorax /')),(find('Head /'),find('Neck /'))]
  for crest in [p for p in self.parts if p['part'].startswith('Crest /')]:pairs.append((crest['part'],find('Head /')))
  for side in (-1,1):
   pairs += [(find('Shoulder /',side),find('Thorax /')),(find('Upper arm /',side),find('Shoulder /',side)),(find('Scythe /',side),find('Upper arm /',side)),(find('Haunch /',side),find('Abdomen /')),(find('Haunch thorn /',side),find('Haunch /',side)),(find('Shank /',side),find('Haunch /',side)),(find('Greave /',side),find('Shank /',side)),(find('Foot /',side),find('Greave /',side))]
   for j in range(3):
    nail=next(p for p in self.parts if p['part'].startswith('Foot claw /') and p['part'].split()[-2:]==[str(side),str(j)]);pairs.append((nail['part'],find('Foot /',side)))
   for j in range(2):
    wing=next(p for p in self.parts if p['part'].startswith('Wing case /') and p['part'].split()[-2:]==[str(side),str(j)])
    vein=next(p for p in self.parts if p['part'].startswith('Wing vein /') and p['part'].split()[-2:]==[str(side),str(j)])
    membrane=next(p for p in self.parts if p['part'].startswith('Wing /') and p['part'].split()[-2:]==[str(side),str(j)])
    pairs += [(wing['part'],find('Thorax /'))]
  for clip in self.m.clips:
   for t in (0.,.18,.55):
    parts=self.posed_parts(self.m.matrices(clip,t))
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  blade=copy.deepcopy(self.find('Scythe /'));blade['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(blade['positions'])];self.assertFalse(parts_overlap(blade,self.find('Upper arm /')))
 def test_thin_veins_have_real_embedded_vertices_in_their_membranes(self):
  # A coarse XY grid can miss a thin diagonal ridge. Its actual stored back
  # vertices must instead occupy the membrane volume; common ownership keeps
  # that attachment rigid throughout the wing's motion.
  for vein in [p for p in self.parts if p['part'].startswith('Wing vein /')]:
   parent=next(p for p in self.parts if p['part'].startswith('Wing /') and p['part'].split()[-2:]==vein['part'].split()[-2:])
   side,j=map(int,vein['part'].split()[-2:]);parent=self.local_feature(parent,side,j);vein=self.local_feature(vein,side,j)
   def occupied(part):
    count=0
    for i in range(0,len(part['positions']),3):
     x,y,z=part['positions'][i:i+3];hits=sorted(rays(parent,x,y))
     if hits and min(hits)+.00005<z<max(hits)-.00005:count+=1
    return count
   self.assertGreaterEqual(occupied(vein),20,vein['part'])
   detached=copy.deepcopy(vein);detached['positions']=[v+(.01 if i%3==2 else 0.) for i,v in enumerate(detached['positions'])]
   self.assertEqual(occupied(detached),0,'floating-vein negative control must fail occupancy')
 def test_coherent_gaze_scythe_spines_wings_and_ground_owners(self):
  for n,(i,a,b) in self.links.items():
   side=n.split()[-2] if n.startswith(('Wing /','Wing case /','Wing vein /','Foot claw /')) else n.split()[-1]
   if n.startswith(('Greave /','Foot /','Foot claw /')):owner=0
   elif n.startswith(('Head /','Crest /','Eye','Pupil /','Mouth /','Nostril /')):owner=4
   elif n.startswith(('Scythe /','Scythe rim /')):owner=6 if side=='-1' else 8
   elif n.startswith(('Wing /','Wing case /','Wing vein /')):owner=9 if side=='-1' else 10
   elif n.startswith(('Abdomen /','Abdomen seam /')):owner=2
   elif n.startswith(('Haunch /','Haunch thorn /')):owner=11 if side=='-1' else 12
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
     if n.startswith(('Greave /','Foot claw /')):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
    if step in (0,16):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Thorax','Abdomen','Head','Scythe','Upper arm','Wing'}<=moved,moved)
 def test_winding_strain_and_nonintersecting_anatomy(self):
  m=self.m
  for p in self.parts:
   if p['part'].startswith(('Thorax /','Head /','Scythe /','Greave /','Wing case /','Shank /','Foot claw /')):self.assertIsNone(crossed_faces(p),p['part'])
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.30,(clip,n))
 def test_attack_moves_actual_blade_tips_relative_to_upper_arm(self):
  from test_gengar_glb import transform
  for side in (-1,1):
   blade=self.find('Scythe /',side);point=min([blade['positions'][i:i+3] for i in range(0,len(blade['positions']),3)],key=lambda p:p[1])
   mats=self.m.matrices('attack',.31);upper=5 if side<0 else 7
   delta=math.dist(transform(mats[upper],point,1.),transform(mats[upper+1],point,1.))
   self.assertGreater(delta,.035,'actual blade tip must sweep independently of its upper arm')
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
