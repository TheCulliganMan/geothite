#!/usr/bin/env python3
"""Original moonlight companion with fitted hollow gold rings and seamless banded ears/tail."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube,fitted_patch
from bovine_sculpt import curved_tube,oval,smooth
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from lugia_sculpt import paddle
from chikorita_sculpt import unit,sub,cross,dot,mul,add
from skin_glb import COORDINATES
HEIGHT=1.05
DARK=(.22,.27,.29);FACE=(.25,.30,.32);GOLD=(.98,.83,.30)
INK=(.025,.035,.040);RED=(.91,.28,.17);WHITE=(1.,.99,.94)
TAIL_POINTS=((0.,.310,-.260),(.050,.340,-.430),(.160,.470,-.560),(.230,.660,-.600),(.250,.800,-.590))

def trunk():
 rows=curved_profiles(((-.31,.016,.025,.30),(-.24,.124,.130,.315),(-.08,.145,.143,.33),(.07,.126,.137,.35),(.17,.077,.100,.40),(.205,.015,.025,.43)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / compact charcoal moonlight companion',DARK,v,faces,True),40.)

def banded_sweep(names,points,widths,depths,cuts,segments=24,steps=6):
 path=points;points,widths=smooth(path,widths,steps);_,depths=smooth(path,depths,steps);rings=[];side=(1.,0.,0.)
 for k,p in enumerate(points):
  tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]));side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
  rings.append([add(p,add(mul(side,widths[k]*math.cos(j*math.tau/segments)),mul(other,depths[k]*math.sin(j*math.tau/segments)))) for j in range(segments)])
 result=[];bounds=(0,*cuts,len(rings)-1)
 for i,(start,end) in enumerate(zip(bounds,bounds[1:])):
  rr=rings[start:end+1];n=segments;v=[p for ring in rr for p in ring];faces=[tuple(reversed(range(n))),tuple((len(rr)-1)*n+j for j in range(n))]
  for k in range(len(rr)-1):
   for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
  result.append(crease_normals(part(names[i],GOLD if i==1 else DARK,v,faces,True),40.))
 return result


def ring_patch(name,parent,cx,cy,rx,ry,inner=.58,th=.003):
 # Closed annular skin follows its stored parent, including the hollow center.
 skin=surface(parent);n=48;v=[];faces=[];radial=(inner,(inner+1.)*.5,1.)
 for offset in (th,-th*.35):
  for radius in radial:
   for j in range(n):
    a=j*math.tau/n;x=cx+rx*radius*math.cos(a);y=cy+ry*radius*math.sin(a);v.append((x,y,skin(y,x)+offset))
 layer=3*n
 for k in range(2):
  for j in range(n):
   a=k*n+j;b=k*n+(j+1)%n;faces.extend(((a,b,b+n,a+n),(a+layer+n,b+layer+n,b+layer,a+layer)))
 for j in range(n):
  a=j;b=(j+1)%n;faces.extend(((b,a,a+layer,b+layer),(a+2*n,b+2*n,b+2*n+layer,a+2*n+layer)))
 return crease_normals(part(name,GOLD,v,faces,True),40.)


def side_ring(name,parent,side,z,y,rz,ry):
 # Project the lateral parent into the same fitted-surface coordinate system.
 q=dict(parent,positions=[v for i in range(0,len(parent['positions']),3) for v in (parent['positions'][i+2],parent['positions'][i+1],side*parent['positions'][i])])
 r=ring_patch(name,q,z,y,rz,ry,th=.003)
 r['positions']=[v for i in range(0,len(r['positions']),3) for v in (side*r['positions'][i+2],r['positions'][i+1],r['positions'][i])]
 r['normals']=[v for i in range(0,len(r['normals']),3) for v in (side*r['normals'][i+2],r['normals'][i+1],r['normals'][i])]
 if side>0:
  for i in range(0,len(r['indices']),3):r['indices'][i+1],r['indices'][i+2]=r['indices'][i+2],r['indices'][i+1]
 return r


def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / rounded charcoal moonlight brow and cheeks',FACE,curved_profiles(((.490,.032,.045,.255),(.523,.095,.092,.242),(.575,.148,.130,.216),(.645,.158,.133,.192),(.714,.131,.108,.174),(.755,.065,.062,.164),(.766,.010,.012,.161)),4),36)
 muzzle=ellipsoid('Muzzle / short charcoal moonlight snout',FACE,(0.,.543,.326),(.072,.031,.038),24,14)
 p=[trunk(),head,muzzle,paddle('Neck / rooted charcoal moonlight throat',DARK,[(0.,.35,.11),(0.,.43,.15),(0.,.54,.20)],[.077,.078,.073],[.073,.075,.071],segments=20,steps=4)]
 p+=banded_sweep(('Tail / thick charcoal moonlight base','Tail band / continuous gold moonlight band','Tail tip / tapered charcoal moonlight end'),TAIL_POINTS,(.045,.085,.090,.060,.004),(.035,.060,.065,.045,.004),(15,19))
 for side in (-1,1):
  path=((side*.096,.700,.160),(side*.140,.810,.137),(side*.200,.950,.108),(side*.240,1.022,.091))
  p+=banded_sweep(('Ear / rounded charcoal moonlight base '+str(side),'Ear band / continuous gold moonlight band '+str(side),'Ear tip / tapered charcoal moonlight end '+str(side)),path,(.045,.060,.035,.003),(.027,.030,.019,.003),(10,13))
  xy=[(side*.042,.617),(side*.067,.645),(side*.125,.648),(side*.123,.616),(side*.102,.588),(side*.064,.591)]
  eye=paint('Eye / red almond moonlight gaze '+str(side),RED,xy,head,.006,3);p.append(eye)
  pupil=marking('Pupil / dark vertical moonlight eye '+str(side),INK,oval(side*.086,.619,.009,.025,20),eye,.0025);p.append(pupil)
  p.append(marking('Glint / white moonlight eye spark '+str(side),WHITE,oval(side*.069,.632,.006,.008,16),eye,.003))
  shoulder=ellipsoid('Shoulder / rounded charcoal moonlight foreleg '+str(side),DARK,(side*.087,.319,.128),(.061,.088,.069),24,16);p.append(shoulder)
  p.append(side_ring('Leg ring / fitted gold foreleg oval '+str(side),shoulder,side,.128,.322,.038,.053))
  haunch=ellipsoid('Haunch / rounded charcoal moonlight hind thigh '+str(side),DARK,(side*.108,.291,-.214),(.074,.096,.090),24,16);p.append(haunch)
  p.append(side_ring('Leg ring / fitted gold hindleg oval '+str(side),haunch,side,-.214,.296,.043,.058))
  for row in (0,1):
   z=.137 if row==0 else -.208;x=side*(.093 if row==0 else .128)
   pts=[(side*(.078 if row==0 else .104),.340 if row==0 else .310,z-.010),(x,.169,z-(.030 if row else 0.)),(x,.056,z+.018)]
   p.append(curved_tube(f'Leg / sturdy charcoal moonlight limb {side} {row}',DARK,pts,[.049 if row==0 else .058,.035,.030],18))
   paw=ellipsoid(f'Paw / four planted charcoal moonlight feet {side} {row}',DARK,(x,.031,z+.039),(.047,.031,.055),24,14);p.append(paw)
   skin=surface(paw)
   for j,dx in enumerate((-.013,.013)):
    pts=[]
    for k in range(8):
     yy=.010+.021*k/7;xx=x+dx;pts.append((xx,yy,skin(yy,xx)+.0006))
    p.append(tube(f'Toe crease / subtle moonlight paw divisions {side} {row} {j}',INK,pts,[.00065]*8,6))
 p.append(ring_patch('Forehead ring / fitted hollow gold moonlight oval',head,0.,.703,.030,.041,th=.004))
 p.append(ellipsoid('Nose / small dark moonlight button',INK,(0.,.558,.361),(.009,.005,.004),16,10))
 skin=surface(muzzle);smile=[]
 for k in range(21):
  x=-.037+.074*k/20;y=.530+.007*(abs(x)/.037)**2;smile.append((x,y,skin(y,x)+.001))
 p.append(tube('Smile / fitted fine moonlight mouth',INK,smile,[.001]*21,8))
 return {'name':'umbreon','version':1,'coordinate_system':COORDINATES,'primitives':p}


def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from umbreon_glb import export_umbreon
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_umbreon())
