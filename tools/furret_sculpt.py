#!/usr/bin/env python3
"""Original Furret with shared-ring banding and a fitted, pointed-eared face."""
import math
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import part,ellipsoid,loft,translate,add,sub,cross,unit,mul,dot
from bovine_sculpt import oval,smooth
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.20
LIGHT=(189/255,173/255,33/255);DARK=(115/255,74/255,16/255)
INK=(0.,0.,0.);WHITE=(1.,1.,1.)
TAIL_KNOTS=((0.,.22,-.19),(-.02,.19,-.35),(-.06,.26,-.53),(-.10,.45,-.66),(-.13,.70,-.70),(-.11,.91,-.64))

def colored_sections(prefix,vertices,segments,rows,sections):
 """Closed colored sections share exact ring positions and exterior normals."""
 faces=[tuple(reversed(range(segments))),tuple((rows-1)*segments+i for i in range(segments))]
 for k in range(rows-1):
  for i in range(segments):
   a=k*segments+i;b=k*segments+(i+1)%segments;faces.append((a,b,b+segments,a+segments))
 whole=part(prefix+' / reference skin',LIGHT,vertices,faces,True)
 normals={tuple(whole['positions'][i:i+3]):whole['normals'][i:i+3] for i in range(0,len(whole['positions']),3)}
 out=[]
 for j,(start,end,color) in enumerate(sections):
  vs=vertices[start*segments:(end+1)*segments];count=end-start+1
  fs=[tuple(reversed(range(segments))),tuple((count-1)*segments+i for i in range(segments))]
  for k in range(count-1):
   for i in range(segments):
    a=k*segments+i;b=k*segments+(i+1)%segments;fs.append((a,b,b+segments,a+segments))
  q=part(prefix+' / joined '+('gold' if color==LIGHT else 'brown')+' band '+str(j),color,vs,fs)
  # Cap normals stay flat inside the solid; exterior seam normals are shared.
  cap_indices=2*(segments-2)*3
  for i in set(q['indices'][cap_indices:]):
   key=tuple(q['positions'][i*3:i*3+3]);q['normals'][i*3:i*3+3]=normals[key]
  out.append(q)
 return out,whole

def body():
 profiles=[(.12,.12,.17,-.14),(.20,.19,.21,-.13),(.30,.205,.20,-.10),(.42,.19,.18,-.065),
  (.52,.17,.16,-.025),(.64,.15,.145,.015),(.73,.14,.14,.035),(.87,.135,.135,.045),
  (.97,.15,.145,.06),(1.05,.21,.175,.09),(1.13,.19,.15,.105),(1.20,.125,.12,.10),(1.225,.012,.012,.10)]
 n=32;vs=[(rx*math.sin(math.tau*i/n),y,z+rz*math.cos(math.tau*i/n)) for y,rx,rz,z in profiles for i in range(n)]
 return colored_sections('Body',vs,n,len(profiles),[(0,2,DARK),(2,3,LIGHT),(3,4,DARK),(4,5,LIGHT),(5,6,DARK),(6,10,LIGHT),(10,12,DARK)])

def tail():
 points,radii=smooth(TAIL_KNOTS,[.135,.15,.135,.10,.066,.004],6);n=24;vs=[];side=None
 for k,p in enumerate(points):
  tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
  if side is None:side=cross(tangent,(0.,1.,0.))
  side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
  vs.extend(add(p,mul(add(mul(side,math.cos(i*math.tau/n)),mul(other,math.sin(i*math.tau/n))),radii[k])) for i in range(n))
 ranges=[(a,min(a+5,len(points)-1),LIGHT if j%2 else DARK) for j,a in enumerate(range(0,len(points)-1,5))]
 return colored_sections('Tail',vs,n,len(points),ranges)[0]

@lru_cache(maxsize=1)
def raw_sculpture():
 p,skin=body();p+=tail()
 for side in (-1,1):
  x=side*.088
  eye=paint('Eye / fitted black Furret gaze '+str(side),INK,oval(x,1.103,.024,.030,20),skin,.005,2);p.append(eye)
  p.append(paint('Glint / small fitted Furret eye light '+str(side),WHITE,oval(x-.006,1.115,.005,.006,12),eye,.003,1))
  for j,y in enumerate((1.073,1.044)):
   p.append(paint('Face mark / fitted brown Furret cheek '+str(side)+' '+str(j),DARK,
    [(side*.121,y+.010),(side*.151,y+.006),(side*.149,y-.005),(side*.124,y-.004)],skin,.004,2))
  ear=loft('Ear / swept pointed brown Furret pinna '+str(side),DARK,
   [(1.125,.046,.030,.084),(1.19,.067,.045,.079),(1.25,.005,.004,.024)],20)
  # Sweep the pinnae outward; Furret's ears are wide points, not upright horns.
  ear['positions']=[v+side*(.135+.85*(ear['positions'][i-i%3+1]-1.125)) if i%3==0 else v for i,v in enumerate(ear['positions'])]
  # Recompute normals after the shear rather than reusing the loft's normals.
  vertices=[tuple(ear['positions'][i:i+3]) for i in range(0,len(ear['positions']),3)]
  faces=[tuple(ear['indices'][i:i+3]) for i in range(0,len(ear['indices']),3)]
  ear=part(ear['part'],DARK,vertices,faces,True);p.append(ear)
  p.append(ellipsoid('Arm / small Furret forelimb '+str(side),LIGHT,(side*.145,.66,.061),(.055,.10,.055),20,14))
  p.append(ellipsoid('Hand / brown Furret forepaw '+str(side),DARK,(side*.176,.60,.094),(.051,.044,.054),20,14))
  p.append(ellipsoid('Leg / rooted Furret haunch '+str(side),DARK,(side*.123,.16,-.10),(.085,.135,.09),24,16))
  p.append(translate(loft('Foot / planted Furret hind paw '+str(side),DARK,
   [(0.,.065,.075,.015),(.027,.088,.12,.038),(.064,.076,.11,.026),(.09,.046,.056,-.005)],24),side*.135,0.,0.))
 nose=paint('Nose / fitted dark Furret nose',INK,oval(0.,1.073,.016,.010,16),skin,.006,2);p.append(nose)
 p.append(paint('Mouth / small fitted Furret smile',INK,[(-.026,1.046),(0.,1.034),(.026,1.046),(.023,1.049),(0.,1.040),(-.023,1.049)],skin,.004,2))
 return {'name':'furret','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(*,normalize=True):
 m=deepcopy(raw_sculpture())
 if normalize:
  lo=min(v for p in m['primitives'] for v in p['positions'][1::3]);hi=max(v for p in m['primitives'] for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  for p in m['primitives']:
   p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])]
   p['normals']=[round(v,7) for v in p['normals']]
 return m
