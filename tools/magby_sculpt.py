#!/usr/bin/env python3
"""Original small live-coal creature with lobed crown and a plain pointed tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,part
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from sculpt_geometry import fitted_adaptive_contour
from goldfish_sculpt import surface
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES
RED=(.85,.43,.40);GOLD=(.97,.88,.45);IVORY=(.98,.97,.92);INK=(.08,.09,.11)
HEIGHT=.77

def paint(name,color,xy,parent,th=.006,sub=3):
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)

def arm_path(s):
 return [(s*.077,.508,.003),(s*.223,.464,.025),(s*.321,.474,.046)]

def bill():
 # A closed annular taper with an open front aperture and recessed head behind.
 rings=[(.126,.697,.110,.066),(.199,.686,.087,.052),(.259,.679,.045,.035),(.273,.679,.034,.027),(.255,.679,.027,.021),(.196,.679,.027,.021),(.143,.679,.027,.021)]
 n=48;vv=[]
 for z,y,rx,ry in rings:vv.extend((rx*math.cos(math.tau*i/n),y+ry*math.sin(math.tau*i/n),z) for i in range(n))
 ff=[]
 for j in range(len(rings)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;ff.append((a,b,b+n,a+n))
 for i in range(n):ff.append((i,(len(rings)-1)*n+i,(len(rings)-1)*n+(i+1)%n,(i+1)%n))
 return crease_normals(part('Bill / rooted yellow live-coal taper with open mouth',GOLD,vv,ff,True),45)

def sculpture(normalize=True):
 body=crease_normals(loft('Body / compact red live-coal pear',RED,curved_profiles(((.157,.075,.073,-.006),(.238,.183,.137,-.006),(.355,.179,.145,-.006),(.460,.119,.103,.000),(.548,.065,.060,.020)),4),40),45)
 neck=loft('Collar / rooted dark live-coal neck wrap',INK,curved_profiles(((.513,.072,.065,.020),(.560,.081,.071,.023),(.598,.076,.066,.024)),3),40)
 head=crease_normals(loft('Head / broad red live-coal face',RED,curved_profiles(((.545,.094,.092,.020),(.605,.167,.148,.020),(.715,.204,.180,.020),(.810,.218,.178,.016),(.872,.188,.146,.010),(.912,.098,.090,.010)),4),40),45);p=[body,neck,head]
 # Overlapping lobes form a low cloud crown, rather than three floating balls.
 for j,(x,y,z,rx,ry,rz) in enumerate(((-.106,.891,.073,.128,.110,.145),(.106,.886,.077,.133,.114,.146),(-.139,.904,-.068,.102,.105,.119),(.128,.915,-.073,.110,.107,.122),(0,.941,-.021,.118,.108,.143))):
  p.append(ellipsoid('Crown lobe / rooted rounded live-coal cloud '+str(j),RED,(x,y,z),(rx,ry,rz),28,18))
 p.append(paint('Belly / fitted pointed yellow live-coal bib',GOLD,[(-.116,.268),(-.126,.339),(-.080,.433),(.028,.528),(.014,.438),(.105,.379),(.120,.310),(.083,.213),(-.060,.205)],body,.005,3))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.035,.792),(.102,.806),(.150,.771),(.143,.675),(.102,.652),(.047,.682))]
  socket=paint('Eye socket / fitted dark live-coal recess '+str(s),INK,xy,head,.005,3);p.append(socket);cx=cy=0.
  for x,y in xy:cx+=x;cy+=y
  cx/=len(xy);cy/=len(xy)
  eye=paint('Eye / fitted tall ivory live-coal gaze '+str(s),IVORY,[(cx+.88*(x-cx),cy+.90*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  pupil=paint('Pupil / fitted broad live-coal stare '+str(s),INK,oval(s*.092,.731,.018,.038,24),eye,.003,2);p.append(pupil)
  p.append(paint('Glint / fitted small ivory live-coal spark '+str(s),IVORY,oval(s*.087,.744,.004,.008,20),pupil,.002,1))
  p.append(paddle('Arm / rooted low red live-coal limb '+str(s),RED,arm_path(s),[.047,.054,.045],[.044,.049,.043],segments=24,steps=4))
  hand=ellipsoid('Hand / rooted red live-coal claw junction '+str(s),RED,(s*.350,.487,.059),(.060,.043,.056),28,18);p.append(hand)
  for j,(dx,dy,dz) in enumerate(((-.024,.015,.040),(.017,.025,.027),(.044,.006,-.008))):
   start=(s*(.350+dx),.487+dy,.059+dz);end=(start[0]+s*.025,start[1]+.031,start[2]+.016)
   p.append(curved_tube(f'Hand claw / rooted ivory live-coal point {s} {j}',IVORY,[start,end],[.017,.0015],20))
  p.append(paddle('Leg / rooted red live-coal shin '+str(s),RED,[(s*.113,.234,.014),(s*.143,.134,.014),(s*.146,.055,.014)],[.056,.042,.037],[.052,.043,.040],segments=24,steps=4))
  p.append(ellipsoid('Foot / grounded broad red live-coal support '+str(s),RED,(s*.148,.040,.056),(.080,.040,.101),28,18))
  for j,dx in enumerate((-.041,0.,.041)):
   x=s*.148+dx;p.append(curved_tube(f'Toe claw / rooted ivory live-coal point {s} {j}',IVORY,[(x,.035,.116),(x,.024,.165),(x,.012,.185)],[.018,.012,.0015],20))
 p.append(curved_tube('Tail / rooted plain pointed red live-coal balance',RED,[(0,.283,-.101),(.062,.256,-.259),(.143,.307,-.383),(.194,.370,-.451)],[.057,.051,.030,.002],28))
 p.append(bill());p.append(ellipsoid('Mouth back / recessed dark live-coal oral chamber',INK,(0,.679,.190),(.030,.024,.025),28,18))
 for i,q in enumerate(p):
  name=q['part'];side=name.split()[-1];parent=None
  for child,owner in (('Belly /','Body /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /'),('Glint /','Pupil /')):
   if name.startswith(child):parent=next(a for a in p if a['part'].startswith(owner) and (owner in ('Body /','Head /') or a['part'].endswith(' '+side)))
  p[i]=skin_shading(q,parent) if parent else crease_normals(q,45.)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Collar /','Eye socket /','Pupil /','Mouth back /')) else (255,255,255) if n.startswith(('Eye /','Glint /','Hand claw /','Toe claw /')) else (255,148,74) if n.startswith(('Bill /','Belly /')) else (189,16,90)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'magby','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'magby.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
