"""Independent checks of all 26 stored Unown glyphs, topology, eye clearance and clips."""
import copy,math,os,unittest
from pathlib import Path
import unown_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,ray_intersections
ROOT=Path(os.environ.get('CRYSTAL_UNOWN_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class UnownSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.neutral={}
  for species in author.SPECIES:
   blob=(ROOT/(species+'.glb')).read_bytes();cls.models[species]=Model(blob);cls.neutral[species]=author.decode_glyph(blob,species)
 def test_all_26_distinct_canonical_forms_fit_existing_limits(self):
  self.assertEqual(len(self.models),26);digests=set()
  for species,m in self.models.items():
   blob=(ROOT/(species+'.glb')).read_bytes();self.assertEqual(blob,author.export_glyph(species));digest=author.geometry_digest(self.neutral[species]);self.assertEqual(digest,author.REST_DIGESTS[species]);self.assertTrue(digest not in digests);digests.add(digest)
   self.assertLess(len(blob),1024*1024);self.assertLess(sum(map(len,m.vertices)),20000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertEqual(len(m.joints),3)
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,self.neutral);assert_model_shading(self,self.neutral)
 def test_dark_glyphs_use_the_exact_verified_pack_palette(self):
  rgb=[((v<<3)|(v>>2))/255 for v in (7,7,8)]
  expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
  for species,m in self.models.items():
   for part,name in zip(m.parts,m.names):
    if not name.startswith(('Ring /','Back /','Glyph /')):continue
    color=m.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']
    for a,b in zip(color,expected):self.assertAlmostEqual(a,b,places=7)
 def test_secondary_loops_are_true_closed_tori_without_overlapping_caps(self):
  loop_forms=set()
  for species,model in self.neutral.items():
   for part in model['primitives']:
    if 'closed loop' not in part['part']:continue
    loop_forms.add(species[-1]);vs=[tuple(part['positions'][i:i+3]) for i in range(0,len(part['positions']),3)];edges=set()
    for i in range(0,len(part['indices']),3):
     a,b,c=[vs[j] for j in part['indices'][i:i+3]]
     for x,y in ((a,b),(b,c),(c,a)):edges.add(tuple(sorted((x,y))))
    self.assertEqual(len(set(vs))-len(edges)+len(part['indices'])//3,0,(species,part['part']))
  self.assertEqual(loop_forms,set('bfgov'))
 def test_one_continuous_socket_and_coherent_eye_per_letter(self):
  for species,m in self.models.items():
   self.assertEqual(sum(n.startswith('Ring /') for n in m.names),1);self.assertEqual(sum(n.startswith('Eye /') for n in m.names),1);self.assertEqual(sum(n.startswith('Pupil /') for n in m.names),1)
   self.assertEqual(m.parents[:-1],[None,0,1])
   for p in range(4):self.assertTrue(all(js==[1,0,0,0] for js in m.ids[p]))
   self.assertTrue(all(n.startswith('Glyph /') for n in m.names[4:]))
 def test_pupil_front_triangles_clear_stored_eye_lens(self):
  for species,m in self.neutral.items():
   eye=m['primitives'][1];pupil=m['primitives'][2];vs=[pupil['positions'][i:i+3] for i in range(0,len(pupil['positions']),3)];checked=0
   # Broad phase over the complete pupil bounds; every subsequent ray still
   # intersects actual stored parent triangles, independent of authoring math.
   low=[min(v[k] for v in vs)-1e-6 for k in range(2)];high=[max(v[k] for v in vs)+1e-6 for k in range(2)]
   parent=[eye['positions'][i:i+3] for i in range(0,len(eye['positions']),3)];indices=[]
   for i in range(0,len(eye['indices']),3):
    tri=eye['indices'][i:i+3];points=[parent[j] for j in tri]
    if all(max(v[k] for v in points)>=low[k] and min(v[k] for v in points)<=high[k] for k in range(2)):indices.extend(tri)
   eye={'positions':eye['positions'],'indices':indices}

   for i in range(0,len(pupil['indices']),3):
    a,b,c=[vs[j] for j in pupil['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(p[k] for p in (a,b,c))/3 for k in range(3)];hits=ray_intersections(eye,*mid[:2]);self.assertTrue(hits);self.assertGreater(mid[2],max(hits),(species,i));checked+=1
   self.assertGreater(checked,8)
 def test_floor_winding_and_glyph_strain_across_all_clips(self):
  for species,m in self.models.items():
   for clip in m.clips:
    for t in (0.,.18,.28,.53,.79,1.):
     mats=m.matrices(clip,t)
     for p,name in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(species,clip,t,name))
      for i in range(0,len(m.indices[p]),3):
       tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(species,clip,t,name,i));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(species,clip,t,name,ratio))
 def test_closed_endpoints_and_articulation(self):
  for species,m in self.models.items():
   for clip in m.clips:
    for t in (0.,1.):
     mats=m.matrices(clip,t)
     for p,vs in enumerate(m.vertices):
      for a,b in zip(vs,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
    mats=m.matrices(clip,.28)
    self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[0],m.posed(0,mats))),.001)
    self.assertTrue(any(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,mats)))>.001 for p in range(4,len(m.parts))))
   for animation in m.doc['animations']:
    self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'});idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
 def test_y_stem_keeps_an_occupied_root_in_its_separate_foot_through_clips(self):
  m=self.models['unown_y'];parts=m.neutral_model()['primitives']
  stem=next(i for i,n in enumerate(m.names) if n=='Glyph / Y authored letter stroke 2')
  foot=next(i for i,n in enumerate(m.names) if n=='Glyph / Y authored letter stroke 3')
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    a=dict(parts[stem],positions=sum(m.posed(stem,mats),[]));b=dict(parts[foot],positions=sum(m.posed(foot,mats),[]))
    self.assertTrue(parts_overlap(a,b),(clip,step,'detached Y foot'))
  detached=copy.deepcopy(parts[foot]);detached['positions']=[v-(.10 if i%3==1 else 0.) for i,v in enumerate(detached['positions'])]
  self.assertFalse(parts_overlap(parts[stem],detached))
if __name__=='__main__':unittest.main()
