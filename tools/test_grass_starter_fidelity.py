"""Reference-volume regressions over stored GLBs, independent of author pins.

CRYSTAL_GRASS_STARTER_FIDELITY_MODELS accepts immutable previous GLBs named
chikorita, bayleef and meganium for rejected-geometry checks.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from sculpt_test_support import horizontal_section_bounds as section,indexed_ray_intersections as rays
from test_steelix_sculpt import occupied

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models'
OVERRIDE=os.environ.get('CRYSTAL_GRASS_STARTER_FIDELITY_MODELS')
def limits(p,k):return min(p['positions'][k::3]),max(p['positions'][k::3])
def points(p):return list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def volume(p):
 vs=points(p);return abs(sum(sum(a[k]*cross(b,c)[k] for k in range(3)) for i in range(0,len(p['indices']),3) for a,b,c in [[vs[j] for j in p['indices'][i:i+3]]])/6)
def xy_area(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])

class GrassStarterFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.parts={}
  for s in ('chikorita','bayleef','meganium'):
   path=Path(OVERRIDE)/(s+'.glb') if OVERRIDE else ROOT/('actor_props/battle_chikorita.glb' if s=='chikorita' else 'battle_species/'+s+'.glb')
   m=Model(path.read_bytes());m=logical_model(m) if 'source_part_ranges' in m.doc['extras'] else m
   cls.models[s]=m;cls.parts[s]=m.neutral_model()['primitives']
 def find(self,s,prefix):
  matches=[p for p in self.parts[s] if p['part'].startswith(prefix)]
  self.assertTrue(matches,(s,'missing anatomy',prefix));return matches[0]
 def test_chikorita_rear_barrel_has_real_profile_depth(self):
  lo,hi=section(self.find('chikorita','Body /'),.25)
  self.assertLess(lo[1],-.37);self.assertGreater(hi[1]-lo[1],.65)
 def test_chikorita_crown_is_broad_and_stays_near_the_centerline(self):
  p=self.find('chikorita','Leaf / curved');q=dict(p,positions=[v for x,y,z in points(p) for v in (x,-z,y)])
  lo,hi=section(q,.22);self.assertGreater(hi[0]-lo[0],.265);self.assertLess(abs((lo[0]+hi[0])/2),.06)
 def test_chikorita_has_red_iris_tissue_and_three_nails_per_paw(self):
  ps=self.parts['chikorita'];claws=[p for p in ps if p['part'].startswith('Claw /')];self.assertEqual(len(claws),12)
  for side in ('left','right'):
   iris=self.find('chikorita','Eye / '+side+' fitted red iris');r,g,b,_=iris['base_color'];self.assertGreater(r,3*g);self.assertGreater(r,2*b)
   for row in ('front','rear'):self.assertEqual(sum(p['part'].startswith('Claw / '+side+' '+row) for p in claws),3)
 def test_chikorita_gaze_and_smile_clear_the_actual_moving_skull(self):
  m=self.models['chikorita']
  for clip,t in ((None,0.),('idle',.25),('attack',.34),('hit',.18)):
   mats=m.matrices(clip,t);head=mats[2]
   def local(i):
    coords=[]
    for p in m.posed(i,mats):
     d=[p[k]-head[12+k] for k in range(3)];coords.extend(sum(head[j*4+k]*d[k] for k in range(3)) for j in range(3))
    return dict(positions=coords,indices=m.indices[i])
   body=local(0)
   for i,name in enumerate(m.names):
    if not name.startswith(('Eye /','Mouth /')):continue
    mark=local(i);vs=points(mark)
    for k in range(0,len(mark['indices']),3):
     a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
     if xy_area(a,b,c)<1e-10:continue
     probes=[[sum(p[d] for p in (a,b,c))/3 for d in range(3)]]+[[sum(p[d] for p in pair)/2 for d in range(3)] for pair in ((a,b),(b,c),(c,a))]
     for x,y,z in probes:
      hits=rays(body,x,y);self.assertTrue(hits,(clip,name));self.assertGreater(z-max(hits),.0006,(clip,name))
 def test_bayleef_neck_and_skull_have_source_volume(self):
  body=self.find('bayleef','Body /')
  for y,minimum in ((.68,.245),(.88,.33)):
   lo,hi=section(body,y);self.assertGreater(hi[0]-lo[0],minimum)
 def test_bayleef_collar_leaves_are_solids_with_broad_folds(self):
  ps=[p for p in self.parts['bayleef'] if p['part'].startswith('Collar / rolled')];self.assertEqual(len(ps),8)
  for p in ps:self.assertGreater(volume(p),.001,(p['part'],'thin collar leaf'))
 def test_bayleef_crown_returns_over_the_back_with_an_open_notch(self):
  p=self.find('bayleef','Crown / asymmetric');self.assertLess(limits(p,0)[1],.23)
  # Actual stored station on either side of the deep re-entrant right edge.
  self.assertFalse(occupied(p,(.10,1.003,-.314)))
  q=dict(p,positions=[v for x,y,z in points(p) for v in (x,-z,y)])
  lo,hi=section(q,.001);self.assertGreater(hi[0]-lo[0],.34);self.assertLess(abs((lo[0]+hi[0])/2),.06)
 def test_evolved_tails_have_thick_roots_and_rise_toward_the_tip(self):
  for s,width in (('bayleef',.15),('meganium',.22)):
   p=self.find(s,'Tail /');self.assertGreater(limits(p,0)[1]-limits(p,0)[0],width,s)
   vs=points(p);front=limits(p,2)[1];back=limits(p,2)[0]
   root=[v[1] for v in vs if v[2]>front-.08];tip=[v[1] for v in vs if v[2]<back+.04]
   self.assertGreater(sum(tip)/len(tip)-sum(root)/len(root),.12,s)
 def test_meganium_has_a_rounded_skull_and_substantial_neck(self):
  p=self.find('meganium','Body /');lo,hi=section(p,1.24)
  self.assertLess(hi[0]-lo[0],.32);self.assertLess(hi[1]-lo[1],.27)
  lo,hi=section(p,.92);self.assertGreater(hi[0]-lo[0],.25)
 def test_meganium_antennae_sweep_back_from_occupied_skull_roots(self):
  body=self.find('meganium','Body /')
  for side,sign in (('left',-1),('right',1)):
   p=self.find('meganium','Antenna / '+side);lo,hi=limits(p,2)
   self.assertLess(lo,-.35);self.assertGreater(hi-lo,.65);self.assertLess(max(abs(x) for x,y,z in points(p)),.24)
   self.assertTrue(occupied(body,(sign*.065,1.270,.31)))
 def test_meganium_pink_front_and_back_and_stamens_follow_real_petal_faces(self):
  m=self.models['meganium'];axis=(0.,.724,.69);v=(0.,.69,-.724)
  def dot(a,b):return sum(x*y for x,y in zip(a,b))
  for clip,t in ((None,0.),('idle',.25),('attack',.34),('hit',.18)):
   mats=m.matrices(clip,t);carrier=mats[3]
   def local(i,a):
    r=(math.cos(a),v[1]*math.sin(a),v[2]*math.sin(a));u=(-math.sin(a),v[1]*math.cos(a),v[2]*math.cos(a));coords=[]
    for p in m.posed(i,mats):
     d=[p[k]-carrier[12+k] for k in range(3)];q=[sum(carrier[j*4+k]*d[k] for k in range(3)) for j in range(3)]
     coords.extend((dot(q,r),dot(q,u),dot(q,axis)))
    return dict(positions=coords,indices=m.indices[i])
   for n in range(1,7):
    angle=math.pi/6+(n-1)*math.tau/6;carrier=mats[15+n]
    for child,parent,both in (('Flower / folded oval petal ','Flower / pale petal rim ',True),('Flower / stamen ','Flower / folded oval petal ',False)):
     self.find('meganium',child+str(n));self.find('meganium',parent+str(n))
     i=m.names.index(child+str(n));j=m.names.index(parent+str(n));mark=local(i,angle);hull=local(j,angle);vs=points(mark);tested=[0,0]
     for k in range(0,len(mark['indices']),3):
      ids=mark['indices'][k:k+3];a,b,c=[vs[q] for q in ids];area=xy_area(a,b,c)
      if abs(area)<1e-7 or (area<0 and not both):continue
      x,y,z=[sum(p[d] for p in (a,b,c))/3 for d in range(3)];hits=rays(hull,x,y);self.assertTrue(hits,(clip,child,n));gap=z-max(hits) if area>0 else min(hits)-z
      self.assertTrue(.0002<gap<.004,(clip,child,n,gap));tested[0 if area>0 else 1]+=1
     self.assertGreater(tested[0],5)
     if both:self.assertGreater(tested[1],5)
 def test_appendage_roots_keep_real_body_support_through_motion(self):
  for s,m in self.models.items():
   probes=[]
   for i,name in enumerate(m.names):
    if not name.startswith(('Tail /','Antenna /','Flower / pale petal rim ')):continue
    # Recover the actual first cap plane instead of assuming its segment count.
    a,b,c=[m.vertices[i][j] for j in m.indices[i][:3]];normal=cross([b[k]-a[k] for k in range(3)],[c[k]-a[k] for k in range(3)]);length=math.sqrt(sum(v*v for v in normal));normal=[v/length for v in normal];cap={}
    for first in range(0,len(m.indices[i]),3):
     ids=m.indices[i][first:first+3];p,q,r=[m.vertices[i][j] for j in ids];n=cross([q[k]-p[k] for k in range(3)],[r[k]-p[k] for k in range(3)])
     if sum(n[k]*normal[k] for k in range(3))<=0 or any(abs(sum((v[k]-a[k])*normal[k] for k in range(3)))>5e-7 for v in (p,q,r)):break
     cap.update((tuple(m.vertices[i][j]),j) for j in ids)
    self.assertGreaterEqual(len(cap),6,(s,name))
    probes += [(i,j) for j in cap.values()]
   self.assertTrue(probes,s)
   for clip,t in ((None,0.),('idle',.25),('attack',.34),('hit',.18)):
    mats=m.matrices(clip,t);body=dict(self.parts[s][0],positions=[v for p in m.posed(0,mats) for v in p])
    for i,j in probes:
     # Read the actual root cap vertices and their stored skin influences.
     self.assertTrue(occupied(body,m.point(i,j,mats)),(s,clip,m.names[i],m.vertices[i][j]))
 def aperture(self,parts):
  jaw=next(p for p in parts if p['part'].startswith('Jaw /'));body=next(p for p in parts if p['part'].startswith('Body /'));front=limits(jaw,2)[1]
  lip_y=max(y for x,y,z in points(jaw) if z>front-.07);y=limits(body,1)[1]-.180;result=[]
  self.assertGreater(y-lip_y,.002)
  for x in (-.025,0.,.025):
   hits=rays(body,x,y);above=rays(body,x,y+.05);self.assertTrue(hits and above)
   backing=max(hits);self.assertGreater(max(above)-backing,.025)
   self.assertTrue(occupied(body,(x,y,backing-.008)))
   self.assertLess(backing+.020,front)
   result += [(x,y,backing+d) for d in (.010,.020)]
  return result
 def test_meganium_mouth_is_empty_in_front_of_occupied_backing(self):
  m=self.models['meganium']
  for clip,t in ((None,0.),('idle',.25),('attack',.34),('hit',.18)):
   mats=m.matrices(clip,t);head=mats[5];parts=[]
   for i,original in enumerate(self.parts['meganium']):
    coords=[]
    for p in m.posed(i,mats):
     d=[p[k]-head[12+k] for k in range(3)];coords.extend(sum(head[j*4+k]*d[k] for k in range(3)) for j in range(3))
    parts.append(dict(original,positions=coords))
   for point in self.aperture(parts):self.assertFalse(any(occupied(p,point) for p in parts),(clip,t,point))

if __name__=='__main__':unittest.main()
