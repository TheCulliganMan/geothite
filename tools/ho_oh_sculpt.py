#!/usr/bin/env python3
"""Original rainbow phoenix with layered tricolor flight feathers and gold tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals
from raptor_bird_sculpt import sheet,paint,vane_xy,feather_depth
from bovine_sculpt import oval,reverse_front
from lugia_sculpt import paddle
from murkrow_sculpt import bill
from wing_insect_sculpt import contour
from bovine_sculpt import curved_tube
from skin_glb import COORDINATES
SPECIES=('ho_oh',);HEIGHT=1.45
RED=(.88,.27,.16);WARM=(.97,.38,.20);WHITE=(.97,.98,.88)
GREEN=(.58,.74,.27);GOLD=(.99,.72,.23);PALE=(1.,.90,.52)
NAVY=(.23,.25,.37);INK=(.065,.075,.10);IRIS=(.90,.28,.15)

def clip_along(xy,root,tip,t,above=True):
 dx,dy=tip[0]-root[0],tip[1]-root[1];dd=dx*dx+dy*dy
 def at(q):return ((q[0]-root[0])*dx+(q[1]-root[1])*dy)/dd
 out=[]
 for a,b in zip(xy,xy[1:]+xy[:1]):
  ta,tb=at(a),at(b);ia=(ta>=t) if above else (ta<=t);ib=(tb>=t) if above else (tb<=t)
  if ia:out.append(a)
  if ia!=ib:
   u=(t-ta)/(tb-ta);out.append((a[0]+u*(b[0]-a[0]),a[1]+u*(b[1]-a[1])))
 return out

def tip_colors(p,feather,xy,root,tip,label,tail=False):
 colors=((.68,PALE,'pale gold'),) if tail else ((.64,WHITE,'ivory band'),(.83,GREEN,'green tip'))
 terminal_outline=xy
 for t,color,name in colors:
  clipped=clip_along(terminal_outline,root,tip,t);cx=sum(x for x,y in clipped)/len(clipped);cy=sum(y for x,y in clipped)/len(clipped)
  inset=[(cx+.965*(x-cx),cy+.965*(y-cy)) for x,y in clipped]
  if not tail and t==.64:terminal_outline=inset
  for face in (-1,1):
   parent=feather if face==1 else reverse_front(feather)
   outline=inset if face==1 else [(-x,y) for x,y in inset]
   # Each green tip is supported by the actual ivory terminal layer.
   if not tail and t==.83:
    parent=next(q for q in p if q['part']==f'Flight color / {label} ivory band {face}')
    if face==-1:parent=reverse_front(parent)
   q=paint(f'{"Tail color" if tail else "Flight color"} / {label} {name} {face}',color,outline,parent,.004,2)
   p.append(q if face==1 else reverse_front(q))

def phoenix_vane(root,tip,width):
 dx,dy=tip[0]-root[0],tip[1]-root[1];length=math.hypot(dx,dy);nx,ny=-dy/length*width,dx/length*width
 def at(t,w):return (root[0]+t*dx+w*nx,root[1]+t*dy+w*ny)
 return contour([(at(0,-.30),at(.25,-.95),at(.76,-1),at(1,-.48)),
                 (at(1,-.48),at(1.07,-.33),at(1.07,.33),at(1,.48)),
                 (at(1,.48),at(.84,.90),at(.29,.88),at(0,.30)),
                 (at(0,.30),at(-.07,.20),at(-.07,-.20),at(0,-.30))],5)

def wing_depth(y,x):return -.022-.10*abs(x)+.023*math.sin((y-.65)*3)

def wings(p):
 for s in (-1,1):
  outline=[(.105,.650),(.24,.865),(.46,1.052),(.70,1.397),(.80,1.602),(.86,1.614),(.79,1.329),(.59,1.04),(.37,.77),(.17,.625)]
  wing=sheet('Wing / curved red rising shoulder '+str(s),RED,[(s*x,y) for x,y in outline],wing_depth,.043,3);p.append(wing)
  for j in range(12):
   root=(s*(.185+.048*j),.752+.063*j);tip=(s*(.450+.065*j),.530+.096*j)
   xy=phoenix_vane(root,tip,.069)
   feather=sheet(f'Flight / separate broad tricolor primary {s} {j}',RED,xy,feather_depth(wing_depth,root,tip,.069,.018,1),.025,2);p.append(feather)
   tip_colors(p,feather,xy,root,tip,f'{s} {j}')
  for j in range(6):
   root=(s*(.17+.075*j),.776+.083*j);tip=(s*(.30+.073*j),.685+.077*j)
   p.append(sheet(f'Covert / warm overlapping shoulder vane {s} {j}',WARM,vane_xy(root,tip,.036),feather_depth(wing_depth,root,tip,.036,.040,1),.018,2))

def tail(p):
 for j in range(9):
  root=((j-4)*.009,.345);tip=((j-4)*.087,.075+abs(j-4)*.013)
  xy=vane_xy(root,tip,.062)
  depth=lambda y,x:-.123-2.85*(.345-y)+.025*abs(x)
  feather=sheet('Tail / broad splayed golden phoenix vane '+str(j),GOLD,xy,depth,.026,2);p.append(feather)
  tip_colors(p,feather,xy,root,tip,str(j),True)

def feet(p):
 for s in (-1,1):
  p.append(ellipsoid('Haunch / rooted ivory feather thigh '+str(s),WHITE,(s*.115,.295,.025),(.099,.143,.112),32,22))
  p.append(curved_tube('Leg / rooted dark phoenix shin '+str(s),NAVY,[(s*.115,.22,.027),(s*.125,.12,.043),(s*.128,.052,.074)],[.044,.039,.041],24))
  p.append(ellipsoid('Foot / grounded dark phoenix knuckle '+str(s),NAVY,(s*.128,.035,.094),(.073,.035,.062),32,20))
  for j,dx in enumerate((-.052,0,.052)):
   end=(s*.128+dx,.022,.208 if j==1 else .184)
   p.append(curved_tube(f'Toe / three forward phoenix digits {s} {j}',NAVY,[(s*.128+dx*.30,.044,.116),(s*.128+dx*.70,.028,.154),end],[.024,.022,.017],20))
   p.append(curved_tube(f'Talon / forward ivory phoenix hook {s} {j}',WHITE,[(end[0],end[1],end[2]-.010),(end[0],.014,end[2]+.025),(end[0],.002,end[2]+.038)],[.017,.012,.002],20))
  p.append(curved_tube('Toe / rear gripping phoenix digit '+str(s),NAVY,[(s*.128,.044,.064),(s*.145,.024,-.014),(s*.159,.02,-.060)],[.023,.018,.013],20))
  p.append(curved_tube('Talon / rear ivory phoenix hook '+str(s),WHITE,[(s*.159,.02,-.052),(s*.165,.012,-.085),(s*.165,.002,-.101)],[.013,.010,.002],20))

def sculpture(species='ho_oh'):
 if species not in SPECIES:raise ValueError('unsupported rainbow phoenix')
 body=crease_normals(loft('Body / tapered red phoenix breast',RED,curved_profiles(((.185,.025,.033,-.025),(.29,.153,.150,-.020),(.46,.202,.177,-.007),(.65,.164,.137,.01),(.78,.086,.079,.023),(.827,.035,.043,.043)),5),44),45)
 neck=paddle('Neck / long curved red phoenix throat',RED,[(0,.74,.025),(0,.91,.085),(0,1.045,.163),(0,1.14,.24)],[.077,.066,.060,.064],[.081,.068,.062,.064])
 head=ellipsoid('Head / compact red phoenix cranium',RED,(0,1.18,.256),(.123,.129,.125),44,28);p=[body,neck,head]
 xy=[(0,.696),(.075,.710),(.151,.629),(.181,.49),(.146,.326),(.077,.230),(0,.219),(-.077,.230),(-.146,.326),(-.181,.49),(-.151,.629),(-.075,.710)]
 p.append(paint('Belly / fitted broad ivory phoenix breast',WHITE,xy,body,.010,4))
 # Neck bands follow the actual curved throat on both visible sides.
 for face in (-1,1):
  parent=neck if face==1 else reverse_front(neck)
  xy=[(-.045,.979),(-.041,1.010),(.041,1.010),(.045,.979)]
  q=paint('Collar / fitted green rainbow band '+str(face),GREEN,xy,parent,.006,3);p.append(q if face==1 else reverse_front(q))
  xy=[(-.042,.970),(-.044,.978),(.044,.978),(.042,.970)]
  q=paint('Collar edge / fitted ivory throat ring '+str(face),WHITE,xy,parent,.006,3);p.append(q if face==1 else reverse_front(q))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.028,1.22),(.078,1.246),(.109,1.212),(.088,1.177),(.045,1.177),(.020,1.194))]
  socket=paint('Eye socket / fitted dark phoenix stare '+str(s),INK,xy,head,.008,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory phoenix lens '+str(s),WHITE,[(cx+.78*(x-cx),cy+.72*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  iris=paint('Iris / restrained red phoenix gaze '+str(s),IRIS,oval(s*.072,1.21,.010,.016,20),eye,.004,3);p.append(iris)
  p.append(paint('Pupil / small dark phoenix center '+str(s),INK,oval(s*.072,1.21,.005,.011,20),iris,.003,2))
 # Five broad rounded gold crest feathers have physical roots in the skull.
 for j in range(5):
  x=(j-2)*.043;y=1.365+.048*(1-abs(j-2)/2)
  p.append(paddle('Crown / rounded golden crest petal '+str(j),GOLD,[(x*.55,1.272,.22),(x*.80,1.32,.183),(x,y,.145),(x,y+.018,.14)],[.024,.035,.028,.009],[.021,.024,.022,.011]))
 p.append(bill('Jaw / long curved gold upper phoenix beak',GOLD,[(.321,.055,1.15,1.198),(.432,.067,1.155,1.223),(.548,.040,1.209,1.249),(.605,.0015,1.218,1.227)]))
 p.append(bill('Jaw / open gold lower phoenix beak',GOLD,[(.319,.052,1.087,1.135),(.398,.058,1.062,1.107),(.492,.037,1.098,1.12),(.547,.0015,1.136,1.144)]))
 p.append(ellipsoid('Mouth / recessed charcoal phoenix oral lining',INK,(0,1.15,.385),(.044,.039,.013),32,22))
 wings(p);tail(p);feet(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'ho_oh.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
