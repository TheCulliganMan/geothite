#!/usr/bin/env python3
"""Stored phoenix skin: coherent paint, occupied roots, planted hooks and open roar."""
import math,unittest
from pathlib import Path
import ho_oh_glb as author
import test_gengar_glb as skin
import test_ho_oh_sculpt as anatomy
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class HoOhSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'ho_oh.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.records={};cls.last=None;cls.cached={}
  for p,metadata in enumerate(cls.m.doc['extras']['source_part_ranges']):
   sources=metadata.get('source_parts',[{'part':cls.m.names[p],'order':metadata.get('source_order'),'vertex_start':0,'vertex_count':len(cls.m.vertices[p]),'index_start':0,'index_count':len(cls.m.indices[p])}])
   for r in sources:cls.records[r['part']]=(p,r)
 def source(self,name,mats):
  if mats is not self.last:self.last=mats;self.cached={}
  p,r=self.records[name]
  if p not in self.cached:self.cached[p]=self.m.posed(p,mats)
  a=r['vertex_start'];b=a+r['vertex_count'];i=r['index_start'];j=i+r['index_count']
  return {'part':name,'positions':[v for point in self.cached[p][a:b] for v in point],'indices':[k-a for k in self.m.indices[p][i:j]]}
 def find(self,prefix):return next(n for n in self.records if n.startswith(prefix))
 def test_canonical_roundtrip_source_ranges_and_allocation_limits(self):
  m=self.m;decoded=author.decode_ho_oh(self.blob)
  self.assertEqual(self.blob,author.export_ho_oh(decoded));self.assertEqual(tuple(p['part'] for p in decoded['primitives']),author.PARTS)
  self.assertEqual(len(decoded['primitives']),207)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(38,19747,25200,10))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  core={'primitives':[p for p in decoded['primitives'] if p['part'].startswith(('Body /','Neck /','Jaw /','Foot /'))]}
  self.assertEqual(len(core['primitives']),6);self.assertEqual(author.geometry_digest(core),'ff91b55266642639ceff0b02e907dd38191e49dcc3724b9060c18a9e0dc4d529')
  self.assertFalse((ROOT/'ho_oh.mesh.json').exists())
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_source_anatomy_has_independent_coherent_carriers(self):
  m=self.m
  for n,(p,r) in self.records.items():
   if n.startswith(('Leg /','Haunch /')):continue
   owner=0 if n.startswith(('Foot /','Toe /','Talon /')) else 2 if n.startswith(('Neck /','Collar')) else 6 if n.startswith(('Tail /','Tail color /')) else 7 if n.startswith('Jaw / open') else 3 if n.startswith(('Head /','Eye','Iris /','Pupil /','Jaw /','Mouth /','Crown /')) else 1
   if n.startswith(('Wing /','Flight /','Flight color /','Covert /')):
    side=n.split()[3] if n.startswith('Flight color /') else n.split()[-1 if n.startswith('Wing /') else -2]
    owner=4 if side=='-1' else 5
   self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[p][r['vertex_start']:r['vertex_start']+r['vertex_count']]),n)
 def test_neutral_endpoints_grounded_hooks_and_yawing_tail_height(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     points=m.posed(p,mats);self.assertGreaterEqual(min(q[1] for q in points),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(points,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if n.startswith(('Foot /','Toes /','Talons /')):
      for a,b in zip(points,m.vertices[p]):self.assertLess(math.dist(a,b),1e-7)
     if n.startswith(('Tail fan /','Tail tint /')):
      for a,b in zip(points,m.vertices[p]):self.assertLess(abs(a[1]-b[1]),1e-7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Wing fan','Wing ivory','Wing green','Tail fan','Jaw'}<=moved,moved)
 def test_winding_and_physical_edge_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri)
      self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
       self.assertTrue(.75<ratio<1.30,(clip,t,n,ratio))
 def test_posed_neck_wings_crest_jaws_tail_and_legs_keep_occupied_roots(self):
  body=self.find('Body /');neck=self.find('Neck /');head=self.find('Head /');pairs=[(neck,body),(head,neck)]
  pairs += [(n,head) for n in self.records if n.startswith(('Crown /','Jaw /','Mouth /'))]
  pairs += [(n,body) for n in self.records if n.startswith(('Wing /','Tail /','Haunch /'))]
  for side in ('-1','1'):
   leg=next(n for n in self.records if n.startswith('Leg /') and n.endswith(' '+side));thigh=next(n for n in self.records if n.startswith('Haunch /') and n.endswith(' '+side));foot=next(n for n in self.records if n.startswith('Foot /') and n.endswith(' '+side));pairs += [(leg,thigh),(foot,leg)]
  for clip in self.m.clips:
   for t in (0.,.18,.55):
    mats=self.m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.source(a,mats),self.source(b,mats)),(clip,t,a,b))
 def test_posed_roar_retains_real_empty_space(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t);carrier=mats[3];parts=[]
    for n in self.records:
     q=self.source(n,mats);points=[]
     for i in range(0,len(q['positions']),3):
      d=[q['positions'][i+k]-carrier[12+k] for k in range(3)]
      points.extend(sum(carrier[4*j+k]*d[k] for k in range(3)) for j in range(3))
     parts.append(dict(q,positions=points))
    probe=anatomy.HoOhTests();probe.ps=parts
    self.assertTrue(probe.jaw_gap(parts),(clip,t))
 def test_literal_neutral_keys_and_rotation_only_cue_contract(self):
  for clip,duration,count in author.CLIPS:
   self.assertEqual(author.pose_angles(clip,0.),[(0.,0.,0.)]*10);self.assertEqual(author.pose_angles(clip,1.),[(0.,0.,0.)]*10)
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle)
   self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
