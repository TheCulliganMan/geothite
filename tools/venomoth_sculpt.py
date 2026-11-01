#!/usr/bin/env python3
"""Original lilac moth with swept veined wings, three antennae and striped oval abdomen."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,tube
from bovine_sculpt import curved_tube,oval
from wing_insect_sculpt import contour
from skarmory_sculpt import blade
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from goldfish_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.16;CENTER=(0.,.48,-.015)
LILAC=(.66,.59,.76);WING=(.80,.75,.87);ABDOMEN=(.64,.61,.69)
VEIN=(.43,.38,.49);IVORY=(.96,.94,.85);EYE=(.83,.94,.98);INK=(.12,.15,.20);WHITE=(1.,1.,1.)
AB_CENTER=(0.,.255,-.025);AB_RADII=(.125,.235,.106)

def wing_depth(x,y):return -.075-.065*abs(x)+.018*(y-.5)**2

def wings(side,rear):
 if not rear:
  outline=contour([((.065,.485),(.17,.75),(.61,1.115),(.985,1.155)),((.985,1.155),(1.09,1.168),(1.01,1.015),(.903,.910)),((.903,.910),(.796,.802),(.843,.665),(.669,.574)),((.669,.574),(.475,.481),(.210,.420),(.065,.485))],10)
  paths=[[(.095,.500),(.250,.695),(.556,.942),(.960,1.110)],[(.104,.493),(.346,.686),(.652,.874),(.942,.962)],[(.104,.485),(.343,.588),(.591,.704),(.810,.747)]]
 else:
  outline=contour([((.065,.449),(.218,.503),(.463,.485),(.582,.421)),((.582,.421),(.681,.373),(.589,.337),(.571,.305)),((.571,.305),(.600,.257),(.652,.204),(.594,.204)),((.594,.204),(.518,.190),(.447,.320),(.358,.317)),((.358,.317),(.240,.300),(.113,.341),(.065,.449))],8)
  paths=[[(.087,.440),(.235,.445),(.418,.421),(.575,.381)],[(.085,.430),(.224,.387),(.367,.366),(.560,.258)]]
 label=f'{side} {int(rear)}';outline=[(side*x,y) for x,y in outline]
 p=[blade('Wing / swept lilac '+('hindwing fan ' if rear else 'forewing sail ')+label,WING,outline,wing_depth,.007)]
 skin=surface(p[0])
 for face in (-1,1):
  for j,path in enumerate(paths):
   points=[(side*x,y,wing_depth(side*x,y)+face*.0083) for x,y in path]
   vein=curved_tube(f'Wing vein / fitted moth spar {side} {int(rear)} {face} {j}',VEIN,points,[.0023]*len(points),6)
   vertices=[]
   for i in range(0,len(vein['positions']),3):
    x,y,z=vein['positions'][i:i+3];vertices.append((x,y,skin(y,x)-.007+z-wing_depth(x,y)))
   faces=[tuple(vein['indices'][i:i+3]) for i in range(0,len(vein['indices']),3)]
   p.append(part(vein['part'],VEIN,vertices,faces,True))
 return p

def abdomen_band(index,y):
 # A single closed zigzag cuff, seated on the actual analytic oval. The
 # meridian/radial frame keeps every point attached around the full abdomen.
 n=80;rings=8;v=[];cx,cy,cz=AB_CENTER;rx,ry,rz=AB_RADII
 for i in range(n):
  a=math.tau*i/n;yy=y+.008*(2/math.pi*math.asin(math.sin(10*a)))
  for j in range(rings):
   b=math.tau*j/rings;dy=.0025*math.cos(b);out=-.0008+.0032*math.sin(b);f=math.sqrt(1.-((yy+dy-cy)/ry)**2)
   v.append((cx+(rx*f+out)*math.sin(a),yy+dy,cz+(rz*f+out)*math.cos(a)))
 faces=[]
 for i in range(n):
  for j in range(rings):faces.append((i*rings+j,i*rings+(j+1)%rings,((i+1)%n)*rings+(j+1)%rings,((i+1)%n)*rings+j))
 return part('Abdomen band / seated zigzag segment seam '+str(index),VEIN,v,faces,True)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Thorax / lilac moth wing engine',LILAC,CENTER,(.135,.145,.120),32,22)
 abdomen=ellipsoid('Abdomen / long gray lavender moth oval',ABDOMEN,AB_CENTER,AB_RADII,40,28)
 head=ellipsoid('Head / round lilac moth face',LILAC,(0,.651,.055),(.155,.156,.138),36,24)
 p=[body,abdomen,head]+[abdomen_band(j,y) for j,y in enumerate((.145,.239,.337))]
 for side in (-1,1):
  eye=ellipsoid('Eye / pale round moth lens '+str(side),EYE,(side*.104,.680,.165),(.085,.099,.063),28,20);p.append(eye)
  pupil=paint('Pupil / fitted dark round moth gaze '+str(side),INK,oval(side*.112,.686,.046,.059,28),eye,.004,3);p.append(pupil)
  p.append(paint('Glint / ivory moth eye reflection '+str(side),WHITE,oval(side*.100,.711,.020,.023,20),pupil,.002,2))
  p.append(paddle('Mandible / curved ivory moth jaw '+str(side),IVORY,[(side*.083,.575,.135),(side*.072,.515,.190),(side*.031,.510,.205)],[.021,.022,.002],[.017,.018,.0015],segments=16,steps=5))
  p.append(paddle('Antenna / lateral pointed lilac crest '+str(side),LILAC,[(side*.048,.748,.069),(side*.098,.850,.040),(side*.161,.967,.012)],[.025,.021,.001],[.019,.014,.001],segments=16,steps=5))
  for rear in (False,True):p.extend(wings(side,rear))
  for j,z in enumerate((.077,-.020,-.107)):
   x=.128+j*.012
   p.append(curved_tube(f'Leg / six slender lilac moth struts {side} {j}',LILAC,[(side*.085,.439,z),(side*(.176+j*.008),.372,z+.055),(side*x,.307,z+.120)],[.016,.013,.008],14))
   p.append(ellipsoid(f'Foot / six tucked narrow moth toes {side} {j}',LILAC,(side*x,.298,z+.132),(.016,.019,.032),18,12))
 p.append(paddle('Antenna / tall central lilac moth crest',LILAC,[(0.,.748,.070),(0.,.918,.025),(.013,1.093,-.008)],[.033,.021,.001],[.022,.014,.001],segments=18,steps=6))
 return {'name':'venomoth','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from venomoth_glb import export_venomoth
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_venomoth())
