#!/usr/bin/env python3
"""Independent stored frog anatomy, surface roots, spiral fit and skin playback."""
import copy,math,unittest
from pathlib import Path
import poli_frog_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,indexed_ray_intersections as rays
from test_kingdra_sculpt import crossed_faces
import test_nidorina_sculpt as attachments
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
def find(ps,prefix,side=None):return next(p for p in ps if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+str(side))))
def pairs(ps):
 for p in ps:
  n=p['part'];side=n.split()[-1];parent=None
  for child,base in [('Belly /','Body /'),('Eye /','Eye socket /'),('Pupil /','Eye /'),('Brow /','Eye socket /'),('Cheek /','Head /'),('Mouth /','Head /')]:
   if n.startswith(child):parent=find(ps,base,side if child in ('Eye /','Pupil /','Brow /') else None)
  if n.startswith('Glove crease /'):parent=find(ps,'Hand /',n.split()[-2])
  if parent:yield p,parent

def roots(test,ps,species):
 helper=attachments.NidorinaTests();helper.ps=ps;overlap=helper.occupied_overlap;body=find(ps,'Body /');head=find(ps,'Head /') if species=='politoed' else body
 if species=='politoed':test.assertTrue(overlap(head,body));test.assertTrue(overlap(find(ps,'Crest /'),head))
 for side in (-1,1):
  arm=find(ps,'Arm /',side);hand=find(ps,'Hand /',side);haunch=find(ps,'Haunch /',side);leg=find(ps,'Leg /',side);foot=find(ps,'Foot /',side)
  for a,b in [(find(ps,'Eye socket /',side),head),(arm,body),(hand,arm),(haunch,body),(leg,haunch),(foot,leg)]:test.assertTrue(overlap(a,b),(species,a['part'],b['part']))
  if species!='politoed':test.assertTrue(overlap(find(ps,'Thumb /',side),hand))
  else:
   for j in range(3):test.assertTrue(overlap(find(ps,'Finger /',f'{side} {j}'),hand));test.assertTrue(overlap(find(ps,'Toe /',f'{side} {j}'),foot))

class PoliFrogTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def test_canonical_geometry_hierarchy_and_existing_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_poli_frog(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(self.runtime[s].neutral_model()),author.PACKED_REST_DIGESTS[s]);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,pivot in author.JOINTS[s]])
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(m.parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,weights in zip(ids,ws) for j,w in zip(js,weights) if w},set(range(len(author.JOINTS[s]))))
 def test_stored_closed_connected_outward_solids(self):assert_closed_models(self,{s:m.neutral_model() for s,m in self.models.items()})
 def test_actual_face_shading(self):assert_model_shading(self,{s:m.neutral_model() for s,m in self.models.items()})
 def test_distinct_species_anatomy_and_floor(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];points=[p for vs in m.vertices for p in vs];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),author.HEIGHTS[s],places=6)
   for prefix,count in [('Body /',1),('Belly /',1),('Spiral /',1),('Eye socket /',2),('Eye /',2),('Pupil /',2),('Arm /',2),('Hand /',2),('Haunch /',2),('Leg /',2),('Foot /',2),('Brow /',2 if s=='poliwrath' else 0),('Thumb /',0 if s=='politoed' else 2),('Finger /',6 if s=='politoed' else 0),('Toe /',6 if s=='politoed' else 0),('Head /',1 if s=='politoed' else 0),('Cheek /',2 if s=='politoed' else 0),('Crest /',1 if s=='politoed' else 0)]:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count,(s,prefix))
  width=lambda s:max(p[0] for vs in self.models[s].vertices for p in vs)-min(p[0] for vs in self.models[s].vertices for p in vs)
  self.assertGreater(width('poliwrath'),width('poliwhirl'))
 def test_volume_roots_and_uncrossed_appendages_survive_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives']
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)];roots(self,posed,s)
     for p in posed:
      if p['part'].startswith(('Arm /','Leg /','Crest /','Finger /')):self.assertIsNone(crossed_faces(p),(s,clip,t,p['part']))
 def test_fitted_caps_follow_actual_posed_parent_triangles(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];rest=list(pairs(ps))
   for clip in m.clips:
    for t in (0.,.31,.67):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for (original,_),(child,parent) in zip(rest,pairs(posed)):
      vs=[child['positions'][i:i+3] for i in range(0,len(child['positions']),3)];old=[original['positions'][i:i+3] for i in range(0,len(original['positions']),3)]
      for k in range(0,len(child['indices']),3):
       ids=child['indices'][k:k+3];a,b,c=[old[i] for i in ids]
       if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
       tri=[vs[i] for i in ids]
       for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
        x,y,z=[sum(weights[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,child['part']));gap=z-max(hits);self.assertGreater(gap,.0001,(s,clip,t,child['part']));self.assertLess(gap,.012,(s,clip,t,child['part'],gap))
 def test_entire_spiral_remains_rooted_in_belly(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];index=next(i for i,n in enumerate(m.names) if n.startswith('Spiral /'));self.assertGreaterEqual(len(m.vertices[index]),161*12)
   for clip in m.clips:
    for t in (0.,.31,.67):
     mats=m.matrices(clip,t);belly_index=next(i for i,n in enumerate(m.names) if n.startswith('Belly /'));parent=dict(ps[belly_index],positions=sum(m.posed(belly_index,mats),[]));points=m.posed(index,mats)
     for ring in range(161):
      gaps=[]
      for x,y,z in points[ring*12:(ring+1)*12]:
       hits=rays(parent,x,y);self.assertTrue(hits,(s,ring));gaps.append(z-max(hits))
      self.assertLess(min(gaps),-.0002,(s,clip,t,ring));self.assertGreater(max(gaps),.001,(s,clip,t,ring));self.assertLess(max(gaps),.025)
 def test_fixed_feet_strain_winding_and_neutral_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=1e9
     for i,n in enumerate(m.names):
      vs=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Toe /')):
       for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,normals,tri);self.assertGreater(cosine,0.,(s,clip,n,step));self.assertGreater(area,1e-11,(s,n))
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,n,step))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_independent_anatomical_motion_and_coherent_belly(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    if n.startswith(('Belly /','Spiral /')):self.assertTrue(all(js==[1,0,0,0] for js in m.ids[i]))
    if n.startswith(('Foot /','Toe /')):self.assertTrue(all(js==[0,0,0,0] for js in m.ids[i]))
   rotations=m.rotations('attack',.31)
   for j in range(1,len(author.JOINTS[s])):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate);self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_upper_arm_caps_stay_buried_in_torso(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];body_index=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
   for i,n in enumerate(m.names):
    if not n.startswith('Arm /'):continue
    # Stored twenty-sided root caps must be entirely inside the torso, not
    # merely touch it at one point while exposing a flat cutoff shoulder.
    ids=set(m.indices[i][:18*3]);self.assertEqual(len(ids),20)
    for clip in m.clips:
     for t in (0.,.31,.67):
      mats=m.matrices(clip,t);body=dict(ps[body_index],positions=sum(m.posed(body_index,mats),[]));points=m.posed(i,mats)
      for v in ids:
       x,y,z=points[v];hits=sorted(rays(body,x,y));self.assertTrue(any(a+.0002<z<b-.0002 for a,b in zip(hits[::2],hits[1::2])),(s,clip,t,n,v))
 def test_runtime_toes_keep_complete_separate_leg_ownership(self):
  for s,m in self.runtime.items():
   for part,meta in zip(m.parts,m.doc['extras']['source_part_ranges']):
    if part['extras']['part'].startswith('Toe /'):
     self.assertEqual(len(meta['source_parts']),3);self.assertEqual(len({p['part'].split()[-2] for p in meta['source_parts']}),1)
 def test_attack_advances_both_hands_with_species_specific_reach(self):
  for s,m in self.models.items():
   mats=m.matrices('attack',.31);minimum={'poliwhirl':.08,'poliwrath':.15,'politoed':.04}[s]
   for i,n in enumerate(m.names):
    if n.startswith('Hand /'):
     self.assertGreater(max(q[2]-p[2] for p,q in zip(m.vertices[i],m.posed(i,mats))),minimum,(s,n))
 def test_strict_reader_rejects_unreviewed_geometry_and_bytes(self):
  for s,b in self.blobs.items():
   changed=bytearray(b);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_poli_frog(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_poli_frog(s,changed)
if __name__=='__main__':unittest.main()
