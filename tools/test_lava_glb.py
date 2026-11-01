#!/usr/bin/env python3
"""Decoded molten necks, level contact layers and a coherent open shell vent."""
import math,unittest
from pathlib import Path
import lava_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class LavaSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def part(self,m,p,mats):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
 def test_canonical_full_geometry_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_lava(s,author.decode_lava(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'slugma':(12,2290,4108,5),'magcargo':(22,5554,10276,6)}[s])
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_face_stalks_and_rigid_vented_shell(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=5 if n.startswith(('Shell /','Shell ridge /','Shell seam /','Chimney /')) else (3 if n.endswith('-1') else 4) if n.startswith(('Eye stalk /','Eye /')) else 2 if n.startswith(('Lava /','Mouth /')) else 1 if n.startswith('Mantle /') else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_level_contact_layer_neutral_endpoints_and_articulation(self):
  for s,m in self.models.items():
   moved=set();floor=[(p,k) for p,vs in enumerate(m.vertices) for k,v in enumerate(vs) if v[1]<.005];self.assertTrue(floor)
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,n))
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.001:moved.add(n.split(' /')[0])
     for p,k in floor:self.assertAlmostEqual(m.point(p,k,mats)[1],m.vertices[p][k][1],places=7)
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Body','Eye stalk','Eye','Mouth','Shell' if s=='magcargo' else 'Mantle'}<=moved,(s,moved))
 def test_winding_and_edge_strain(self):
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
 def test_actual_posed_stalks_eyes_and_shell_seat(self):
  for s,m in self.models.items():
   body=next(p for p,n in enumerate(m.names) if n.startswith('Body /'));pairs=[]
   for p,n in enumerate(m.names):
    if n.startswith(('Eye stalk /','Shell /')):pairs.append((p,body))
    if n.startswith('Eye / broad'):pairs.append((p,m.names.index('Eye stalk / rising rounded lava horn '+n.split()[-1])))
    if n.startswith('Eye / slim'):pairs.append((p,m.names.index('Eye / broad yellow oval '+n.split()[-1])))
   if s=='magcargo':
    shell=next(p for p,n in enumerate(m.names) if n.startswith('Shell /'));outer=m.names.index('Chimney / hollow stone vent');pairs.append((outer,shell))
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t)
     for a,b in pairs:self.assertTrue(parts_overlap(self.part(m,a,mats),self.part(m,b,mats)),(s,clip,t,m.names[a],m.names[b]))
 def test_posed_molten_face_and_mantle_folds_follow_actual_body(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.55):
     mats=m.matrices(clip,t);body=self.part(m,0,mats);top=dict(body);top['positions']=[v for k in range(0,len(body['positions']),3) for v in (body['positions'][k],body['positions'][k+2],body['positions'][k+1])]
     for p,n in enumerate(m.names):
      if not n.startswith(('Lava /','Mouth /','Mantle /')):continue
      upper=n.startswith('Mantle /');support=top if upper else body
      for x,y,z in set(map(tuple,m.posed(p,mats))):
       hits=ray_intersections(support,x,z if upper else y);self.assertTrue(hits,(s,clip,t,n));self.assertLess(abs((y if upper else z)-max(hits)),.035 if upper else .012,(s,clip,t,n))
 def test_rocking_shell_keeps_vent_open_above_its_recessed_floor(self):
  m=self.models['magcargo'];floor=m.names.index('Chimney / recessed molten floor');vent=m.names.index('Chimney / hollow stone vent');cx=(min(v[0] for v in m.vertices[floor])+max(v[0] for v in m.vertices[floor]))/2;cz=(min(v[2] for v in m.vertices[floor])+max(v[2] for v in m.vertices[floor]))/2;lip=max(v[1] for v in m.vertices[vent])
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t);a=mats[5]
    def undo(v):
     d=[v[k]-a[12+k] for k in range(3)];return [sum(a[4*i+k]*d[k] for k in range(3)) for i in range(3)]
    parts=[]
    for p in range(len(m.parts)):
     points=[undo(v) for v in m.posed(p,mats)];parts.append({'positions':[v for x,y,z in points for v in (x,z,y)],'indices':m.indices[p]})
    for dx,dz in ((0,0),(.002,0),(-.002,0),(0,.002),(0,-.002)):
     self.assertFalse(ray_intersections(parts[vent],cx+dx,cz+dz));floor_hits=ray_intersections(parts[floor],cx+dx,cz+dz);self.assertTrue(floor_hits);hits=[h for q in parts for h in ray_intersections(q,cx+dx,cz+dz)];self.assertAlmostEqual(max(hits),max(floor_hits),places=6);self.assertGreater(lip-max(hits),.085)
 def test_noncanonical_data_rejected(self):
  for s,blob in self.blobs.items():
   with self.assertRaises(ValueError):author.decode_lava(blob[:-1],s)
if __name__=='__main__':unittest.main()
