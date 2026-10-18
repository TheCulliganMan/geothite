#!/usr/bin/env python3
"""Independently decoded dragon poses, supported ribbons, rooted anatomy and fixed contacts."""
import copy,math,os,unittest
from pathlib import Path
import dragon_family_glb as author
import test_dragon_family_sculpt as anatomy
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from test_seahorse_glb import occupied_overlap,inverse_rigid
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,surface_distances,indexed_ray_intersections as rays
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_DRAGON_FAMILY_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class DragonSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.logical={s:logical_model(m) for s,m in cls.runtime.items()};cls.models={s:m.neutral_model() for s,m in cls.logical.items()}
 def part(self,m,i,mats):return dict(part=m.names[i],positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i])
 def find(self,m,prefix,side=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (side is None or n.endswith(' '+str(side))))
 def test_exact_source_packing_hierarchy_inverse_binds_and_limits(self):
  for s,m in self.logical.items():
   self.assertEqual(self.blobs[s],author.export_dragon_family(s));self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_dragon_family(self.blobs[s],s)),author.REST_DIGESTS[s])
   self.assertEqual(len(m.names),{'dratini':17,'dragonair':21,'dragonite':60}[s]);self.assertEqual(len(m.joints),15 if s=='dragonite' else 7);self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,v in author.JOINTS[s]])
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLessEqual(len(self.blobs[s]),1024*1024)
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_closed_outward_anatomy_and_stored_corner_shading(self):assert_closed_models(self,self.models);assert_model_shading(self,self.models)
 def test_every_control_moves_independently_and_complete_digits_keep_their_carriers(self):
  for s,m in self.logical.items():
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(len(author.JOINTS[s]))))
   rotations=m.rotations('attack',.34)
   for j in range(1,len(author.JOINTS[s])):
    isolated=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[j]=rotations[j];mats=m.matrices(rotations=isolated)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,j))
   for i,n in enumerate(m.names):
    owner=None
    if n.startswith(('Eye socket /','Eye white /','Iris /','Pupil /','Glint /','Mouth /','Nostril /','Head /','Muzzle /','Horn /','Forehead /')):owner=2
    elif n.startswith(('Foot /','Foot claw /')):owner=0
    elif n.startswith(('Hand /','Hand claw /')):owner=9 if n.split()[-2 if n.startswith('Hand claw /') else -1]=='-1' else 10
    elif n.startswith(('Orb /','Orb glint /')):owner=2 if n.endswith('0') else 6
    if owner is not None:self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]),(s,n));self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[i]))
 def test_fixed_floor_and_soles_positive_winding_strain_and_neutral_endpoints(self):
  for s,m in self.logical.items():
   fixed=[(i,k) for i,vs in enumerate(m.vertices) for k,v in enumerate(vs) if v[1]<1e-7];self.assertTrue(fixed,s)
   for clip in m.clips:
    for t in (0.,.18,.34,.46,.76,1.):
     mats=m.matrices(clip,t);self.assertEqual(mats[0],IDENTITY)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if m.names[i].startswith(('Foot /','Foot claw /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12,(s,m.names[i],k))
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.8<ratio<1.2,(s,clip,t,m.names[i],ratio))
     for i,k in fixed:self.assertEqual(m.point(i,k,mats),m.vertices[i][k],(s,clip,t,m.names[i]))
     if t in (0.,1.):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_heads_ears_pearls_arms_wings_and_leg_roots_through_real_poses(self):
  for s,m in self.logical.items():
   body=self.find(m,'Body /');head=self.find(m,'Head /');pairs=[(head,body),(self.find(m,'Muzzle /'),head)]
   for i,n in enumerate(m.names):
    if n.startswith(('Horn /','Antenna /','Ear fin /')):pairs.append((i,head))
    elif n.startswith('Orb /'):pairs.append((i,body))
   if s=='dragonite':
    pairs.append((self.find(m,'Tail /'),body))
    for side in (-1,1):
     arm=self.find(m,'Arm /',side);hand=self.find(m,'Hand /',side);haunch=self.find(m,'Haunch /',side);leg=self.find(m,'Leg /',side);foot=self.find(m,'Foot /',side);wing=self.find(m,'Wing /',side);border=self.find(m,'Wing border /',side);spar=self.find(m,'Wing spar /',side)
     pairs += [(arm,body),(hand,arm),(haunch,body),(leg,haunch),(foot,leg),(border,body),(wing,border),(spar,body),(spar,border)]
     for i,n in enumerate(m.names):
      if n.startswith(('Hand claw /','Foot claw /')) and n.split()[-2]==str(side):pairs.append((i,hand if n.startswith('Hand') else foot))
      if n.startswith('Wing rib /') and n.split()[-2]==str(side):pairs += [(i,wing),(i,spar)]
   for clip in m.clips:
    for t in (0.,.18,.34,.76):
     mats=m.matrices(clip,t);parts={i:self.part(m,i,mats) for i in {v for p in pairs for v in p}}
     for child,parent in pairs:self.assertTrue(occupied_overlap(parts[child],parts[parent]),(s,clip,t,m.names[child],m.names[parent]))
 def test_posed_fitted_vertices_edges_and_centres_keep_real_parent_support(self):
  for s,m in self.logical.items():
   for clip in m.clips:
    for t in (.18,.34,.76):
     mats=m.matrices(clip,t);parts=[self.part(m,i,mats) for i in range(len(m.parts))];helper=anatomy.DragonFamilyTests();helper.ms={s:dict(primitives=parts)}
     for species,p,parent in helper.pairs():
      i=m.names.index(p['part']);j=m.names.index(parent['part']);self.assertTrue(all(ids==m.ids[i][0] and ws==[1.,0.,0.,0.] for ids,ws in zip(m.ids[i],m.weights[i])),(s,p['part']));self.assertTrue(all(ids==m.ids[i][0] and ws==[1.,0.,0.,0.] for ids,ws in zip(m.ids[j],m.weights[j])),(s,parent['part']))
      matrix=mats[m.ids[i][0][0]];p=inverse_rigid(p,matrix);parent=inverse_rigid(parent,matrix)
      for k in range(0,len(p['positions']),3):
       x,y,z=p['positions'][k:k+3];hits=rays(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),.019,(s,p['part']))
      for k in range(0,len(p['indices']),3):
       a,b,c=[p['positions'][3*v:3*v+3] for v in p['indices'][k:k+3]]
       if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
       for weights in ((1/3,1/3,1/3),(.5,.5,0),(0,.5,.5),(.5,0,.5)):
        x,y,z=[sum(v[q]*w for v,w in zip((a,b,c),weights)) for q in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertGreater(z-max(hits),.0001,(s,p['part'],k//3))
 def test_ventral_ribbons_keep_actual_three_dimensional_skin_support_through_poses(self):
  for s,m in self.logical.items():
   ribbon=self.find(m,'Tail underside /' if s=='dragonite' else 'Belly /');parent=self.find(m,'Tail /' if s=='dragonite' else 'Body /')
   for clip,t in (('idle',.31),('attack',.34),('hit',.46)):
    mats=m.matrices(clip,t);body=self.part(m,parent,mats);points=set(tuple(p) for p in m.posed(ribbon,mats))
    for gap in surface_distances(body,points,.007):self.assertLess(gap,.007,(s,clip,t))
   bad=[(x,y,z+author.HEIGHTS[s]*2) for x,y,z in m.vertices[ribbon][::19]]
   self.assertTrue(all(gap>.007 for gap in surface_distances(self.models[s]['primitives'][parent],bad,.007)),s)
 def test_continuous_bodies_and_guardian_tail_remain_uncrossed(self):
  for s,m in self.logical.items():
   i=self.find(m,'Tail /' if s=='dragonite' else 'Body /')
   for clip,t in (('idle',.31),('attack',.34),('hit',.46)):
    self.assertIsNone(crossed_faces(self.part(m,i,m.matrices(clip,t))),(s,clip,t))
 def test_source_byte_and_species_mutations_are_rejected(self):
  for s,b in self.blobs.items():
   for bad in (b[:-1],b+b'bad',b[:20]+bytes([b[20]^1])+b[21:]):
    with self.assertRaises(ValueError):author.decode_dragon_family(bad,s)
   with self.assertRaises(ValueError):author.decode_dragon_family(b,next(v for v in author.SPECIES if v!=s))
   model=copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0]+=.0001
   with self.assertRaises(ValueError):author.export_dragon_family(s,model)

if __name__=='__main__':unittest.main()
