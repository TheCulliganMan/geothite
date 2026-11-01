#!/usr/bin/env python3
"""Decoded seal supports, coherent faces and rooted tail/flipper articulation."""
import copy,math,os,unittest
from pathlib import Path
import seal_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,crossed_faces,indexed_ray_intersections as rays,assert_visible_surface_stroke
ROOT=Path(os.environ.get('CRYSTAL_SEAL_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class SealSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def find(self,m,prefix,ending=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (ending is None or n.split()[-len(ending):]==ending))
 def test_canonical_limits_hierarchy_weights_and_identity_binds(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_seal(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());count=10 if s=='seel' else 9;self.assertEqual(len(m.joints),count)
   self.assertEqual(m.parents[:count],[None,0,1,1,0,0,3,6,6,2] if s=='seel' else [None,0,1,1,1,1,3,6,6])
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual({j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w},set(range(count)))
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.);self.assertTrue(all(j==0 for j,w in zip(js,ws) if w==0.))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_anatomical_owners_keep_faces_folds_and_fans_coherent(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith('Body /'):self.assertTrue(owners<={0,1,2,3,6});continue
    if n.startswith('Flipper /'):expected={4 if n.endswith('-1') else 5}
    elif n.startswith('Flipper fold /'):expected={4 if n.split()[-2]=='-1' else 5}
    elif n.startswith('Tail fluke /'):expected={7 if n.endswith('-1') else 8}
    elif s=='seel' and n.startswith(('Tongue /','Tongue crease /')):expected={9}
    else:expected={2}
    self.assertEqual(owners,expected,(s,n))
 def test_flippers_flukes_heads_horns_muzzles_and_canines_have_occupied_roots(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];find=lambda prefix,ending=None:self.find(m,prefix,ending);body=find('Body /');head=find('Head /') if s=='seel' else body;muzzle=find('Muzzle /');pairs=[(head,muzzle),(head,find('Horn /'))]
   if s=='seel':pairs += [(body,head),(muzzle,find('Mouth /')),(find('Mouth /'),find('Tongue /')),(muzzle,find('Nose /'))]
   for i,n in enumerate(m.names):
    if n.startswith(('Flipper /','Tail fluke /')):pairs.append((body,i))
    elif n.startswith(('Tusk /','Fang /')):pairs.append((muzzle,i))
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(ps[find('Flipper /')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,ps[body]))
 def test_floor_endpoints_winding_strain_and_local_articulation(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8);floor=float('inf')
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);floor=min(floor,min(p[1] for p in points));self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      for v,p in enumerate(m.vertices[i]):
       if p[1]<1e-7:self.assertAlmostEqual(points[v][1],0.,places=7)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     self.assertLess(abs(floor),1e-7)
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   rotations=m.rotations('attack',.31)
   for joint in range(1,len(m.joints)):
    isolate=[[0.,0.,0.,1.] for _ in m.doc['nodes']];isolate[joint]=rotations[joint];mats=m.matrices(rotations=isolate)
    self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,mats))),.0005,(s,joint))
 def test_dewgong_curved_body_does_not_cross_itself_in_motion(self):
  m=self.models['dewgong'];i=self.find(m,'Body /');body=m.neutral_model()['primitives'][i]
  for clip in m.clips:
   posed=dict(body,positions=sum(m.posed(i,m.matrices(clip,.31)),[]));self.assertIsNone(crossed_faces(posed),clip)
 def test_posed_fitted_gaze_and_reflections_clear_actual_support(self):
  for s,m in self.models.items():
   ps=m.neutral_model()['primitives'];find=lambda prefix,ending=None:self.find(m,prefix,ending);pairs=[]
   for i,n in enumerate(m.names):
    if n.startswith('Eye /'):parent=find('Head /') if s=='seel' else find('Body /')
    elif n.startswith(('Eye sheen /','Eye glint /')):parent=find('Eye /',[n.split()[-1]])
    elif n.startswith('Nose glint /'):parent=find('Nose /')
    else:continue
    pairs.append((i,parent))
   for clip in m.clips:
    mats=m.matrices(clip,.31);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(ps)]
    for mark,parent in pairs:
     p=posed[mark];vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
     for k in range(0,len(p['indices']),3):
      ids=p['indices'][k:k+3];a,b,c=[m.vertices[mark][j] for j in ids]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      a,b,c=[vs[j] for j in ids]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
      x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=rays(posed[parent],x,y);self.assertTrue(h,(s,m.names[mark]));self.assertGreater(z-max(h),.0001,(s,clip,m.names[mark]))
 def test_source_cues_own_attack_and_hit_clocks(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})

 def test_dewgong_smile_has_occupied_muzzle_roots_across_its_width_through_clips(self):
  m=self.models['dewgong'];mouth=self.find(m,'Mouth /');muzzle=self.find(m,'Muzzle /')
  for clip in m.clips:
   for t in (0.,.31,.67):
    mats=m.matrices(clip,t)
    a={'positions':sum(m.posed(mouth,mats),[]),'indices':m.indices[mouth]}
    b={'positions':sum(m.posed(muzzle,mats),[]),'indices':m.indices[muzzle]}
    self.assertTrue(parts_overlap(a,b),(clip,t,'detached smile'))
    xl,xh=min(a['positions'][::3]),max(a['positions'][::3])
    yl,yh=min(a['positions'][1::3]),max(a['positions'][1::3])
    for section in range(1,16):
     x=xl+(xh-xl)*section/16;root=front=-float('inf')
     for row in range(1,80):
      y=yl+(yh-yl)*row/80
      aa=sorted({round(z,10) for z in rays(a,x,y)})
      bb=sorted({round(z,10) for z in rays(b,x,y)})
      for al,ah in zip(aa[::2],aa[1::2]):
       for bl,bh in zip(bb[::2],bb[1::2]):
        root=max(root,min(ah,bh)-max(al,bl));front=max(front,ah-bh)
     self.assertGreater(root,.00015,(clip,t,section,'unrooted section'))
     self.assertGreater(front,.0003,(clip,t,section,'buried smile'))

 assert_surface_stroke=assert_visible_surface_stroke

 def surface_strokes(self, species, model, matrices):
  parts=model.neutral_model()['primitives']
  posed=[dict(p,positions=sum(model.posed(i,matrices),[])) for i,p in enumerate(parts)]
  for i,name in enumerate(model.names):
   if name.startswith('Brow /'):parent=self.find(model,'Head /')
   elif name.startswith('Tongue crease /'):parent=self.find(model,'Tongue /')
   elif name.startswith('Flipper fold /'):parent=self.find(model,'Flipper /',[name.split()[-2]])
   else:continue
   mark,support=posed[i],posed[parent]
   if species=='seel' and name.startswith('Flipper fold /'):
    def top(p):return dict(p,positions=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,-z,y)])
    mark,support=top(mark),top(support)
   yield name,mark,support

 def test_folds_brows_and_tongue_crease_keep_continuous_visible_occupied_roots(self):
  for species,model in self.models.items():
   for clip in model.clips:
    for t in (0.,.31,.67):
     strokes=list(self.surface_strokes(species,model,model.matrices(clip,t)))
     self.assertEqual(len(strokes),7 if species=='seel' else 4)
     for name,mark,parent in strokes:self.assert_surface_stroke(mark,parent,(species,clip,t,name))

 def test_surface_stroke_oracle_rejects_floating_and_buried_details(self):
  for species,model in self.models.items():
   for name,mark,parent in self.surface_strokes(species,model,model.matrices()):
    for offset in (-.01,.01):
     broken=dict(mark,positions=[v+(offset if i%3==2 else 0.) for i,v in enumerate(mark['positions'])])
     with self.assertRaises(AssertionError):self.assert_surface_stroke(broken,parent,(species,name,offset))
if __name__=='__main__':unittest.main()
