#!/usr/bin/env python3
"""Original stocky tongue bearer with a recessed mouth and broad pink tongue."""
import argparse,json
from functools import lru_cache
from copy import deepcopy
from pathlib import Path
from sculpt_geometry import part,ellipsoid as sphere,loft,curved_profiles,linear,crease_normals,fitted_adaptive_contour
from bovine_sculpt import oval
from lugia_sculpt import paddle as swept
from slow_pair_sculpt import forelimb
from goldfish_sculpt import surface
from cubone_sculpt import recessed,skin_shading
from moltres_sculpt import flowing
from skin_glb import COORDINATES
HEIGHT=1.18
PINK=(.94,.64,.70);TONGUE=(.95,.69,.75);CREAM=(.96,.91,.69)
WHITE=(.95,.96,.95);INK=(.17,.13,.19);GROOVE=(.89,.60,.67)

TONGUE_POINTS=((0,1.025,.025),(0,.945,.425),(-.012,.713,.704),(-.022,.466,.902),(-.022,.392,.932))
def arm_points(side):return ((side*.268,.861,.028),(side*.421,.86,.120),(side*.494,.806,.223),(side*.501,.767,.270),(side*.501,.750,.285))

def ellipsoid(name,color,center,radii,segments=28,rings=18):
 cap=(64,40) if name.startswith('Head /') else (28,18) if name.startswith('Thigh /') else (20,14) if name.startswith('Hand tip /') else (24,16)
 return sphere(name,color,center,radii,min(segments,cap[0]),min(rings,cap[1]))
def paddle(name,color,points,widths,depths):
 return swept(name,color,points,widths,depths,segments=32 if name.startswith('Tongue /') else 28)
def paint(name,color,xy,parent,th=.006,sub=3):
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)
def recessed_lining(name,color,outline,parent,th=.001):
 # Clip the actual front head triangles to the convex oral outline. Sharing
 # their planes removes the lifted chords and irregular shadow fringe of an
 # approximate cap, while retaining the physical recess beneath a closed layer.
 if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:outline=list(reversed(outline))
 points=[tuple(parent['positions'][i:i+3]) for i in range(0,len(parent['positions']),3)];vertices=[];faces=[];lookup={}
 def vertex(point):
  key=tuple(round(v,10) for v in point)
  if key not in lookup:lookup[key]=len(vertices);vertices.append(key)
  return lookup[key]
 for k in range(0,len(parent['indices']),3):
  poly=[points[i] for i in parent['indices'][k:k+3]];a,b,c=poly
  if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
  for a,b in zip(outline,outline[1:]+outline[:1]):
   clipped=[]
   def distance(q):return (b[0]-a[0])*(q[1]-a[1])-(b[1]-a[1])*(q[0]-a[0])
   for u,v in zip(poly,poly[1:]+poly[:1]):
    du,dv=distance(u),distance(v);inside_u,inside_v=du>=-1e-12,dv>=-1e-12
    if inside_u:clipped.append(u)
    if inside_u!=inside_v:
     fraction=du/(du-dv);clipped.append(tuple(u[j]+fraction*(v[j]-u[j]) for j in range(3)))
   poly=clipped
   if len(poly)<3:break
  if len(poly)<3:continue
  ids=[vertex(q) for q in poly]
  for j in range(1,len(ids)-1):
   if len({ids[0],ids[j],ids[j+1]})==3:faces.append((ids[0],ids[j],ids[j+1]))
 edges={}
 for face in faces:
  for a,b in zip(face,face[1:]+face[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
 if any(len(uses)>2 for uses in edges.values()):raise ValueError('nonmanifold oral lining front')
 count=len(vertices);positions=[(x,y,z+offset) for offset in (th,-th*.35) for x,y,z in vertices];closed=faces+[tuple(i+count for i in reversed(face)) for face in faces]
 for uses in edges.values():
  if len(uses)==1:
   a,b=uses[0];closed.append((b,a,a+count,b+count))
 return part(name,color,positions,closed,True)

def fitted_shading(parts):
 def find(prefix,side=None):return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+side)))
 for i,p in enumerate(parts):
  name=p['part'];side=name.split()[-1]
  for child,parent in (('Mouth lining /','Head /'),('Tongue groove /','Tongue /'),('Belly band /','Body /'),('Eye /','Head /'),('Eye glint /','Eye /'),('Nostril /','Head /'),('Thigh ring /','Thigh /'),('Thigh center /','Thigh ring /')):
   if name.startswith(child):parts[i]=skin_shading(p,find(parent,side if parent in ('Eye /','Thigh /','Thigh ring /') else None));break
 return parts

@lru_cache(maxsize=1)
def anatomy():
 body=loft('Body / stocky rounded pink Lickitung torso',PINK,curved_profiles(((.13,.07,.09,-.04),(.25,.24,.23,-.03),(.43,.33,.28,-.012),(.64,.32,.255,.015),(.83,.26,.205,.05),(.98,.13,.12,.08),(1.025,.035,.04,.08)),4),32)
 neck=ellipsoid('Neck / rooted short pink Lickitung throat',PINK,(0,1.007,.08),(.15,.17,.14),36,24)
 head=recessed(ellipsoid('Head / rounded pink Lickitung head with physical oral recess',PINK,(0,1.20,.09),(.304,.28,.263),80,56),[(0,1.05,.215,.125,.177)])
 # The recessed helper rebuilds a part; retain this species' original palette.
 head['base_color']=linear(PINK)
 p=[body,neck,head]
 mouth=skin_shading(recessed_lining('Mouth lining / fitted broad dark Lickitung oral bowl',INK,oval(0,1.050,.202,.111,48),head,.001),head);p.append(mouth)
 tongue=paddle('Tongue / rooted long broad flattened pink Lickitung tongue',TONGUE,TONGUE_POINTS,[.140,.152,.151,.115,.004],[.048,.050,.048,.049,.003]);p.append(tongue)
 # A shallow fitted marking follows the actual stored tongue, not a floating rod.
 p.append(skin_shading(paint('Tongue groove / fitted long subtle Lickitung center line',GROOVE,[(-.008,.455),(-.007,.84),(.004,.84),(.003,.455)],tongue,.004,4),tongue))
 for j,(low,high,w) in enumerate(((.26,.365,.18),(.405,.555,.235),(.603,.760,.23),(.805,.928,.16))):
  xy=[(-w*.66,low),(-w,low+.045),(-w,high-.04),(-w*.78,high),(w*.78,high),(w,high-.04),(w,low+.045),(w*.66,low)]
  p.append(skin_shading(paint('Belly band / fitted broad cream Lickitung abdomen '+str(j),CREAM,flowing(xy,3),body,.007,3),body))
 for s in (-1,1):
  eye=paint('Eye / fitted small dark Lickitung eye '+str(s),INK,oval(s*.157,1.277,.019,.028,28),head,.004,3);p.append(eye)
  p.append(paint('Eye glint / fitted white Lickitung gleam '+str(s),WHITE,oval(s*.153,1.287,.006,.007,20),eye,.002,2))
  p.append(paint('Nostril / fitted subtle Lickitung nostril '+str(s),GROOVE,oval(s*.038,1.17,.003,.008,20),head,.003,2))
  shoulder=ellipsoid('Shoulder / rooted round pink Lickitung shoulder '+str(s),PINK,(s*.269,.839,.021),(.125,.136,.133),36,24);p.append(shoulder)
  p.append(forelimb('Arm / continuous broad pink Lickitung forelimb and palm '+str(s),PINK,arm_points(s),[.109,.126,.081,.059,.002],[.11,.104,.080,.044,.002],segments=24))
  p.append(ellipsoid('Hand tip / rooted small white Lickitung nail '+str(s),WHITE,(s*.501,.76,.279),(.024,.025,.036),28,20))
  thigh=ellipsoid('Thigh / rooted powerful pink Lickitung haunch '+str(s),PINK,(s*.256,.280,.020),(.173,.213,.225),40,28);p.append(thigh)
  ring=skin_shading(paint('Thigh ring / fitted cream Lickitung leg ring '+str(s),CREAM,oval(s*.272,.306,.076,.093,40),thigh,.005,3),thigh);p.append(ring)
  p.append(skin_shading(paint('Thigh center / fitted pink Lickitung ring center '+str(s),PINK,oval(s*.272,.306,.043,.059,32),ring,.003,3),ring))
  foot=ellipsoid('Foot / grounded pink Lickitung foot '+str(s),PINK,(s*.269,.055,.156),(.107,.055,.162),36,24);p.append(foot)
  p.append(ellipsoid('Toe / rooted broad white Lickitung toe tip '+str(s),WHITE,(s*.269,.048,.282),(.075,.039,.075),36,24))
 p.append(paddle('Tail / rooted heavy upward curling pink Lickitung tail',PINK,[(0,.327,-.206),(.028,.502,-.480),(.025,.80,-.615),(.025,1.00,-.489),(.025,1.08,-.38)],[.190,.244,.223,.14,.003],[.139,.174,.148,.10,.002]))
 return fitted_shading([crease_normals(q,45.) for q in p])

@lru_cache(maxsize=2)
def sculpture(normalize=True):
 p=deepcopy(anatomy());low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 if normalize:
  for q in p:
   q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'lickitung','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'lickitung.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
