#!/usr/bin/env python3
"""Original reclining Seel: continuous raised tail, lobed paddles and playful face."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,ellipsoid,curved_profiles,fitted_patch,crease_normals,tube
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,smooth
from jigglypuff_sculpt import pillow_ear
from chikorita_sculpt import linear
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.03
WHITE=(.86,.92,.97);IVORY=(.99,.97,.88);CREAM=(.99,.90,.66);INK=(.10,.14,.16);TEAL=(.25,.49,.47);PINK=(.81,.56,.69)

def pillow(name,color,xy,z,depth):
 # Reuse the closed rounded perimeter; transform normals by inverse scale.
 # An explicit centroid keeps reviewed signed-zero normals on old and new
 # Python summation implementations, without normalizing unrelated zeros.
 center=tuple(math.fsum(p[i] for p in xy)/len(xy) for i in (0,1))
 q=pillow_ear(name,xy,center=center);q['base_color']=linear(color);scale=depth/.060
 q['positions']=[v if i%3!=2 else z+(v-.01)*scale for i,v in enumerate(q['positions'])]
 normals=[]
 for i in range(0,len(q['normals']),3):
  x,y,zz=q['normals'][i:i+3];zz/=scale;l=math.sqrt(x*x+y*y+zz*zz);normals.extend((x/l,y/l,zz/l))
 q['normals']=normals;return q

def horizontal(q):
 # Proper quarter turn around X: local XY outlines become floor paddles.
 for field in ('positions','normals'):
  q[field]=[v for i in range(0,len(q[field]),3) for x,y,z in [q[field][i:i+3]] for v in (x,z,-y)]
 return q

def surface_stroke(name,color,points,radii,parent,segments=10):
 # Smooth the original XY controls first, then seat every resulting ring in
 # the stored surface. Interpolating sparse depths left the narrow folds and
 # brows floating between controls. Preserve their outlines and round radii.
 centers,rr=smooth(points,radii);skin=surface(parent)
 centers=[(x,y,skin(y,x)+r*.35) for (x,y,z),r in zip(centers,rr)]
 return crease_normals(tube(name,color,centers,rr,segments),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 rows=((- .28,.050,.060,.235),(-.19,.210,.180,.235),(.02,.280,.190,.230),(.20,.260,.220,.250),(.39,.215,.220,.310),(.57,.140,.160,.430),(.72,.075,.110,.620),(.84,.047,.070,.770),(.87,.025,.035,.800))
 body=horizontal(loft('Body / continuous reclining seal and rising tail stem',WHITE,curved_profiles(rows,4),40))
 head=ellipsoid('Head / broad round ice-blue seal cranium',WHITE,(0,.275,.315),(.232,.223,.215),40,28);p=[body,head];hs=surface(head)
 for sign in (-1,1):
  xy=contour((((.075,-.055),(.255,-.015),(.400,-.005),(.494,-.066)),((.494,-.066),(.541,-.086),(.535,-.142),(.502,-.165)),((.502,-.165),(.548,-.177),(.530,-.220),(.489,-.234)),((.489,-.234),(.519,-.255),(.474,-.295),(.424,-.288)),((.424,-.288),(.328,-.286),(.221,-.192),(.075,-.055))),8)
  paddle=pillow('Flipper / rounded three-lobed front paddle '+str(sign),WHITE,[(sign*x,y) for x,y in xy],.055,.037);fs=surface(paddle)
  p.append(horizontal(deepcopy(paddle)))
  for j,path in enumerate((((.230,-.080),(.320,-.080),(.430,-.090)),((.280,-.140),(.360,-.180),(.440,-.220)))):
   points=[(sign*x,y,fs(y,sign*x)+.0016) for x,y in path]
   p.append(horizontal(surface_stroke(f'Flipper fold / supported rounded paddle crease {sign} {j}',(.55,.65,.74),points,[.0014]*len(points),paddle,8)))
  xy=contour((((.010,.762),(.047,.894),(.035,.956),(.098,1.017)),((.098,1.017),(.172,1.091),(.290,1.104),(.329,1.053)),((.329,1.053),(.370,1.016),(.362,.974),(.328,.966)),((.328,.966),(.347,.926),(.324,.888),(.290,.900)),((.290,.900),(.285,.855),(.242,.845),(.211,.868)),((.211,.868),(.150,.901),(.095,.788),(.010,.762))),8)
  p.append(pillow('Tail fluke / raised rounded three-lobed fan '+str(sign),WHITE,[(sign*x,y) for x,y in xy],-.855,.040))
  x=sign*.102;y=.339;eye=fitted_patch('Eye / fitted glossy dark seal oval '+str(sign),INK,oval(x,y,.023,.035,32),hs,.005);p.append(eye)
  p.append(fitted_patch('Eye sheen / fitted muted teal lower crescent '+str(sign),TEAL,oval(x,y-.017,.013,.008,24),surface(eye),.0018))
  p.append(fitted_patch('Eye glint / fitted bright seal reflection '+str(sign),IVORY,oval(x-sign*.004,y+.013,.008,.011,24),surface(eye),.002))
  points=[]
  for k in range(17):
   t=k/16;xx=x-.025+.050*t;yy=.389+.017*math.sin(math.pi*t);points.append((xx,yy,hs(yy,xx)+.003))
  p.append(surface_stroke('Brow / supported gentle seal arch '+str(sign),INK,points,[.0021]*len(points),head))
 horn=loft('Horn / short tapered ice-breaking crown',WHITE,curved_profiles(((.445,.065,.056,.302),(.487,.052,.046,.309),(.550,.030,.029,.320),(.619,.013,.012,.333),(.654,.0025,.003,.343)),4),32);p.append(horn)
 xy=contour((((0,.253),(.072,.272),(.123,.284),(.169,.258)),((.169,.258),(.219,.235),(.215,.166),(.165,.139)),((.165,.139),(.101,.121),(.066,.167),(0,.155)),((0,.155),(-.066,.167),(-.101,.121),(-.165,.139)),((-.165,.139),(-.215,.166),(-.219,.235),(-.169,.258)),((-.169,.258),(-.123,.284),(-.072,.272),(0,.253))),8)
 muzzle=pillow('Muzzle / continuous cream bilobed cheek pad',CREAM,xy,.493,.061);p.append(muzzle)
 mouth=ellipsoid('Mouth / rooted dark playful opening',INK,(0,.141,.532),(.087,.050,.041),32,22);p.append(mouth)
 tongue=ellipsoid('Tongue / broad rounded lilac playful tongue',PINK,(0,.111,.555),(.075,.045,.026),32,22);p.append(tongue)
 ts=surface(tongue);points=[(0,y,ts(y,0)+.002) for y in (.115,.122,.129,.136,.142)];p.append(surface_stroke('Tongue crease / fitted central short fold',(.50,.34,.45),points,[.0018]*len(points),tongue))
 nose=ellipsoid('Nose / small glossy charcoal seal nose',INK,(0,.245,surface(muzzle)(.245,0)+.008),(.027,.015,.018),32,20);p.append(nose)
 p.append(fitted_patch('Nose glint / fitted tiny seal highlight',IVORY,oval(-.005,.250,.007,.003,20),surface(nose),.001))
 for sign in (-1,1):
  p.append(curved_tube('Tusk / curved short ivory seal canine '+str(sign),IVORY,[(sign*.133,.220,.535),(sign*.140,.183,.567),(sign*.136,.146,.583),(sign*.116,.139,.588)],[.018,.024,.017,.003],24))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye /','Eye sheen /','Eye glint /','Nose glint /')):p[i]=crease_normals(q,35)
 return {'name':'seel','version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());p=m['primitives']
 if not normalize:return m
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return m
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from seal_glb import export_seel
 blob=export_seel();(args.output/'seel.glb').write_bytes(blob);print(len(blob),'bytes')
if __name__=='__main__':main()
