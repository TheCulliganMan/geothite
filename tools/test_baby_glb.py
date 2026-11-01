#!/usr/bin/env python3
"""Baby skin contracts: planted feet, shared facial supports and coherent appendages."""
import math,unittest
from pathlib import Path
import baby_glb as author
import test_gengar_glb as skin
import test_donphan_sculpt as joins
from skin_glb_test_support import IDENTITY
from test_axolotl_sculpt import ray_intersections
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BabySkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def test_canonical_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_baby(s,author.decode_baby(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'pichu':(24,9462,9104,10),'cleffa':(17,14716,24376,9),'igglybuff':(19,16964,28684,6)}[s])
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_pichu_original_closed_anatomy_and_shading(self):
  models={'pichu':author.decode_baby(self.blobs['pichu'],'pichu')};assert_closed_models(self,models);assert_model_shading(self,models)
  names=self.models['pichu'].names
  for prefix,count in [('Head /',1),('Ear /',2),('Ear inset /',2),('Collar /',1),('Cheek /',2),('Tail /',2),('Foot /',2)]:self.assertEqual(sum(n.startswith(prefix) for n in names),count)
 def test_pichu_face_and_ear_insets_fit_actual_stored_supports(self):
  m=author.decode_baby(self.blobs['pichu'],'pichu');parts=m['primitives']
  def find(prefix):return next(q for q in parts if q['part'].startswith(prefix))
  for p in parts:
   n=p['part'];parent=None
   if n.startswith(('Eye /','Cheek /','Nose /','Mouth /')):parent=find('Head /')
   elif n.startswith('Eye glint /'):parent=find('Eye / fitted large black mouse oval '+n.split()[-1])
   elif n.startswith('Ear inset /'):parent=find('Ear / broad black pointed mouse ear '+n.split()[-1])
   elif n.startswith('Tongue /'):parent=find('Mouth /')
   if parent:
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(n,x,y));self.assertLess(abs(z-max(hits)),.008,(n,x,y))
 def test_fitted_face_and_appendage_owners(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=0 if n.startswith('Foot /') else 2 if n.startswith(('Head /','Eye /','Eye glint /','Eye border /','Iris /','Pupil /','Cheek /','Nose /','Mouth /','Tongue /','Forehead /')) else author.BANKS[s]['tuft'] if n.startswith(('Tuft /','Tuft crease /')) else author.BANKS[s]['crown'] if n.startswith('Crown /') else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_floor_fixed_feet_neutral_endpoints_and_visible_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,n))
      if n.startswith('Foot /'):self.assertEqual(posed,m.vertices[p])
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Body','Arm','Eye border' if s=='igglybuff' else 'Eye'}<=moved,(s,moved))
   self.assertTrue(({'Ear','Tail'} if s=='pichu' else {'Ear','Tail','Tuft'} if s=='cleffa' else {'Crown'})<=moved,(s,moved))
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
 def test_actual_posed_limb_crown_ear_and_tail_roots(self):
  for s,m in self.models.items():
   pairs=[]
   def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
   body=find('Body /');head=find('Head /') if s=='pichu' else body
   for p,n in enumerate(m.names):
    if n.startswith(('Arm /','Leg /')):pairs.append((p,body))
    elif n.startswith('Foot /') and s!='pichu':pairs.append((p,body))
    elif n.startswith('Ear /'):pairs.append((p,head))
    elif n.startswith('Tuft /'):pairs.append((p,head))
   if s=='pichu':pairs += [(head,body),(find('Tail / thin'),body),(find('Tail / broad'),find('Tail / thin'))]+[(p,find('Leg / supporting yellow shin '+n.split()[-1])) for p,n in enumerate(m.names) if n.startswith('Foot /')]
   if s=='cleffa':pairs.append((find('Tail /'),body))
   if s=='igglybuff':pairs += [(find('Crown / rounded rooted balloon tuft lobe '+str(j)),body) for j in (0,1)]+[(find('Crown / rounded rooted balloon tuft lobe 2'),find('Crown / rounded rooted balloon tuft lobe '+str(j))) for j in (0,1)]
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t)
     for a,b in pairs:
      def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
      self.assertTrue(joins.DonphanTests.overlap(self,part(a),part(b)),(s,clip,t,m.names[a],m.names[b]))
 def test_noncanonical_data_rejected(self):
  for s,blob in self.blobs.items():
   with self.assertRaises(ValueError):author.decode_baby(blob[:-1],s)
if __name__=='__main__':unittest.main()
