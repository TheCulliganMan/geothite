#!/usr/bin/env python3
"""Original small live-coal creature with lobed crown and a plain pointed tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,part
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
RED=(.85,.43,.40);GOLD=(.97,.88,.45);IVORY=(.98,.97,.92);INK=(.08,.09,.11)
HEIGHT=.77

def bill():
 # A closed annular taper with an open front aperture and recessed head behind.
 rings=[(.126,.723,.137,.080),(.205,.712,.107,.062),(.278,.705,.047,.038),(.290,.705,.034,.027),(.271,.705,.027,.021),(.196,.705,.027,.021),(.143,.705,.027,.021)]
 n=48;vv=[]
 for z,y,rx,ry in rings:vv.extend((rx*math.cos(math.tau*i/n),y+ry*math.sin(math.tau*i/n),z) for i in range(n))
 ff=[]
 for j in range(len(rings)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;ff.append((a,b,b+n,a+n))
 for i in range(n):ff.append((i,(len(rings)-1)*n+i,(len(rings)-1)*n+(i+1)%n,(i+1)%n))
 return crease_normals(part('Bill / rooted yellow live-coal taper with open mouth',GOLD,vv,ff,True),45)

def sculpture():
 body=ellipsoid('Body / compact red live-coal pear',RED,(0,.370,-.006),(.174,.205,.136),44,28)
 neck=loft('Collar / rooted dark live-coal neck wrap',INK,curved_profiles(((.513,.072,.065,.020),(.560,.081,.071,.023),(.598,.076,.066,.024)),3),40)
 head=ellipsoid('Head / broad red live-coal face',RED,(0,.750,.020),(.220,.179,.171),48,30);p=[body,neck,head]
 # Overlapping lobes form a low cloud crown, rather than three floating balls.
 for j,(x,y,z,rx,ry,rz) in enumerate(((-.111,.876,.067,.123,.109,.131),(.110,.869,.074,.129,.117,.136),(-.140,.892,-.073,.096,.105,.107),(.128,.909,-.070,.103,.106,.115),(0,.935,-.020,.116,.111,.128))):
  p.append(ellipsoid('Crown lobe / rooted rounded live-coal cloud '+str(j),RED,(x,y,z),(rx,ry,rz),40,26))
 p.append(paint('Belly / fitted pointed yellow live-coal bib',GOLD,[(-.116,.268),(-.126,.339),(-.080,.433),(.028,.528),(.014,.438),(.105,.379),(.120,.310),(.083,.213),(-.060,.205)],body,.005,4))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.037,.821),(.122,.832),(.151,.773),(.139,.698),(.099,.680),(.057,.712))]
  socket=paint('Eye socket / fitted dark live-coal recess '+str(s),INK,xy,head,.005,3);p.append(socket);cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted tall ivory live-coal gaze '+str(s),IVORY,[(cx+.88*(x-cx),cy+.90*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  pupil=paint('Pupil / fitted narrow live-coal stare '+str(s),INK,oval(s*.090,.754,.008,.033,28),eye,.003,3);p.append(pupil)
  p.append(paint('Glint / fitted small ivory live-coal spark '+str(s),IVORY,oval(s*.087,.766,.002,.006,20),pupil,.002,2))
  path=[(s*.084,.504,.004),(s*.200,.573,.010),(s*.310,.672,.014),(s*.394,.747,.037)]
  p.append(paddle('Arm / rooted raised red live-coal limb '+str(s),RED,path,[.045,.056,.053,.046],[.044,.054,.053,.045]))
  hand=ellipsoid('Hand / rooted red live-coal claw junction '+str(s),RED,(s*.411,.766,.047),(.065,.069,.052),36,24);p.append(hand)
  for j,(dx,dy,dz) in enumerate(((-.037,.037,.015),(.016,.061,.008),(.050,.016,.016))):
   start=(s*(.411+dx),.766+dy,.047+dz);end=(start[0]+s*.010,start[1]+.047,start[2]+.022)
   p.append(curved_tube(f'Hand claw / rooted ivory live-coal point {s} {j}',IVORY,[start,end],[.017,.0015],24))
  p.append(paddle('Leg / rooted red live-coal shin '+str(s),RED,[(s*.090,.245,-.005),(s*.132,.157,.034),(s*.135,.047,.054)],[.049,.038,.031],[.047,.039,.032]))
  p.append(ellipsoid('Foot / grounded small red live-coal support '+str(s),RED,(s*.134,.029,.079),(.064,.029,.075),36,24))
  for j,dx in enumerate((-.027,.027)):
   x=s*.134+dx;p.append(curved_tube(f'Toe claw / rooted ivory live-coal point {s} {j}',IVORY,[(x,.031,.125),(x,.019,.174),(x,.010,.196)],[.018,.012,.0015],24))
 p.append(curved_tube('Tail / rooted plain pointed red live-coal balance',RED,[(0,.300,-.090),(.075,.286,-.245),(.161,.364,-.376),(.225,.436,-.428)],[.054,.051,.031,.002],32))
 p.append(bill());p.append(ellipsoid('Mouth back / recessed dark live-coal oral chamber',INK,(0,.705,.195),(.030,.024,.023),32,22))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'magby','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'magby.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
