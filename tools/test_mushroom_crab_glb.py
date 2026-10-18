#!/usr/bin/env python3
"""Decoded fungal caps, fitted markings, six appendages and planted walking feet."""
import copy,math,unittest
from pathlib import Path
import mushroom_crab_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MushroomCrabTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,parts,prefix,ending=None):return next(p for p in parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_family_anatomy(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_mushroom_crab(s));self.assertEqual(len(m.parts),56 if s=='paras' else 36);self.assertEqual(len(m.joints),11 if s=='paras' else 10)
   self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),author.HEIGHTS[s],places=6)
   for prefix,count in [('Eye /',2),('Leg /',4),('Foot /',4),('Foreclaw /',2),('Mushroom cap /',2 if s=='paras' else 1),('Mushroom spot /',10 if s=='paras' else 9),('Pupil /',2 if s=='paras' else 0)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{s:m.neutral_model() for s,m in self.models.items()})
 def test_actual_face_normals(self):assert_model_shading(self,{s:m.neutral_model() for s,m in self.models.items()})
 def test_fitted_gaze_mouth_and_golden_cap_spots(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives']
   for mark in parts:
    n=mark['part'];side=[n.split()[-1]];top=False
    if n.startswith('Pupil /'):parent=self.find(parts,'Eye /',side)
    elif n.startswith('Glint /'):parent=self.find(parts,'Pupil /',side)
    elif n.startswith('Mouth /'):parent=self.find(parts,'Muzzle /')
    elif n.startswith('Mushroom spot /'):parent=self.find(parts,'Mushroom cap /',[n.split()[-2]]);top=True
    else:continue
    def project(p):return dict(p,positions=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i],-p['positions'][i+2],p['positions'][i+1])])
    if top:mark=project(mark);parent=project(parent)
    vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
     q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,(s,n));self.assertGreater(q[2]-max(hits),.0001,(s,n))
 def test_anatomical_owners_and_fixed_feet(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith('Foot /'):expected={0}
    elif n.startswith('Leg /'):
     side,j=map(int,n.split()[-2:]);expected={0,1,5+(0 if side<0 else 2)+j}
    elif n.startswith('Foreclaw /'):expected={3 if n.endswith('-1') else 4}
    elif n.startswith('Mushroom'):
     label=int(n.split()[-2] if n.startswith(('Mushroom spot /','Mushroom gill /')) else n.split()[-1]);expected={9 if s=='parasect' or label<0 else 10}
    elif n.startswith(('Body /','Carapace seam /')):expected={1}
    else:expected={2}
    self.assertEqual(owners,expected,(s,n))
 def test_occupied_leg_claw_face_and_fungal_roots_survive_motion(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives'];find=lambda prefix,ending=None:self.find(parts,prefix,ending)['part'];body=find('Body /');head=find('Head /');nose=find('Muzzle /');pairs=[(head,body),(nose,head)]
   for p in parts:
    n=p['part'];last=[n.split()[-1]]
    if n.startswith(('Eye /','Forehead bump /','Foreclaw /')):pairs.append((n,head))
    elif n.startswith(('Leg /','Mushroom stalk /')):pairs.append((n,body))
    elif n.startswith('Foot /'):pairs.append((n,find('Leg /',n.split()[-2:])))
    elif n.startswith('Mandible /'):pairs.append((n,nose))
    elif n.startswith('Mushroom cap /'):pairs.append((n,find('Mushroom stalk /',last)))
    elif n.startswith('Mushroom underside /'):pairs.append((n,find('Mushroom cap /',last)))
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed={p['part']:dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)}
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,a,b))
   detached=copy.deepcopy(self.find(parts,'Foreclaw /'));detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,self.find(parts,'Body /')))
 def test_floor_endpoints_winding_strain_and_independent_motion(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8)
     for i,n in enumerate(m.names):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,n));displacement=max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))
      if n.startswith('Foot /'):self.assertLess(displacement,1e-7)
      elif displacement>.002:moved.add(n.split(' /')[0])
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     if step in (0,8):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Head','Foreclaw','Leg','Mushroom cap'}<=moved,(s,moved))
 def test_source_cue_contract(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
