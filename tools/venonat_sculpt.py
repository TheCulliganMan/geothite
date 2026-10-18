#!/usr/bin/env python3
"""Original fuzzy insect orb with pink compound lenses, ivory feelers, claw hands and planted feet."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,tube,crease_normals
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.10;CENTER=(0.,.50,0.)
PURPLE=(.49,.40,.62);EYE=(.88,.39,.58);SHEEN=(.98,.67,.78)
IVORY=(.94,.88,.72);WHITE=(.99,.98,.95);PINK=(.78,.48,.59);CREASE=(.53,.45,.33)

def fuzzy_body():
 n=72;rows=44;v=[];faces=[]
 for k in range(rows+1):
  t=math.pi*(k+.025)/(rows+.05)
  for j in range(n):
   a=j*math.tau/n;grain=max(0.,math.sin(13*a+3*t))*max(0.,math.cos(11*t-2*a));fur=.075*grain**3*math.sin(t)**.35
   r=1.+fur;v.append((.34*math.sin(t)*math.sin(a)*r,.50+.40*math.cos(t)*r,.29*math.sin(t)*math.cos(a)*r))
 faces=[tuple(reversed(range(n))),tuple(rows*n+j for j in range(n))]
 for k in range(rows):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / continuous fuzzy purple insect orb',PURPLE,v,faces,True),45.)

def compound_eye(side):
 q=ellipsoid('Eye / faceted pink compound insect lens '+str(side),EYE,(side*.145,.625,.230),(.125,.150,.085),18,12)
 center=(side*.145,.625,.230);angle=side*.20;c=math.cos(angle);s=math.sin(angle);v=[]
 for i in range(0,len(q['positions']),3):
  x,y,z=[q['positions'][i+j]-center[j] for j in range(3)];v.append((center[0]+c*x+s*z,center[1]+y,center[2]-s*x+c*z))
 faces=[tuple(q['indices'][i:i+3]) for i in range(0,len(q['indices']),3)]
 return part(q['part'],EYE,v,faces,False)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=fuzzy_body();p=[body]
 for side in (-1,1):
  eye=compound_eye(side);p.append(eye)
  p.append(paint('Eye sheen / pale pink compound lens glint '+str(side),SHEEN,oval(side*.132,.691,.019,.023,18),eye,.003,3))
  p.append(curved_tube('Antenna / curved ivory insect feeler '+str(side),WHITE,[(side*.095,.805,.105),(side*.175,.962,.085),(side*.330,1.075,.095)],[.010,.009,.006],14))
  tip=ellipsoid('Antenna tip / broad ivory feeler leaf '+str(side),WHITE,(side*.365,1.091,.095),(.058,.020,.017),22,12);p.append(tip)
  p.append(paddle('Fang / paired ivory insect mandible '+str(side),WHITE,[(side*.062,.490,.300),(side*.070,.448,.316),(side*.054,.407,.306)],[.018,.021,.004],[.017,.018,.004],segments=14,steps=4))
  p.append(curved_tube('Arm / rooted purple insect forearm '+str(side),PURPLE,[(side*.265,.423,.115),(side*.308,.381,.168),(side*.292,.368,.223)],[.039,.030,.026],16))
  p.append(ellipsoid('Hand / small ivory insect palm '+str(side),IVORY,(side*.291,.370,.236),(.043,.050,.029),22,14))
  for j in range(3):
   y=.398-j*.026
   p.append(curved_tube(f'Finger / three curled ivory hand claws {side} {j}',IVORY,[(side*.290,y,.244),(side*.258,y+.006,.267),(side*.244,y-.006,.272)],[.015,.014,.006],12))
  p.append(curved_tube('Leg / short braced ivory insect ankle '+str(side),IVORY,[(side*.146,.190,.008),(side*.152,.130,.016),(side*.151,.070,.032)],[.055,.045,.038],18))
  foot=ellipsoid('Foot / two broad planted ivory insect pads '+str(side),IVORY,(side*.151,.043,.034),(.106,.043,.132),28,16);p.append(foot)
  skin=surface(foot)
  for j,dx in enumerate((-.029,.029)):
   points=[]
   for k in range(10):
    y=.011+.031*k/9;x=side*.151+dx;points.append((x,y,skin(y,x)+.0007))
   p.append(tube(f'Toe crease / three rounded insect toes {side} {j}',CREASE,points,[.0009]*10,6))
 p.append(ellipsoid('Nose / rounded pink insect snout',PINK,(0.,.490,.292),(.072,.035,.040),28,18))
 for j,(x,tip) in enumerate(((-.090,(-.163,.982,-.028)),(0.,(.010,1.009,-.023)),(.090,(.142,.966,-.037)))):
  p.append(paddle('Fur crest / pointed purple crown tuft '+str(j),PURPLE,[(x,.824,-.015),(x*.95,.900,-.024),tip],[.046,.034,.0015],[.032,.027,.0015],segments=16,steps=4))
 return {'name':'venonat','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from venonat_glb import export_venonat
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_venonat())
