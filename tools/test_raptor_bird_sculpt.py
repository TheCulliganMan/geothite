#!/usr/bin/env python3
"""Stored raptor-bird anatomy, physical roots and fitted feather/face checks."""
import json,os,unittest,math
from pathlib import Path
from raptor_bird_sculpt import sculpture,SPECIES,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bovine_sculpt import reverse_front
from test_kingdra_sculpt import crossed_faces
RIG_READY=os.environ.get('CRYSTAL_RAPTOR_BIRD_TEST_RIG_READY')=='1'
ROOT=Path(os.environ.get('CRYSTAL_RAPTOR_BIRD_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class RaptorBirdTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.ms={};cls.skin_sources={}
  for s in SPECIES:
   cls.skin_sources[s]=(ROOT/(s+'.glb')).exists()
   if cls.skin_sources[s]:
    from raptor_bird_glb import read_raptor_bird
    cls.ms[s]=read_raptor_bird(ROOT/(s+'.glb'),s)
   else:cls.ms[s]=json.loads((ROOT/(s+'.mesh.json')).read_bytes())
 def find(self,s,prefix,suffix=None):return next(p for p in self.ms[s]['primitives'] if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    aa=ray_intersections(a,xl+(xh-xl)*i/25,yl+(yh-yl)*j/25);bb=ray_intersections(b,xl+(xh-xl)*i/25,yl+(yh-yl)*j/25)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_parts(self):
  from pidgeotto_contours import components
  models={s:dict(primitives=[q for p in m['primitives'] for q in (components(p) if s=='fearow' and p['part'].startswith('Mane /') else [p])]) for s,m in self.ms.items()}
  assert_closed_models(self,models)
 def test_finite_unit_normals_follow_actual_faces(self):assert_model_shading(self,self.ms)
 def test_regeneration_preserved_height_floor_and_budget(self):
  for s,m in self.ms.items():
   mode=RIG_READY or self.skin_sources[s]
   if self.skin_sources[s]:
    from raptor_bird_glb import geometry_digest
    self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s,rig_ready=mode)))
   else:self.assertEqual(m,sculpture(s,rig_ready=mode))
   self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHTS[s],places=6)
   self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),40000 if mode else 85000);
   if mode:self.assertLessEqual(sum(len(p['positions'])//3 for p in m['primitives']),20000)
   self.assertEqual(len(m['primitives']),len({p['part'] for p in m['primitives']}))
 def test_species_specific_crest_and_feather_counts(self):
  for s,m in self.ms.items():
   for prefix,n in (('Wing /',2),('Primary /',16),('Rear primary /',16),('Covert /',12),('Rear covert /',12),('Tail /',5),('Leg /',2),('Foot /',2),('Toe / three',6),('Toe / rear',2),('Talon /',8),('Crest /',5 if s=='pidgeot' else 0),('Crown /',1 if s=='fearow' else 0),('Primary tip /',32 if s=='fearow' else 0),('Rear primary tip /',32 if s=='fearow' else 0)):
    if s=='fearow' and prefix in ('Rear primary /','Rear covert /','Rear primary tip /'):n=0
    self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),n,(s,prefix))
 def test_body_and_neck_have_no_nonadjacent_crossed_faces(self):
  for s in SPECIES:
   for prefix in ('Body /','Neck /'):self.assertIsNone(crossed_faces(self.find(s,prefix)),(s,prefix))
 def test_feather_solids_and_swept_crest_have_no_crossed_faces(self):
  for s,m in self.ms.items():
   for p in m['primitives']:
    if p['part'].startswith(('Primary /','Rear primary /','Covert /','Rear covert /','Crest /','Crown /','Tail /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_neck_head_bill_crest_and_mane_have_physical_support(self):
  for s,m in self.ms.items():
   b=self.find(s,'Body /');neck=self.find(s,'Neck /');head=self.find(s,'Head /')
   self.assertTrue(self.overlap(b,neck),s);self.assertTrue(self.overlap(neck,head),s)
   for p in m['primitives']:
    if p['part'].startswith(('Bill /','Crest /','Crown /')):self.assertTrue(self.overlap(p,head),(s,p['part']))
    elif p['part'].startswith('Mane /'):self.assertTrue(self.overlap(p,neck),(s,p['part']))
 def test_all_wing_layers_and_rear_tail_feathers_are_rooted(self):
  for s,m in self.ms.items():
   b=self.find(s,'Body /')
   for sign in (-1,1):
    wing=self.find(s,'Wing /',sign);self.assertTrue(self.overlap(wing,b),(s,sign))
    for p in m['primitives']:
     if p['part'].startswith(('Primary /','Covert /','Rear primary /','Rear covert /')) and p['part'].split()[-2]==str(sign):self.assertTrue(self.overlap(p,wing),(s,p['part']))
   for p in m['primitives']:
    if p['part'].startswith('Tail /'):self.assertTrue(self.overlap(p,b),(s,p['part']))
 def test_pink_feet_toes_and_talons_have_real_roots(self):
  for s in SPECIES:
   for sign in (-1,1):
    leg=self.find(s,'Leg /',sign);foot=self.find(s,'Foot /',sign);self.assertTrue(self.overlap(leg,self.find(s,'Body /')));self.assertTrue(self.overlap(leg,foot))
    for j in range(3):
     toe=self.find(s,'Toe / three',f'{sign} {j}');claw=self.find(s,'Talon / forward',f'{sign} {j}');self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,claw))
    toe=self.find(s,'Toe / rear',sign);self.assertTrue(self.overlap(toe,foot));self.assertTrue(self.overlap(toe,self.find(s,'Talon / rear',sign)))
 def test_detached_wing_negative_control_is_rejected(self):
  from sculpt_geometry import translate
  import copy
  for s in SPECIES:
   wing=translate(copy.deepcopy(self.find(s,'Wing /',1)),0,0,1)
   self.assertFalse(self.overlap(wing,self.find(s,'Body /')),s)
 def bill_gap(self,parts):
  upper=next(p for p in parts if p['part'].startswith('Bill / long'))
  lower=next(p for p in parts if p['part'].startswith('Bill / open'))
  lo=max(min(upper['positions'][2::3]),min(lower['positions'][2::3]));hi=min(max(upper['positions'][2::3]),max(lower['positions'][2::3]))
  # Rotate the stored triangles so the independent ray oracle measures +Y.
  rotated=[]
  for p in parts:
   q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i],p['positions'][i+2],-p['positions'][i+1])];rotated.append(q)
  u=rotated[parts.index(upper)];l=rotated[parts.index(lower)]
  for t in (.65,.75,.85):
   z=lo+(hi-lo)*t;uh=[-y for y in ray_intersections(u,0,z)];lh=[-y for y in ray_intersections(l,0,z)]
   if not uh or not lh:return False
   top=min(uh);bottom=max(lh)
   if top-bottom<.015:return False
   for p in rotated:
    hits=sorted(-y for y in ray_intersections(p,0,z))
    for a,b in zip(hits[::2],hits[1::2]):
     if min(b,top-.0002)>max(a,bottom+.0002):return False
  return True
 def test_fearow_has_actual_open_space_between_its_stored_mandibles(self):self.assertTrue(self.bill_gap(self.ms['fearow']['primitives']))
 def test_filled_bill_negative_control_is_rejected(self):
  from sculpt_geometry import ellipsoid
  parts=self.ms['fearow']['primitives'];up=self.find('fearow','Bill / long');low=self.find('fearow','Bill / open');zlo=max(min(up['positions'][2::3]),min(low['positions'][2::3]));zhi=min(max(up['positions'][2::3]),max(low['positions'][2::3]));z=zlo+(zhi-zlo)*.75
  # A thin wall spans the whole local jaw-height range and must fill the gap.
  ymin=min(low['positions'][1::3]);ymax=max(up['positions'][1::3]);filler=ellipsoid('Invalid bill filler',(.5,.5,.5),(0,(ymin+ymax)/2,z),(.02,(ymax-ymin)*.55,.02),24,16)
  self.assertFalse(self.bill_gap(parts+[filler]))
 def pairs(self):
  for s,m in self.ms.items():
   for p in m['primitives']:
    n=p['part'];sign=n.split()[-1];parent=None
    if n.startswith('Breast /'):parent=self.find(s,'Body /')
    elif n.startswith('Throat /'):parent=self.find(s,'Neck /')
    elif n.startswith(('Mask /','Eye socket /','Cheek /')):parent=self.find(s,'Head /')
    elif n.startswith('Eye /'):parent=self.find(s,'Mask /' if s=='pidgeot' else 'Eye socket /',sign)
    elif n.startswith('Pupil /'):parent=self.find(s,'Eye /',sign)
    elif n.startswith('Glint /'):parent=self.find(s,'Pupil /',sign)
    elif n.startswith(('Primary tip /','Rear primary tip /')):
     sign,j,face=n.split()[-3:];parent=self.find(s,'Rear primary /' if n.startswith('Rear') else 'Primary /',f'{sign} {j}')
     if s=='fearow':
      from raptor_bird_glb import authoring_transform
      from fearow_sculpt import wing_frame
      lo,k=authoring_transform(s)
      def inspect(q):
       q=dict(q,positions=[v/k+(lo if i%3==1 else 0.) for i,v in enumerate(q['positions'])])
       return wing_frame(q,int(sign),True)
      p,parent=inspect(p),inspect(parent)
     if face=='-1':parent=reverse_front(parent);p=reverse_front(p)
    elif n.startswith('Wing field /'):
     sign,face=n.split()[-2:];parent=self.find(s,'Wing /',sign)
     if face=='-1':parent=reverse_front(parent);p=reverse_front(p)
    if parent:
     if s=='fearow' and n.startswith(('Eye socket /','Eye /','Pupil /')):
      from pidgeotto_sculpt import frame
      p,parent=frame(p,int(sign)*math.radians(55),True),frame(parent,int(sign)*math.radians(55),True)
     if s=='pidgeot':
      from raptor_bird_glb import authoring_transform
      lo,k=authoring_transform(s)
      if n.startswith(('Mask /','Eye /','Pupil /','Glint /')):
       a=-int(sign)*math.radians(53);c,sn=math.cos(a),math.sin(a)
       def inspect(q):
        q=dict(q);q['positions']=[v for x,y,z in zip(q['positions'][::3],q['positions'][1::3],q['positions'][2::3]) for v in (c*x+sn*(z-.045*k),y,-sn*x+c*(z-.045*k))];return q
       p,parent=inspect(p),inspect(parent)
      elif n.startswith('Wing field /'):
       from raptor_bird_sculpt import folded_wing
       def inspect(q):
        q=dict(q);q['positions']=[v/k+(lo if i%3==1 else 0.) for i,v in enumerate(q['positions'])]
        return folded_wing(q,int(n.split()[-2]),True)
       p,parent=inspect(p),inspect(parent)
     yield s,p,parent
 def test_fitted_faces_breasts_and_both_sides_of_pale_tips_have_mesh_support(self):
  for s,p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),.018,(s,p['part']))
 def test_fitted_front_centers_and_edges_clear_stored_parent(self):
  for s,p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
    # Only the outward front cap faces +Z; back and walls are intentional roots.
    u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
    if normal[2]<=.5*math.sqrt(sum(x*x for x in normal)):continue
    for q in (tuple((a[i]+b[i]+c[i])/3 for i in range(3)),tuple((a[i]+b[i])/2 for i in range(3)),tuple((b[i]+c[i])/2 for i in range(3)),tuple((c[i]+a[i])/2 for i in range(3))):
     hits=ray_intersections(parent,q[0],q[1]);self.assertTrue(hits);self.assertGreater(q[2]-max(hits),.0001,(s,p['part'],k//3))

if __name__=='__main__':unittest.main()
