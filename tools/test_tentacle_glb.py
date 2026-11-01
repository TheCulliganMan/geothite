"""Decoded jelly anatomy, mesh-supported eyes/lobes and grounded tendril controls."""
import copy,math,os,unittest
from pathlib import Path
import tentacle_glb as author
from tentacle_sculpt import normalized,paths
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
from skin_glb_test_support import IDENTITY
ROOT=Path(os.environ.get('CRYSTAL_TENTACLE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class TentacleSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:Model(b) for s,b in cls.blobs.items()};cls.neutral={s:author.decode_tentacle(b,s) for s,b in cls.blobs.items()}
 def find(self,s,prefix):return next(p for p in self.neutral[s]['primitives'] if p['part'].startswith(prefix))
 def test_canonical_geometry_and_existing_limits(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_tentacle(s));self.assertEqual(author.geometry_digest(self.neutral[s]),author.REST_DIGESTS[s]);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(len(p)//3 for p in m.indices),40000);self.assertLessEqual(len(m.parts),64);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertAlmostEqual(min(v[1] for vs in m.vertices for v in vs),0.,places=7);self.assertAlmostEqual(max(v[1] for vs in m.vertices for v in vs),1.13,places=6)
 def test_closed_connected_outward_parts(self):assert_closed_models(self,self.neutral)
 def test_unit_normals_follow_actual_faces(self):assert_model_shading(self,self.neutral)
 def test_different_bell_widths_tendril_counts_and_side_feelers(self):
  for s,m in self.models.items():
   self.assertEqual(sum(n.startswith('Tentacle /') for n in m.names),2 if s=='tentacool' else 10);self.assertEqual(sum(n.startswith('Arm /') for n in m.names),0 if s=='tentacool' else 2)
   bell=self.find(s,'Bell /');width=max(bell['positions'][::3])-min(bell['positions'][::3]);height=max(bell['positions'][1::3])-min(bell['positions'][1::3]);self.assertGreater(width/height,1.1 if s=='tentacool' else 1.6)
  cool=self.find('tentacool','Bell /');cruel=self.find('tentacruel','Bell /');self.assertGreater(max(cruel['positions'][::3]),max(cool['positions'][::3])*1.4)
 def test_actual_tendril_roots_are_inside_the_mantle_volume(self):
  for s in author.SPECIES:
   mantle=self.find(s,'Mantle /')
   for path in paths(s):
    x,y,z=normalized(s,path[0]);hits=ray_intersections(mantle,x,y);self.assertGreaterEqual(len(hits),2);self.assertLess(min(hits),z);self.assertGreater(max(hits),z)
 def test_large_red_lobes_have_actual_bell_volume_roots(self):
  for s in author.SPECIES:
   bell=self.find(s,'Bell /')
   for p in self.neutral[s]['primitives']:
    if not p['part'].startswith('Jewel / large'):continue
    x=(min(p['positions'][::3])+max(p['positions'][::3]))/2;y=(min(p['positions'][1::3])+max(p['positions'][1::3]))/2
    parent=ray_intersections(bell,x,y);gem=ray_intersections(p,x,y);self.assertTrue(parent and gem);self.assertLess(min(gem),max(parent));self.assertGreater(max(gem),max(parent))
 def test_fitted_gaze_and_highlights_clear_their_actual_parents(self):
  for s,model in self.neutral.items():
   for p in model['primitives']:
    n=p['part'];last=n.split()[-1]
    if n.startswith('Eye /'):parent=self.find(s,'Mantle /')
    elif n.startswith('Pupil /'):parent=next(q for q in model['primitives'] if q['part'].startswith('Eye /') and q['part'].endswith(' '+last))
    elif n.startswith('Jewel glint /'):parent=next(q for q in model['primitives'] if q['part'].startswith('Jewel / large') and q['part'].endswith(' '+last))
    else:continue
    vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)];checked=0
    for i in range(0,len(p['indices']),3):
     a,b,c=[vs[j] for j in p['indices'][i:i+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     mid=[sum(q[k] for q in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits,(s,n));self.assertGreater(mid[2],max(hits),(s,n));checked+=1
    self.assertGreater(checked,8)
 def test_gaze_is_not_hidden_by_the_bell_at_production_view_angles(self):
  def projected(p,pitch,shift=0.):
   q=dict(p);c=math.cos(pitch);sn=math.sin(pitch);q['positions']=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,c*y-sn*(z+shift),sn*y+c*(z+shift))];return q
  def fraction(bell,eyes,pitch,shift=0.):
   bell=projected(bell,pitch);visible=total=0
   for eye in eyes:
    p=projected(eye,pitch,shift);vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)];original=[eye['positions'][i:i+3] for i in range(0,len(eye['positions']),3)]
    for i in range(0,len(p['indices']),3):
     # Test the gaze surface, excluding the closed patch's buried root walls.
     aa,bb,cc=[original[j] for j in p['indices'][i:i+3]]
     if (bb[0]-aa[0])*(cc[1]-aa[1])-(bb[1]-aa[1])*(cc[0]-aa[0])<=1e-10:continue
     a,b,c=[vs[j] for j in p['indices'][i:i+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     mid=[sum(v[k] for v in (a,b,c))/3 for k in range(3)];hits=ray_intersections(bell,*mid[:2]);visible+=not hits or mid[2]>max(hits);total+=1
   self.assertGreater(total,50);return visible/total
  for s,m in self.neutral.items():
   bell=self.find(s,'Bell /');eyes=[p for p in m['primitives'] if p['part'].startswith('Eye /')]
   for pitch in (.35,.5,.65):
    self.assertEqual(fraction(bell,eyes,pitch),1.,(s,pitch))
    self.assertLess(fraction(bell,eyes,pitch,-.08),.95,'negative control must reproduce hidden upper gaze')
 def test_visible_pupil_area_clears_bell_mantle_and_eye_surfaces(self):
  def project(p,angle):
   c=math.cos(angle);sn=math.sin(angle);q=dict(p);q['positions']=[v for x,y,z in zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]) for v in (x,c*y-sn*z,sn*y+c*z)];return q
  for s,model in self.neutral.items():
   for angle in (.35,.65,.9):
    parts=[project(p,angle) for p in model['primitives']];visible=total=0.
    for p in parts:
     if not p['part'].startswith('Pupil /'):continue
     parents=[q for q in parts if q['part'].startswith(('Bell /','Mantle /')) or q['part'].startswith('Eye /') and q['part'].split()[-1]==p['part'].split()[-1]];vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
     for i in range(0,len(p['indices']),3):
      a,b,c=[vs[j] for j in p['indices'][i:i+3]];area=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
      if area<=1e-10:continue
      mid=[sum(v[k] for v in (a,b,c))/3 for k in range(3)];hits=[z for parent in parents for z in ray_intersections(parent,*mid[:2])];total+=area
      if not hits or mid[2]>max(hits):visible+=area
    self.assertGreater(total,.0001);self.assertAlmostEqual(visible/total,1.,places=6,msg=(s,angle))
 def test_local_normal_pack_rgb555_palette(self):
  for s,m in self.models.items():
   for p,n in zip(m.parts,m.names):
    if n.startswith(('Bell /','Tentacle /','Arm /')):rgb=(11,20,31)
    elif n.startswith('Jewel / large'):rgb=(30,6,10) if s=='tentacool' else (26,2,2)
    else:continue
    values=[((v<<3)|(v>>2))/255 for v in rgb];linear=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in values];color=m.doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
    for a,b in zip(color,linear):self.assertAlmostEqual(a,b,places=7,msg=(s,n))
 def test_hierarchy_inverse_binds_and_coherent_faces(self):
  for s,m in self.models.items():
   self.assertEqual(len(m.joints),6 if s=='tentacool' else 24);self.assertEqual(m.parents[:2],[None,0])
   for j in range(len(paths(s))):self.assertEqual(m.parents[2+2*j:4+2*j],[1,2+2*j])
   for mat in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   for i,n in enumerate(m.names):
    if n.startswith(('Tentacle /','Arm /')):continue
    self.assertTrue(all(js==[1,0,0,0] for js in m.ids[i]),n)
 def test_all_controls_move_without_dragging_neighbor_tendrils(self):
  for s,m in self.models.items():
   for j in range(2,len(m.joints)):
    rotations=[[0.,0.,0.,1.] for _ in m.doc['nodes']];rotations[j]=m.rotations('attack',.28)[j];mats=m.matrices(rotations=rotations);moving=[]
    for i,n in enumerate(m.names):
     displacement=max(math.dist(a,b) for a,b in zip(m.vertices[i],m.posed(i,mats)))
     own=(n.startswith('Tentacle /') and j in (2+2*int(n.split()[-1]),3+2*int(n.split()[-1]))) or (n.startswith('Arm /') and j==len(m.joints)-2+(int(n.split()[-1])==1))
     if own:moving.append(displacement)
     else:self.assertLess(displacement,1e-7,(s,j,n))
    self.assertGreater(max(moving),.0003,(s,j))
 def test_pinned_tips_winding_and_strain_through_every_clip(self):
  for s,m in self.models.items():
   floor=[(i,j) for i,vs in enumerate(m.vertices) for j,v in enumerate(vs) if v[1]<1e-7];self.assertTrue(floor)
   for clip in m.clips:
    for t in (0.,.18,.28,.52,.78,1.):
     mats=m.matrices(clip,t)
     for i,vs in enumerate(m.vertices):
      posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,t,m.names[i]))
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,m.names[i],k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(vs[a],vs[b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,m.names[i],ratio))
     for i,j in floor:self.assertLess(math.dist(m.vertices[i][j],m.point(i,j,mats)),1e-7)
 def test_neutral_endpoints_and_rotation_only_source_cues(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     mats=m.matrices(clip,t)
     for i,vs in enumerate(m.vertices):
      for a,b in zip(vs,m.posed(i,mats)):self.assertLess(math.dist(a,b),1e-7)
   for animation in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
 def test_mutated_geometry_bytes_and_species_are_rejected(self):
  for s,b in self.blobs.items():
   m=copy.deepcopy(self.neutral[s]);m['primitives'][0]['positions'][0]+=.001
   with self.assertRaises(ValueError):author.export_tentacle(s,m)
   blob=bytearray(b);blob[-8]^=1
   with self.assertRaises(ValueError):author.decode_tentacle(bytes(blob),s)
  with self.assertRaises(ValueError):author.export_tentacle('lapras')
if __name__=='__main__':unittest.main()
