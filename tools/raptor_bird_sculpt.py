#!/usr/bin/env python3
"""Original swept-crested Pidgeot and long-billed Fearow, with layered vanes."""
import argparse,json,math,struct
from pathlib import Path
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals,part,fitted_contour
from bovine_sculpt import curved_tube,oval,reverse_front
from wing_insect_sculpt import contour
from goldfish_sculpt import surface,zy
from murkrow_sculpt import bill
from skin_glb import COORDINATES

SPECIES=('pidgeot','fearow')
HEIGHTS={'pidgeot':1.35,'fearow':1.23}
BROWN=(.61,.40,.20);LIGHT_BROWN=(.73,.50,.27);CREAM=(.97,.90,.63)
GOLD=(.94,.76,.31);RED=(.90,.38,.27);PINK=(.78,.49,.56)
INK=(.075,.07,.09);WHITE=(.98,.97,.91);TALON=(.88,.87,.78)

def sheet(name,color,xy,depth,th=.018,sub=2):
 return crease_normals(fitted_contour(name,color,xy,depth,th,sub),35)

def paint(name,color,xy,parent,th=.006,sub=3):
 return sheet(name,color,xy,surface(parent),th,sub)

def geometry_functions(rig_ready):
 # Legacy callers keep their exact stored geometry. Rig topology spends samples
 # on the curved outline and only refines fitted caps where parent clearance
 # requires it, instead of subdividing tiny pupils and flat feathers uniformly.
 if not rig_ready:return sheet,paint
 from sculpt_geometry import fitted_adaptive_contour
 def rig_sheet(name,color,xy,depth,th=.018,sub=2):
  return sheet(name,color,xy,depth,th,0 if name.startswith(('Primary /','Rear primary /','Covert /','Rear covert /','Tail /')) else min(sub,1))
 def rig_paint(name,color,xy,parent,th=.006,sub=3):
  return fitted_adaptive_contour(name,color,xy,parent,th,initial_subdivisions=0,max_surface_error=th*.40)
 return rig_sheet,rig_paint

def vane_xy(root,tip,width):
 dx,dy=tip[0]-root[0],tip[1]-root[1];length=math.hypot(dx,dy)
 nx,ny=-dy/length*width,dx/length*width
 def point(t,w):return (root[0]+t*dx+w*nx,root[1]+t*dy+w*ny)
 return contour([(point(0,-.30),point(.25,-.95),point(.76,-1),point(1,0)),
                 (point(1,0),point(.84,.90),point(.29,.88),point(0,.30)),
                 (point(0,.30),point(-.07,.20),point(-.07,-.20),point(0,-.30))],8)

def feather_depth(base,root,tip,width,offset,bank):
 dx,dy=tip[0]-root[0],tip[1]-root[1];dd=dx*dx+dy*dy
 def at(y,x):
  t=((x-root[0])*dx+(y-root[1])*dy)/dd
  w=((x-root[0])*(-dy)+(y-root[1])*dx)/math.sqrt(dd)
  return base(y,x)+offset+bank*.009*math.sin(math.pi*max(0,min(1,t)))*max(0,1-(w/width)**2)
 return at

def pale_tip(p,feather,xy,root,tip,prefix,rig_ready=False):
 sheet,paint=geometry_functions(rig_ready)
 dx,dy=tip[0]-root[0],tip[1]-root[1];dd=dx*dx+dy*dy
 def along(q):return ((q[0]-root[0])*dx+(q[1]-root[1])*dy)/dd
 clipped=[]
 for a,b in zip(xy,xy[1:]+xy[:1]):
  ta,tb=along(a),along(b)
  if ta>=.70:clipped.append(a)
  if (ta>=.70)!=(tb>=.70):
   t=(.70-ta)/(tb-ta);clipped.append((a[0]+t*(b[0]-a[0]),a[1]+t*(b[1]-a[1])))
 cx=sum(x for x,y in clipped)/len(clipped);cy=sum(y for x,y in clipped)/len(clipped)
 inset=[(cx+.92*(x-cx),cy+.92*(y-cy)) for x,y in clipped]
 for face in (-1,1):
  parent=feather if face==1 else reverse_front(feather)
  patch=paint(prefix+' '+str(face),CREAM,inset if face==1 else [(-x,y) for x,y in inset],parent,.004,2)
  p.append(patch if face==1 else reverse_front(patch))

# Fixed orthonormal fold frame. The broad feather faces turn around the body;
# an XY fan's bounding box alone is not a folded profile wing.
_FOLD_U=(.18,-.8,-math.sqrt(1.-.18**2-.8**2))
_FOLD_V=(0.,-_FOLD_U[2]/math.sqrt(1.-_FOLD_U[0]**2),_FOLD_U[1]/math.sqrt(1.-_FOLD_U[0]**2))
# The old fan spans diagonally in XY, rather than along X. Align that actual
# shoulder-to-tip axis before folding it down the body.
_FOLD_ANGLE=math.atan2(.70,.83)
WING_U=tuple(math.cos(_FOLD_ANGLE)*u-math.sin(_FOLD_ANGLE)*v for u,v in zip(_FOLD_U,_FOLD_V))
WING_V=tuple(math.sin(_FOLD_ANGLE)*u+math.cos(_FOLD_ANGLE)*v for u,v in zip(_FOLD_U,_FOLD_V))
WING_W=(WING_U[1]*WING_V[2]-WING_U[2]*WING_V[1],-WING_U[0]*WING_V[2],WING_U[0]*WING_V[1])
def folded_wing(q,side,inverse=False):
 q=deepcopy(q);old=(side*.105,.59,-.02);new=(side*.245,.570,-.020);scale=.42
 for field in ('positions','normals'):
  values=[]
  for x,y,z in zip(q[field][::3],q[field][1::3],q[field][2::3]):
   if inverse:
    a=[(v-new[i])/scale if field=='positions' else v for i,v in enumerate((x,y,z))];a[0]*=side
    b=[sum(a[k]*axis[k] for k in range(3)) for axis in (WING_U,WING_V,WING_W)];b[0]*=side
    values.extend(b[i]+(old[i] if field=='positions' else 0.) for i in range(3))
   else:
    a=[v-old[i] if field=='positions' else v for i,v in enumerate((x,y,z))];a[0]*=side
    b=[a[0]*WING_U[i]+a[1]*WING_V[i]+a[2]*WING_W[i] for i in range(3)];b[0]*=side
    values.extend(b[i]*scale+new[i] if field=='positions' else b[i] for i in range(3))
  q[field]=values
 return q

def wings(p,fear,rig_ready=False):
 sheet,paint=geometry_functions(rig_ready)
 start=len(p)
 for sign in (-1,1):
  depth=lambda y,x:-.02-.14*abs(x)+.028*math.sin(3*y)
  outline=contour([((.095,.56),(.21,.78),(.35,.98),(.57,1.08)),
    ((.57,1.08),(.70,1.15),(.85,1.26),(.94,1.30)),
    ((.94,1.30),(.89,1.13),(.74,.97),(.60,.84)),
    ((.60,.84),(.43,.68),(.29,.57),(.095,.56))],9)
  wing=sheet('Wing / curved rooted upper spar '+str(sign),BROWN,[(sign*x,y) for x,y in outline],depth,.033,2);p.append(wing)
  for j in range(8):
   root=(sign*(.21+.059*j),.66+.065*j);tip=(sign*(.43+.085*j),.50+.115*j)
   xy=vane_xy(root,tip,.065 if fear else .072)
   feather=sheet(f'Primary / layered flight vane {sign} {j}',LIGHT_BROWN if fear else CREAM,xy,feather_depth(depth,root,tip,.065 if fear else .072,.027,1),.019 if fear else .014,1);p.append(feather)
   rear=sheet(f'Rear primary / layered upper flight vane {sign} {j}',LIGHT_BROWN if fear else CREAM,xy,feather_depth(depth,root,tip,.065 if fear else .072,-.023,-1),.020 if fear else .014,1);p.append(rear)
   if fear:
    # A pale tip is fitted to both faces of the actual closed vane.
    pale_tip(p,feather,xy,root,tip,f'Primary tip / supported pale terminal {sign} {j}',rig_ready)
    pale_tip(p,rear,xy,root,tip,f'Rear primary tip / supported pale terminal {sign} {j}',rig_ready)
  for j in range(6):
   root=(sign*(.20+.073*j),.71+.068*j);tip=(sign*(.37+.076*j),.60+.071*j)
   xy=vane_xy(root,tip,.044)
   p.append(sheet(f'Covert / smaller layered shoulder vane {sign} {j}',LIGHT_BROWN if fear else BROWN,xy,feather_depth(depth,root,tip,.044,.037,1),.020 if fear else .014,1))
   p.append(sheet(f'Rear covert / smaller upper shoulder vane {sign} {j}',LIGHT_BROWN if fear else BROWN,xy,feather_depth(depth,root,tip,.044,-.029,-1),.023 if fear else .015,1))
 if not fear:
  for i in range(start,len(p)):
   side=-1 if sum(p[i]['positions'][::3])<0 else 1;p[i]=folded_wing(p[i],side)
   if p[i]['part'].startswith('Wing /'):
    from lugia_sculpt import paddle
    p[i]=paddle(p[i]['part'],BROWN,
      [(side*.145,.535,-.025),(side*.245,.555,-.030),(side*.275,.430,-.110),(side*.290,.285,-.220)],
      [.040,.055,.055,.018],[.060,.095,.110,.035],segments=18,steps=5)

def feet(p,fear=True):
 for s in (-1,1):
  p.append(curved_tube('Leg / rooted pink shin '+str(s),PINK,[(s*.092,.275,.018),(s*.105,.156,.020),(s*.108,.041,.041)] if fear else [(s*.108,.211,.020),(s*.108,.110,.020),(s*.108,.041,.020)],[.028,.021,.024],20))
  p.append(ellipsoid('Foot / grounded pink knuckle '+str(s),PINK,(s*.108,.023,.057),(.053,.023,.050),26,16))
  for j,dx in enumerate((-.039,0,.039)):
   end=(s*.108+dx,.015,.154 if j==1 else .135)
   p.append(curved_tube(f'Toe / three forward digits {s} {j}',PINK,[(s*.108+dx*.20,.031,.077),(s*.108+dx*.65,.023,.110),end],[.014,.012,.010],16))
   p.append(curved_tube(f'Talon / forward ivory hook {s} {j}',TALON,[(end[0],end[1],end[2]-.005),(end[0],.009,end[2]+.025),(end[0],.002,end[2]+.037)],[.010,.007,.0015],16))
  p.append(curved_tube('Toe / rear gripping digit '+str(s),PINK,[(s*.108,.031,.035),(s*.13,.019,-.025),(s*.14,.014,-.050)],[.013,.011,.008],16))
  p.append(curved_tube('Talon / rear ivory hook '+str(s),TALON,[(s*.14,.014,-.045),(s*.15,.008,-.068),(s*.15,.002,-.082)],[.008,.006,.0015],16))

def tails(p,fear,rig_ready=False):
 sheet,paint=geometry_functions(rig_ready)
 for j in range(5):
  x=(j-2)*.060
  xy=vane_xy((x*.20,.355),(x,.172+abs(j-2)*.012),.043)
  feather=sheet('Tail / splayed rear feather '+str(j),LIGHT_BROWN if fear else RED,xy,lambda y,x:-.145-2.25*(.355-y),.018,2)
  if not fear:
   for i in range(0,len(feather['positions']),3):
    y=feather['positions'][i+1];feather['positions'][i+1]=.30+(.355-y)*1.05
   # The tail's broad plane rises from the rump in Crystal's standing pose.
   from sculpt_geometry import part as geometry_part
   vs=list(zip(feather['positions'][::3],feather['positions'][1::3],feather['positions'][2::3]));faces=[tuple(feather['indices'][i:i+3]) for i in range(0,len(feather['indices']),3)]
   # Reflection in Y changes handedness. Rebuild the stored faces and normals.
   feather=geometry_part(feather['part'],RED,vs,[tuple(reversed(f)) for f in faces],True)
  p.append(feather)

def pidgeot_face(p,head,rig_ready=False):
 sheet,paint=geometry_functions(rig_ready)
 start=len(p);head=deepcopy(head)
 head['positions']=[v+(.125 if i%3==1 else -.045 if i%3==2 else 0.) for i,v in enumerate(head['positions'])]
 from pidgeotto_sculpt import frame,mark
 for sign in (-1,1):
  angle=sign*math.radians(53);local=frame(head,angle,True)
  uv=lambda points:[(sign*x,y+.17) for x,y in points]
  mask=mark('Mask / hooked charcoal cheek stripe '+str(sign),INK,uv(((-.050,.724),(.003,.754),(.084,.744),(.100,.713),(.077,.681),(.064,.622),(.037,.646),(.039,.694),(-.036,.688))),local,.006);p.append(frame(mask,angle))
  white=mark('Eye / angular fitted ivory '+str(sign),WHITE,uv(((-.038,.721),(.009,.737),(.065,.730),(.054,.704),(.005,.696),(-.031,.702))),mask,.004);p.append(frame(white,angle))
  pupil=mark('Pupil / restrained dark gaze '+str(sign),INK,oval(sign*.013,.889,.012,.013,16),white,.003);p.append(frame(pupil,angle))
  glint=mark('Glint / tiny ivory gaze '+str(sign),WHITE,oval(sign*.008,.893,.003,.004,12),pupil,.002);p.append(frame(glint,angle))
 upper=bill('Bill / short hooked pink upper beak',PINK,[(.160,.063,.824,.873),(.230,.079,.822,.877),(.304,.043,.803,.849),(.330,.003,.793,.808)]);p.append(upper)
 p.append(bill('Bill / dark short seam',INK,[(.162,.059,.819,.825),(.236,.060,.815,.821),(.299,.024,.805,.811),(.313,.002,.803,.809)]))
 p.append(bill('Bill / short pink lower beak',PINK,[(.160,.057,.782,.817),(.221,.060,.776,.814),(.287,.030,.795,.807),(.310,.002,.803,.807)]))
 from lugia_sculpt import paddle
 for j in (-2,-1,0,1,2):
  gold=j in (-2,0,2);side=-1 if j<0 else 1 if j>0 else 0
  if gold:
   x=side*.05;points=[(x*.5,.925,.145),(x*1.3,1.055,.095),(x*1.9,1.115-abs(side)*.025,-.175),(x*2.8,1.005-abs(side)*.025,-.645+abs(side)*.05)]
   widths=[.026,.052,.045,.0015];depths=[.025,.032,.027,.0015]
  else:
   x=side*.065;points=[(x*.6,.925,.145),(x*1.6,1.005,.075),(x*2.,1.025,-.205),(x*2.9,.935,-.625)]
   widths=[.023,.039,.032,.0015];depths=[.018,.023,.020,.0015]
  p.append(paddle('Crest / long swept ribbon '+str(j),GOLD if gold else RED,points,widths,depths,segments=12,steps=5))
 for q in p[start:]:q['positions']=[v+(-.125 if i%3==1 else .045 if i%3==2 else 0.) for i,v in enumerate(q['positions'])]

def fearow_face(p,head,rig_ready=False):
 sheet,paint=geometry_functions(rig_ready)
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.029,.986),(.085,.998),(.110,.977),(.083,.950),(.039,.953))]
  socket=paint('Eye socket / sharp dark bill-side gaze '+str(s),INK,xy,head,.009,3);p.append(socket)
  white=paint('Eye / sharp fitted ivory '+str(s),WHITE,[(s*x,y) for x,y in ((.036,.982),(.083,.990),(.099,.976),(.079,.957),(.044,.959))],socket,.006,3);p.append(white)
  pupil=paint('Pupil / small stern gaze '+str(s),INK,oval(s*.080,.974,.008,.014,20),white,.004,3);p.append(pupil)
 upper=bill('Bill / long tapering pink upper lance',PINK,[(.468,.047,.921,.960),(.566,.054,.898,.940),(.748,.026,.862,.888),(.854,.002,.836,.842)]);p.append(upper)
 p.append(bill('Bill / open lower pink mandible',PINK,[(.469,.038,.890,.916),(.530,.036,.858,.887),(.613,.025,.827,.846),(.667,.002,.856,.860)]))
 crown=[(-.092,.993),(-.103,1.105),(-.054,1.066),(-.040,1.202),(-.012,1.117),(.024,1.242),(.045,1.116),(.075,1.192),(.079,1.088),(.107,1.136),(.086,.999)]
 p.append(sheet('Crown / five pointed upright red feathers',RED,crown,lambda y,x:.403,.037,2))
 mane=[(-.065,.682),(-.125,.813),(-.087,.792),(-.100,.901),(-.056,.858),(-.032,.965),(.001,.885),(.038,.976),(.062,.867),(.100,.928),(.084,.790),(.130,.850),(.075,.680)]
 p.append(sheet('Mane / ragged pale rear neck ruff',CREAM,mane,lambda y,x:.094+.10*(y-.68),.030,2))

def sculpture(species,*,rig_ready=False,with_transform=False):
 if species=='fearow':
  from fearow_sculpt import sculpture as fearow
  return fearow(rig_ready=rig_ready,with_transform=with_transform)
 sheet,paint=geometry_functions(rig_ready)
 if species not in SPECIES:raise ValueError('unsupported raptor bird')
 fear=species=='fearow'
 if not fear:
  from sculpt_geometry import fitted_triangle_contour
  def paint(name,color,xy,parent,th=.006,sub=3):return fitted_triangle_contour(name,color,xy,parent,th,precision=6)
 rows=curved_profiles(((.188,.045,.045,-.018),(.265,.129,.145,-.025),(.405,.201,.189,-.020),(.525,.186,.167,-.005),(.620,.112,.103,.015),(.692,.039,.041,.036)),4)
 if not fear:rows=curved_profiles(((.132,.055,.080,-.040),(.222,.160,.195,-.050),(.382,.236,.250,-.065),(.512,.220,.233,-.035),(.620,.137,.136,.030),(.692,.070,.075,.061)),4)
 body=crease_normals(loft('Body / tapered individual bird breast',BROWN,rows,40),45);p=[body]
 if fear:
  neck=curved_tube('Neck / long bent narrow fishing neck',BROWN,[(0,.580,.030),(0,.687,.104),(0,.826,.255),(0,.940,.358)],[.077,.055,.043,.041],28)
  head=ellipsoid('Head / narrow long-billed cranium',LIGHT_BROWN,(0,.948,.400),(.126,.097,.105),40,26)
 else:
  neck=crease_normals(loft('Neck / broad crested eagle neck',BROWN,curved_profiles(((.555,.125,.118,.036),(.641,.113,.108,.052),(.739,.106,.104,.066)),4),32),45)
  head=ellipsoid('Head / rounded crested eagle cranium',BROWN,(0,.750,.070),(.166,.145,.154),40,26)
 p.extend((neck,head))
 if fear:
  xy=[(-.145,.548),(-.182,.458),(-.119,.478),(-.118,.408),(-.067,.446),(-.042,.373),(.005,.437),(.041,.385),(.073,.455),(.116,.416),(.112,.481),(.174,.458),(.140,.547)]
  p.append(paint('Breast / fitted ragged ochre feather bib',LIGHT_BROWN,xy,body,.011,3))
 else:
  xy=contour([((0,.223),(-.130,.247),(-.233,.382),(-.196,.534)),
    ((-.196,.534),(-.097,.640),(-.064,.642),(0,.651)),
    ((0,.651),(.064,.642),(.097,.640),(.196,.534)),
    ((.196,.534),(.233,.382),(.130,.247),(0,.223))],9)
  p.append(paint('Breast / fitted broad cream belly',CREAM,xy,body,.012,4))
  xy=[(-.058,.571),(-.077,.614),(-.080,.681),(-.050,.722),(.050,.722),(.080,.681),(.077,.614),(.058,.571)]
  p.append(paint('Throat / fitted continuous cream neck',CREAM,xy,neck,.011,4))
  xy=[(-.075,.655),(-.108,.689),(-.081,.739),(0,.772),(.081,.739),(.108,.689),(.075,.655),(0,.626)]
  p.append(paint('Cheek / fitted cream throat',CREAM,xy,head,.009,3))
 wings(p,fear,rig_ready);feet(p,fear);tails(p,fear,rig_ready)
 (fearow_face if fear else pidgeot_face)(p,head,rig_ready)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
  if not fear:
   q['positions']=list(struct.unpack('<'+str(len(q['positions']))+'f',struct.pack('<'+str(len(q['positions']))+'f',*q['positions'])))
   q.update(crease_normals(q,40.));q['normals']=[round(v,7) or 0. for v in q['normals']]
 model={'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
 return (model,(lo,scale)) if with_transform else model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--rig-ready',action='store_true');args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s,rig_ready=args.rig_ready),separators=(',',':')))
if __name__=='__main__':main()
