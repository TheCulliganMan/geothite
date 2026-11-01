#!/usr/bin/env python3
"""Connected larva bodies, closed cream seam, seated rings and coherent crawling skins."""
import copy,math,unittest
from pathlib import Path
import larva_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class LarvaTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,parts,prefix,ending=None):return next(p for p in parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_distinct_family_anatomy(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_larva(s));self.assertEqual(len(m.parts),33 if s=='caterpie' else 24);self.assertEqual(len(m.joints),12);self.assertFalse((ROOT/(s+'.mesh.json')).exists())
   self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),.95,places=6)
   for prefix,count in [('Body /',1),('Belly /',1 if s=='caterpie' else 0),('Side ring /',10 if s=='caterpie' else 0),('Proleg /',6 if s=='caterpie' else 14),('Horn branch /',2 if s=='caterpie' else 0),('Horn /',1 if s=='weedle' else 0)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_connected_solids(self):assert_closed_models(self,{s:m.neutral_model() for s,m in self.models.items()})
 def test_actual_face_normals(self):assert_model_shading(self,{s:m.neutral_model() for s,m in self.models.items()})
 def test_fitted_eye_and_nose_reflections(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives']
   for mark in parts:
    n=mark['part']
    if n.startswith('Glint /'):parent=self.find(parts,'Eye /',[n.split()[-1]])
    elif n.startswith('Nose glint /'):parent=self.find(parts,'Nose /')
    else:continue
    vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
     q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,(s,n));self.assertGreater(q[2]-max(hits),.0001,(s,n))
 def test_anatomical_owners_and_eight_body_carriers(self):
  for s,m in self.models.items():
   body_owners=set()
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith('Proleg / planted'):expected={0}
    elif n.startswith(('Horn branch /','Horn tip /')):expected={10 if n.endswith('-1') else 11}
    elif n.startswith('Horn /'):expected={10}
    elif n.startswith('Tail sting /'):expected={11}
    elif n.startswith('Tail tip /'):expected={1}
    elif n.startswith(('Head /','Eye','Glint /','Muzzle','Nose','Horn stem /')):expected={9}
    else:
     self.assertTrue(owners<={0,1,2,3,4,5,6,7,8},(s,n,owners));body_owners|=owners;continue
    self.assertEqual(owners,expected,(s,n))
   self.assertEqual(body_owners,set(range(9)),s)
 def test_caterpie_cream_green_seam_is_exact_through_all_clips(self):
  m=self.models['caterpie'];parts=m.neutral_model()['primitives'];a=next(i for i,n in enumerate(m.names) if n.startswith('Body /'));b=next(i for i,n in enumerate(m.names) if n.startswith('Belly /'))
  left={tuple(p):i for i,p in enumerate(m.vertices[a])};right={tuple(p):i for i,p in enumerate(m.vertices[b])};common=left.keys()&right.keys();self.assertEqual(len(common),114)
  pairs=[(left[p],right[p]) for p in common]
  for i,j in pairs:self.assertEqual(m.ids[a][i],m.ids[b][j]);self.assertEqual(m.weights[a][i],m.weights[b][j])
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8);aa=m.posed(a,mats);bb=m.posed(b,mats)
    for i,j in pairs:self.assertLess(math.dist(aa[i],bb[j]),1e-7)
 def test_side_rings_remain_hollow_and_seated_fully_on_green_body(self):
  m=self.models['caterpie'];parts=m.neutral_model()['primitives'];parent=self.find(parts,'Body /')
  for p in parts:
   if not p['part'].startswith('Side ring /'):continue
   side=int(p['part'].split()[-2]);projected=dict(parent,positions=[v for i in range(0,len(parent['positions']),3) for v in (parent['positions'][i+2],parent['positions'][i+1],side*parent['positions'][i])]);depths=[]
   for k in range(0,len(p['positions']),3):
    x,y,z=p['positions'][k:k+3];hits=rays(projected,z,y);self.assertTrue(hits,p['part']);depths.append(side*x-max(hits))
   self.assertLess(max(depths),.007);self.assertLess(min(depths),-.0002)
 def test_occupied_head_proleg_fork_and_sting_roots_survive_motion(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives'];find=lambda prefix,ending=None:self.find(parts,prefix,ending)['part'];body_names=[p['part'] for p in parts if p['part'].startswith(('Body /','Belly /'))];head=find('Head /');pairs=[(head,body_names)]
   for p in parts:
    n=p['part'];side=[n.split()[-1]]
    if n.startswith(('Proleg /','Tail tip /','Tail sting /')):pairs.append((n,body_names))
    elif n.startswith('Eye rim /'):pairs.append((n,[head]))
    elif n.startswith('Eye /'):pairs.append((n,[find('Eye rim /',side)] if s=='caterpie' else [head]))
    elif n.startswith('Muzzle /'):pairs.append((n,[head]))
    elif n.startswith('Muzzle lower /'):pairs.append((n,[find('Muzzle /')]))
    elif n.startswith(('Horn stem /','Horn /','Nose /')):pairs.append((n,[head]))
    elif n.startswith('Horn branch /'):pairs.append((n,[find('Horn stem /')]))
    elif n.startswith('Horn tip /'):pairs.append((n,[find('Horn branch /',side)]))
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed={p['part']:dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)}
     for a,parents in pairs:self.assertTrue(any(parts_overlap(posed[a],posed[b]) for b in parents),(s,clip,t,a,parents))
   detached=copy.deepcopy(self.find(parts,'Head /'));detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,parts[0]))
 def test_floor_endpoints_winding_strain_and_independent_face_and_tips(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8)
     for i,n in enumerate(m.names):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,n));displacement=max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))
      if n.startswith('Proleg / planted'):self.assertLess(displacement,1e-7)
      elif displacement>.001:moved.add(n.split(' /')[0])
      for a,b in zip(posed,m.vertices[i]):
       if b[1]<1e-7:self.assertLess(math.dist(a,b),1e-7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     if step in (0,8):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Body','Head','Proleg'}<=moved,(s,moved))
   mats=m.matrices('attack',.31)
   for i,n in enumerate(m.names):
    parent=8 if n.startswith('Eye /') else 9 if n.startswith(('Horn branch /','Horn /')) else 1 if n.startswith('Tail sting /') else None
    if parent is not None:self.assertGreater(max(math.dist(m.posed(i,mats)[v],skin.transform(mats[parent],p)) for v,p in enumerate(m.vertices[i])),.001,(s,n))
 def test_source_cue_contract(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
