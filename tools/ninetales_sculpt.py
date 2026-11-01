#!/usr/bin/env python3
"""Original elegant nine-tailed fox with cream mane, swept crest and nine gold-tipped plumes."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube,fitted_patch
from bovine_sculpt import curved_tube,oval,smooth
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from chikorita_sculpt import unit,sub,cross,dot,mul,add
from skin_glb import COORDINATES
HEIGHT=1.30
CREAM=(.98,.94,.68);FACE=(.99,.95,.74);MANE=(1.,.97,.77);TIP=(.93,.67,.24)
EAR=(.63,.47,.28);INK=(.14,.10,.085);RED=(.91,.24,.16);WHITE=(1.,1.,.98)

def tail_paths():
 paths=[]
 for j in range(7):
  s=(j-3)/3;paths.append(((s*.025,.365,-.285),(s*.16,.44,-.52),(s*.43,.64+.27*(1-s*s),-.75),(s*.63,.45+.32*(1-s*s),-.94+.08*abs(s)),(s*.69,.31+.32*(1-s*s),-.87)))
 for side in (-1,1):paths.append(((side*.025,.365,-.285),(side*.20,.30,-.48),(side*.45,.23,-.43),(side*.58,.17,-.08),(side*.48,.16,.25)))
 return tuple(paths)
TAIL_PATHS=tail_paths()

def trunk():
 rows=curved_profiles(((-.36,.016,.025,.35),(-.28,.116,.133,.36),(-.10,.133,.145,.38),(.08,.115,.140,.405),(.19,.073,.100,.455),(.225,.015,.025,.48)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / elegant cream nine-tail fox torso',CREAM,v,faces,True),40.)

def mane():
 rows=curved_profiles(((.365,.070,.070,0.),(.420,.140,.120,0.),(.500,.170,.140,0.),(.600,.120,.100,0.),(.670,.070,.060,0.)),3);n=48;v=[];faces=[]
 for k,(y,rx,rz,_) in enumerate(rows):
  for j in range(n):
   a=j*math.tau/n;t=k/(len(rows)-1);fur=(.5+.5*math.cos(7*a))**2*math.sin(math.pi*t)
   v.append((rx*math.sin(a)*(1.+.10*fur),y-.040*fur,.14+rz*math.cos(a)*(1.+.10*fur)))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Mane / continuous full cream fox throat',MANE,v,faces,True),45.)

def tails():
 result=[];n=20;seam=13
 for j,path in enumerate(TAIL_PATHS):
  widths=(.040,.080,.135,.105,.003) if j<7 else (.040,.100,.140,.105,.003)
  depths=(.035,.060,.085,.060,.003) if j<7 else (.035,.065,.065,.040,.003)
  points,widths=smooth(path,widths,4);_,depths=smooth(path,depths,4);rings=[];side=(1.,0.,0.)
  for k,p in enumerate(points):
   tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]));side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
   rings.append([add(p,add(mul(side,widths[k]*math.cos(i*math.tau/n)),mul(other,depths[k]*math.sin(i*math.tau/n)))) for i in range(n)])
  for label,color,rr in [('Tail / flowing cream fox plume ',CREAM,rings[:seam+1]),('Tail tip / continuous golden fox end ',TIP,rings[seam:])]:
   v=[p for ring in rr for p in ring];faces=[tuple(reversed(range(n))),tuple((len(rr)-1)*n+i for i in range(n))]
   for k in range(len(rr)-1):
    for i in range(n):a=k*n+i;b=k*n+(i+1)%n;faces.append((a,b,b+n,a+n))
   result.append(crease_normals(part(label+str(j),color,v,faces,True),40.))
 return result

def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / long cream fox muzzle and tapered brow',FACE,curved_profiles(((.565,.025,.035,.320),(.600,.075,.100,.320),(.660,.119,.152,.290),(.735,.137,.131,.240),(.805,.104,.092,.205),(.845,.048,.048,.190),(.853,.010,.012,.185)),4),32)
 p=[trunk(),head,mane(),paddle('Neck / graceful upright cream fox throat',CREAM,[(0.,.40,.12),(0.,.51,.16),(0.,.64,.22)],[.069,.071,.067],[.070,.071,.065],segments=20,steps=4)]
 p.append(paddle('Mane tuft / long cream chest point',MANE,[(0.,.510,.255),(0.,.408,.280),(0.,.296,.205)],[.086,.080,.0015],[.035,.032,.0015],segments=18,steps=4))
 p+=tails()
 for side in (-1,1):
  ear=paddle('Ear / sharp cream fox pinna '+str(side),CREAM,[(side*.095,.788,.185),(side*.150,.885,.161),(side*.207,.978,.133),(side*.240,1.030,.116)],[.045,.052,.028,.0015],[.026,.023,.014,.0015],segments=18,steps=4);p.append(ear)
  xy=[(side*.110,.813),(side*.141,.927),(side*.223,1.005),(side*.187,.900),(side*.148,.831)]
  p.append(paint('Ear inset / fitted warm tan fox pinna '+str(side),EAR,xy,ear,.003,3))
  xy=[(side*.040,.718),(side*.074,.752),(side*.123,.757),(side*.117,.726),(side*.093,.708),(side*.062,.708)]
  eye=paint('Eye / sharp red nine-tail fox gaze '+str(side),RED,xy,head,.006,3);p.append(eye)
  p.append(marking('Pupil / dark narrow nine-tail fox eye '+str(side),INK,oval(side*.085,.735,.006,.017,18),eye,.0025))
  p.append(marking('Glint / white nine-tail fox eye spark '+str(side),WHITE,oval(side*.082,.740,.0035,.005,14),eye,.003))
  p.append(paddle('Cheek fur / swept cream fox whisker '+str(side),MANE,[(side*.100,.699,.201),(side*.150,.658,.160),(side*.179,.584,.110)],[.035,.039,.0015],[.023,.022,.0015],segments=14,steps=4))
  shoulder=ellipsoid('Shoulder / joined cream fox foreleg '+str(side),CREAM,(side*.080,.397,.146),(.052,.072,.061),22,14);p.append(shoulder)
  for row in (0,1):
   z=.155 if row==0 else -.245;x=side*(.090 if row==0 else .123)
   if row==1:p.append(ellipsoid('Haunch / rounded cream fox hind thigh '+str(side),CREAM,(side*.106,.333,-.242),(.075,.103,.100),24,16))
   p.append(curved_tube(f'Leg / slender cream fox limb {side} {row}',CREAM,[(side*(.074 if row==0 else .102),.413 if row==0 else .351,z-.010),(x,.188,z-(.045 if row else 0.)),(x,.055,z+.017)],[.045 if row==0 else .057,.030,.027],18))
   paw=ellipsoid(f'Paw / four planted cream fox feet {side} {row}',CREAM,(x,.029,z+.038),(.043,.029,.054),22,12);p.append(paw)
   skin=surface(paw);pts=[]
   for k in range(8):
    yy=.010+.018*k/7;pts.append((x,yy,skin(yy,x)+.0006))
   p.append(tube(f'Toe crease / fine cream fox paw division {side} {row}',EAR,pts,[.00065]*8,6))
 p.append(paddle('Crest / swept central cream fox crown',MANE,[(0.,.806,.186),(0.,.925,.116),(.040,1.010,-.020)],[.043,.070,.0015],[.030,.035,.0015],segments=18,steps=4))
 for side in (-1,1):p.append(paddle('Crest / swept paired cream fox crown '+str(side),MANE,[(side*.040,.805,.166),(side*.080,.908,.080),(side*.160,.947,-.045)],[.030,.040,.0015],[.022,.027,.0015],segments=14,steps=4))
 skin=surface(head);p.append(ellipsoid('Nose / small dark nine-tail fox button',INK,(0.,.640,skin(.640,0.)+.002),(.009,.006,.004),16,10))
 smile=[]
 for k in range(21):
  x=-.043+.086*k/20;y=.612+.007*(abs(x)/.043)**2;smile.append((x,y,skin(y,x)+.001))
 p.append(tube('Smile / fitted nine-tail fox muzzle crease',INK,smile,[.001]*21,8))
 return {'name':'ninetales','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from ninetales_glb import export_ninetales
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_ninetales())
