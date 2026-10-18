#!/usr/bin/env python3
"""Independent stored psychic-cat anatomy, fitted gaze and articulated looping-tail checks."""
import copy,math,unittest
from pathlib import Path
import mew_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays,crossed_faces
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MewTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'mew.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives'];cls.named={p['part']:p for p in cls.parts}
 def find(self,prefix,side=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+str(side))))
 def test_canonical_schema_floor_body_height_and_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_mew());self.assertEqual(len(m.parts),40);self.assertEqual(len(m.joints),13);self.assertFalse((ROOT/'mew.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),.78,places=6)
  ears=[p for q in self.parts if q['part'].startswith('Ear /') for p in q['positions'][1::3]]
  self.assertAlmostEqual(max(ears),.78,places=6,msg='ears, not the tail, must establish standing height')
  for prefix,count in [('Ear /',2),('Ear inset /',2),('Finger /',6),('Foot /',2),('Toe crease /',4),('Tail /',1),('Tail tip /',1)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'mew':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'mew':self.m.neutral_model()})
 def fitted_pairs(self):
  for p in self.parts:
   n=p['part'];side=n.split()[-1]
   if n.startswith('Eye socket /'):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',side)
   elif n.startswith('Iris /'):parent=self.find('Eye /',side)
   elif n.startswith('Pupil /'):parent=self.find('Iris /',side)
   elif n.startswith('Glint /'):parent=self.find('Iris /',side)
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',side)
   else:continue
   yield p,parent
 def test_fitted_lenses_and_ear_insets_clear_stored_parents(self):
  for mark,parent in self.fitted_pairs():
   for i in range(0,len(mark['positions']),3):
    x,y,z=mark['positions'][i:i+3];hits=rays(parent,x,y);self.assertTrue(hits,mark['part']);self.assertLess(abs(z-max(hits)),.012,mark['part'])
   for k in range(0,len(mark['indices']),3):
    a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    samples=[tuple((a[j]+b[j]+c[j])/3 for j in range(3)),*[tuple((u[j]+v[j])/2 for j in range(3)) for u,v in [(a,b),(b,c),(c,a)]]]
    for q in samples:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,mark['part']);self.assertGreater(q[2]-max(hits),.0001,(mark['part'],q))
 def test_smile_and_toe_creases_follow_real_surfaces(self):
  for p in self.parts:
   n=p['part']
   if n.startswith('Mouth /'):parent=self.find('Head /')
   elif n.startswith('Toe crease /'):parent=self.find('Foot /',n.split()[-2])
   else:continue
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=rays(parent,x,y);self.assertTrue(hits,n);self.assertLess(abs(z-max(hits)),.004,n)
 def test_anatomical_owners_and_three_tail_carriers(self):
  m=self.m
  for i,n in enumerate(m.names):
   if n.startswith('Tail /'):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w};self.assertEqual(owners,{8,9,10});continue
   owner=3 if n.startswith(('Head /','Eye','Iris /','Pupil /','Glint /','Mouth /')) else 2 if n.startswith('Neck /') else 10 if n.startswith('Tail tip /') else 1
   if n.startswith(('Haunch /','Leg /','Foot /','Toe crease /')):
    side=n.split()[-2] if n.startswith('Toe crease /') else n.split()[-1];owner=6 if side=='-1' else 7
   elif n.startswith(('Arm /','Hand /','Finger /')):
    side=n.split()[-2] if n.startswith('Finger /') else n.split()[-1];owner=4 if side=='-1' else 5
   elif n.startswith(('Ear /','Ear inset /')):owner=11 if n.endswith('-1') else 12
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),n)
 def posed_parts(self,mats):return [dict(p,positions=sum(self.m.posed(i,mats),[])) for i,p in enumerate(self.parts)]
 def test_occupied_head_hand_ear_leg_and_tail_roots_survive_motion(self):
  names=self.m.names;find=lambda prefix,s=None:self.find(prefix,s)['part'];pairs=[(find('Neck /'),find('Body /')),(find('Head /'),find('Neck /')),(find('Tail /'),find('Body /')),(find('Tail tip /'),find('Tail /'))]
  for side in (-1,1):
   pairs += [(find('Ear /',side),find('Head /')),(find('Arm /',side),find('Body /')),(find('Hand /',side),find('Arm /',side)),(find('Haunch /',side),find('Body /')),(find('Leg /',side),find('Haunch /',side)),(find('Foot /',side),find('Leg /',side))]
   pairs += [(n,find('Hand /',side)) for n in names if n.startswith('Finger /') and n.split()[-2]==str(side)]
  for clip in self.m.clips:
   for t in (0.,.18,.55):
    parts={p['part']:p for p in self.posed_parts(self.m.matrices(clip,t))}
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  tip=copy.deepcopy(self.find('Tail tip /'));tip['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(tip['positions'])]
  self.assertFalse(parts_overlap(tip,self.find('Tail /')))
 def test_floor_neutral_endpoints_independent_motion_and_tail_height(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(17):
    mats=m.matrices(clip,step/16)
    for i,n in enumerate(m.names):
     points=m.posed(i,mats);self.assertGreaterEqual(min(q[1] for q in points),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(points,m.vertices[i]))>.002:moved.add(n.split(' /')[0])
     if n.startswith(('Tail /','Tail tip /')):
      for a,b in zip(points,m.vertices[i]):self.assertLess(abs(a[1]-b[1]),1e-7)
    if step in (0,16):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Ear','Arm','Hand','Foot','Tail','Tail tip'}<=moved,moved)
 def test_winding_strain_and_nonintersecting_anatomy(self):
  m=self.m
  for p in self.parts:
   if p['part'].startswith(('Head /','Ear /','Foot /','Tail /','Arm /')):self.assertIsNone(crossed_faces(p),p['part'])
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.30,(clip,n))
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
