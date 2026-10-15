#!/usr/bin/env python3
"""Original slender Suicune with an open diamond crest and broad flowing ribbons."""
import math
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import part,ellipsoid,loft,translate,crease_normals
from bovine_sculpt import oval,smooth
from aqua_rabbit_sculpt import ribbon
from raptor_bird_sculpt import paint
from raikou_sculpt import side_frame,plate
from skin_glb import COORDINATES

HEIGHT=1.60
BLUE=(90/255,189/255,1.);PURPLE=(115/255,16/255,165/255)
WHITE=(1.,1.,1.);INK=(0.,0.,0.);CREST=(.27,.67,.72);RED=(.80,.18,.24)
RIBBON_PATH=tuple((x,y,z) for x,y,z in (
 (.12,.64,-.55),(.35,.87,-.50),(.52,.98,-.22),(.55,.84,.08),
 (.44,.54,.34),(.33,.32,.44),(.46,.18,.27),(.64,.34,-.05),
 (.66,.67,-.51),(.56,.95,-.94),(.45,1.16,-1.13)))

def diamond_crest():
 outer=[(0.,1.215),(.19,1.50),(0.,1.94),(-.19,1.50)]
 inner=[(0.,1.345),(.090,1.50),(0.,1.79),(-.090,1.50)]
 vertices=[(x,y,.52-.60*(y-1.215)+offset) for offset in (.027,-.027) for loop in (outer,inner) for x,y in loop]
 faces=[]
 for i in range(4):
  j=(i+1)%4
  faces.extend(((i,j,j+4,i+4),(i+8,i+12,j+12,j+8),
                (i+8,j+8,j,i),(i+4,j+4,j+12,i+12)))
 return crease_normals(part('Crest / continuous hollow Suicune diamond',CREST,vertices,faces),35.)

def wind_mane():
 q=loft('Mane / continuous purple Suicune wind mantle',PURPLE,
  [(-.30,.14,.15,1.00),(-.12,.30,.19,1.12),(.18,.31,.17,1.15),
   (.43,.29,.11,1.10),(.70,.23,.13,1.16),(.88,.025,.020,1.16)],32)
 for key in ('positions','normals'):
  values=q[key];q[key]=[v for i in range(0,len(values),3) for v in (values[i],values[i+2],-values[i+1])]
 return q

def marking(name,zy,parent,side):
 q=paint(name,WHITE,[(-side*z,y) for z,y in zy],side_frame(parent,side),.005,3)
 return side_frame(q,side,True)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Body / slender blue Suicune torso',BLUE,(0.,.58,-.12),(.205,.23,.52),32,18)
 neck=ellipsoid('Neck / graceful raised Suicune neck',BLUE,(0.,.85,.29),(.17,.31,.22),28,16)
 head=ellipsoid('Head / narrow blue Suicune skull',BLUE,(0.,1.12,.45),(.17,.16,.235),28,16)
 p=[body,neck,head]
 p.append(paint('Bib / fitted tapering white Suicune throat',WHITE,
  [(-.065,1.02),(-.090,.86),(-.045,.64),(0.,.61),(.045,.64),(.090,.86),(.065,1.02),(0.,1.075)],neck,.007,3))
 p.append(ellipsoid('Muzzle / narrow white Suicune muzzle',WHITE,(0.,1.068,.665),(.125,.068,.16),24,14))
 p.append(ellipsoid('Nose / small dark Suicune nose',INK,(0.,1.083,.817),(.039,.023,.019),16,10))
 for side in (-1,1):
  for j,(z,y,w,h) in enumerate(((.14,.665,.09,.085),(-.13,.70,.095,.075),(-.39,.64,.073,.075))):
   p.append(marking('Body diamond / fitted white Suicune flank '+str(side)+' '+str(j),[(z,y+h),(z+w,y),(z,y-h),(z-w,y)],body,side))
  for rear,z in ((False,.23),(True,-.48)):
   label='rear' if rear else 'fore';x=side*.155
   leg=translate(loft('Leg / long tapered '+label+' Suicune leg '+str(side),BLUE,
    [(.055,.042,.063,z+.075),(.16,.045,.060,z+.05),(.30,.050,.065,z-.005),
     (.44,.067,.080,z-.025),(.60,.083,.092,z),(.69,.043,.050,z-.025)],24),x,0.,0.);p.append(leg)
   if rear:
    p.append(marking('Leg diamond / fitted white Suicune hind mark '+str(side),[(z-.01,.53),(z+.04,.48),(z-.005,.43),(z-.05,.48)],leg,side))
   p.append(translate(loft('Foot / planted '+label+' Suicune paw '+str(side),BLUE,
    [(0.,.050,.067,z+.095),(.022,.072,.096,z+.100),(.059,.068,.086,z+.080),(.085,.039,.048,z+.063)],24),x,0.,0.))
   for j,dx in enumerate((-.047,0.,.047)):
    p.append(ellipsoid('Toe / slender '+label+' Suicune toe '+str(side)+' '+str(j),BLUE,(x+dx,.029,z+.177),(.026,.028,.035),16,10))
  x=side*.106;outline=[(x-side*.022,1.184),(x+side*.024,1.181),(x+side*.021,1.157),(x-side*.019,1.160)]
  socket=paint('Eye socket / fitted dark Suicune stare '+str(side),INK,outline,head,.006,2);p.append(socket)
  white=paint('Eye / fitted white Suicune sclera '+str(side),WHITE,[(x+(a-x)*.82,1.172+(b-1.172)*.75) for a,b in outline],socket,.003,2);p.append(white)
  iris=paint('Iris / fitted red Suicune iris '+str(side),RED,oval(x,1.172,.006,.006,12),white,.003,1);p.append(iris)
  p.append(paint('Pupil / narrow dark Suicune pupil '+str(side),INK,oval(x,1.172,.0025,.004,12),iris,.002,1))
  path=[(side*x,y,z) for x,y,z in RIBBON_PATH]
  widths=[.023,.041,.045,.039,.044,.045,.042,.042,.044,.035,.004]
  points,ww=smooth(path,widths,4);_,depths=smooth(path,[.006]*10+[.002],4)
  p.append(crease_normals(ribbon('Ribbon / broad streaming Suicune tail '+str(side),WHITE,points,ww,depths),35.))
 p.append(plate('Crest base / fitted swept Suicune forehead armor',CREST,
  [(-.17,1.24),(-.10,1.28),(0.,1.27),(.10,1.28),(.17,1.24),(.08,1.19),(0.,1.16),(-.08,1.19)],lambda y,x:.59-2.0*x*x,.025))
 p.append(diamond_crest());p.append(wind_mane())
 return {'name':'suicune','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(*,normalize=True):
 model=deepcopy(raw_sculpture())
 if normalize:
  low=min(v for p in model['primitives'] for v in p['positions'][1::3]);high=max(v for p in model['primitives'] for v in p['positions'][1::3]);scale=HEIGHT/(high-low)
  for p in model['primitives']:
   p['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])]
   p['normals']=[round(v,7) for v in p['normals']]
 return model
