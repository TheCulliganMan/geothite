#!/usr/bin/env python3
"""Original continuous spiked ghost head and detached three-clawed spectral hands."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,crease_normals,fitted_adaptive_contour
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint,sheet
from wing_insect_sculpt import contour
from goldfish_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.10;CENTER=(0.,.55,0.)
PURPLE=(.48,.43,.61);INK=(.095,.06,.13);WHITE=(.97,.97,.98);PINK=(.73,.43,.56)

def fitted(name,color,xy,parent,th):
 return fitted_adaptive_contour(name,color,xy,parent,th,surface_at=surface(parent))

def eye_lens(side,xy,parent):
 def convex(y,x):return -.025+.275*math.sqrt(1.-(x/.34)**2-((y-.55)/.30)**2)
 lens=sheet('Eye / fitted slanted white ghost lens '+str(side),WHITE,xy,convex,.008,3)
 skin=surface(parent)
 for i in range(0,len(lens['positions']),3):
  x,y,z=lens['positions'][i:i+3]
  if z<convex(y,x):lens['positions'][i+2]=skin(y,x)-.002
 return crease_normals(lens,35.)

def body():
 right=[(0.,.85),(.13,.85),(.48,1.00),(.35,.63),(.52,.61),(.35,.49),(.43,.41),(.30,.38),(.33,.25),(.19,.28),(.20,.08),(0.,.24)]
 outline=right+[(-x,y) for x,y in reversed(right[1:-1])]
 # A common peripheral loop joins the convex front and back of the head.
 # Concentric rings keep the ears and serrated shroud in the same closed solid.
 outline=[(a[0]+(b[0]-a[0])*k/3,a[1]+(b[1]-a[1])*k/3) for a,b in zip(outline,outline[1:]+outline[:1]) for k in range(3)]
 n=len(outline);v=[];faces=[];rings=(.025,.16,.32,.48,.64,.79,.91,.97,1.)
 for sign in (1,-1):
  for r in rings:
   for x,y in outline:
    distance=math.sqrt((x/.34)**2+((y-.55)/.30)**2)
    radius=r/distance if r<1. else 1.
    z=-.025+sign*(.275 if sign>0 else .185)*math.sqrt(max(0.,1-r*r))
    v.append((radius*x,.55+radius*(y-.55),z))
 count=n*len(rings)
 front=len(v);v.append((0.,.55,.250));back=len(v);v.append((0.,.55,-.210))
 for i in range(n):faces += [(front,(i+1)%n,i),(back,count+i,count+(i+1)%n)]
 for layer in range(2):
  offset=layer*count
  for k in range(len(rings)-1):
   for i in range(n):
    a=offset+k*n+i;b=offset+k*n+(i+1)%n
    faces.append((a,b,b+n,a+n) if layer==0 else (a,a+n,b+n,b))
 # The front/back peripheral loops coincide; no zero-area seam faces are added.
 return crease_normals(part('Body / continuous angular ghost head ears and ragged shroud',PURPLE,v,faces,True),35.)

def palm_center(side):return (side*.49,.35,.15)
def finger_root(side,j):return (side*(.49+(j-1)*.047),.327,.181)

@lru_cache(maxsize=1)
def raw_sculpture():
 skull=body();p=[skull]
 for side in (-1,1):
  xy=[(.027,.550),(.266,.659),(.258,.559),(.232,.520),(.145,.518)]
  eye=eye_lens(side,[(side*x,y) for x,y in xy],skull);p.append(eye)
  p.append(paint('Pupil / fine vertical black ghost slit '+str(side),INK,oval(side*.102,.552,.004,.014,14),eye,.002,1))
 mouth=contour([((-.263,.479),(-.105,.423),(.105,.423),(.263,.479)),
   ((.263,.479),(.197,.241),(-.197,.241),(-.263,.479))],8)
 rim=fitted('Mouth rim / broad open curved spectral grin',INK,mouth,skull,.010);p.append(rim)
 inner=[(x*.91,.353+(y-.353)*.88) for x,y in mouth]
 p.append(fitted('Mouth / fitted pink spectral grin interior',PINK,inner,rim,.003))
 for side in (-1,1):
  for j,(x,y,w,length) in enumerate(((.074,.439,.028,.055),(.190,.453,.026,.051))):
   xy=[(side*(x-w),y+.007),(side*(x+w),y+.007),(side*x,y-length)]
   p.append(paint(f'Upper tooth / pointed purple ghost grin {side} {j}',PURPLE,xy,skull,.024,2))
  for j,(x,y,w,length) in enumerate(((.046,.293,.023,.043),(.123,.311,.022,.040))):
   xy=[(side*(x-w),y-.008),(side*(x+w),y-.008),(side*x,y+length)]
   p.append(paint(f'Lower tooth / pointed purple ghost grin {side} {j}',PURPLE,xy,skull,.024,2))
  p.append(ellipsoid('Hand / detached broad spectral palm '+str(side),PURPLE,palm_center(side),(.092,.069,.065),28,18))
  for j in range(3):
   x,y,z=finger_root(side,j);dx=side*(j-1)*.020
   p.append(curved_tube(f'Finger / three curved pointed ghost claws {side} {j}',PURPLE,[(x,y,z),(x+dx*.4,.287,.232),(x+dx,.216,.277),(x+dx*.8,.158+(j==1)*-.023,.290)],[.034,.036,.020,.0018],20))
 return {'name':'haunter','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from haunter_glb import export_haunter
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_haunter())
