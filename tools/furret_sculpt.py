#!/usr/bin/env python3
"""Original cream-and-brown Furret with a full curved body and banded tail."""
import math,struct
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import part,ellipsoid,loft,translate,add,sub,cross,unit,mul,dot,curved_profiles,crease_normals,fitted_triangle_contour,fitted_contour
from bovine_sculpt import oval,smooth
from lugia_sculpt import paddle
from yanma_sculpt import clip
from skin_glb import COORDINATES
HEIGHT=1.20
LIGHT=(.97,.89,.75);DARK=(.55,.37,.29);INK=(.045,.033,.028);WHITE=(1.,.99,.95);PINK=(.95,.48,.52)
TAIL_KNOTS=((0.,.270,-.180),(-.050,.255,-.440),(-.145,.325,-.650),(-.285,.490,-.820),(-.405,.710,-.900),(-.445,.915,-.885),(-.450,1.040,-.810))
TAIL,RADII=smooth(TAIL_KNOTS,[.135,.195,.185,.150,.118,.071,.003],6)

def colored_sections(prefix,vertices,segments,rows,sections):
 """Closed colored sections share exact ring positions and exterior normals."""
 def faces(count):
  fs=[tuple(reversed(range(segments))),tuple((count-1)*segments+i for i in range(segments))]
  for k in range(count-1):
   for i in range(segments):
    a=k*segments+i;b=k*segments+(i+1)%segments;fs.append((a,b,b+segments,a+segments))
  return fs
 whole=part(prefix+' / reference skin',LIGHT,vertices,faces(rows),True)
 normals={tuple(whole['positions'][i:i+3]):whole['normals'][i:i+3] for i in range(0,len(whole['positions']),3)};out=[]
 for j,(start,end,color) in enumerate(sections):
  q=part(prefix+' / joined '+('cream' if color==LIGHT else 'brown')+' band '+str(j),color,vertices[start*segments:(end+1)*segments],faces(end-start+1))
  for i in set(q['indices'][2*(segments-2)*3:]):q['normals'][i*3:i*3+3]=normals[tuple(q['positions'][i*3:i*3+3])]
  out.append(q)
 return out,whole

def body():
 profiles=curved_profiles(((.090,.110,.125,-.110),(.150,.220,.215,-.130),(.240,.263,.260,-.130),
  (.340,.258,.255,-.115),(.460,.226,.218,-.065),(.570,.195,.185,.010),(.720,.140,.160,.075),
  (.870,.143,.175,.125),(.980,.158,.190,.165),(1.080,.176,.215,.195),(1.190,.184,.180,.170),
  (1.245,.157,.142,.125),(1.280,.088,.079,.110),(1.290,.006,.006,.110)),3)
 n=32;vs=[]
 for y,rx,rz,z in profiles:
  for i in range(n):
   angle=math.tau*i/n;x=rx*math.sin(angle);front=z+rz*math.cos(angle)
   # A real shallow mouth well, closed by the continuous facial backing.
   if math.cos(angle)>.65:
    u=max(0.,1.-(x/.050)**2-((y-1.075)/.048)**2);front-=.045*u*u
   vs.append((x,y,front))
 def row(y):return next(i for i,p in enumerate(profiles) if abs(p[0]-y)<1e-10)
 parts,skin=colored_sections('Body',vs,n,len(profiles),[(0,row(.240),DARK),(row(.240),row(.460),LIGHT),
  (row(.460),row(.570),DARK),(row(.570),row(1.190),LIGHT),(row(1.190),len(profiles)-1,DARK)])
 # The neck saddle is actual coloured body tissue. Both halves share the
 # original exterior corners and normals, with matching buried cut faces.
 source_normals={tuple(skin['positions'][i:i+3]):skin['normals'][i:i+3] for i in range(0,len(skin['positions']),3)}
 halves=[];first,last=row(.570),row(1.190);count=last-first+1;width=n//2+1
 for name,color,angles in [('Body / joined cream neck front',LIGHT,list(range(24,32))+list(range(9))),
   ('Back coat / continuous brown neck saddle',DARK,list(range(8,25)))]:
  points=[vs[k*n+i] for k in range(first,last+1) for i in angles]
  faces=[tuple(reversed(range(width))),tuple((count-1)*width+i for i in range(width))]
  exterior=[]
  for k in range(count-1):
   for i in range(width-1):
    a=k*width+i;exterior.append((a,a+1,a+width+1,a+width))
   a=k*width; b=a+width-1;faces.append((b,a,a+width,b+width))
  q=part(name,color,points,faces+exterior)
  # Keep the whole loft's smooth outside normals; caps retain their own normals.
  cap_triangles=(2*(width-2)+2*(count-1))*3
  for i in set(q['indices'][cap_triangles:]):
   q['normals'][i*3:i*3+3]=source_normals[tuple(q['positions'][i*3:i*3+3])]
  halves.append(q)
 parts[3:4]=halves
 return parts,skin

def tail():
 n=24;vs=[];side=None
 for k,p in enumerate(TAIL):
  tangent=unit(sub(TAIL[min(k+1,len(TAIL)-1)],TAIL[max(0,k-1)]))
  if side is None:side=cross(tangent,(0.,1.,0.))
  side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
  vs.extend(add(p,mul(add(mul(side,math.cos(i*math.tau/n)),mul(other,math.sin(i*math.tau/n))),RADII[k])) for i in range(n))
 return colored_sections('Tail',vs,n,len(TAIL),[(0,5,DARK),(5,10,LIGHT),(10,16,DARK),(16,22,LIGHT),(22,27,DARK),(27,len(TAIL)-1,LIGHT)])[0]

def paint(name,color,xy,parent,th=.003):return fitted_triangle_contour(name,color,xy,parent,th,precision=5)
@lru_cache(maxsize=1)
def raw_sculpture():
 p,skin=body();p+=tail()
 for s in (-1,1):
  eye=paint('Eye / rounded fitted black Furret gaze '+str(s),INK,oval(s*.0792,1.158,.024,.028,24),skin,.004);p.append(eye)
  p.append(paint('Glint / small ivory Furret eye '+str(s),WHITE,oval(s*.0792-.0064,1.1684,.0048,.0056,14),eye,.0015))
  for j,y in enumerate((1.105,1.065)):
   p.append(paint(f'Face mark / two short brown Furret cheek bars {s} {j}',DARK,[(s*.1112,y+.011),(s*.1416,y+.005),(s*.1392,y-.006),(s*.1112,y-.001)],skin,.003))
  outline=[(x*.8,y) for x,y in ((.125,1.190),(.217,1.140),(.350,1.290),(.251,1.260),(.192,1.300))]
  # The cream point occupies the ear's full thickness, including its back.
  for label,color,above in [('Ear / broad swept brown Furret pinna ',DARK,False),('Ear tip / solid cream Furret terminal point ',LIGHT,True)]:
   xy=clip(outline,0,.254,above)
   p.append(crease_normals(fitted_contour(label+str(s),color,[(s*x,y) for x,y in xy],lambda y,x:.105,.035,0),40))
  p.append(paddle('Arm / short cream Furret forelimb '+str(s),LIGHT,[(s*.095,.762,.074),(s*.213,.664,.119),(s*.232,.600,.143)],[.038,.043,.034],[.039,.039,.030],segments=18,steps=4))
  p.append(ellipsoid('Hand / rounded brown Furret forepaw '+str(s),DARK,(s*.233,.581,.145),(.040,.046,.041),20,14))
  p.append(ellipsoid('Leg / upright rooted brown Furret haunch '+str(s),DARK,(s*.149,.137,-.085),(.090,.107,.100),24,16))
  foot=translate(loft('Foot / planted cream Furret hind paw '+str(s),LIGHT,[(0.,.067,.090,.035),(.027,.092,.125,.037),(.061,.079,.105,.024),(.083,.043,.052,-.004)],24),s*.149,0.,0.)
  # Round three modest toe lobes into the actual paw, preserving its flat sole.
  vs=[]
  for i in range(0,len(foot['positions']),3):
   x,y,z=foot['positions'][i:i+3];local=x-s*.149
   if z>.095:z+=.007*math.cos(local*math.pi/.030)**2
   vs.append((x,y,z))
  p.append(part(foot['part'],LIGHT,vs,[tuple(foot['indices'][i:i+3]) for i in range(0,len(foot['indices']),3)],True))
 p.append(paint('Forehead / fitted brown Furret cap point',DARK,[(-.068,1.199),(0.,1.132),(.068,1.199)],skin,.003))
 p.append(paint('Nose / tiny dark Furret nose',INK,oval(0.,1.121,.011,.007,16),skin,.003))
 mouth=paint('Mouth / recessed dark Furret mouth lining',INK,[(0.,1.108),(.024,1.084),(.019,1.050),(0.,1.042),(-.019,1.050),(-.024,1.084)],skin,.002);p.append(mouth)
 p.append(paint('Tongue / fitted pink Furret lower mouth',PINK,oval(0.,1.062,.015,.018,20),mouth,.001))
 for j,(root,mid,tip) in enumerate([((-.446,.925,-.877),(-.486,1.006,-.856),(-.506,1.064,-.808)),((-.436,.926,-.874),(-.401,1.008,-.830),(-.378,1.063,-.778))]):
  p.append(paddle('Tail tuft / two closed cream terminal fur points '+str(j),LIGHT,[root,mid,tip],[.028,.031,.0015],[.028,.025,.0015],segments=14,steps=3))
 return dict(name='furret',version=1,coordinate_system=COORDINATES,primitives=p)

def sculpture(*,normalize=True):
 m=deepcopy(raw_sculpture())
 if normalize:
  lo=min(v for p in m['primitives'] for v in p['positions'][1::3]);hi=max(v for p in m['primitives'] for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
  for p in m['primitives']:
   values=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])]
   p['positions']=list(struct.unpack('<'+'f'*len(values),struct.pack('<'+'f'*len(values),*values)))
   if not p['part'].startswith(('Body /','Back coat /','Tail /')):p.update(crease_normals(p,40.))
   p['normals']=[round(v,7) or 0. for v in p['normals']]
 return m
