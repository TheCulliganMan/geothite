#!/usr/bin/env python3
"""Stored middle poison-pin skins: anatomical supports, real roots and coherent fitted spots."""
import copy,math,os,unittest
from pathlib import Path
import nido_middle_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import indexed_ray_intersections as rays
import test_nidorina_sculpt as female_tests
import test_nidorino_sculpt as male_tests
def source_test(species,ps):
 test=(female_tests.NidorinaTests if species=='nidorina' else male_tests.NidorinoTests)();test.ps=ps;return test
def fitted_pairs(ps,species):return source_test(species,ps).pairs()
ROOT=Path(os.environ.get('CRYSTAL_NIDO_MIDDLE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class NidoMiddleSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def test_exact_geometry_hierarchy_inverse_binds_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_nido_middle(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(self.runtime[s].neutral_model()),author.PACKED_REST_DIGESTS[s])
   expected=(59,35,19478,32064,15) if s=='nidorina' else (74,39,19860,32172,13)
   self.assertEqual((len(m.parts),len(self.runtime[s].parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices),len(m.joints)),expected);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for name,p,pivot in author.JOINTS[s]]);self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(author.JOINTS[s]))))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_rigid_paws_face_ears_and_per_limb_support_packing(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owner=0 if n.startswith(('Foot /','Foot claw /','Claw /')) else 1 if n.startswith(('Body /','Breast /','Flank spot /','Back spine /')) else 2 if n.startswith(('Head /','Muzzle /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Nostril /','Fang /','Head spot /')) else (3 if n.endswith('-1') else 4) if n.startswith(('Ear /','Ear inset /','Ear fold /')) else (2 if s=='nidorina' else 5) if n.startswith(('Mouth /','Jaw /')) else 6 if n.startswith('Horn /') else None
    if owner is not None:self.assertEqual({j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w},{owner},(s,n))
   runtime=self.runtime[s]
   for p,meta in zip(runtime.parts,runtime.doc['extras']['source_part_ranges']):
    names=[q['part'] for q in meta.get('source_parts',[])] or [p['extras']['part']];legs=set()
    for n in names:
     kind=n.split(' /')[0]
     if kind in ('Hand claw','Foot claw','Haunch spot'):legs.add((n.split()[-2],))
     elif kind in ('Claw','Leg spot'):legs.add(tuple(n.split()[-3:-1]))
    self.assertLessEqual(len(legs),1,(s,p['extras']['part'],'cross-limb support'))
 def test_floor_strain_winding_and_neutral_endpoints(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=1e9
     for i,n in enumerate(m.names):
      vs=m.posed(i,mats);ns=m.posed(i,mats,True);floor=min(floor,min(v[1] for v in vs));self.assertGreaterEqual(min(v[1] for v in vs),-1e-7,(s,clip,n))
      if n.startswith(('Foot /','Foot claw /','Claw /')):
       for p,q in zip(vs,m.vertices[i]):self.assertLess(math.dist(p,q),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(vs,ns,tri);self.assertGreater(cosine,0.,(s,clip,n,step));self.assertGreater(area,1e-11)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(vs[a],vs[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n,step))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_every_carrier_has_independent_motion(self):
  for s,m in self.models.items():
   rotations=m.rotations('attack',.31)
   for j in range(1,len(author.JOINTS[s])):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[j]=rotations[j];mats=m.matrices(rotations=isolate);self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_actual_roots_and_uncrossed_surfaces_survive_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives']
   for clip in m.clips:
    for t in (0.,.13,.31,.67,.83):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)];test=source_test(s,posed)
     if s=='nidorina':
      test.test_head_muzzle_ears_fangs_tail_and_spines_have_volume_roots();test.test_all_limbs_and_claws_have_volume_roots();test.test_ears_tail_spines_and_limbs_do_not_cross_themselves()
     else:
      test.test_head_jaw_horn_ears_fangs_tail_and_spines_have_volume_roots();test.test_all_four_leg_chains_and_twelve_claws_have_volume_roots();test.test_horn_ears_tail_spines_and_legs_do_not_cross_themselves()
 def test_fitted_caps_follow_actual_posed_parent(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];authored=list(fitted_pairs(ps,s))
   for clip in m.clips:
    for t in (0.,.31,.67):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for (rest,_),(child,parent) in zip(authored,fitted_pairs(posed,s)):
      original=[rest['positions'][i:i+3] for i in range(0,len(rest['positions']),3)];points=[child['positions'][i:i+3] for i in range(0,len(child['positions']),3)]
      for k in range(0,len(child['indices']),3):
       ids=child['indices'][k:k+3];a,b,c=[original[i] for i in ids]
       if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
       tri=[points[i] for i in ids]
       for ws in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
        x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,clip,t,child['part']));gap=z-max(hits);self.assertGreater(gap,.0001,(s,clip,t,child['part'],gap));self.assertLess(gap,.012,(s,clip,t,child['part'],gap))
        
 def test_strict_reader_rejects_geometry_and_byte_mutations(self):
  for s in author.SPECIES:
   changed=bytearray(self.blobs[s]);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_nido_middle(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_nido_middle(s,changed)
if __name__=='__main__':unittest.main()
