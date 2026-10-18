#!/usr/bin/env python3
"""Original genetic psychic cat with narrow gaze, three-finger hands and a heavy tail."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,crease_normals
from bovine_sculpt import oval,curved_tube,smooth
from lugia_sculpt import paddle
from abra_sculpt import surface
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.45
PALE=(.83,.81,.86);PURPLE=(.60,.38,.64);INK=(.10,.08,.13)
WHITE=(.98,.98,.97);IRIS=(.54,.31,.62)
TAIL_POINTS=((0.,.63,-.06),(-.05,.45,-.35),(-.33,.44,-.56),(-.63,.65,-.59),(-.70,.99,-.53),(-.55,1.13,-.45),(-.34,.99,-.39),(-.27,.80,-.38))
TAIL_RADII=(.12,.12,.105,.090,.075,.065,.058,.035);TAIL_STEPS=8
BACK_POINTS=((0.,1.01,-.12),(0.,1.15,-.245),(0.,1.31,-.235),(0.,1.44,-.060))

@lru_cache(maxsize=1)
def raw_sculpture():
 body=loft('Body / tapered pale genetic torso and waist',PALE,curved_profiles(
  ((.54,.080,.085,-.020),(.68,.150,.115,-.025),(.86,.105,.090,-.015),
   (.99,.125,.100,0.),(1.09,.220,.145,0.),(1.17,.210,.120,.025),
   (1.23,.130,.085,.035),(1.27,.070,.060,.045)),4),36)
 head=loft('Head / angular genetic cranium cheeks and long muzzle',PALE,curved_profiles(
  ((1.28,.055,.070,.170),(1.31,.095,.100,.160),(1.38,.160,.140,.105),
   (1.47,.190,.140,.045),(1.56,.170,.115,.015),(1.60,.110,.080,.005),(1.61,.015,.012,0.)),4),36)
 p=[body,ellipsoid('Pelvis / rounded purple lower abdomen',PURPLE,(0.,.650,.025),(.150,.160,.145),28,18),head]
 p.append(paddle('Neck / slender pale genetic throat',PALE,[(0.,1.19,.035),(0.,1.29,.095),(0.,1.35,.120)],[.067,.059,.056],[.060,.055,.052],segments=24,steps=4))
 p.append(curved_tube('Back tube / continuous genetic spinal arch',PALE,BACK_POINTS,[.049,.043,.042,.043],20))
 face=surface(head)
 for side in (-1,1):
  p.append(paddle('Horn / rooted blunt genetic cranial projection '+str(side),PALE,
   [(side*.115,1.55,-.005),(side*.140,1.65,-.020),(side*.150,1.72,-.035)],[.043,.035,.030],[.036,.030,.027],segments=20,steps=4))
  x=side*.105;y=1.449
  xy=[(x+side*u,y+v) for u,v in ((-.045,.005),(.043,.040),(.036,-.018),(-.025,-.030))]
  outer=[(x+1.08*(u-x),y+1.08*(v-y)) for u,v in xy]
  socket=paint('Eye socket / fitted stern genetic rim '+str(side),INK,outer,head,.005,3);p.append(socket)
  eye=paint('Eye / narrow ivory genetic lens '+str(side),WHITE,xy,socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted violet genetic gaze '+str(side),IRIS,oval(x+side*.007,y-.001,.018,.022,20),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / narrow black genetic center '+str(side),INK,oval(x+side*.007,y-.001,.006,.017,16),iris,.002,1);p.append(pupil)
  p.append(paint('Glint / tiny ivory genetic eye spark '+str(side),WHITE,oval(x+side*.004,y+.009,.0035,.005,12),iris,.003,1))
  p.append(paint('Nostril / tiny fitted genetic muzzle slit '+str(side),INK,oval(side*.023,1.335,.003,.0025,12),head,.002,0))
  p.append(paddle('Upper arm / pale genetic shoulder and humerus '+str(side),PALE,
   [(side*.205,1.115,.015),(side*.286,1.005,.010),(side*.320,.920,.035)],[.067,.046,.037],[.058,.044,.036],segments=20,steps=4))
  p.append(paddle('Forearm / tapered pale genetic forearm '+str(side),PALE,
   [(side*.320,.920,.035),(side*.351,.850,.061),(side*.380,.790,.100)],[.041,.047,.033],[.038,.042,.030],segments=20,steps=4))
  p.append(ellipsoid('Hand / three-finger genetic palm '+str(side),PALE,(side*.393,.755,.120),(.046,.058,.039),24,14))
  paths=[[(.374,.750,.140),(.345,.700,.175),(.360,.670,.180)],
         [(.395,.720,.130),(.410,.650,.150),(.400,.620,.180)],
         [(.425,.750,.130),(.460,.690,.140),(.455,.650,.170)]]
  for j,path in enumerate(paths):
   path=[(side*x,y,z) for x,y,z in path]
   p.append(curved_tube(f'Finger / long rounded genetic digits {side} {j}',PALE,path,[.016,.014,.013],12))
   p.append(ellipsoid(f'Fingertip / round genetic contact pad {side} {j}',PALE,path[-1],(.020,.021,.020),16,10))
  p.append(ellipsoid('Haunch / powerful pale genetic thigh '+str(side),PALE,(side*.170,.570,.005),(.145,.240,.150),28,20))
  p.append(curved_tube('Shin / digitigrade pale genetic hock '+str(side),PALE,
   [(side*.180,.415,.015),(side*.155,.280,.035),(side*.175,.095,.090)],[.075,.055,.050],20))
  p.append(ellipsoid('Foot / broad pale genetic ankle pad '+str(side),PALE,(side*.178,.063,.130),(.090,.063,.135),24,16))
  for j,(dx,z) in enumerate(((-.065,.235),(.065,.225),(0.,.035))):
   p.append(ellipsoid(f'Toe / three rounded genetic foot pads {side} {j}',PALE,(side*.178+dx,.035,z),(.045,.035,.057),20,12))
 mouth=[]
 for i in range(19):
  x=-.055+.110*i/18;y=1.302+.003*(abs(x)/.055)**2
  mouth.append((x,y,face(y,x)+.0013))
 p.append(tube('Mouth / fitted restrained genetic muzzle seam',INK,mouth,[.0013]*len(mouth),8))
 points,radii=smooth(TAIL_POINTS,TAIL_RADII,TAIL_STEPS)
 p.append(crease_normals(tube('Tail / continuous heavy purple genetic sweep',PURPLE,points,radii,24),35.))
 p.append(ellipsoid('Tail tip / rounded purple genetic terminal',PURPLE,(-.29,.840,-.38),(.065,.120,.060),24,16))
 return {'name':'mewtwo','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from mewtwo_glb import export_mewtwo
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_mewtwo())
if __name__=='__main__':main()
