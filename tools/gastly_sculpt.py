#!/usr/bin/env python3
"""Original dark gaseous orb, broad hostile white gaze and irregular vapor halo."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,tube
from raptor_bird_sculpt import paint
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.05;CENTER=(0.,.50,0.)
BODY=(.13,.105,.145);INK=(.055,.035,.065);WHITE=(.97,.965,.97)
MOUTH=(.68,.43,.56);GAS=(.56,.40,.66);GAS_LIGHT=(.62,.47,.70)

def cloud_radius(a):return .378+.030*math.sin(7*a+.3)+.021*math.sin(11*a-.9)+.010*math.sin(17*a)

def halo():
 n=144;k=12;v=[];faces=[]
 for i in range(n):
  a=math.tau*i/n;r=cloud_radius(a);w=.081+.014*math.sin(5*a+.7)
  for j in range(k):
   b=math.tau*j/k;inner=.230+.010*math.sin(5*a);radius=inner+(r+w-inner)*(.5+.5*math.cos(b))
   v.append((radius*math.cos(a),.50+radius*math.sin(a),-.065+(.170+.022*math.sin(5*a))*math.sin(b)+.024*math.sin(3*a)))
 for i in range(n):
  for j in range(k):faces.append((i*k+j,((i+1)%n)*k+j,((i+1)%n)*k+(j+1)%k,i*k+(j+1)%k))
 return part('Gas halo / continuous irregular scalloped purple vapor',GAS,v,faces,True)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Body / smooth round dark ghost orb',BODY,CENTER,(.305,.30,.265),48,30)
 p=[body,halo()]
 for i in range(9):
  a=math.tau*i/9+.12;rad=cloud_radius(a);sign=1 if i%3 else -1;points=[];radii=[]
  for k in range(14):
   t=k/13;angle=a+sign*.23*math.sin(math.pi*t*.80);radius=rad-.015+.170*t-.027*t*t
   points.append((radius*math.cos(angle),.50+radius*math.sin(angle),-.065+.100*math.sin(a*3+.4)+.070*math.sin(math.pi*t+2*a)))
   radii.append(.047*(1-t)**.85+.003)
  p.append(tube('Gas curl / tapered drifting comma '+str(i),GAS_LIGHT if i%3==0 else GAS,points,radii,10))
 for side in (-1,1):
  xy=contour([((.024,.509),(.085,.546),(.188,.622),(.246,.657)),
    ((.246,.657),(.279,.603),(.279,.486),(.215,.433)),
    ((.215,.433),(.154,.387),(.067,.397),(.024,.509))],10)
  eye=paint('Eye / broad angular white ghost gaze '+str(side),WHITE,[(side*x,y) for x,y in xy],body,.008,3);p.append(eye)
  pupil=[(side*(.073+.0045*math.cos(j*math.tau/16)),.495+.014*math.sin(j*math.tau/16)) for j in range(16)]
  p.append(paint('Pupil / tiny vertical black ghost pupil '+str(side),INK,pupil,eye,.002,1))
 mouth=contour([((-.181,.382),(-.080,.330),(.080,.330),(.181,.382)),
    ((.181,.382),(.123,.252),(-.123,.252),(-.181,.382))],16)
 rim=paint('Mouth rim / broad curved dark open grin',INK,mouth,body,.009,3);p.append(rim)
 cx=0.;cy=.315
 inside=[(cx+.88*(x-cx),cy+.84*(y-cy)) for x,y in mouth]
 p.append(paint('Mouth / fitted muted pink ghost grin',MOUTH,inside,rim,.003,3))
 for side in (-1,1):
  xy=[(side*.150,.367),(side*.111,.352),(side*.126,.305)]
  p.append(paint('Fang / two narrow ivory ghost canines '+str(side),WHITE,xy,body,.020,3))
 return {'name':'gastly','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from gastly_glb import export_gastly
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_gastly())
