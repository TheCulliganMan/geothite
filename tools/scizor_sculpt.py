#!/usr/bin/env python3
"""Original red steel mantis with open articulated pincers and four wing leaves."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint,sheet
from wing_insect_sculpt import contour
from abra_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.28
RED=(.77,.20,.23);DARK_RED=(.57,.14,.17);INK=(.075,.075,.080)
PLATE=(.29,.31,.30);WING=(.79,.83,.85);VEIN=(.48,.53,.56)
GOLD=(.96,.80,.23);IVORY=(.94,.94,.87)
PINCER_CENTER=(.48,.88)

def pincer_parts(side):
 cx,cy=PINCER_CENTER
 upper=contour([((-.090,-.075),(-.151,-.010),(-.145,.100),(-.084,.153)),
  ((-.084,.153),(.020,.228),(.133,.158),(.158,.062)),
  ((.158,.062),(.184,-.025),(.136,-.037),(.088,.003)),
  ((.088,.003),(.025,.058),(-.024,.058),(-.033,-.009)),
  ((-.033,-.009),(-.026,-.044),(-.040,-.070),(-.090,-.075))],7)
 lower=contour([((-.104,-.055),(-.062,-.072),(.031,-.084),(.129,-.062)),
  ((.129,-.062),(.181,-.072),(.157,-.112),(.105,-.133)),
  ((.105,-.133),(.013,-.174),(-.093,-.145),(-.117,-.087)),
  ((-.117,-.087),(-.124,-.066),(-.119,-.057),(-.104,-.055))],7)
 def world(xy):return [(side*(cx+x),cy+y) for x,y in xy]
 def depth(y,x):
  u=side*x-cx;v=y-cy
  return .103+.034*max(0.,1.-(u/.18)**2-(v/.23)**2)
 upper_outline=upper;lower_outline=lower
 upper=sheet('Pincer upper / rounded red steel fixed claw '+str(side),RED,world(upper),depth,.068,1)
 lower=sheet('Pincer lower / rounded red steel hinged jaw '+str(side),RED,world(lower),depth,.062,1)
 p=[upper,lower]
 # Dark shallow lips line real empty space between separate jaw solids.
 edge=upper_outline[21:29]
 p.append(paint('Pincer upper lining / fitted dark steel crushing lip '+str(side),INK,
  world([(x,y+.006) for x,y in edge]+[(x,y+.024) for x,y in reversed(edge)]),upper,.003,2))
 edge=lower_outline[:8]
 p.append(paint('Pincer lower lining / fitted dark steel crushing bed '+str(side),INK,
  world([(x,y-.006) for x,y in edge]+[(x,y-.023) for x,y in reversed(edge)]),lower,.003,2))
 x=side*(cx-.067);y=cy+.067
 rim=paint('Pincer eye rim / fitted dark false-eye oval '+str(side),PLATE,oval(x,y,.038,.055,28),upper,.005,2);p.append(rim)
 eye=paint('Pincer eye / fitted yellow false-eye oval '+str(side),GOLD,oval(x,y,.026,.043,24),rim,.003,2);p.append(eye)
 p.append(paint('Pincer pupil / fitted dark false-eye center '+str(side),INK,oval(x,y,.010,.028,20),eye,.002,1))
 return p

@lru_cache(maxsize=1)
def raw_sculpture():
 body=loft('Body / angular red thorax and narrow steel waist',RED,curved_profiles(
  ((.56,.045,.045,-.015),(.66,.050,.050,-.005),(.76,.112,.088,.005),(.85,.158,.108,.010),(.91,.120,.085,.025),(.95,.052,.047,.030)),4),28)
 abdomen=loft('Abdomen / long rounded hanging red steel shell',RED,curved_profiles(
  ((.275,.025,.030,-.055),(.32,.080,.075,-.060),(.43,.145,.119,-.050),(.57,.151,.120,-.035),(.66,.105,.075,-.020),(.69,.045,.040,-.015)),4),32)
 head=loft('Head / angular red steel mantis mask',RED,curved_profiles(
  ((.984,.016,.023,.088),(1.020,.060,.063,.070),(1.080,.130,.085,.045),
   (1.145,.146,.086,.005),(1.195,.089,.060,-.010),(1.215,.019,.019,-.020)),3),24)
 p=[body,abdomen,head]
 p.append(curved_tube('Neck / short rooted red steel throat',DARK_RED,[(0.,.915,.028),(0.,.970,.040),(0.,1.025,.060)],[.040,.034,.035],20))
 p.append(paddle('Crest / tall central steel mantis point',RED,[(0.,1.145,-.014),(0.,1.285,-.025),(0.,1.420,-.043)],[.057,.039,.0025],[.038,.026,.0025],segments=20,steps=4))
 for side in (-1,1):
  p.append(paddle('Crest / swept outer steel mantis point '+str(side),RED,[(side*.079,1.145,-.010),(side*.157,1.242,-.024),(side*.215,1.330,-.036)],[.046,.033,.0025],[.032,.023,.0025],segments=20,steps=4))
  x=side*.075;y=1.088
  xy=[(x+side*u,y+v) for u,v in ((-.042,.012),(.035,.038),(.033,-.023),(-.027,-.031))]
  rim=paint('Eye socket / fitted dark stern mantis brow '+str(side),INK,xy,head,.005,3);p.append(rim)
  xy=[(x+.84*(u-x),y+.85*(v-y)) for u,v in xy]
  eye=paint('Eye / fitted golden steel mantis gaze '+str(side),GOLD,xy,rim,.003,2);p.append(eye)
  p.append(paint('Pupil / fitted narrow black mantis slit '+str(side),INK,oval(x-side*.002,y-.001,.006,.020,16),eye,.0025,2))
  p.append(ellipsoid('Shoulder / rounded red steel armor socket '+str(side),RED,(side*.154,.866,.009),(.071,.078,.077),24,16))
  p.append(curved_tube('Upper arm / slim red steel humerus '+str(side),RED,[(side*.184,.848,.017),(side*.240,.812,.028),(side*.278,.806,.037)],[.026,.023,.022],16))
  p.append(curved_tube('Forearm / slim rising red steel ulna '+str(side),RED,[(side*.276,.808,.037),(side*.324,.820,.058),(side*.389,.841,.090)],[.026,.024,.030],16))
  p += pincer_parts(side)
  p.append(ellipsoid('Hip armor / rounded red steel coxa '+str(side),RED,(side*.130,.571,-.014),(.079,.067,.081),24,16))
  p.append(curved_tube('Thigh / slender red steel femur '+str(side),RED,[(side*.151,.548,-.005),(side*.184,.442,.016),(side*.213,.343,.034)],[.035,.030,.027],16))
  p.append(paddle('Greave / long pointed red steel shin armor '+str(side),RED,[(side*.212,.355,.034),(side*.232,.184,.080),(side*.239,.069,.132)],[.036,.067,.043],[.027,.046,.032],segments=24,steps=4))
  p.append(ellipsoid('Foot / grounded narrow red steel pad '+str(side),RED,(side*.240,.035,.145),(.047,.035,.082),24,16))
  p.append(paddle('Toe cap / broad pointed pale steel nail '+str(side),IVORY,[(side*.240,.047,.191),(side*.241,.030,.222),(side*.242,.017,.252)],[.035,.023,.002],[.014,.010,.0015],segments=20,steps=4))
  for j,(root,tip,w) in enumerate((((side*.085,.866,-.073),(side*.352,1.068,-.163),.057),((side*.083,.831,-.082),(side*.386,.649,-.151),.064))):
   mid=tuple((a+b)/2 for a,b in zip(root,tip))
   wing=paddle(f'Wing / four pale steel mantis leaves {side} {j}',WING,[root,mid,tip],[.016,w,.006],[.010,.013,.004],segments=20,steps=5);p.append(wing)
   skin=surface(wing);vein=[]
   for k in range(13):
    t=.025+.94*k/12;x=root[0]+t*(tip[0]-root[0]);y=root[1]+t*(tip[1]-root[1]);vein.append((x,y,skin(y,x)+.001))
   p.append(tube(f'Wing vein / fitted central gray mantis rib {side} {j}',VEIN,vein,[.0015]*len(vein),8))
 p.append(paint('Chest plate / fitted dark steel thorax keel',PLATE,[(0.,.768),(.035,.831),(.033,.916),(-.033,.916),(-.029,.832)],body,.006,3))
 for j,(y,w) in enumerate(((.432,.103),(.511,.119),(.590,.098))):
  p.append(paint('Abdominal plate / three fitted dark steel chevrons '+str(j),PLATE,[(0.,y-.042),(w,y+.023),(w*.88,y+.050),(-w*.88,y+.050),(-w,y+.023)],abdomen,.005,3))
 return {'name':'scizor','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from scizor_glb import export_scizor
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_scizor())
if __name__=='__main__':main()
