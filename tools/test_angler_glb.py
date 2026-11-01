#!/usr/bin/env python3
"""Decoded angler fins, flexible stalks, coherent lights and ground contact."""
import hashlib,math,struct,unittest
from pathlib import Path
import angler_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class AnglerSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def part(self,m,p,mats):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
 def test_canonical_original_triangle_geometry_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_angler(s,author.decode_angler(self.blobs[s],s)));self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'chinchou':(21,12754,11116,10),'lanturn':(29,17971,15572,12)}[s]);self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   h=hashlib.sha256()
   for p in m.neutral_model()['primitives']:
    for i in p['indices']:h.update(struct.pack('<3f',*p['positions'][3*i:3*i+3]))
    h.update(struct.pack('<4f',*p['base_color']))
   self.assertEqual(h.hexdigest(),author.TRIANGLE_DIGESTS[s])
   if s=='lanturn':
    other={'primitives':[p for p in m.neutral_model()['primitives'] if not p['part'].startswith('Mask /')]}
    self.assertEqual(len(other['primitives']),28);self.assertEqual(author.geometry_digest(other),'1cc08ef37293e186351c9799d5cb65f2f61c13ecc3361ab201c2dcf89d750263')
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_face_lure_and_fin_fold_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=2 if n.startswith(('Fin / broad','Fin fold /')) and n.endswith('-1') else 3 if n.startswith(('Fin / broad','Fin fold /')) else (8 if n.endswith('-1') else 9) if n.startswith('Fin / soft') else ((5 if n.endswith('-1') else 7) if s=='chinchou' else (5 if 'large' in n else 7)) if n.startswith('Lure /') else 11 if n.startswith('Tail fin /') else 1 if n.startswith(('Body /','Mask /','Eye /','Pupil /','Mouth /')) else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_floor_height_neutral_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,n))
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.002:moved.add(n.split(' /')[0])
      if n.startswith('Fin / soft'):
       for a,b in zip(posed,m.vertices[p]):self.assertAlmostEqual(a[1],b[1],places=7)
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Body','Fin','Stalk','Lure','Eye'}<=moved,(s,moved))
   if s=='lanturn':self.assertTrue({'Tail','Tail fin'}<=moved)
 def test_winding_and_physical_edge_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.31,.55,.80):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,n,ratio))
 def test_actual_posed_fin_stalk_bulb_and_tail_joins(self):
  for s,m in self.models.items():
   def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
   pairs=[(p,0) for p,n in enumerate(m.names) if n.startswith('Fin /')]
   if s=='chinchou':
    for side in ('-1','1'):
     stalk=find('Stalk / arched flexible antenna '+side);pairs += [(stalk,0),(stalk,find('Lure / hanging golden teardrop '+side))]
   else:
    main=find('Stalk / single');branch=find('Stalk / short');tail=find('Tail /');pairs += [(main,0),(main,branch),(main,find('Lure / golden large')),(branch,find('Lure / golden small')),(tail,0)]+[(p,tail) for p,n in enumerate(m.names) if n.startswith('Tail fin /')]
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t)
     for a,b in pairs:self.assertTrue(parts_overlap(self.part(m,a,mats),self.part(m,b,mats)),(s,clip,t,m.names[a],m.names[b]))
 def test_fitted_light_centers_and_fin_folds_remain_on_actual_support(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.55):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      if n.startswith('Lure / pale'):prefix='Lure / hanging golden teardrop '+n.split()[-1] if s=='chinchou' else 'Lure / golden '+('large' if 'large' in n else 'small')
      elif n.startswith('Fin fold /'):prefix='Fin / broad rounded pectoral '+n.split()[-1]
      else:continue
      parent=next(i for i,q in enumerate(m.names) if q.startswith(prefix));owner=m.ids[parent][0][0];a=mats[owner]
      def local(part):
       points=[]
       for v in m.posed(part,mats):
        d=[v[k]-a[12+k] for k in range(3)];points.extend(sum(a[4*j+k]*d[k] for k in range(3)) for j in range(3))
       return {'positions':points,'indices':m.indices[part]}
      support=local(parent);mark=local(p)
      for i in range(0,len(mark['positions']),3):
       x,y,z=mark['positions'][i:i+3];hits=ray_intersections(support,x,y);self.assertTrue(hits,(s,n));self.assertLess(abs(z-max(hits)),.023 if n.startswith('Lure /') else .008,(s,n))
 def test_attack_strokes_fins_and_lures_sway_independently(self):
  for s,m in self.models.items():
   mats=m.matrices('attack',.18)
   for p,n in enumerate(m.names):
    if n.startswith('Fin / broad'):
     tips=[k for k,v in enumerate(m.vertices[p]) if abs(v[0])>.45*author.HEIGHTS[s]/author.authoring_transform(s)[1]];self.assertTrue(tips)
     self.assertLess(sum(m.point(p,k,mats)[1]-m.vertices[p][k][1] for k in tips)/len(tips),-.02,(s,n))
    if n.startswith(('Lure / hanging','Lure / golden')):
     owner=m.ids[p][0][0];self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[owner-1],v))>.005 for k,v in enumerate(m.vertices[p])),(s,n))
 def test_noncanonical_data_rejected(self):
  for s,b in self.blobs.items():
   with self.assertRaises(ValueError):author.decode_angler(b[:-1],s)
if __name__=='__main__':unittest.main()
