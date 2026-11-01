#!/usr/bin/env python3
"""Independent stored bat skin, face ownership and wing deformation checks."""
import math,os,unittest
from pathlib import Path
import bat_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_BAT_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
COUNTS={'zubat':(28,2368,3352,10),'golbat':(32,3762,4784,10),'crobat':(28,4653,4116,14)}
class BatSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={n:Model((ROOT/(n+'.glb')).read_bytes()) for n in author.SPECIES}
 def test_pupils_are_rooted_in_actual_eye_surfaces_through_clips(self):
  from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
  for species in ['golbat','crobat']:
   m=self.models[species]
   for side in [-1,1]:
    parent=next(i for i,n in enumerate(m.names) if n==('Eye / fitted ivory wedge ' if species=='golbat' else 'Eye / angry fitted yellow wedge ')+str(side))
    child=next(i for i,n in enumerate(m.names) if n==('Eye / narrow black pupil ' if species=='golbat' else 'Eye / narrow vertical pupil ')+str(side))
    for clip in m.clips:
     for t in [0.,.31,.67]:
      mats=m.matrices(clip,t);eye={'positions':[v for p in m.posed(parent,mats) for v in p],'indices':m.indices[parent]}
      points=m.posed(child,mats);pupil={'positions':[v for p in points for v in p],'indices':m.indices[child]}
      self.assertTrue(parts_overlap(eye,pupil),(species,side,clip,t,'detached pupil'))
      for k in range(0,len(m.indices[child]),3):
       ids=m.indices[child][k:k+3];a,b,c=[m.vertices[child][i] for i in ids]
       if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
       tri=[points[i] for i in ids]
       for weights in [(1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5)]:
        x,y,z=[sum(weights[j]*tri[j][axis] for j in range(3)) for axis in range(3)]
        hits=rays(eye,x,y);self.assertTrue(hits,(species,side,clip,t))
        gap=z-max(hits);self.assertGreater(gap,.0003);self.assertLess(gap,.0045)
 def test_neutral_anatomy_and_reader_bounds(self):
  for n,m in self.models.items():
   blob=(ROOT/(n+'.glb')).read_bytes();neutral=author.decode_bat(blob,n)
   self.assertEqual(author.geometry_digest(neutral),author.REST_DIGESTS[n])
   self.assertEqual((len(m.parts),sum(len(p) for p in m.vertices),sum(len(p)//3 for p in m.indices),len(m.joints)),COUNTS[n])
   self.assertLess(len(blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   self.assertFalse((ROOT/(n+'.mesh.json')).exists())
 def test_standard_hierarchy_binds_and_weights(self):
  for n,m in self.models.items():
   self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS[n]])
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):
     self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<len(m.joints) for j in js))
 def test_hollow_faces_eyes_fangs_and_ear_insets_keep_common_owners(self):
  for n,m in self.models.items():
   for p,name in enumerate(m.names):
    if name.startswith(('Body /','Face /','Mouth /','Eye /','Fang /')):self.assertTrue(all(js==[1,0,0,0] for js in m.ids[p]),(n,name))
    if name.startswith('Ear /'):
     owner=8 if sum(v[0] for v in m.vertices[p])<0. else 9
     self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p]),(n,name))
    if name.startswith(('Leg /','Toe /')):self.assertTrue(all(js==[0,0,0,0] for js in m.ids[p]))
 def test_connected_wing_fields_and_distinct_lower_pair(self):
  for n,m in self.models.items():
   for p,name in enumerate(m.names):
    if name.startswith('Wing /'):
     used={j for ids,ws in zip(m.ids[p],m.weights[p]) for j,w in zip(ids,ws) if w>0}
     left=sum(v[0] for v in m.vertices[p])<0.
     self.assertEqual(used,({1,10,11} if left else {1,12,13}) if 'lower' in name else ({1,2,3,4} if left else {1,5,6,7}))
 def test_motion_winding_and_edge_strain(self):
  for n,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8)
     for p,name in enumerate(m.names):
      posed=m.posed(p,mats)
      if name.startswith(('Leg /','Toe /')):self.assertEqual(posed,m.vertices[p])
      normals=m.posed(p,mats,True)
      for i in range(0,len(m.indices[p]),3):
       tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
       self.assertGreater(cosine,0.,(n,clip,step,name,i));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
        self.assertTrue(.7<ratio<1.3,(n,clip,step,name,ratio))
 def test_cue_relative_endpoints_body_brace_and_tip_delay(self):
  for n,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   body=[m.rotations('attack',i/100)[1][0] for i in range(101)]
   self.assertGreater(body[20],max(body)*.9)
   for a in m.doc['animations']:
    idle=a['name'].endswith('.idle');self.assertEqual(a['extras']['loopSuggested'],idle)
    self.assertEqual(a['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
