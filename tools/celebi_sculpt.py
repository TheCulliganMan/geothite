#!/usr/bin/env python3
"""Original time-travel fairy with a swept onion head and blue-tipped feelers."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from abra_sculpt import surface
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=.98
HEAD=(.75,.88,.51);BODY=(.57,.77,.29);DARK=(.36,.61,.23)
MINT=(.85,.94,.69);WING=(.86,.95,.84);INK=(.095,.12,.105)
BLUE=(.16,.56,.77);DEEP=(.075,.29,.43);WHITE=(.98,.99,.95)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Body / small rounded green fairy torso',BODY,(0.,.365,-.012),(.115,.160,.099),28,18)
 head=loft('Head / continuous swept onion bulb and curled point',HEAD,curved_profiles(
  ((.48,.090,.072,.045),(.54,.190,.145,.036),(.67,.280,.205,.015),
   (.83,.260,.195,-.006),(.97,.200,.160,-.040),(1.07,.130,.113,-.120),
   (1.16,.065,.070,-.220),(1.21,.024,.029,-.290),(1.23,.0018,.0018,-.325)),4),40)
 p=[body,ellipsoid('Abdomen / darker rounded fairy lower body',DARK,(0.,.255,-.025),(.094,.092,.085),24,16),head]
 face=surface(head)
 p.append(paint('Chest / fitted pale leaf collar',MINT,[(0.,.36),(.070,.45),(.055,.49),(-.055,.49),(-.070,.45)],body,.005,2))
 for side in (-1,1):
  x=side*.134;y=.725
  socket=paint('Eye socket / fitted dark fairy oval '+str(side),INK,oval(x,y,.073,.108,28),head,.006,3);p.append(socket)
  eye=paint('Eye / fitted ivory fairy lens '+str(side),WHITE,oval(x,y,.059,.092,24),socket,.004,2);p.append(eye)
  iris=paint('Iris / large blue fairy gaze '+str(side),BLUE,oval(x-side*.010,y,.043,.077,24),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / deep blue upright fairy center '+str(side),DEEP,oval(x-side*.013,y-.002,.019,.055,20),iris,.002,1);p.append(pupil)
  p.append(paint('Glint / fitted white fairy eye spark '+str(side),WHITE,oval(x-side*.020,y+.039,.011,.018,16),iris,.003,1))
  start=face(.90,side*.090)-.006
  p.append(curved_tube('Antenna / curved green fairy feeler '+str(side),HEAD,
   [(side*.090,.90,start),(side*.108,1.067,.152),(side*.170,1.205,.104)],[.012,.0105,.0075],12))
  p.append(curved_tube('Antenna tip / fitted blue terminal '+str(side),BLUE,
   [(side*.150,1.176,.116),(side*.185,1.228,.092),(side*.222,1.253,.077)],[.009,.0065,.0018],12))
  p.append(paddle('Arm / slender reaching fairy arm '+str(side),HEAD,
   [(side*.085,.439,.006),(side*.145,.414,.024),(side*.205,.405,.045)],[.030,.026,.023],[.026,.023,.021],segments=16,steps=3))
  p.append(ellipsoid('Hand / soft three-finger fairy palm '+str(side),HEAD,(side*.213,.404,.047),(.038,.028,.032),20,12))
  for j in range(3):
   p.append(curved_tube(f'Finger / three pointed fairy digits {side} {j}',HEAD,
    [(side*.230,.394+.010*j,.030+.017*j),(side*(.264+.006*(j==1)),.391+.013*j,.026+.020*j)], [.012,.0028],10))
  p.append(paddle('Wing / rounded pale fairy leaf '+str(side),WING,
   [(side*.072,.42,-.078),(side*.237,.562,-.133),(side*.388,.682,-.180)],[.023,.071,.010],[.009,.012,.004],segments=20,steps=4))
  p.append(curved_tube('Leg / short rooted fairy shank '+str(side),DARK,
   [(side*.063,.249,-.009),(side*.075,.170,.006),(side*.078,.125,.027)],[.044,.041,.038],16))
  p.append(ellipsoid('Foot / planted green fairy teardrop '+str(side),DARK,(side*.080,.070,.027),(.051,.070,.061),24,14))
 for j in (-1,0,1):
  path=[]
  for i in range(17):
   t=i/16;y=.93+.27*t;x=j*.075*(1.-t)**1.2
   path.append((x,y,face(y,x)+.0015))
  p.append(tube('Head groove / fitted onion fold '+str(j),DARK,path,[.0012]*len(path),6))
 mouth=[]
 for i in range(17):
  x=-.044+.088*i/16;y=.55+.012*(abs(x)/.044)**2
  mouth.append((x,y,face(y,x)+.0015))
 p.append(tube('Mouth / fitted little fairy smile',INK,mouth,[.0015]*len(mouth),8))
 return {'name':'celebi','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from celebi_glb import export_celebi
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_celebi())
if __name__=='__main__':main()
