#!/usr/bin/env python3
"""Original mechanical steel orbs, beveled horseshoes and slotted screws."""
import argparse,copy,json,math
from pathlib import Path
from functools import lru_cache
from sculpt_geometry import part,loft,ellipsoid,fitted_contour,fitted_patch,crease_normals,tube,fitted_adaptive_contour
from abra_sculpt import surface,polygon_caps
from bovine_sculpt import oval
from wing_insect_sculpt import contour
from bivalve_sculpt import top_view
from skin_glb import COORDINATES
from cubone_sculpt import skin_shading
SPECIES=('magnemite','magneton');HEIGHT=.96
UNIT_POSES={'magnemite':((0,0,0,0),),'magneton':((0,.360,0,0),(-.190,.025,.045,2.0),(.190,.025,.045,-2.0))}
BODY=(.69,.81,.83);STEEL=(.61,.64,.67);WHITE=(.99,.99,.98);INK=(.13,.14,.17);RED=(.83,.35,.35);BLUE=(.31,.67,.84)
def beveled_plate(name,color,xy,front=.058,back=-.048,bevel=.007):
 # Remove redundant points along straight pole edges before offsetting.
 while len(xy)>3:
  kept=[b for a,b,c in zip(xy[-1:]+xy[:-1],xy,xy[1:]+xy[:1]) if abs((b[0]-a[0])*(c[1]-b[1])-(b[1]-a[1])*(c[0]-b[0]))>1e-10]
  if len(kept)==len(xy):break
  xy=kept
 if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(xy,xy[1:]+xy[:1]))<0:xy=list(reversed(xy))
 inner=[];n=len(xy)
 for a,b,c in zip(xy[-1:]+xy[:-1],xy,xy[1:]+xy[:1]):
  dx,dy=b[0]-a[0],b[1]-a[1];d=math.hypot(dx,dy);n1=(-dy/d,dx/d);dx,dy=c[0]-b[0],c[1]-b[1];d=math.hypot(dx,dy);n2=(-dy/d,dx/d);sx,sy=n1[0]+n2[0],n1[1]+n2[1];d=math.hypot(sx,sy);sx/=d;sy/=d;amount=min(bevel/max(.25,sx*n1[0]+sy*n1[1]),bevel*2);inner.append((b[0]+sx*amount,b[1]+sy*amount))
 v=[(x,y,z) for outline,z in ((inner,front),(xy,front-bevel),(xy,back+bevel),(inner,back)) for x,y in outline]
 caps=polygon_caps(inner);f=list(caps)+[tuple(3*n+i for i in reversed(q)) for q in caps]
 for k in range(3):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((b,a,a+n,b+n))
 return crease_normals(part(name,color,v,f,True),40)
def cross_outline(x,y,r,w):return [(x+a,y+b) for a,b in ((-w,r),(w,r),(w,w),(r,w),(r,-w),(w,-w),(w,-r),(-w,-r),(-w,-w),(-r,-w),(-r,w),(-w,w))]
def translate(q,x,y,z):
 q['positions']=[v+(x,y,z)[i%3] for i,v in enumerate(q['positions'])];return q

def unit():
 body=ellipsoid('Sphere / smooth blue-silver magnetic steel body',BODY,(0,0,0),(.20,.205,.180),48,30);p=[body];ss=surface(body)
 border=skin_shading(fitted_adaptive_contour('Eye border / fitted fine dark mechanical eye rim',INK,oval(0,.036,.096,.100,40),body,.005,surface_at=ss,initial_subdivisions=1),body);eye=skin_shading(fitted_adaptive_contour('Eye white / fitted broad pale mechanical eye',WHITE,oval(0,.036,.092,.096,40),border,.003,surface_at=surface(border),initial_subdivisions=1),border);p.extend((border,eye));p.append(fitted_patch('Pupil / fitted tiny mechanical eye point',INK,oval(0,.049,.0037,.0045,24),surface(eye),.002))
 for j,(x,y) in enumerate(((-.125,-.097),(.129,-.050))):
  z=ss(y,x)-.009;disc=top_view(loft('Face bolt / beveled circular steel screw '+str(j),STEEL,((0,.019,.019,0),(.005,.028,.028,0),(.014,.031,.031,0),(.024,.028,.028,0),(.027,.025,.025,0)),32));translate(disc,x,y,z);p.append(disc)
  p.append(fitted_contour('Face slot / fitted dark cross screw recess '+str(j),INK,cross_outline(x,y,.017,.005),surface(disc),.002,subdivisions=0))
 shaft=loft('Crown shaft / rooted vertical steel screw stem',STEEL,((.188,.026,.026,0),(.200,.027,.027,0),(.281,.025,.025,0),(.290,.024,.024,0)),32);p.append(shaft)
 pts=[]
 for k in range(85):
  t=k/84;a=t*math.tau*3.5;pts.append((.027*math.cos(a),.214+.066*t,.027*math.sin(a)))
 p.append(crease_normals(tube('Thread / continuous fine helical steel screw ridge',STEEL,pts,[.0034]*len(pts),10),50))
 head=crease_normals(loft('Crown head / beveled broad flat steel screw cap',STEEL,((.279,.061,.061,0),(.284,.074,.074,0),(.305,.074,.074,0),(.311,.062,.062,0)),40),40);p.append(head)
 p.append(top_view(fitted_contour('Crown slot / fitted dark cross screw recess',INK,cross_outline(0,0,.051,.012),surface(top_view(head)),.002,subdivisions=0),True))
 xy=contour((((.49,.13),(.36,.16),(.15,.16),(.15,0)),((.15,0),(.15,-.16),(.36,-.16),(.49,-.13)),((.49,-.13),(.49,-.108),(.49,-.087),(.49,-.065)),((.49,-.065),(.34,-.076),(.24,-.075),(.24,0)),((.24,0),(.24,.075),(.34,.076),(.49,.065)),((.49,.065),(.49,.087),(.49,.108),(.49,.13))),12)
 for sign in (-1,1):
  pieces=[beveled_plate('Magnet / solid open beveled steel horseshoe '+str(sign),STEEL,[(sign*x,y) for x,y in xy])]
  for top in (True,False):
   ya,yb=(.066,.136) if top else (-.136,-.066);outline=[(sign*x,y) for x,y in ((.437,ya),(.536,ya),(.536,yb),(.437,yb))]
   pieces.append(beveled_plate('Pole / flat '+('red' if (top==(sign<0)) else 'blue')+' magnetic terminal '+str(sign)+' '+str(top),RED if (top==(sign<0)) else BLUE,outline,front=.060,back=-.050,bevel=.005))
  cx=sign*.170;angle=sign*.28;c,s=math.cos(angle),math.sin(angle)
  for q in pieces:
   q['positions']=[v for i in range(0,len(q['positions']),3) for x,y,z in [q['positions'][i:i+3]] for v in (cx+(x-cx)*c-y*s,(x-cx)*s+y*c,z)];q['normals']=[v for i in range(0,len(q['normals']),3) for x,y,z in [q['normals'][i:i+3]] for v in (x*c-y*s,x*s+y*c,z)];p.append(q)
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye border /','Eye white /','Pupil /','Face slot /','Crown slot /')):
   p[i]=crease_normals(q,35)
   parent=body if q['part'].startswith('Eye border /') else (border if q['part'].startswith('Eye white /') else (eye if q['part'].startswith('Pupil /') else None))
   if parent:p[i]=skin_shading(p[i],parent)
 return p

@lru_cache(maxsize=2)
def raw_sculpture(species):
 if species not in SPECIES:raise ValueError(species)
 poses=UNIT_POSES[species]
 p=[];base=unit()
 for j,(cx,cy,cz,angle) in enumerate(poses):
  c,s=math.cos(angle),math.sin(angle)
  for q in copy.deepcopy(base):
   q['part']+=' / unit '+str(j);q['positions']=[v for i in range(0,len(q['positions']),3) for x,y,z in [q['positions'][i:i+3]] for v in (cx+x*c-y*s,cy+x*s+y*c,cz+z)];q['normals']=[v for i in range(0,len(q['normals']),3) for x,y,z in [q['normals'][i:i+3]] for v in (x*c-y*s,x*s+y*c,z)];p.append(q)
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(species,normalize=True):
 m=copy.deepcopy(raw_sculpture(species));p=m['primitives']
 if not normalize:return m
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return m
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--only',choices=SPECIES);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in ([args.only] if args.only else SPECIES):
  from magnet_glb import export_magnet
  blob=export_magnet(s);(args.output/(s+'.glb')).write_bytes(blob);print(s,len(blob),'bytes',flush=True)
if __name__=='__main__':main()
