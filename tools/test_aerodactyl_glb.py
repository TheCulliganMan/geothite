#!/usr/bin/env python3
"""Decoded fossil skin geometry, rooted anatomy and genuinely empty articulated jaws."""
import copy,math,os,unittest
from pathlib import Path
import aerodactyl_glb as author
import test_aerodactyl_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import occupied_overlap,inverse_rigid
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,indexed_ray_intersections as rays
PATH=Path(os.environ.get('CRYSTAL_AERODACTYL_TEST_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/aerodactyl.glb'))

class AerodactylSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=PATH.read_bytes();cls.runtime=Model(cls.blob);cls.m=logical_model(cls.runtime);cls.model=cls.m.neutral_model()
 def part(self,i,mats):return dict(part=self.m.names[i],positions=[v for p in self.m.posed(i,mats) for v in p],normals=[v for p in self.m.posed(i,mats,True) for v in p],indices=self.m.indices[i])
 def find(self,prefix):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix))
 def test_exact_source_packing_hierarchy_inverse_binds_and_limits(self):
  m=self.m;self.assertEqual(self.blob,author.export_aerodactyl());self.assertEqual(author.geometry_digest(self.model),author.REST_DIGESTS['aerodactyl']);self.assertEqual(author.geometry_digest(author.decode_aerodactyl(self.blob)),author.REST_DIGESTS['aerodactyl'])
  self.assertEqual(len(m.names),56);self.assertEqual(len(m.joints),9);self.assertEqual(m.parents[:9],[p for n,p,v in author.JOINTS['aerodactyl']]);self.assertLessEqual(len(self.runtime.parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blob),1024*1024)
  self.assertEqual(sum(map(len,m.vertices)),18907);self.assertEqual(sum(len(i)//3 for i in m.indices),31516)
  for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_actual_corner_normals(self):assert_closed_models(self,{'aerodactyl':self.model});assert_model_shading(self,{'aerodactyl':self.model})
 def test_independent_controls_and_complete_jaw_wing_and_digit_carriers(self):
  m=self.m;self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(9)))
  rotations=m.rotations('attack',.34)
  for j in range(1,9):
   isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
   self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,j)
  for i,n in enumerate(m.names):
   if n.startswith(('Body /','Leg /')):continue
   owner=author.skin_weights('aerodactyl',n,(0.,0.,0.))[0][0]
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),n);self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]),n)
 def test_fixed_soles_positive_winding_strain_and_neutral_endpoints(self):
  m=self.m;floor=[(i,k) for i,vs in enumerate(m.vertices) for k,p in enumerate(vs) if p[1]<1e-7];self.assertTrue(floor)
  for clip in m.clips:
   for t in (0.,.18,.34,.46,.76,1.):
    mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
    for i,vs in enumerate(m.vertices):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,m.names[i])
     if m.names[i].startswith(('Foot /','Foot claw /')):self.assertEqual(posed,vs)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,m.names[i],k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.8<math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b])<1.2,(clip,t,m.names[i]))
    for i,k in floor:self.assertEqual(m.point(i,k,mats),m.vertices[i][k])
    if t in (0.,1.):
     for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_actual_roots_stay_occupied_through_independent_poses(self):
  m=self.m;pairs=[];body=self.find('Body /');head=self.find('Skull /');jaw=self.find('Jaw /')
  pairs.append((head,body))
  for i,n in enumerate(m.names):
   side=n.split()[-2] if n.startswith(('Wing finger /','Wing claw /','Foot claw /')) else n.split()[-1]
   if n.startswith('Jaw hinge /'):pairs.extend(((i,head),(i,jaw)))
   elif n.startswith('Skull horn /'):pairs.append((i,head))
   elif n.startswith('Tooth /'):pairs.append((i,head if ' upper ' in n else jaw))
   elif n.startswith('Wing /'):pairs.extend(((i,body),(i,self.find('Wing arm / continuous bent leading fossil arm '+side))))
   elif n.startswith('Wing arm /'):pairs.append((i,body))
   elif n.startswith(('Wing finger /','Wing claw /')):pairs.append((i,self.find('Wing arm / continuous bent leading fossil arm '+side)))
   elif n.startswith('Leg /'):pairs.extend(((i,body),(i,self.find('Foot / grounded narrow fossil pad '+side))))
   elif n.startswith('Foot claw /'):pairs.append((i,self.find('Foot / grounded narrow fossil pad '+side)))
   elif n.startswith('Tail /'):pairs.append((i,body))
   elif n.startswith('Tail tip /'):pairs.append((i,self.find('Tail /')))
  for clip in m.clips:
   for t in (0.,.18,.34,.46,.76):
    mats=m.matrices(clip,t);parts={i:self.part(i,mats) for i in {v for p in pairs for v in p}}
    for child,parent in pairs:self.assertTrue(occupied_overlap(parts[child],parts[parent]),(clip,t,m.names[child],m.names[parent]))
 def test_articulated_jaw_retains_real_empty_space_and_closed_negative_control(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.34,.46,.76):
    mats=m.matrices(clip,t);helper=anatomy.AerodactylTests();helper.ps=[self.part(i,mats) for i in range(len(m.parts))];helper.test_open_jaw_is_actual_empty_space_between_stored_solids()
  helper=anatomy.AerodactylTests();helper.ps=self.model['primitives'];helper.test_closed_mouth_negative_control_has_no_open_clearance()
 def test_posed_fitted_face_palate_and_tongue_keep_actual_support(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.34,.46,.76):
    mats=m.matrices(clip,t);helper=anatomy.AerodactylTests();helper.ps=[]
    for i in range(len(m.parts)):
     p=self.part(i,mats)
     if not m.names[i].startswith(('Body /','Leg /')):
      matrix=mats[m.ids[i][0][0]];p=inverse_rigid(p,matrix)
     helper.ps.append(p)
    helper.test_fitted_face_palate_and_tongue_vertices_follow_stored_surfaces();helper.test_fitted_front_triangles_clear_actual_parent()
 def test_source_byte_and_species_mutations_are_rejected(self):
  for bad in (self.blob[:-1],self.blob+b'bad',self.blob[:20]+bytes([self.blob[20]^1])+self.blob[21:]):
   with self.assertRaises(ValueError):author.decode_aerodactyl(bad)
  with self.assertRaises(ValueError):author.export_fossil('charizard')
  model=copy.deepcopy(author.authoring_model('aerodactyl'));model['primitives'][0]['positions'][0]+=.0001
  with self.assertRaises(ValueError):author.export_aerodactyl(model)

if __name__=='__main__':unittest.main()
