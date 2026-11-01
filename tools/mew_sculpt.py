#!/usr/bin/env python3
"""Original small psychic cat with fitted gaze, long feet and a looping whip tail."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,crease_normals,translate
from bovine_sculpt import oval,curved_tube,smooth
from lugia_sculpt import paddle
from abra_sculpt import surface
from raptor_bird_sculpt import paint
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=.78
PINK=(.94,.72,.80);INNER=(.79,.49,.61);WHITE=(.99,.98,.96)
INK=(.11,.09,.13);BLUE=(.21,.55,.74);DEEP=(.06,.23,.35)
TAIL_STEPS=10
TAIL_RADII=(.030,.025,.023,.021,.019,.018,.017,.016,.015)
TAIL_POINTS=((0.,.35,-.12),(0.,.25,-.40),(.39,.29,-.57),(.61,.59,-.55),(.50,.88,-.33),(.25,.96,-.28),(-.10,.93,-.33),(-.41,.83,-.43),(-.61,.98,-.43))

@lru_cache(maxsize=1)
def raw_sculpture():
 body=loft('Body / compact pear-shaped pink psychic cat',PINK,curved_profiles(
  ((.22,.070,.080,-.025),(.28,.130,.130,-.040),(.40,.150,.135,-.040),
   (.52,.100,.090,-.015),(.61,.068,.072,.022),(.66,.040,.050,.035)),4),32)
 neck=paddle('Neck / short continuous pink cat throat',PINK,[(0.,.59,.02),(0.,.66,.06),(0.,.72,.10)],[.068,.066,.062],[.065,.064,.061],segments=24,steps=4)
 head=loft('Head / rounded feline cheeks and small tapered chin',PINK,curved_profiles(
  ((.68,.070,.100,.140),(.73,.160,.150,.115),(.83,.235,.180,.090),
   (.97,.220,.170,.065),(1.06,.155,.130,.040),(1.10,.060,.055,.025),(1.108,.008,.010,.025)),4),36)
 p=[body,neck,head];face=surface(head)
 for side in (-1,1):
  x=side*.115;y=.881
  local=contour([((-.034,.054),(-.032,.077),(-.012,.088),(.005,.078)),((.005,.078),(.029,.078),(.044,.055),(.041,.031)),((.041,.031),(.047,-.017),(.041,-.063),(.014,-.065)),((.014,-.065),(-.022,-.070),(-.053,-.005),(-.034,.054))],6)
  xy=[(x+side*u,y+v) for u,v in local]
  outer=[(x+1.08*(u-x),y+1.05*(v-y)) for u,v in xy]
  socket=paint('Eye socket / fine fitted psychic-cat rim '+str(side),INK,outer,head,.005,3);p.append(socket)
  eye=paint('Eye / tapered ivory psychic-cat lens '+str(side),WHITE,xy,socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted blue psychic-cat gaze '+str(side),BLUE,oval(x+side*.006,y-.003,.027,.058,24),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / deep blue upright psychic-cat center '+str(side),DEEP,oval(x+side*.008,y-.004,.013,.039,20),iris,.002,1);p.append(pupil)
  p.append(paint('Glint / fitted ivory psychic-cat eye spark '+str(side),WHITE,oval(x+side*.001,y+.030,.009,.014,16),iris,.003,1))
  ear=paddle('Ear / rooted soft pointed psychic-cat pinna '+str(side),PINK,
   [(side*.14,1.025,.035),(side*.178,1.125,.015),(side*.215,1.197,-.005)],[.057,.043,.0025],[.040,.028,.0025],segments=20,steps=4);p.append(ear)
  p.append(paint('Ear inset / fitted dusky pink inner pinna '+str(side),INNER,[(side*.14,1.045),(side*.19,1.165),(side*.202,1.095)],ear,.003,2))
  p.append(paddle('Arm / short reaching pink psychic-cat arm '+str(side),PINK,
   [(side*.090,.526,.005),(side*.158,.500,.035),(side*.222,.486,.065)],[.040,.034,.027],[.035,.030,.026],segments=18,steps=3))
  p.append(ellipsoid('Hand / tiny three-finger psychic-cat palm '+str(side),PINK,(side*.228,.484,.066),(.039,.029,.034),20,12))
  for j in range(3):
   p.append(curved_tube(f'Finger / short rounded psychic-cat digits {side} {j}',PINK,
    [(side*.246,.473+.010*j,.045+.018*j),(side*(.272+.006*(j==1)),.472+.012*j,.043+.020*j)],[.012,.006],10))
  p.append(ellipsoid('Haunch / rounded pink psychic-cat thigh '+str(side),PINK,(side*.111,.305,-.012),(.096,.122,.109),24,16))
  p.append(curved_tube('Leg / short pink psychic-cat ankle '+str(side),PINK,
   [(side*.113,.279,.005),(side*.115,.214,.036),(side*.116,.151,.073)],[.056,.049,.043],18))
  paw=loft('Foot / long tapered psychic-cat hind paw '+str(side),PINK,curved_profiles(((.001,.013,.015,.15),(.04,.057,.050,.13),(.12,.070,.060,.09),(.22,.055,.045,.04),(.29,.032,.030,.01)),4),28)
  p.append(translate(paw,side*.116,0.,0.))
  paw_skin=surface(p[-1])
  for j,dx in enumerate((-.018,.018)):
   points=[]
   for i in range(9):
    y=.025+.050*i/8;x=side*.116+dx
    points.append((x,y,paw_skin(y,x)+.0012))
   p.append(tube(f'Toe crease / fitted psychic-cat paw divisions {side} {j}',INNER,points,[.0009]*len(points),6))
 mouth=[]
 for i in range(17):
  x=-.035+.070*i/16;y=.753+.009*(abs(x)/.035)**2
  mouth.append((x,y,face(y,x)+.0013))
 p.append(tube('Mouth / little fitted psychic-cat smile',INK,mouth,[.0013]*len(mouth),8))
 tail_points,tail_radii=smooth(TAIL_POINTS,TAIL_RADII,TAIL_STEPS)
 p.append(crease_normals(tube('Tail / continuous looping psychic whip',PINK,tail_points,tail_radii,16),35.))
 p.append(ellipsoid('Tail tip / oval pink psychic-cat terminal',PINK,(-.634,1.042,-.414),(.048,.115,.041),24,18))
 return {'name':'mew','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from mew_glb import export_mew
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_mew())
if __name__=='__main__':main()
