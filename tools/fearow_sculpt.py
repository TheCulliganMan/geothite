#!/usr/bin/env python3
"""Original airborne Fearow: forward neck, sagittal comb and tucked claws.

Crystal's complete animation supplies the flight stance. Inspected game orbits
and the Home render supply depth and full colour; no foreign mesh is an input.
"""
import math,struct
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,fitted_triangle_contour
from bovine_sculpt import curved_tube,oval
from goldfish_sculpt import zy
from murkrow_sculpt import bill
from pidgeotto_sculpt import frame
from skin_glb import COORDINATES

HEIGHT=1.23
BROWN=(.74,.46,.20);LIGHT_BROWN=(.83,.55,.25);PINK=(.94,.65,.72)
RED=(.93,.29,.33);CREAM=(.99,.96,.79);INK=(.10,.075,.08);WHITE=(.99,.98,.93)
TALON=(.92,.89,.80)
WING_ANGLE=-math.radians(38)
WING_SWEEP=math.radians(35)

def wing_frame(q,side,inverse=False):
 """Turn the actual closed fan, including fitted tips, around its rooted spar."""
 q=deepcopy(q);old=(side*.105,.59,-.02);new=(side*.12,.735,-.065)
 c,s=math.cos(WING_ANGLE),math.sin(WING_ANGLE)
 cy,sy=math.cos(WING_SWEEP),side*math.sin(WING_SWEEP)
 for field in ('positions','normals'):
  out=[]
  for xyz in zip(q[field][::3],q[field][1::3],q[field][2::3]):
   if inverse:
    x,y,z=[v-(new[i] if field=='positions' else 0.) for i,v in enumerate(xyz)]
    x,z=cy*x-sy*z,sy*x+cy*z
    xyz=(x,c*y+s*z,-s*y+c*z)
    out.extend(v+(old[i] if field=='positions' else 0.) for i,v in enumerate(xyz))
   else:
    x,y,z=[v-(old[i] if field=='positions' else 0.) for i,v in enumerate(xyz)]
    xyz=(x,c*y-s*z,s*y+c*z)
    x,y,z=xyz;xyz=(cy*x+sy*z,y,-sy*x+cy*z)
    out.extend(v+(new[i] if field=='positions' else 0.) for i,v in enumerate(xyz))
  q[field]=out
 return q

def feet(p):
 for s in (-1,1):
  p.append(curved_tube('Leg / tucked pink hock '+str(s),PINK,[(s*.105,.490,-.10),(s*.110,.377,-.145),(s*.120,.341,-.010)],[.027,.019,.020],16))
  p.append(ellipsoid('Foot / raised pink knuckle '+str(s),PINK,(s*.120,.335,.005),(.042,.031,.036),20,12))
  for j,dx in enumerate((-.030,0.,.030)):
   end=(s*.120+dx,.245,.075 if j==1 else .060)
   p.append(curved_tube(f'Toe / three curled forward digits {s} {j}',PINK,[(s*.120+dx*.35,.340,.024),(s*.120+dx,.294,.094 if j==1 else .080),end],[.012,.011,.008],12))
   p.append(curved_tube(f'Talon / forward ivory hook {s} {j}',TALON,[end,(end[0],.232,end[2]-.014),(end[0],.239,end[2]-.035)],[.008,.006,.0015],12))
  p.append(curved_tube('Toe / rear gripping digit '+str(s),PINK,[(s*.120,.342,-.008),(s*.148,.288,-.064),(s*.148,.258,-.035)],[.012,.010,.007],12))
  p.append(curved_tube('Talon / rear ivory hook '+str(s),TALON,[(s*.148,.258,-.035),(s*.148,.247,-.020),(s*.148,.257,-.002)],[.007,.005,.0015],12))

def sculpture(*,rig_ready=False,with_transform=False):
 from raptor_bird_sculpt import wings,tails,sheet
 body=crease_normals(loft('Body / long canted airborne breast',BROWN,curved_profiles(((.425,.035,.070,-.130),(.485,.125,.235,-.145),(.585,.177,.292,-.130),(.695,.163,.268,-.100),(.765,.107,.179,-.040),(.802,.036,.065,.005)),4),32),45)
 neck=curved_tube('Neck / forward arched fishing neck',BROWN,[(0,.720,.050),(0,.840,.205),(0,.795,.355),(0,.740,.455)],[.068,.047,.035,.038],24)
 head=ellipsoid('Head / narrow long-billed cranium',LIGHT_BROWN,(0,.735,.477),(.084,.073,.095),32,20)
 p=[body,neck,head]
 def mark(name,color,xy,parent,th=.004):return fitted_triangle_contour(name,color,xy,parent,th,precision=6)
 p.append(mark('Breast / fitted ragged ochre feather bib',LIGHT_BROWN,[(-.100,.742),(-.149,.665),(-.105,.678),(-.104,.605),(-.056,.640),(-.032,.570),(.003,.630),(.041,.580),(.066,.648),(.112,.614),(.106,.678),(.148,.665),(.100,.742)],body,.007))
 start=len(p);wings(p,True,rig_ready)
 # Each closed primary already has two real faces. A second coincident bank
 # made the old wing look like stacked tiles, rather than a feathered surface.
 p[start:]=[q for q in p[start:] if not q['part'].startswith(('Rear primary /','Rear covert /','Rear primary tip /'))]
 # Recolour the existing original fan before turning all of its closed solids.
 from raptor_bird_sculpt import BROWN as OLD_BROWN,LIGHT_BROWN as OLD_LIGHT,CREAM as OLD_CREAM
 for i in range(start,len(p)):
  side=-1 if sum(p[i]['positions'][::3])<0 else 1
  if p[i]['part'].startswith('Wing /'):
   from lugia_sculpt import paddle
   p[i]=paddle(p[i]['part'],BROWN,[(side*.095,.560,-.020),(side*.30,.800,-.055),(side*.61,1.080,-.095),(side*.93,1.300,-.150)],[.070,.063,.042,.008],[.050,.048,.033,.008],segments=16,steps=4)
  col=tuple(p[i]['base_color'][:3]);p[i]['base_color']=list((BROWN if col==OLD_BROWN else LIGHT_BROWN if col==OLD_LIGHT else CREAM if col==OLD_CREAM else col))+[1.]
  p[i]=wing_frame(p[i],side)
 feet(p)
 start=len(p);tails(p,True,rig_ready)
 for q in p[start:]:
  q['base_color']=list(LIGHT_BROWN)+[1.]
  j=int(q['part'].split()[-1]);layer=.004*abs(j-2)
  q['positions']=[v+(.22+layer if i%3==1 else -.18 if i%3==2 else 0.) for i,v in enumerate(q['positions'])]
 for s in (-1,1):
  a=s*math.radians(55);local=frame(head,a,True)
  uv=lambda points:[(s*x,y) for x,y in points]
  socket=mark('Eye socket / sharp dark bill-side gaze '+str(s),INK,uv(((-.428,.746),(-.395,.773),(-.351,.770),(-.355,.744),(-.396,.730),(-.426,.732))),local,.004)
  white=mark('Eye / sharp fitted ivory '+str(s),WHITE,uv(((-.418,.747),(-.394,.766),(-.360,.764),(-.363,.747),(-.395,.736),(-.416,.737))),socket,.0025)
  pupil=mark('Pupil / small stern gaze '+str(s),INK,oval(-s*.399,.747,.006,.009,12),white,.0015)
  p.extend(frame(q,a) for q in (socket,white,pupil))
 p.append(bill('Bill / long tapering pink upper lance',PINK,[(.532,.033,.707,.749),(.603,.035,.681,.715),(.803,.018,.553,.574),(.985,.0015,.432,.439)]))
 p.append(bill('Bill / open lower pink mandible',PINK,[(.532,.029,.685,.704),(.594,.028,.621,.646),(.690,.017,.528,.543),(.777,.0015,.503,.509)]))
 # The comb is broad in the sagittal profile and narrow in the front view.
 crown=[(-.417,.782),(-.396,.856),(-.447,.830),(-.421,.953),(-.481,.867),(-.486,.999),(-.522,.880),(-.558,.969),(-.553,.853),(-.608,.905),(-.573,.767)]
 p.append(zy(sheet('Crown / five pointed sagittal red feathers',RED,crown,lambda y,x:.005,.010,0)))
 # A pale upper-back ruff wraps the nape, with separate peaks along its profile.
 from lugia_sculpt import paddle
 from pidgeotto_sculpt import join
 ruff=[]
 for j in range(-3,4):
  x=j*.026
  ruff.append(paddle('nape feather',CREAM,[(x*.25,.742,.055),(x*.90,.805,-.008-.006*abs(j)),(x*1.6,.918-.018*abs(j),-.085-.010*abs(j))],[.023,.030,.0015],[.020,.025,.0015],segments=10,steps=3))
 p.append(join('Mane / seven rooted pale nape feathers',ruff))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);k=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) for i,v in enumerate(q['positions'])]
  q['positions']=list(struct.unpack('<'+'f'*len(q['positions']),struct.pack('<'+'f'*len(q['positions']),*q['positions'])))
  q.update(crease_normals(q,40.));q['normals']=[round(v,7) or 0. for v in q['normals']]
 model=dict(name='fearow',version=1,coordinate_system=COORDINATES,primitives=p)
 return (model,(lo,k)) if with_transform else model
