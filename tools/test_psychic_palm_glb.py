"""Decoded psychic eggs/palm: supported identities, intact cavities and bending rooted canopy."""
import copy,math,os,struct,unittest
from pathlib import Path
import psychic_palm_glb as author
import test_exeggcute_sculpt as eggs
import test_exeggutor_sculpt as palm
from test_gengar_glb import Model,winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import indexed_ray_intersections as rays,ray_intersections as unindexed_rays
ROOT=Path(os.environ.get('PSYCHIC_PALM_SKIN_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def inverse_points(p,frame):
 p=dict(p);p['positions']=[sum(frame[4*i+j]*(q[j]-frame[12+j]) for j in range(3)) for q in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for i in range(3)];return p
def overlap(a,b):
 # Same occupied-interval test, grid and tolerance as the original anatomy
 # oracle; spatial indexing only excludes triangles outside each ray column.
 xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
 if xh<=xl or yh<=yl:return False
 for i in range(1,19):
  for j in range(1,23):
   x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23;aa=sorted(rays(a,x,y));bb=sorted(rays(b,x,y))
   if any(min(ah,bh)-max(al,bl)>.00025 for al,ah in zip(aa[::2],aa[1::2]) for bl,bh in zip(bb[::2],bb[1::2])):return True
 return False
def assert_roots(test,s,parts):
 def find(prefix,suffix=None):return next(p for p in parts if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 if s=='exeggcute':
  for j in (3,4,5):test.assertTrue(any(overlap(find('Egg /',j),find('Egg /',i)) for i in (0,1,2)),('raised egg',j))
 else:
  trunk=find('Trunk /')
  for p in parts:
   n=p['part']
   if n.startswith(('Head /','Leg /','Frond /')):test.assertTrue(overlap(trunk,p),n)
   elif n.startswith('Claw /'):test.assertTrue(overlap(find('Foot /',n.split()[-2]),p),n)
  for sign in (-1,1):test.assertTrue(overlap(find('Leg /',sign),find('Foot /',sign)),sign)
class PsychicPalmSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.runtime={s:Model(b) for s,b in cls.blobs.items()};cls.models={s:logical_model(m) for s,m in cls.runtime.items()}
 def posed(self,m,mats):return dict(name=m.doc['extras']['source_name'],primitives=[dict(part=n,positions=[v for p in m.posed(i,mats) for v in p],normals=[v for p in m.posed(i,mats,True) for v in p],indices=m.indices[i],base_color=m.doc['materials'][m.parts[i]['material']]['pbrMetallicRoughness']['baseColorFactor']) for i,n in enumerate(m.names)])
 def test_canonical_logical_geometry_ranges_names_and_existing_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_psychic_palm(s));self.assertEqual(author.geometry_digest(m.neutral_model()),author.REST_DIGESTS[s]);self.assertEqual(author.geometry_digest(author.decode_psychic_palm(self.blobs[s],s)),author.REST_DIGESTS[s]);self.assertEqual(m.names,list(author.PARTS[s]));self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(i)//3 for i in m.indices)),(46,19868,35472) if s=='exeggcute' else (70,19278,30396));self.assertLessEqual(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(self.runtime[s].parts),64);self.assertLessEqual(len(m.doc['accessors']),256)
 def test_original_neutral_bounds_and_all_source_colors_preserved(self):
  expected={'exeggcute':((-.9152057,.8299585),(0.,1.22),(-.3704536,.5838285)),'exeggutor':((-.5299351,.5301029),(0.,1.22),(-.4289918,.2375828))}
  for s,m in self.models.items():
   for axis,(lo,hi) in enumerate(expected[s]):self.assertEqual((min(p[axis] for vs in m.vertices for p in vs),max(p[axis] for vs in m.vertices for p in vs)),struct.unpack('<2f',struct.pack('<2f',lo,hi)))
   old=(author.eggs if s=='exeggcute' else author.palm).sculpture()
   for p,q in zip(old['primitives'],m.neutral_model()['primitives']):self.assertEqual(p['part'],q['part']);self.assertEqual(author.packed(p['base_color'],5126),author.packed(q['base_color'],5126))
 def test_untouched_eggs_yolk_trunk_heads_feet_claws_and_frond_geometry(self):
  for s,m in self.models.items():
   old=(author.eggs if s=='exeggcute' else author.palm).sculpture()
   for p,q in zip(old['primitives'],m.neutral_model()['primitives']):
    if not p['part'].startswith(('Egg /','Yolk /') if s=='exeggcute' else ('Trunk /','Leg /','Foot /','Claw /','Head /','Frond /')):continue
    for key,c in (('positions',5126),('normals',5126),('indices',5125)):self.assertEqual(author.packed(p[key],c),author.packed(q[key],c),(s,p['part'],key))
 def test_hierarchy_inverse_binds_and_every_control_used(self):
  for s,m in self.models.items():
   count=7 if s=='exeggcute' else 30;self.assertEqual(len(m.joints),count)
   expected=[None,0,0,0,1,2,3] if s=='exeggcute' else [None,0,1,2,2,2,0,0]+[p for j in range(11) for p in (2,8+2*j)]
   self.assertEqual(m.parents[:count],expected);self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for js,ww in zip(ids,ws) for j,w in zip(js,ww) if w},set(range(count)))
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
 def test_coherent_expressions_intact_yolk_and_complete_fixed_palm_feet(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    j=None
    if s=='exeggcute' and not n.startswith('Egg /'):j=1+author.egg_owner(n)
    elif s=='exeggutor':
     if n.startswith(('Foot /','Sole /','Claw /')):j=0
     elif n.startswith(('Head /','Eye white /','Pupil /','Lid /','Mouth /','Tongue /','Slack tongue /','Fang /','Smile lining /')):j=3+author.palm_head(n)
    if j is not None:self.assertTrue(all(ids==[j,0,0,0] and ws==[1.,0.,0.,0.] for ids,ws in zip(m.ids[i],m.weights[i])),(s,n))
 def test_each_control_moves_and_independent_faces_and_fronds_do_not_drag_neighbors(self):
  for s,m in self.models.items():
   for j in range(1,len(m.joints)):
    qs=[[0.,0.,0.,1.] for _ in m.doc['nodes']];qs[j]=m.rotations('attack',.30)[j];mats=m.matrices(rotations=qs);movement=[max(math.dist(a,b) for a,b in zip(vs,m.posed(i,mats))) for i,vs in enumerate(m.vertices)];self.assertGreater(max(movement),.0001,(s,j))
    for i,n in enumerate(m.names):
     if s=='exeggutor' and n.startswith(('Foot /','Sole /','Claw /')):self.assertLess(movement[i],1e-7,(s,j,n))
     if s=='exeggutor' and j in (3,4,5) and n.startswith('Head /') and int(n.split()[-1])!=j-3:self.assertLess(movement[i],1e-7,(s,j,n))
     if s=='exeggutor' and j>=8 and n.startswith(('Frond /','Midrib /')) and int(n.split()[-1])!=(j-8)//2:self.assertLess(movement[i],1e-7,(s,j,n))
 def test_ground_contact_positive_winding_and_bounded_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     mats=m.matrices(clip,t)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,t,m.names[i]))
      if s=='exeggutor' and m.names[i].startswith(('Foot /','Sole /','Claw /')):self.assertEqual(posed,vs)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,m.names[i],ratio))
 def test_actual_pile_yolk_coconut_limb_claw_and_frond_roots_survive_poses(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.30,.55,.80):
     model=self.posed(m,m.matrices(clip,t));h=eggs.EggTests() if s=='exeggcute' else palm.PalmTests();h.m=model;h.ps=model['primitives']
     assert_roots(self,s,h.ps)
 def test_indexed_ray_hits_match_the_original_triangle_oracle(self):
  for s,m in self.models.items():
   for p in m.neutral_model()['primitives']:
    if not p['part'].startswith(('Egg /','Yolk /','Trunk /','Head /','Frond /','Foot /')):continue
    xl,xh=min(p['positions'][::3]),max(p['positions'][::3]);yl,yh=min(p['positions'][1::3]),max(p['positions'][1::3])
    for i in range(3):
     for j in range(4):
      x=xl+(xh-xl)*(i+.371)/3;y=yl+(yh-yl)*(j+.619)/4;self.assertEqual(sorted(rays(p,x,y)),sorted(unindexed_rays(p,x,y)),(s,p['part'],x,y))
 def test_all_five_egg_expressions_remain_visible_during_source_cues(self):
  m=self.models['exeggcute']
  for clip in m.clips:
   for t in (.18,.55):
    model=self.posed(m,m.matrices(clip,t));shells=[p for p in model['primitives'] if p['part'].startswith('Egg /')]
    for i,n in enumerate(m.names):
     if not n.startswith('Mouth /'):continue
     ids=m.indices[i];source=m.vertices[i];posed=model['primitives'][i]['positions'];fronts=[]
     for k in range(0,len(ids),3):
      a,b,c=[source[v] for v in ids[k:k+3]]
      if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])>1e-10:fronts.append(tuple(sum(posed[3*v+axis] for v in ids[k:k+3])/3 for axis in range(3)))
     samples=fronts[::max(1,len(fronts)//25)];visible=0
     for x,y,z in samples:
      hits=[h for p in shells for h in rays(p,x,y)];self.assertTrue(hits);visible+=z-max(hits)>.0001
     self.assertGreater(visible,len(samples)*.70,(clip,t,n,visible,len(samples)))
 def test_broken_egg_remains_hollow_with_a_rooted_yolk(self):
  m=self.models['exeggcute']
  for clip in m.clips:
   for t in (.18,.30,.55,.80):
    mats=m.matrices(clip,t);model=self.posed(m,mats);h=eggs.EggTests();h.m=model;h.ps=[inverse_points(p,mats[4]) for p in model['primitives']];h.test_broken_shell_is_hollow_and_yolk_is_rooted()
 def test_fitted_faces_leaf_midribs_and_trunk_bands_follow_actual_posed_parents(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.55):
     mats=m.matrices(clip,t);model=self.posed(m,mats);parts=[]
     for p in model['primitives']:
      n=p['part'];j=0
      if s=='exeggcute':j=1+author.egg_owner(n)
      elif n.startswith(('Frond /','Midrib /')):j=8+2*int(n.split()[-1])
      elif n.startswith(('Trunk /','Trunk crease /')):j=1
      elif n.startswith(('Head /','Eye white /','Pupil /','Lid /','Mouth /','Tongue /','Slack tongue /','Fang /','Smile lining /')):j=3+author.palm_head(n)
      parts.append(inverse_points(p,mats[j]))
     h=eggs.EggTests() if s=='exeggcute' else palm.PalmTests();h.m=model;h.ps=parts
     if s=='exeggcute':h.test_markings_and_lines_follow_actual_stored_parents();h.test_front_marking_triangles_clear_stored_support()
     else:h.test_marking_vertices_follow_actual_parents();h.test_front_marking_triangles_clear_stored_support();h.test_leaf_midribs_and_trunk_segments_are_supported()
 def test_detached_egg_and_frond_negative_controls_are_rejected(self):
  for s,m in self.models.items():
   model=self.posed(m,m.matrices('attack',.30));h=eggs.EggTests() if s=='exeggcute' else palm.PalmTests();h.m=model;h.ps=model['primitives'];p=h.find('Egg /' if s=='exeggcute' else 'Frond /',5 if s=='exeggcute' else 0);p['positions']=[v+(3. if i%3==0 else 0.) for i,v in enumerate(p['positions'])]
   with self.assertRaises(AssertionError):
    assert_roots(self,s,h.ps)
 def test_neutral_endpoints_rotation_only_cues_and_mutation_rejection(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     for mat in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   for animation in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
   bad=copy.deepcopy(m.neutral_model());bad['name']=s;bad['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_psychic_palm(s,bad)
   for blob in (b'bad',self.blobs[s][:-4],self.blobs[s]+b' '):
    with self.assertRaises(ValueError):author.decode_psychic_palm(blob,s)
if __name__=='__main__':unittest.main()
