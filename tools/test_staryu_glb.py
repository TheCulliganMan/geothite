"""Stored star anatomy, ruby fitting, planted tips and radial skin regressions."""
import math,unittest
from pathlib import Path
import staryu_glb as author
from test_gengar_glb import Model,winding_alignment
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class StaryuSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'staryu.glb').read_bytes();cls.m=Model(cls.blob);cls.neutral=author.decode_starfish(cls.blob,'staryu')
 def test_canonical_anatomy_and_existing_limits(self):
  self.assertEqual(self.blob,author.export_staryu());self.assertEqual(author.geometry_digest(self.neutral),author.REST_DIGESTS['staryu'])
  self.assertEqual((len(self.m.parts),sum(map(len,self.m.vertices)),sum(len(p)//3 for p in self.m.indices),len(self.m.joints)),(10,1982,2852,12))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(self.m.doc['accessors']),256);self.assertFalse((ROOT/'staryu.mesh.json').exists())
 def test_outward_closed_geometry_and_shading(self):
  assert_closed_models(self,{'staryu':self.neutral});assert_model_shading(self,{'staryu':self.neutral})
 def test_five_arms_and_two_floor_contacts(self):
  body=self.m.vertices[0];low=min(p[1] for p in body);contacts={tuple(p) for p in body if p[1]<low+1e-7}
  self.assertEqual(len(contacts),2);self.assertAlmostEqual(sum(p[0] for p in contacts),0.,places=7)
  # Outer tip extrema in five different radial sectors, using stored coordinates.
  center=author.normalized((0.,.40,0.));rays=[math.pi/2+i*math.tau/5 for i in range(5)]
  extreme=[]
  for angle in rays:
   direction=(math.cos(angle),math.sin(angle));p=max(body,key=lambda p:(p[0]-center[0])*direction[0]+(p[1]-center[1])*direction[1]);extreme.append(tuple(p));self.assertGreater(math.dist(p[:2],center[:2]),.54)
  self.assertEqual(len(set(extreme)),5)
 def test_ruby_and_glint_are_in_front_of_the_body(self):
  parts=self.neutral['primitives'];body=parts[0];gem=next(p for p in parts if p['part'].startswith('Core /'));glint=next(p for p in parts if p['part'].startswith('Core glint /'))
  center=author.normalized((0.,.40,0.));self.assertGreater(max(ray_intersections(gem,*center[:2])),max(ray_intersections(body,*center[:2])))
  # Inspect each stored front glint triangle centroid against the real ruby shell.
  ps=[glint['positions'][i:i+3] for i in range(0,len(glint['positions']),3)]
  checked=0
  for i in range(0,len(glint['indices']),3):
   tri=[ps[j] for j in glint['indices'][i:i+3]];a,b,c=tri
   if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=0:continue
   mid=[sum(p[k] for p in tri)/3 for k in range(3)];hits=ray_intersections(gem,*mid[:2]);self.assertTrue(hits);self.assertGreater(mid[2],max(hits));checked+=1
  self.assertGreater(checked,3)
 def test_stored_radial_ridges_clear_the_real_body(self):
  body=self.neutral['primitives'][0]
  for ridge in self.neutral['primitives'][1:6]:
   vs=[ridge['positions'][i:i+3] for i in range(0,len(ridge['positions']),3)];checked=0
   for i in range(0,len(ridge['indices']),3):
    a,b,c=[vs[j] for j in ridge['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(p[k] for p in (a,b,c))/3 for k in range(3)];hits=ray_intersections(body,*mid[:2]);self.assertTrue(hits);self.assertGreater(mid[2]-max(hits),.002);checked+=1
   self.assertGreater(checked,50)
 def test_pack_body_and_core_palette(self):
  for index,rgb555 in [(0,(23,17,11)),(8,(31,5,3))]:
   color=self.neutral['primitives'][index]['base_color'];rgb=[((v<<3)|(v>>2))/255 for v in rgb555]
   for a,b in zip(color,[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]):self.assertAlmostEqual(a,b,places=7)
 def test_core_mount_and_glint_share_one_owner(self):
  m=self.m;self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS['staryu']])
  for p,name in enumerate(m.names):
   for js,ws in zip(m.ids[p],m.weights[p]):self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<12 for j in js))
   if name.startswith(('Core','Collar')):self.assertTrue(all(js==[1,0,0,0] for js in m.ids[p]),name)
  for i in range(5):self.assertTrue(any(2+2*i in js or 3+2*i in js for js in m.ids[0]))
 def test_floor_contacts_winding_and_strain_across_clips(self):
  m=self.m
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for p,name in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
     for i,v in enumerate(m.vertices[p]):
      if v[1]<.001:self.assertLess(math.dist(posed[i],v),1e-12,(clip,name,i))
     for i in range(0,len(m.indices[p]),3):
      tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
 def test_neutral_endpoints_and_early_cue_articulation(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,1.):
    mats=m.matrices(clip,t)
    for p,vertices in enumerate(m.vertices):
     for a,b in zip(vertices,m.posed(p,mats)):self.assertLess(math.dist(a,b),1e-7)
   changed=sum(math.dist(a,b)>1e-5 for a,b in zip(m.vertices[0],m.posed(0,m.matrices(clip,.25))));self.assertGreater(changed,200)
  for animation in m.doc['animations']:
   self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
   idle=animation['name'].endswith('.idle');self.assertEqual(animation['extras']['loopSuggested'],idle);self.assertEqual(animation['extras']['cueRelative'],not idle)
  angles=author.pose_angles('staryu','attack',.17);late=author.pose_angles('staryu','attack',.70);self.assertGreater(abs(angles[2][0]),abs(late[2][0]))
if __name__=='__main__':unittest.main()
