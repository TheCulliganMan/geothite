#!/usr/bin/env python3
"""Original Dewgong with one curved seal body, flowing fans and fitted gentle face."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,fitted_patch,fitted_contour,crease_normals,tube
from seel_sculpt import pillow,surface_stroke
from bovine_sculpt import oval,smooth,curved_tube
from abra_sculpt import surface
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.03
WHITE=(.94,.97,.99);IVORY=(.99,.99,.97);INK=(.11,.12,.14);BROWN=(.35,.30,.27);FOLD=(.59,.65,.71)

def curved_body():
 points=[(0,.920,.240),(0,.860,.250),(0,.755,.230),(0,.565,.190),(0,.395,.067),(0,.245,-.023),(0,.180,-.205),(0,.275,-.407),(0,.495,-.573),(0,.725,-.618),(0,.860,-.590),(0,.966,-.520)]
 widths=[.012,.115,.180,.181,.225,.250,.230,.170,.100,.065,.048,.040]
 depths=[.012,.102,.153,.174,.175,.160,.150,.105,.075,.050,.040,.036]
 pp,ww=smooth(points,widths,4);_,dd=smooth(points,depths,4);n=40;v=[]
 for k,(x,y,z) in enumerate(pp):
  a=pp[max(k-1,0)];b=pp[min(k+1,len(pp)-1)];dy=b[1]-a[1];dz=b[2]-a[2];length=math.hypot(dy,dz);ny=dz/length;nz=-dy/length
  for j in range(n):angle=j*math.tau/n;v.append((x+ww[k]*math.sin(angle),y+dd[k]*math.cos(angle)*ny,z+dd[k]*math.cos(angle)*nz))
 f=[tuple(reversed(range(n))),tuple((len(pp)-1)*n+j for j in range(n))]
 for k in range(len(pp)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return part('Body / continuous curved head neck belly and hooked tail',WHITE,v,f,True)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=curved_body();skin=surface(body);p=[body]
 for sign in (-1,1):
  xy=contour((((.148,.493),(.210,.437),(.313,.362),(.412,.372)),((.412,.372),(.427,.315),(.456,.320),(.438,.271)),((.438,.271),(.419,.218),(.434,.199),(.397,.170)),((.397,.170),(.379,.123),(.342,.115),(.312,.145)),((.312,.145),(.225,.228),(.170,.360),(.148,.493))),10)
  flipper=pillow('Flipper / broad flowing scalloped seal fan '+str(sign),WHITE,[(sign*x,y) for x,y in xy],.110,.042);p.append(flipper)
  fs=surface(flipper)
  # Curved short folds end before the rim and follow the actual solid fan.
  for j,path in enumerate([[(.211,.380),(.264,.295),(.323,.230),(.363,.190)],[(.253,.364),(.310,.313),(.373,.278)]]):
   pts=[(sign*x,y,fs(y,sign*x)+.002) for x,y in path];p.append(surface_stroke('Flipper fold / supported flowing crease '+str(sign)+' '+str(j),FOLD,pts,[.0018]*len(pts),flipper))
  x=sign*.085;y=.786;outline=contour((((x-.025,y-.034),(x-.022,y+.008),(x-.015,y+.040),(x+.006,y+.046)),((x+.006,y+.046),(x+.029,y+.044),(x+.034,y+.010),(x+.025,y-.036)),((x+.025,y-.036),(x+.004,y-.030),(x-.007,y-.032),(x-.025,y-.034))),10)
  eye=fitted_contour('Eye / fitted tall dark tapered seal eye '+str(sign),INK,outline,skin,.005,subdivisions=3);p.append(eye)
  p.append(fitted_patch('Eye sheen / fitted soft brown lower reflection '+str(sign),BROWN,oval(x,y-.018,.015,.010,24),surface(eye),.002))
  p.append(fitted_patch('Eye glint / fitted bright ivory seal sparkle '+str(sign),IVORY,oval(x+.002,y+.015,.007,.012,24),surface(eye),.002))
 xy=contour((((.017,.936),(.069,1.008),(.159,1.094),(.275,1.137)),((.275,1.137),(.311,1.076),(.328,1.089),(.321,1.024)),((.321,1.024),(.313,.984),(.331,.962),(.300,.926)),((.300,.926),(.298,.885),(.264,.870),(.224,.902)),((.224,.902),(.150,.960),(.076,.909),(.017,.936))),10)
 for sign in (-1,1):p.append(pillow('Tail fluke / broad curved scalloped fan '+str(sign),WHITE,[(sign*x,y) for x,y in xy],-.523,.036))
 muzzle=ellipsoid('Muzzle / rounded white seal snout',WHITE,(0,.715,skin(.715,0)+.025),(.142,.052,.066),40,24);p.append(muzzle);ms=surface(muzzle)
 # Retain the canonical centroid (including its signed zero) across Python's
 # old and compensated sum implementations; the actual nose stays identical.
 nose_outline=oval(0,.740,.019,.006,28)
 nose_center=tuple(math.fsum(q[i] for q in nose_outline)/len(nose_outline) for i in (0,1))
 nose=fitted_patch('Nose / small fitted charcoal nostril',INK,nose_outline,ms,.003,center=nose_center);p.append(nose)
 p.append(fitted_patch('Nose glint / fitted tiny ivory reflection',IVORY,oval(-.006,.742,.004,.0015,16),surface(nose),.001))
 pts=[]
 for k in range(25):
  x=-.107+.214*k/24;y=.689+.85*x*x;pts.append((x,y,0.))
 # Fit every smoothed centerline sample to the actual muzzle triangles. Merely
 # offsetting sparse controls let the interpolated narrow tube float above it.
 centers,_=smooth(pts,[.0020]*len(pts))
 centers=[(x,y,ms(y,x)+.0006) for x,y,_ in centers]
 p.append(crease_normals(tube('Mouth / supported gentle white-seal smile',INK,centers,[.0020]*len(centers),10),35.))
 for sign in (-1,1):
  x=sign*.083;y=.697;z=ms(y,x)
  p.append(curved_tube('Fang / small rooted ivory lower canine '+str(sign),IVORY,[(x,y,z+.004),(sign*.080,.668,z+.019),(sign*.070,.645,z+.024)],[.013,.014,.0025],20))
 horn=loft('Horn / short swept tapered white crown',WHITE,curved_profiles(((.881,.044,.038,.236),(.919,.033,.030,.248),(.960,.018,.017,.266),(.995,.002,.0025,.286)),4),32);p.append(horn)
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye /','Eye sheen /','Eye glint /','Nose /','Nose glint /')):p[i]=crease_normals(q,35)
 return {'name':'dewgong','version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());p=m['primitives']
 if not normalize:return m
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return m
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from seal_glb import export_dewgong
 blob=export_dewgong();(args.output/'dewgong.glb').write_bytes(blob);print(len(blob),'bytes')
if __name__=='__main__':main()
