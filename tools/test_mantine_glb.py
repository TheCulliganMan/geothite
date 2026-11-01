#!/usr/bin/env python3
"""Decoded two-tone ray wing seams, seated markings and attached Remora."""
import hashlib,math,struct,unittest
from pathlib import Path
import mantine_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
from sea_sculpt import underside
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MantineSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'mantine.glb').read_bytes();cls.m=skin.Model(cls.blob)
 def part(self,p,mats):return {'positions':[v for point in self.m.posed(p,mats) for v in point],'normals':[v for point in self.m.posed(p,mats,True) for v in point],'indices':self.m.indices[p]}
 def find(self,prefix):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix))
 def test_canonical_original_triangle_geometry_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_mantine(author.decode_mantine(self.blob)));self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(32,19485,27416,13));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  zs=[v[2] for part in m.vertices for v in part];self.assertEqual(m.doc['extras']['reference_span'],max(zs)-min(zs));self.assertGreater(max(zs)-min(zs),2.*author.HEIGHT)
  h=hashlib.sha256()
  for p in m.neutral_model()['primitives']:
   for i in p['indices']:h.update(struct.pack('<3f',*p['positions'][3*i:3*i+3]))
   h.update(struct.pack('<4f',*p['base_color']))
  self.assertEqual(h.hexdigest(),author.TRIANGLE_DIGEST)
  other={'primitives':[p for p in m.neutral_model()['primitives'] if not p['part'].startswith(('Back /','Wing border /','Remora mouth /','Remora eye /','Remora glint /'))]}
  self.assertEqual(len(other['primitives']),24);self.assertEqual(author.geometry_digest(other),'541ff047c2b76639172432964a8ddaa140c2321232185f563fc6842834240053')
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_face_tail_layers_and_remora_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=9 if n.startswith(('Tail leaf /','Tail leaf back /')) else 6 if n.startswith('Cephalic fin /') and n.endswith('-1') else 7 if n.startswith('Cephalic fin /') else 12 if n.startswith('Remora tail /') else 10 if n.startswith('Remora') and not n.startswith('Remora / slender') else 1 if n.startswith(('Body /','Back /','Eye /','Mouth /','Gill /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_neutral_endpoints_fixed_belly_contact_and_articulation(self):
  m=self.m;moved=set();body=self.find('Body /')
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
    for a,b in zip(m.posed(body,mats),m.vertices[body]):self.assertAlmostEqual(a[1],b[1],places=7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Wing','Wing border','Cephalic fin','Tail','Tail leaf','Remora','Remora tail'}<=moved,moved)
 def test_two_wing_layers_keep_every_shared_seam_exactly_coincident(self):
  m=self.m
  for side in ('-1','1'):
   a=self.find('Wing / curved blue back '+side);b=self.find('Wing / pale curved underside '+side)
   aa={tuple(v):k for k,v in enumerate(m.vertices[a])};bb={tuple(v):k for k,v in enumerate(m.vertices[b])};shared=aa.keys()&bb.keys();self.assertGreater(len(shared),45)
   for clip in m.clips:
    for t in (.18,.31,.55,.80):
     mats=m.matrices(clip,t)
     for v in shared:self.assertLess(math.dist(m.point(a,aa[v],mats),m.point(b,bb[v],mats)),1e-7,(clip,side,v))
 def test_winding_and_physical_edge_strain(self):
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
 def test_actual_posed_wing_paddle_tail_and_remora_joins(self):
  m=self.m;f=self.find;body=f('Body /');tail=f('Tail /');leaf=f('Tail leaf /');fish=f('Remora / attached');attachment=f('Remora / dorsal');stem=f('Remora / slender')
  pairs=[(body,tail),(tail,leaf),(leaf,f('Tail leaf back /')),(fish,f('Remora nose /')),(fish,stem),(stem,f('Remora tail /')),(fish,attachment),(attachment,f('Wing / pale curved underside 1'))]
  pairs += [(p,body) for p,n in enumerate(m.names) if n.startswith(('Wing /','Cephalic fin /'))]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.part(a,mats),self.part(b,mats)),(clip,t,m.names[a],m.names[b]))
 def test_seated_wing_zigzags_and_remora_markings_follow_support(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     parent=None;mark=self.part(p,mats);sign=n.split()[-1]
     if n.startswith('Wing border /'):
      parent=self.part(self.find('Wing / pale curved underside '+sign),mats);mark=underside(mark);parent=underside(parent)
     elif n.startswith('Remora eye /'):parent=self.part(self.find('Remora / attached'),mats)
     elif n.startswith('Remora glint /'):parent=self.part(self.find('Remora eye / fitted tiny dark oval '+sign),mats)
     elif n.startswith('Remora mouth /'):parent=self.part(self.find('Remora nose /'),mats)
     if parent is None:continue
     # The full marking remains seated, including narrow zigzag endpoints.
     for i in range(0,len(mark['positions']),3):
      x,y,z=mark['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(clip,n));self.assertLess(abs(z-max(hits)),.017,(clip,n))
 def test_attack_lifts_wings_and_remora_above_the_resting_contact_plane(self):
  m=self.m;mats=m.matrices('attack',.18)
  for side in ('-1','1'):
   p=self.find('Wing / curved blue back '+side);tips=[k for k,v in enumerate(m.vertices[p]) if abs(v[0])>.90*author.SCALE];self.assertTrue(tips)
   self.assertGreater(sum(m.point(p,k,mats)[1]-m.vertices[p][k][1] for k in tips)/len(tips),.20)
   base=2 if side=='-1' else 4
   self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[base],m.vertices[p][k]))>.005 for k in tips),'unmarked wing tip does not flex independently')
  p=self.find('Remora / attached');self.assertGreater(sum(a[1]-b[1] for a,b in zip(m.posed(p,mats),m.vertices[p]))/len(m.vertices[p]),.06)
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_mantine(self.blob[:-1])
if __name__=='__main__':unittest.main()
