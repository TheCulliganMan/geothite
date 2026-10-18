#!/usr/bin/env python3
"""Stored gas skins: coherent cratered faces, joined orbs and phased rooted vapor."""
import copy,math,unittest
from pathlib import Path
import gas_pair_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,indexed_ray_intersections as rays
import test_gas_pair_sculpt as anatomy_checks
from test_steelix_sculpt import occupied
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class GasSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,m,prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
 def test_canonical_limits_hierarchy_weights_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_gas(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());count=12 if s=='koffing' else 14;self.assertEqual(len(m.joints),count)
   parents=[None,0]+([1,1] if s=='weezing' else []);base=len(parents)
   for cloud in range(2):parents += [0]+[base+cloud*5]*4
   self.assertEqual(m.parents[:count],parents)
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(1,count)))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_faces_fangs_and_occupied_cloud_roots(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w};expected=None
    if n.startswith('Gas cloud /'):expected={author.gas_base(s)+5*int(n.split()[-1])}
    elif n.startswith('Gas lobe /'):
     cloud,k=map(int,n.split()[-2:]);core=author.gas_base(s)+5*cloud;self.assertEqual(owners,{core,core+1+k});continue
    elif n.startswith('Main stalk /'):expected={1,2}
    elif n.startswith('Rear stalk /'):expected={1,3}
    elif n.startswith('Rear node /'):expected={3}
    elif n.startswith('Small body /') or 'small Weezing' in n:expected={2}
    else:expected={1}
    self.assertEqual(owners,expected,(s,n))
 def test_linked_orbs_canines_and_cloud_lobes_have_real_roots_through_clips(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];find=lambda n:self.find(m,n);pairs=[]
   for i,n in enumerate(m.names):
    if n.startswith('Gas lobe /'):pairs.append((find('Gas cloud / floating pale '+s+' cloud core '+n.split()[-2]),i))
    elif n.startswith('Fang /'):pairs.append((find('Small body /' if 'small Weezing' in n else 'Body /'),i))
   if s=='weezing':pairs += [(find('Body /'),find('Main stalk /')),(find('Main stalk /'),find('Small body /')),(find('Body /'),find('Rear stalk /')),(find('Rear stalk /'),find('Rear node /'))]
   for clip in m.clips:
    for t in (0.,.31,.67):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[pairs[0][1]]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[pairs[0][0]]))
 def test_floor_endpoints_winding_and_bounded_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_independent_orbs_clouds_and_lobes_with_distinct_idle_phases(self):
  for s,m in self.models.items():
   for joint in range(1,len(m.joints)):
    distance=0.
    for t in (.13,.31,.57,.83):
     isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=m.rotations('idle',t)[joint];mats=m.matrices(rotations=isolate)
     distance=max(distance,max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))))
    self.assertGreater(distance,.0005,(s,joint))
   indices=[author.gas_base(s)+cloud*5+1+k for cloud in range(2) for k in range(4)];series=[tuple(round(v,6) for t in (.13,.31,.57,.83) for v in m.rotations('idle',t)[j]) for j in indices];self.assertEqual(len(set(series)),8)
 def test_fitted_expression_and_emblem_clear_support_during_motion(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];helper=anatomy_checks.GasTests();helper.ps=ps;lookup={n:i for i,n in enumerate(m.names)};pairs=[(lookup[p['part']],lookup[parent['part']]) for p,parent in helper.pairs()]
   for clip in m.clips:
    mats=m.matrices(clip,.31);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for mark,parent in pairs:
     vs=[posed[mark]['positions'][i:i+3] for i in range(0,len(posed[mark]['positions']),3)]
     for k in range(0,len(m.indices[mark]),3):
      ids=m.indices[mark][k:k+3];a,b,c=[m.vertices[mark][j] for j in ids]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      x,y,z=[sum(vs[j][axis] for j in ids)/3 for axis in range(3)];hits=rays(posed[parent],x,y);self.assertTrue(hits,(s,clip,m.names[mark]));self.assertGreater(z-max(hits),.0001,(s,clip,m.names[mark]))
 def test_thirteen_craters_per_orb_stay_empty_when_posed(self):
  from gas_pair_sculpt import VENTS,unit
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];helper=anatomy_checks.GasTests();bodies,convert,_=helper.fixture(s)
   for clip in m.clips:
    mats=m.matrices(clip,.31)
    for prefix,c,r in bodies:
     i=self.find(m,prefix);owner=2 if prefix.startswith('Small') else 1;posed=dict(ps[i],positions=sum(m.posed(i,mats),[]))
     for d in VENTS:
      point=convert(tuple(c[k]+r[k]*unit(d)[k]*.98 for k in range(3)));self.assertFalse(occupied(posed,skin.transform(mats[owner],point)),(s,clip,prefix,d))
 def test_strict_readers_reject_mutations(self):
  for s,blob in self.blobs.items():
   changed=bytearray(blob);changed[-8]^=1
   with self.assertRaises(ValueError):author.decode_gas(bytes(changed),s)
   changed=copy.deepcopy(author.authoring_model(s));changed['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_gas(s,changed)
if __name__=='__main__':unittest.main()
