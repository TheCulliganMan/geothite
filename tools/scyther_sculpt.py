#!/usr/bin/env python3
"""Original green mantis with curved ivory scythes, wing cases and clawed greaves."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint,sheet
from wing_insect_sculpt import contour
from abra_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.18
GREEN=(.48,.70,.31);DARK=(.25,.43,.23);CREAM=(.91,.88,.63)
IVORY=(.95,.97,.94);WING=(.97,.95,.79);VEIN=(.57,.65,.58);INK=(.075,.10,.07)

def scythe(side):
 cx=.285;cy=.790
 xy=contour([((-.015,-.035),(.008,.008),(.103,.037),(.177,.041)),
  ((.177,.041),(.286,.057),(.360,.008),(.369,-.082)),
  ((.369,-.082),(.396,-.180),(.385,-.284),(.352,-.390)),
  ((.352,-.390),(.316,-.292),(.258,-.234),(.193,-.211)),
  ((.193,-.211),(.177,-.136),(.121,-.076),(.055,-.061)),
  ((.055,-.061),(.022,-.060),(-.002,-.047),(-.015,-.035))],8)
 def world(points):return [(side*(cx+x),cy+y) for x,y in points]
 blade=sheet('Scythe / long curved ivory mantis blade '+str(side),IVORY,world(xy),lambda y,x:.089+.018*math.sin(math.pi*max(0.,min(1.,(side*x-cx)/.40))),.023,1)
 edge=[]
 for x,y in xy[:23]:
  dx=.18-x;dy=-.15-y;l=math.hypot(dx,dy);edge.append((x+.0012*dx/l,y+.0012*dy/l))
 inner=[]
 for i,(x,y) in enumerate(edge):
  dx=.18-x;dy=-.15-y;l=math.hypot(dx,dy);w=.013*math.sin(math.pi*(i+.6)/(len(edge)+.2))**.5
  inner.append((x+dx*w/l,y+dy*w/l))
 rim=paint('Scythe rim / fitted green mantis blade spine '+str(side),GREEN,world(edge+list(reversed(inner))),blade,.003,2)
 return [blade,rim]

@lru_cache(maxsize=1)
def raw_sculpture():
 thorax=ellipsoid('Thorax / rounded green mantis breast',GREEN,(0.,.823,.010),(.145,.160,.120),28,18)
 waist=loft('Waist / slender pale mantis stalk',CREAM,curved_profiles(((.53,.050,.045,-.02),(.64,.053,.047,-.01),(.75,.065,.055,.01),(.80,.074,.060,.015)),4),24)
 abdomen=loft('Abdomen / rounded tapered green mantis shell',GREEN,curved_profiles(((.286,.030,.027,-.06),(.34,.095,.075,-.065),(.46,.146,.114,-.060),(.58,.138,.104,-.04),(.64,.081,.060,-.025),(.65,.035,.030,-.02)),4),32)
 head=loft('Head / angular green mantis mask and tapered muzzle',GREEN,curved_profiles(((.963,.018,.024,.115),(1.005,.068,.067,.090),(1.075,.124,.088,.047),(1.145,.140,.080,.0),(1.190,.088,.052,-.021),(1.205,.016,.012,-.03)),4),28)
 p=[thorax,waist,abdomen,head]
 p.append(curved_tube('Neck / short pale green mantis throat',CREAM,[(0.,.924,.027),(0.,.977,.053),(0.,1.018,.076)],[.041,.034,.032],20))
 p.append(paddle('Crest / slender backward mantis antenna',DARK,[(0.,1.146,-.057),(0.,1.285,-.092),(.004,1.442,-.122)],[.024,.014,.0018],[.018,.011,.0018],segments=16,steps=4))
 for side in (-1,1):
  p.append(paddle('Crest / swept green mantis cranial point '+str(side),GREEN,[(side*.070,1.136,-.004),(side*.141,1.205,-.009),(side*.207,1.246,-.020)],[.045,.034,.002],[.029,.021,.002],segments=20,steps=4))
  x=side*.075;y=1.080
  xy=[(x+side*u,y+v) for u,v in ((-.041,.012),(.035,.038),(.033,-.021),(-.028,-.031))]
  rim=paint('Eye socket / fitted stern green mantis brow '+str(side),DARK,xy,head,.005,3);p.append(rim)
  xy=[(x+.85*(u-x),y+.85*(v-y)) for u,v in xy]
  eye=paint('Eye / fitted ivory mantis lens '+str(side),IVORY,xy,rim,.003,2);p.append(eye)
  p.append(paint('Pupil / fitted narrow black mantis slit '+str(side),INK,oval(x+side*.005,y-.002,.006,.019,16),eye,.0025,2))
  p.append(paint('Nostril / tiny fitted mantis muzzle pore '+str(side),DARK,oval(side*.027,1.005,.003,.0025,12),head,.002,1))
  p.append(ellipsoid('Shoulder / large rounded green mantis armor '+str(side),GREEN,(side*.151,.850,.005),(.096,.100,.091),28,18))
  p.append(curved_tube('Upper arm / slender pale mantis elbow stalk '+str(side),CREAM,[(side*.194,.835,.016),(side*.244,.807,.034),(side*.304,.780,.092)],[.026,.023,.024],16))
  p += scythe(side)
  p.append(ellipsoid('Haunch / teardrop green mantis coxa '+str(side),GREEN,(side*.142,.453,-.027),(.085,.124,.085),28,18))
  p.append(paddle('Haunch thorn / pointed green upper leg armor '+str(side),GREEN,[(side*.165,.490,-.027),(side*.184,.585,-.035)],[.048,.002],[.037,.002],segments=16,steps=3))
  p.append(curved_tube('Shank / slender pale mantis knee stalk '+str(side),CREAM,[(side*.171,.384,-.006),(side*.195,.299,.019),(side*.217,.249,.040)],[.035,.028,.026],16))
  p.append(paddle('Greave / flared green mantis lower leg armor '+str(side),GREEN,[(side*.214,.278,.034),(side*.229,.145,.080),(side*.236,.060,.129)],[.032,.076,.070],[.027,.049,.045],segments=24,steps=4))
  p.append(ellipsoid('Foot / planted broad green mantis pad '+str(side),GREEN,(side*.237,.035,.128),(.073,.035,.095),24,16))
  for j,dx in enumerate((-.052,0.,.052)):
   p.append(curved_tube(f'Foot claw / three pointed ivory mantis nails {side} {j}',IVORY,[(side*.237+dx,.042,.181),(side*.238+dx*1.12,.031,.219+(j==1)*.020),(side*.240+dx*1.13,.012,.247+(j==1)*.022)],[.021,.016,.0018],16))
  for j,(root,tip,w) in enumerate((((side*.071,.861,-.080),(side*.475,1.175,-.157),.065),((side*.072,.831,-.085),(side*.432,.985,-.177),.050))):
   mid=tuple((a+b)/2 for a,b in zip(root,tip));wing=paddle(f'Wing case / four green-rimmed mantis leaves {side} {j}',GREEN,[root,mid,tip],[.016,w,.005],[.010,.012,.004],segments=20,steps=5);p.append(wing)
   dx=tip[0]-root[0];dy=tip[1]-root[1];length=math.hypot(dx,dy);nx=-dy/length;ny=dx/length
   def at(t,v):return (root[0]+dx*t+nx*v,root[1]+dy*t+ny*v)
   xy=contour([(at(.035,-.008),at(.20,-w*.62),at(.63,-w*.80),at(.965,0.)),(at(.965,0.),at(.63,w*.80),at(.20,w*.62),at(.035,.008)),(at(.035,.008),at(.02,.005),at(.02,-.005),at(.035,-.008))],8)
   membrane=paint(f'Wing / fitted pale mantis membrane {side} {j}',WING,xy,wing,.003,2)
   front=surface(wing);back=surface(reverse_front(wing))
   for k in range(0,len(membrane['positions']),3):
    x,y,z=membrane['positions'][k:k+3]
    if z<front(y,x):membrane['positions'][k+2]=-back(y,-x)-.003
   membrane=crease_normals(membrane,35.);p.append(membrane)
   skin=surface(membrane);vein=[]
   for k in range(13):
    t=.08+.82*k/12;x,y=at(t,0.);vein.append((x,y,skin(y,x)+.001))
   p.append(tube(f'Wing vein / fitted central gray mantis rib {side} {j}',VEIN,vein,[.0015]*len(vein),8))
 p.append(paint('Breast plate / fitted pale mantis throat shield',CREAM,[(0.,.735),(.044,.805),(.039,.924),(-.039,.924),(-.044,.805)],thorax,.005,3))
 af=surface(abdomen)
 for j,(y,w) in enumerate(((.366,.109),(.456,.140),(.548,.133))):
  pts=[]
  for k in range(25):
   x=-w+2*w*k/24;yy=y+.009*(x/w)**2;pts.append((x,yy,af(yy,x)+.0015))
  p.append(tube('Abdomen seam / three fitted green mantis shell divisions '+str(j),DARK,pts,[.0015]*len(pts),8))
 hs=surface(head);pts=[]
 for i in range(19):
  x=-.045+.090*i/18;y=.994+.006*(abs(x)/.045)**2;pts.append((x,y,hs(y,x)+.0015))
 p.append(tube('Mouth / fitted restrained mantis muzzle seam',DARK,pts,[.0015]*len(pts),8))
 return {'name':'scyther','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from scyther_glb import export_scyther
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_scyther())
if __name__=='__main__':main()
