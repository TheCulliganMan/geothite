#!/usr/bin/env python3
"""Original hollow Shellder valves and layered spiked Cloyster armor."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,curved_profiles,fitted_patch,fitted_contour,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from abra_sculpt import surface
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('shellder','cloyster');HEIGHT={s:1.05 for s in SPECIES}
LAVENDER=(.59,.55,.77);PURPLE=(.57,.51,.68);HOOD=(.58,.65,.73);ORB=(.22,.20,.33)
INK=(.10,.11,.14);WHITE=(.98,.99,.97);PINK=(.87,.60,.70)

def cup(name,color,cy,w,h,d,cz,thickness=.028,wave=0,side=0):
 n=32 if side else 64;m=n+1 if side else n;radii=(.12,.25,.40,.56,.72,.86,1.);v=[(0,cy,cz-d)];shrink=.91
 def contour_xy(r,j,scale=1):
  a=(-math.pi/2 if side>0 else math.pi/2)+j*math.pi/n if side else j*math.tau/n;mod=1+wave*math.cos(6*a)+wave*.30*math.sin(10*a+.4)
  return (w*r*math.cos(a)*mod*scale,cy+h*r*math.sin(a)*mod*scale)
 for r in radii:
  for j in range(m):x,y=contour_xy(r,j);v.append((x,y,cz-d*math.sqrt(max(0,1-r*r))))
 inner=len(v);v.append((0,cy,cz-d+thickness))
 for r in radii:
  for j in range(m):x,y=contour_xy(r,j,shrink);v.append((x,y,cz-(d-thickness)*math.sqrt(max(0,1-r*r))))
 f=[]
 for j in range(n):f.extend(((0,1+(j+1)%m,1+j),(inner,inner+1+j,inner+1+(j+1)%m)))
 for k in range(len(radii)-1):
  for j in range(n):
   a=1+k*m+j;b=1+k*m+(j+1)%m;c=a+m;e=b+m;f.extend(((a,b,e,c),(inner+a,inner+c,inner+e,inner+b)))
 last=1+(len(radii)-1)*m
 for j in range(n):a=last+j;b=last+(j+1)%m;f.append((a,b,inner+b,inner+a))
 if side:
  for edge in (0,m-1):
   path=[0]+[1+k*m+edge for k in range(len(radii))]
   for a,b in zip(path,path[1:]):
    face=(a,b,inner+b,inner+a);f.append(tuple(reversed(face)) if edge else face)
 return crease_normals(part(name,color,v,f,True),45)

def tilt_point(point,angle,cy,cz):
 x,y,z=point;y-=cy;z-=cz;c=math.cos(angle);s=math.sin(angle)
 return (x,cy+c*y-s*z,cz+s*y+c*z)
def tilt(p,angle,cy,cz):
 p=dict(p)
 for key in ('positions','normals'):
  p[key]=[v for i in range(0,len(p[key]),3) for v in tilt_point(p[key][i:i+3],angle,cy if key=='positions' else 0.,cz if key=='positions' else 0.)]
 return p
def turn(p,angle,cz=-.220):
 p=dict(p);c=math.cos(angle);s=math.sin(angle)
 for key in ('positions','normals'):
  out=[]
  for i in range(0,len(p[key]),3):
   x,y,z=p[key][i:i+3];z-=cz if key=='positions' else 0.
   out.extend((x*c+z*s,y,-x*s+z*c+(cz if key=='positions' else 0.)))
  p[key]=out
 return p

def top_view(q,inverse=False):
 q=dict(q)
 for field in ('positions','normals'):
  q[field]=[v for i in range(0,len(q[field]),3) for x,y,z in [q[field][i:i+3]] for v in ((x,z,-y) if inverse else (x,-z,y))]
 return q

def tongue():
 rows=curved_profiles(((.095,.052,.035,.370),(.18,.068,.037,.305),(.28,.088,.031,.220),(.395,.085,.026,.180),(.43,.024,.008,.177)),4);n=40;v=[]
 for z,w,d,y in rows:v.extend((w*math.sin(j*math.tau/n),y+d*math.cos(j*math.tau/n),z) for j in range(n))
 f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part('Tongue / long broad hanging pink mollusk paddle',PINK,v,f,True),45)

def ribbon_inside(xy,span,amount=.38):
 upper=[];lower=[]
 for k in range(21):
  x=-span+2*span*k/20;hits=[]
  for a,b in zip(xy,xy[1:]+xy[:1]):
   if min(a[0],b[0])<=x<max(a[0],b[0]):hits.append(a[1]+(b[1]-a[1])*(x-a[0])/(b[0]-a[0]))
  lo=min(hits);hi=max(hits);mid=(lo+hi)/2;upper.append((x,mid+(hi-lo)*amount));lower.append((x,mid-(hi-lo)*amount))
 return upper+list(reversed(lower))

@lru_cache(maxsize=2)
def raw_sculpture(species):
 if species not in SPECIES:raise ValueError(species)
 p=[];adult=species=='cloyster'
 if not adult:
  upper_angle=.90;lower_angle=-.65
  shell=cup('Shell / hollow lavender upper hood',LAVENDER,.49,.300,.250,.180,.035);original_shell=shell;shell=tilt(shell,upper_angle,.49,.035);p.append(shell)
  p.append(tilt(cup('Lining / hollow dark upper hood interior',(.37,.36,.42),.49,.300*.955,.250*.955,.180-.021,.037,.009),upper_angle,.49,.035))
  jaw=tilt(cup('Jaw / hollow lavender lower valve',LAVENDER,.250,.255,.150,.110,.060),lower_angle,.250,.060);p.append(jaw)
  p.append(tilt(cup('Jaw lining / hollow dark lower valve interior',(.37,.36,.42),.250,.255*.955,.150*.955,.110-.021,.062,.009),lower_angle,.250,.060))
  body=ellipsoid('Body / round dark exposed mollusk',INK,(0,.485,.005),(.210,.190,.150),40,28);p.append(body);skin=surface(body)
  crest=ellipsoid('Hinge / seated lavender rear valve crest',LAVENDER,(0,.635,-.120),(.140,.060,.110),32,22);p.append(crest)
  for sign in (-1,1):
   x=sign*.220;y=.630;z=surface(original_shell)(y,x)-.018;root=tilt_point((x,y,z),upper_angle,.49,.035)
   p.append(curved_tube('Horn / rooted short upper valve point '+str(sign),LAVENDER,[root,(root[0]+sign*.065,root[1]+.085,root[2]-.025),(root[0]+sign*.135,root[1]+.175,root[2]-.080)],[.047,.030,.0025],24))
   x=sign*.230;y=.355;z=surface(original_shell)(y,x)-.017;root=tilt_point((x,y,z),upper_angle,.49,.035)
   p.append(curved_tube('Rear horn / rooted low valve point '+str(sign),LAVENDER,[root,(root[0]+sign*.110,root[1]-.015,root[2]-.070),(root[0]+sign*.155,root[1]-.028,root[2]-.100)],[.036,.021,.0025],20))
   x=sign*.103;y=.562;eye=fitted_contour('Eye white / fitted large pale mollusk oval '+str(sign),WHITE,oval(x,y,.044,.058,24),skin,.007,subdivisions=2);p.append(eye)
   p.append(fitted_patch('Pupil / tiny fitted mollusk point '+str(sign),INK,oval(x+.004,.580,.0035,.0045,20),surface(eye),.0025))
  t=tongue();p.append(t);ss=surface(top_view(t));pts=[(0,-z,ss(-z,0)+.0028) for z in (.155,.210,.265,.320,.375)]
  p.append(top_view(curved_tube('Tongue crease / supported pink paddle fold',(.61,.37,.47),pts,[.0020]*len(pts),10),True))
  rear=reverse_front(original_shell);ss=surface(rear)
  for j,x in enumerate((-.145,0,.145)):
   pts=[(x,y,ss(y,x)+.0018) for y in [.490+.185*k/24 for k in range(25)]]
   p.append(tilt(reverse_front(curved_tube('Shell groove / supported rear hood division '+str(j),(.40,.36,.58),pts,[.0018]*len(pts),10)),upper_angle,.49,.035))
 else:
  shell=cup('Shell guide',PURPLE,.500,.310,.420,.170,-.050,wave=.065)
  for side in (-1,1):p.append(turn(cup('Shell / hollow wavy armored valve '+str(side),PURPLE,.500,.310,.420,.170,-.050,wave=.065,side=side),side*.12))
  hood=cup('Inner hood / hollow scalloped blue-gray mantle',HOOD,.500,.230,.340,.250,.045,wave=.040);p.append(hood)
  p.append(cup('Mantle lining / recessed charcoal inner shell interior',(.34,.35,.39),.500,.230*.955,.340*.955,.250-.021,.047,.009,wave=.040))
  body=ellipsoid('Body / round purple central mollusk orb',ORB,(0,.500,-.080),(.165,.194,.140),40,28);p.append(body);skin=surface(body)
  for sign in (-1,1):
   for j,(a,length) in enumerate(((-.90,.13),(-.35,.25),(.20,.28),(.95,.30))):
    mod=1+.065*math.cos(6*a)+.065*.30*math.sin(10*a+.4);x=sign*.310*.92*math.cos(a)*mod;y=.500+.420*.92*math.sin(a)*mod;z=surface(shell)(y,x)-.018
    end=(x+sign*length*math.cos(a),y+length*math.sin(a),z+.050)
    p.append(turn(curved_tube('Outer spike / rooted tapered armor lance '+str(sign)+' '+str(j),PURPLE,[(x,y,z),((x+end[0])/2,(y+end[1])/2,z+.015),end],[.048 if j==3 else .044,.027,.0025],24),sign*.12))
   xy=[(sign*x,y) for x,y in ((.020,.535),(.125,.601),(.115,.514),(.071,.516))]
   eye=fitted_contour('Eye white / fitted sharp pale mollusk eye '+str(sign),WHITE,xy,skin,.008,subdivisions=3);p.append(eye)
   p.append(fitted_patch('Pupil / fitted sharp mollusk point '+str(sign),INK,oval(sign*.076,.542,.0035,.008,20),surface(eye),.0025))
  rear=reverse_front(shell);z=-surface(rear)(.795,0)+.018
  p.append(curved_tube('Rear crown / rooted long purple armor spike',PURPLE,[(0,.795,z),(0,.965,z-.030),(0,1.140,z-.150)],[.054,.034,.0025],28))
  z=surface(hood)(.770,0)-.018
  p.append(curved_tube('Inner horn / rooted forward blue-gray mantle lance',HOOD,[(0,.770,z),(-.040,.835,z+.040),(-.135,.975,z+.180)],[.045,.027,.0025],24))
  xy=contour((((-.106,.447),(-.047,.407),(.046,.410),(.106,.447)),((.106,.447),(.062,.382),(-.065,.382),(-.106,.447))),16)
  border=fitted_contour('Grin border / fitted dark curved mollusk grin',INK,xy,skin,.008,subdivisions=3);p.append(border)
  teeth=fitted_contour('Teeth / fitted pale four-panel mollusk grin',WHITE,ribbon_inside(xy,.095),surface(border),.003,subdivisions=3);p.append(teeth);ss=surface(teeth)
  for j,x in enumerate((-.044,0,.044)):
   outline=ribbon_inside(xy,.095);hits=[]
   for a,b in zip(outline,outline[1:]+outline[:1]):
    if min(a[0],b[0])<=x<max(a[0],b[0]):hits.append(a[1]+(b[1]-a[1])*(x-a[0])/(b[0]-a[0]))
   lo=min(hits)+.003;hi=max(hits)-.003;pts=[(x,y,ss(y,x)+.002) for y in (lo,(lo+hi)/2,hi)];p.append(curved_tube('Tooth gap / supported dark grin division '+str(j),INK,pts,[.0015]*len(pts),10))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye white /','Pupil /','Grin border /','Teeth /')):p[i]=crease_normals(q,35)
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(species,normalize=True):
 m=deepcopy(raw_sculpture(species));p=m['primitives']
 if not normalize:return m
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[species]/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return m
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--only',choices=SPECIES);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from bivalve_glb import export_bivalve
 for s in ([args.only] if args.only else SPECIES):
  blob=export_bivalve(s);(args.output/(s+'.glb')).write_bytes(blob);print(s,len(blob),'bytes',flush=True)
if __name__=='__main__':main()
