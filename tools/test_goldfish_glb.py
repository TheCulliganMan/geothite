#!/usr/bin/env python3
"""Independent stored goldfish skin decoding, coherent paint, roots and empty mouths."""
import copy,math,os,unittest
from pathlib import Path
import goldfish_glb as author
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
import test_goldfish_sculpt as anatomy
from test_seahorse_glb import occupied_overlap,inverse_rigid
from sculpt_test_support import assert_closed_models,assert_model_shading,indexed_ray_intersections as rays
from skin_glb_test_support import IDENTITY
from sculpt_geometry import ellipsoid

ROOT=Path(os.environ.get('CRYSTAL_GOLDFISH_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class GoldfishSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES}
  cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,matrices):return dict(part=m.names[i],positions=[v for p in m.posed(i,matrices) for v in p],indices=m.indices[i],normals=[v for p in m.posed(i,matrices,True) for v in p])
 def test_exact_source_packing_hierarchy_bind_and_existing_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_goldfish(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_goldfish(self.blobs[s],s)),author.REST_DIGESTS[s])
   self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertEqual(len(m.names),{'goldeen':36,'seaking':70,'magikarp':30}[s]);self.assertEqual(len(m.joints),{'goldeen':6,'seaking':8,'magikarp':9}[s])
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_actual_corner_shading(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_independent_controls_and_coherent_painted_assemblies(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(1,len(author.JOINTS[s]))))
   for i,name in enumerate(m.names):
    owner=author.owner(s,name);self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[i]),(s,name));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
   rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
 def test_waterline_rigid_strain_winding_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True)
      for p,q in zip(vs,posed):self.assertAlmostEqual(p[1],q[1],places=7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertAlmostEqual(math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]),1.,places=6)
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_actual_appendage_roots_through_decoded_poses(self):
  for s,m in self.logical.items():
   body=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76):
     mats=m.matrices(clip,t);b=self.part(m,body,mats)
     for i,n in enumerate(m.names):
      if n.startswith(('Tail /','Pectoral /','Dorsal /','Pelvic /','Horn /','Whisker /','Lip /','Mouth /')):self.assertTrue(occupied_overlap(self.part(m,i,mats),b),(s,clip,t,n))
 def test_posed_fitted_vertices_edges_and_centres_clear_actual_parents(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.76):
     mats=m.matrices(clip,t);parts=[inverse_rigid(self.part(m,i,mats),mats[author.owner(s,n)]) for i,n in enumerate(m.names)]
     helper=anatomy.GoldfishTests();helper.models={s:dict(primitives=parts)}
     for name,p,parent in helper.pairs():
      for k in range(0,len(p['positions']),3):
       x,y,z=p['positions'][k:k+3];hits=rays(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),.02,(s,p['part']))
      for k in range(0,len(p['indices']),3):
       a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
       if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
       for ww in ((1/3,1/3,1/3),(.5,.5,0),(0,.5,.5),(.5,0,.5)):
        x,y,z=[sum(v[q]*w for v,w in zip((a,b,c),ww)) for q in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertGreater(z-max(hits),.0001,(s,p['part'],k//3))
 def test_whole_posed_mouth_stays_empty_and_filler_is_rejected(self):
  helper=anatomy.GoldfishTests()
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (0.,.18,.34,.76):
     mats=m.matrices(clip,t);parts=[inverse_rigid(self.part(m,i,mats),mats[1]) for i in range(len(m.parts))];self.assertTrue(helper.empty_mouth(parts),(s,clip,t))
   lip=next(p for p in self.models[s]['primitives'] if p['part'].startswith('Lip /'));y=(min(lip['positions'][1::3])+max(lip['positions'][1::3]))/2;z=max(lip['positions'][2::3]);filler=ellipsoid('Bad filled mouth',(.5,.5,.5),(0,y,z-.005),(.04,.06,.015),20,14)
   self.assertFalse(helper.empty_mouth(self.models[s]['primitives']+[filler]),s)
 def test_corrupt_bytes_source_and_species_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_goldfish(bad,s)
   other=next(x for x in author.SPECIES if x!=s)
   with self.assertRaises(ValueError):author.decode_goldfish(b,other)
   m=copy.deepcopy(author.authoring_model(s));m['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_goldfish(s,m)

if __name__=='__main__':unittest.main()
