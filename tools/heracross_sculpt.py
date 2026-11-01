#!/usr/bin/env python3
"""Original single-horn beetle: armored body, flattened fork and hooked claws."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint,sheet
from abra_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.16
BLUE=(.23,.43,.57);BELLY=(.31,.51,.63);DARK=(.12,.29,.39)
INK=(.055,.075,.085);GOLD=(.94,.79,.40);WHITE=(.96,.96,.91)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=loft('Body / broad rounded blue beetle abdomen',BLUE,curved_profiles(
  ((.25,.09,.07,.0),(.29,.19,.15,-.005),(.39,.25,.19,-.015),(.57,.26,.19,-.015),(.70,.24,.17,-.005),(.78,.18,.14,.0)),4),36)
 thorax=ellipsoid('Thorax / broad blue beetle collar',BLUE,(0.,.755,.0),(.255,.155,.185),32,18)
 head=ellipsoid('Head / rounded blue armored beetle mask',BLUE,(0.,.920,.020),(.222,.205,.178),36,22)
 p=[body,thorax,head]
 # The belly plates follow real stored armor; grooves divide the broad central plate.
 p.append(paint('Belly / fitted pale central armored plate',BELLY,[(-.125,.32),(.125,.32),(.150,.41),(.14,.67),(-.14,.67),(-.15,.41)],body,.006,3))
 face=surface(body)
 for side in (-1,1):
  pts=[]
  for j in range(21):
   y=.315+.36*j/20;x=side*(.122+.025*math.sin(math.pi*j/20));pts.append((x,y,face(y,x)+.006))
  p.append(tube('Armor groove / fitted longitudinal abdomen seam '+str(side),DARK,pts,[.0025]*len(pts),8))
 # Rounded outline retains a single continuous, closed forked horn.
 horn=[(-.050,.925),(-.064,1.10),(-.066,1.27),(-.091,1.325),(-.135,1.345),(-.165,1.331),(-.160,1.371),(-.131,1.45),(-.120,1.555),(-.107,1.586),(-.092,1.574),(-.078,1.53),(-.035,1.421),(0.,1.384),(.035,1.421),(.091,1.538),(.113,1.582),(.129,1.571),(.131,1.54),(.127,1.463),(.156,1.375),(.165,1.336),(.134,1.349),(.093,1.331),(.066,1.275),(.064,1.10),(.050,.925)]
 p.append(sheet('Horn / continuous flattened forked beetle beam',BLUE,horn,lambda y,x:.167+.070*(y-.925)+.015*math.exp(-(x/.065)**2),.042,2))
 hf=surface(head)
 for side in (-1,1):
  x=side*.128;y=.967
  rim=paint('Eye socket / fitted dark beetle rim '+str(side),INK,oval(x,y,.050,.064,28),head,.005,3);p.append(rim)
  iris=paint('Eye / fitted golden beetle gaze '+str(side),GOLD,oval(x,y,.043,.056,28),rim,.003,2);p.append(iris)
  pupil=paint('Pupil / deep oval beetle center '+str(side),INK,oval(x-side*.006,y-.003,.025,.040,24),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / small ivory beetle highlight '+str(side),WHITE,oval(x-side*.011,y+.020,.009,.014,16),pupil,.0025,1))
  root=(side*.125,1.065,.078);tip=(side*.208,1.205,.125)
  p.append(curved_tube('Antenna / slender blue bead feeler '+str(side),BLUE,[root,(side*.176,1.133,.142),tip],[.010,.009,.008],12))
  p.append(ellipsoid('Antenna bead / rounded blue feeler terminal '+str(side),BLUE,tip,(.030,.031,.028),20,14))
  p.append(ellipsoid('Shoulder / rounded beetle armor socket '+str(side),BLUE,(side*.250,.738,.003),(.091,.083,.087),24,16))
  p.append(curved_tube('Upper arm / thin exposed beetle arm '+str(side),BLUE,[(side*.285,.718,.013),(side*.344,.675,.032),(side*.390,.630,.060)],[.029,.023,.024],16))
  # Broad tapered forearm and hand are one shell. Small outer teeth make the silhouette serrated.
  p.append(paddle('Forearm / broad pointed beetle gauntlet '+str(side),BLUE,[(side*.386,.640,.060),(side*.452,.558,.087),(side*.493,.477,.112)], [.035,.066,.079],[.028,.045,.043],segments=24,steps=4))
  for j,(x,y) in enumerate(((.474,.556),(.513,.491))):
   p.append(paddle(f'Gauntlet tooth / two pointed armor edges {side} {j}',BLUE,[(side*(x-.018),y+.022,.09),(side*(x+.035),y-.027,.09)],[.026,.0025],[.025,.0025],segments=12,steps=2))
  p.append(ellipsoid('Hand / broad blue claw knuckle '+str(side),BLUE,(side*.485,.458,.128),(.064,.043,.060),24,14))
  for j,dx in enumerate((-.028,.030)):
   x=side*(.485+dx)
   p.append(curved_tube(f'Hand claw / paired hooked ivory beetle digits {side} {j}',WHITE,[(x,.448,.160),(x+side*.010,.405,.184),(x+side*.007,.365,.163)],[.021,.018,.002],16))
  p.append(ellipsoid('Haunch / large rounded blue beetle coxa '+str(side),BLUE,(side*.205,.306,.005),(.125,.142,.132),28,18))
  p.append(paddle('Thigh thorn / pointed upper leg armor '+str(side),BLUE,[(side*.266,.343,-.020),(side*.311,.414,-.025)],[.050,.0025],[.040,.0025],segments=16,steps=3))
  p.append(curved_tube('Shin / short blue beetle leg '+str(side),BLUE,[(side*.225,.223,.013),(side*.242,.151,.039),(side*.249,.100,.067)],[.061,.043,.041],20))
  p.append(ellipsoid('Foot / planted oval blue beetle tarsus '+str(side),BLUE,(side*.253,.060,.107),(.065,.060,.093),24,16))
  p.append(curved_tube('Foot claw / broad forward ivory beetle hook '+str(side),WHITE,[(side*.253,.070,.155),(side*.259,.042,.226),(side*.263,.018,.264)],[.035,.029,.002],20))
 mouth=[]
 for i in range(25):
  x=-.092+.184*i/24;y=.847+.005*math.cos(2.*math.pi*x/.092)
  mouth.append((x,y,hf(y,x)+.0018))
 p.append(tube('Mouth / fitted scalloped beetle mask seam',DARK,mouth,[.0018]*len(mouth),8))
 # Collar and shell seams follow their own physical armor surfaces.
 back=reverse_front(body);bf=surface(back);pts=[]
 for i in range(23):
  y=.29+.44*i/22;pts.append((0.,y,-bf(y,0.)-.002))
 p.append(tube('Back seam / fitted closed wing-case division',DARK,pts,[.002]*len(pts),8))
 return {'name':'heracross','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from heracross_glb import export_heracross
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_heracross())
if __name__=='__main__':main()
