#!/usr/bin/env python3
"""Decoded Hitmontop skin: fixed spin point, raised spiked feet and coherent tail ball."""
import math,os,unittest
from pathlib import Path
import hitmontop_glb as author
import test_gengar_glb as skin
import test_donphan_sculpt as joins
from skin_glb_test_support import IDENTITY,transform
ROOT=Path(os.environ.get('CRYSTAL_HITMONTOP_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))/'hitmontop.glb'
class HitmontopSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.m=skin.Model(cls.blob)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_hitmontop(author.decode_hitmontop(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(41,10934,17604,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(0,0,0) if n.startswith(('Eye /','Mouth /')) else (255,255,255) if n.startswith(('Pupil /','Ankle wrap /','Foot spike /','Wrist wrap /','Tail spike /')) else (82,82,173) if n.startswith(('Body /','Foot /','Tail ball /','Helmet crease /')) else (198,156,33)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_gaze_spiked_feet_fitted_soles_and_tail_point_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith('Spin point /') else 1 if n.startswith(('Head /','Helmet flange /','Eye /','Pupil /','Helmet crease /','Mouth /')) else (4 if n.endswith('-1') else 5) if n.startswith(('Arm /','Wrist wrap /','Hand /')) else 10 if n.startswith(('Tail ball /','Tail spike /')) else None
   if n.startswith(('Leg /','Ankle wrap /','Foot /','Foot spike /','Foot sole /')):
    side=n.split()[-2] if n.startswith('Foot spike /') else n.split()[-1];owner=6 if side=='-1' else 7
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_fixed_spin_point_raised_feet_endpoints_and_articulation(self):
  m=self.m;moved=set();body=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12);body_top=max(v[1] for v in m.posed(body,mats))
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith('Spin point /'):self.assertEqual(posed,m.vertices[p]);self.assertEqual(min(v[1] for v in posed),0.)
     if n.startswith('Foot /'):self.assertGreater(min(v[1] for v in posed),body_top)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Head','Eye','Arm','Wrist wrap','Hand','Leg','Foot','Foot sole','Foot spike','Tail','Tail ball','Tail spike'}<=moved,moved)
 def test_winding_and_edge_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,t,n,ratio))
 def test_actual_posed_neck_balance_point_wrapped_limbs_spikes_and_tail_roots(self):
  m=self.m
  def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Body /');head=find('Head /');neck=find('Neck /');tail=find('Tail /');ball=find('Tail ball /')
  pairs=[(neck,body),(head,neck),(find('Spin point /'),head),(tail,body),(ball,tail),(find('Tail spike /'),ball)]
  for side in ('-1','1'):
   arm=find('Arm / rooted curved tan handstand forelimb '+side);wrist=find('Wrist wrap / rooted ivory handstand cuff '+side);leg=find('Leg / rooted short bent tan handstand shin '+side);ankle=find('Ankle wrap / rooted ivory handstand foot band '+side);foot=find('Foot / broad blue three-spiked handstand boot '+side)
   pairs += [(arm,body),(wrist,arm),(find('Hand / round tan handstand fist '+side),wrist),(leg,body),(ankle,leg),(foot,leg),(foot,ankle),(find('Helmet flange / thick swept tan headstand brim '+side),head)]
   pairs += [(find('Foot spike / three rooted ivory handstand points '+side+' '+str(j)),foot) for j in range(3)]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:
     def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
     self.assertTrue(joins.DonphanTests.overlap(self,part(a),part(b)),(clip,t,m.names[a],m.names[b]))
 def test_idle_completes_one_forward_turn_with_a_smooth_quaternion_wrap(self):
  m=self.m;pivot=author.JOINTS[1][2];point=author.pivot((.20,-1.20,0.));previous=None;total=0.
  for k in range(161):
   mats=m.matrices('idle',k/160);q=transform(mats[1],point);direction=(q[0]-pivot[0],q[2]-pivot[2])
   if previous is not None:
    step=math.atan2(previous[0]*direction[1]-previous[1]*direction[0],previous[0]*direction[0]+previous[1]*direction[1]);self.assertLess(step,0.);self.assertGreater(step,-.08);total+=step
   previous=direction
  self.assertAlmostEqual(total,-math.tau,places=5)
  for t in (1.-1e-5,1e-5):
   mats=m.matrices('idle',t)
   for p in range(len(m.parts)):
    self.assertLess(max(math.dist(a,b) for a,b in zip(m.posed(p,mats),m.vertices[p])),.00002)
  before=m.matrices('idle',.999);after=m.matrices('idle',.001)
  left=transform(before[1],point);right=transform(after[1],point)
  self.assertLess(math.dist(left,right),.002)
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_hitmontop(self.blob[:-1])
if __name__=='__main__':unittest.main()
