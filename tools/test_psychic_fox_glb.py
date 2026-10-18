#!/usr/bin/env python3
"""Stored psychic fox skins: planted claws, fitted ears and articulated held spoons."""
import copy,math,unittest
from pathlib import Path
import psychic_fox_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class PsychicFoxSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,m,prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
 def test_canonical_limits_hierarchy_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_psychic(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());count=17 if s=='alakazam' else 15;self.assertEqual(len(m.joints),count)
   self.assertEqual(m.parents[:count],[None,0,1,1,3,4,1,6,7,0,0,2,2]+([2,2,5,8] if s=='alakazam' else [1,13]))
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(count)))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_feature_and_digit_owners(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    expected=None
    if n.startswith('Foot /'):expected=0
    elif n.startswith('Hand /'):expected=5 if n.split()[-2 if n.startswith('Hand / curled finger') else -1]=='-1' else 8
    elif n.startswith('Ear /'):expected=11 if n.endswith('-1') else 12
    elif n.startswith(('Head /','Eye /')):expected=2
    elif n.startswith('Moustache /'):expected=13 if n.endswith('-1') else 14
    elif n.startswith('Spoon /'):expected=15 if n.endswith('-1') else 16
    elif n.startswith('Armor / shoulder'):expected=1
    if expected is not None:
     self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{expected},(s,n))
 def test_fixed_floor_bounded_strain_winding_and_source_clock_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith('Foot /'):
       for a,b in zip(points,m.vertices[i]):self.assertLess(math.dist(a,b),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_every_carrier_has_independent_articulation(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for joint in range(1,len(m.joints)):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,joint))
 def test_actual_solid_roots_remain_occupied_in_motion(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];find=lambda n:self.find(m,n);body=find('Body /');head=find('Head /');pairs=[(body,head),(body,find('Armor / shaped chest'))]
   for side in (-1,1):
    suffix=' '+str(side);upper=find('Arm / tapered upper'+suffix);fore=find('Arm / broad tapered forearm'+suffix);hand=find('Hand / palm'+suffix);thigh=find('Leg / shaped bent haunch'+suffix);shin=find('Leg / narrow hock'+suffix);foot=find('Foot / raised instep'+suffix)
    pairs += [(head,find('Ear / pointed closed fox'+suffix)),(body,upper),(upper,fore),(fore,hand),(body,thigh),(thigh,shin),(shin,foot)]
    for j in range(3):
     pairs += [(hand,find('Hand / curled finger'+suffix+' '+str(j))),(foot,find('Foot / splayed toe'+suffix+' '+str(j))),(find('Foot / splayed toe'+suffix+' '+str(j)),find('Foot / pale claw'+suffix+' '+str(j)))]
    if s=='alakazam':pairs += [(head,find('Moustache / sweeping serrated blade'+suffix))]
   if s=='abra':pairs += [(body,find('Tail / heavy')),(find('Tail / heavy'),find('Tail / dark'))]
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[head]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[body]))
 def test_spoons_stay_held_by_curled_fingers_and_move_relative_to_wrists(self):
  m=self.models['alakazam'];ps=m.neutral_model()['primitives'];find=lambda n:self.find(m,n)
  for side in (-1,1):
   suffix=' '+str(side);stem=find('Spoon / held tapered stem'+suffix);bowl=find('Spoon / oval bowl'+suffix);inset=find('Spoon / recessed bowl surface'+suffix);fingers=[find('Hand / curled finger'+suffix+' '+str(j)) for j in range(3)]
   for clip in m.clips:
    for t in (0.,.13,.31,.58,.83):
     mats=m.matrices(clip,t);posed={i:dict(ps[i],positions=sum(m.posed(i,mats),[])) for i in [stem,bowl,inset]+fingers}
     self.assertTrue(any(parts_overlap(posed[stem],posed[f]) for f in fingers),(clip,t,side))
     for a,b in ((stem,bowl),(bowl,inset)):self.assertTrue(parts_overlap(posed[a],posed[b]),(clip,t,m.names[a],m.names[b]))
   mats=m.matrices('attack',.31);wrist=5 if side<0 else 8;self.assertGreater(max(math.dist(q,skin.transform(mats[wrist],p)) for p,q in zip(m.vertices[bowl],m.posed(bowl,mats))),.005)
 def test_fitted_ear_interiors_and_gaze_clear_actual_support_in_motion(self):
  from sculpt_test_support import indexed_ray_intersections as rays
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];pairs=[]
   for i,n in enumerate(m.names):
    if n.startswith('Ear / inset'):parent=self.find(m,'Ear / pointed closed fox '+n.split()[-1])
    elif n.startswith('Eye /'):parent=self.find(m,'Head /')
    else:continue
    pairs.append((i,parent))
   for clip in m.clips:
    mats=m.matrices(clip,.31);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for mark,parent in pairs:
     vs=[posed[mark]['positions'][i:i+3] for i in range(0,len(posed[mark]['positions']),3)]
     for k in range(0,len(m.indices[mark]),3):
      ids=m.indices[mark][k:k+3];a,b,c=[m.vertices[mark][j] for j in ids]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      x,y,z=[sum(vs[j][axis] for j in ids)/3 for axis in range(3)];hits=rays(posed[parent],x,y);self.assertTrue(hits,(s,clip,m.names[mark]));self.assertGreater(z-max(hits),.0001,(s,clip,m.names[mark]))
 def test_strict_reader_rejects_mutation(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_psychic(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_psychic(s,changed)
if __name__=='__main__':unittest.main()
