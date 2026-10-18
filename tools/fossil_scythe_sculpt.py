#!/usr/bin/env python3
"""Original domed four-claw fossil and swept-head, paired-scythe adult sculptures."""
import argparse
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals
from bovine_sculpt import curved_tube,oval,reverse_front
from lugia_sculpt import paddle
from murkrow_sculpt import bill
from fire_dragon_sculpt import fitted
from skin_glb import COORDINATES
SPECIES=('kabuto','kabutops');HEIGHTS={'kabuto':.60,'kabutops':1.20}
BROWN=(.65,.45,.20);RIDGE=(.72,.53,.27);DARK=(.19,.14,.105);INK=(.025,.022,.020);IVORY=(.96,.93,.82);RED=(.80,.19,.13)
TAIL=((0.,.36,-.12),(0.,.29,-.30),(0.,.20,-.48),(0.,.22,-.67))
def small_leg(side,j):
 z=(-.11,.13)[j];return ((side*.22,.14,z),(side*.34,.095,z+.025),(side*.36,.033,z+.10))
def adult_arm(side):return ((side*.14,.77,-.005),(side*.28,.66,.045),(side*.39,.78,.10))
def scythe(side):return ((side*.39,.78,.10),(side*.52,.96,.12),(side*.68,.75,.17),(side*.60,.48,.27),(side*.44,.31,.36))
def kabuto():
 shell=crease_normals(loft('Shell / broad domed Kabuto carapace',BROWN,curved_profiles(((.145,.30,.27,-.02),(.185,.365,.31,-.02),(.27,.35,.29,-.02),(.36,.27,.23,-.02),(.425,.16,.145,-.025),(.45,.012,.012,-.025)),4),40),35.)
 underside=ellipsoid('Underside / dark recessed Kabuto face',INK,(0.,.126,.025),(.29,.100,.255),32,20);p=[shell,underside]
 for side in (-1,1):
  eye=fitted('Eye lens / fitted pale Kabuto underside lens '+str(side),IVORY,oval(side*.115,.145,.040,.047,24),underside,.005);p.append(eye)
  iris=fitted('Iris / fitted red Kabuto underside gaze '+str(side),RED,oval(side*.115,.146,.021,.032,20),eye,.0025);p.append(iris)
  p.append(fitted('Pupil / fitted dark Kabuto gaze slit '+str(side),INK,oval(side*.115,.147,.006,.022,16),iris,.0015))
  # Two dark optic pits sit on the brown carapace as well as the underside gaze.
  p.append(fitted('Shell pore / fitted black Kabuto optic pit '+str(side),INK,oval(side*.195,.315,.020,.014,24),shell,.003))
  for j in range(2):
   pts=small_leg(side,j);p.append(curved_tube(f'Leg / four rooted Kabuto claw stems {side} {j}',BROWN,pts,[.047,.039,.022],18))
   x,y,z=pts[-1];p.append(ellipsoid(f'Foot / four planted Kabuto claw roots {side} {j}',BROWN,(x,.021,z),(.030,.021,.039),20,12))
   p.append(paddle(f'Claw / four hooked ivory Kabuto tips {side} {j}',IVORY,[(x,.031,z),(x-side*.012,.025,z+.055),(x-side*.042,.012,z+.092)],[.027,.020,.002],[.019,.013,.0015],segments=14,steps=4))
 # A fitted dark rear inset keeps the carapace readable from behind.
 p.append(reverse_front(fitted('Rear inset / supported dark Kabuto horseshoe',DARK,oval(0.,.26,.195,.085,32),reverse_front(shell),.0035)))
 return p

def kabutops():
 body=crease_normals(loft('Body / tapered Kabutops armored torso',BROWN,curved_profiles(((.265,.05,.06,-.045),(.36,.15,.13,-.04),(.50,.17,.14,-.035),(.64,.19,.14,-.025),(.77,.155,.11,-.015),(.86,.07,.065,.005)),4),28),32.)
 neck=loft('Neck / narrow Kabutops armored neck',DARK,curved_profiles(((.75,.065,.06,.005),(.91,.07,.065,.045),(.99,.06,.055,.06)),3),24)
 head=bill('Head / broad angular Kabutops hammer shell',BROWN,[(-.13,.13,.92,1.11),(-.02,.27,.86,1.13),(.12,.29,.85,1.075),(.27,.18,.88,.995),(.32,.035,.915,.96)]);p=[body,neck,head]
 for side in (-1,1):
  p.append(paddle('Head wing / two swept Kabutops crown blades '+str(side),BROWN,[(side*.19,1.025,-.035),(side*.34,1.13,-.09),(side*.49,1.285,-.18)],[.105,.092,.002],[.070,.045,.0015],segments=16,steps=4))
  socket=fitted('Eye socket / fitted narrow Kabutops glare '+str(side),INK,[(side*x,y) for x,y in ((.085,.941),(.205,.961),(.20,1.015),(.10,.976))],head,.005);p.append(socket)
  white=fitted('Eye lens / fitted ivory Kabutops glare '+str(side),IVORY,[(side*x,y) for x,y in ((.102,.950),(.188,.966),(.183,.999),(.113,.975))],socket,.0025);p.append(white)
  p.append(fitted('Pupil / fitted black Kabutops slit '+str(side),INK,oval(side*.14,.968,.005,.008,20),white,.002))
  pts=adult_arm(side);p.append(curved_tube('Upper arm / rooted Kabutops shoulder '+str(side),BROWN,pts[:2],[.060,.045],22));p.append(ellipsoid('Elbow / Kabutops elbow hinge '+str(side),DARK,pts[1],(.049,.047,.047),20,12))
  p.append(curved_tube('Forearm / raised Kabutops scythe carrier '+str(side),BROWN,pts[1:],[.042,.037],22));p.append(ellipsoid('Wrist / Kabutops scythe root '+str(side),BROWN,pts[-1],(.050,.050,.046),20,12))
  p.append(paddle('Scythe / broad swept ivory Kabutops blade '+str(side),IVORY,scythe(side),[.040,.055,.075,.045,.0015],[.030,.032,.026,.017,.0012],segments=18,steps=4))
  p.append(ellipsoid('Thigh / powerful Kabutops hind limb '+str(side),BROWN,(side*.145,.28,-.035),(.095,.17,.105),24,16))
  p.append(curved_tube('Shin / angled Kabutops lower leg '+str(side),BROWN,[(side*.16,.25,.0),(side*.19,.13,.075),(side*.19,.052,.15)],[.060,.048,.035],20))
  p.append(ellipsoid('Foot / planted Kabutops toe junction '+str(side),BROWN,(side*.19,.034,.17),(.078,.034,.105),24,14))
  for j,dx in enumerate((-.048,0.,.048)):
   p.append(paddle(f'Toe / three pointed ivory Kabutops talons {side} {j}',IVORY,[(side*.19+dx*.7,.035,.225),(side*.19+dx,.025,.285),(side*.19+dx*1.2,.012,.335 if j==1 else .315)],[.026,.020,.0015],[.019,.013,.0012],segments=12,steps=4))
 for j,(y,w) in enumerate(((.36,.115),(.445,.145),(.535,.153),(.625,.155),(.713,.123))):
  xy=[(-w,y+.027),(0.,y+.018),(w,y+.027),(w*.93,y-.017),(0.,y-.030),(-w*.93,y-.017)]
  p.append(fitted('Rib plate / five supported ivory Kabutops chest plates '+str(j),IVORY,xy,body,.006))
 for j,y in enumerate((.43,.58,.72)):
  p.append(paddle('Back spur / three rooted Kabutops dorsal points '+str(j),RIDGE,[(0.,y,-.13),(0.,y+.025,-.22),(0.,y+.07,-.28)],[.052,.040,.0015],[.038,.026,.0012],segments=12,steps=4))
 p.append(paddle('Tail / continuous tapered Kabutops tail',BROWN,TAIL,[.09,.105,.065,.002],[.09,.07,.04,.0015],segments=20,steps=4))
 return p
@lru_cache(maxsize=2)
def anatomy(s):
 if s not in SPECIES:raise ValueError('unknown fossil scythe species')
 return tuple((kabuto if s=='kabuto' else kabutops)())
def sculpture(species,normalize=True):
 p=deepcopy(list(anatomy(species)));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for field in ('positions','normals'):q[field]=[0. if v==0 else v for v in q[field]]
 return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=p)
def main():
 from fossil_scythe_glb import export_fossil_scythe
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_fossil_scythe(s))
if __name__=='__main__':main()
