#!/usr/bin/env python3
"""Original slender sun cat with wide ears, ruby forehead gem and a long bifurcated tail."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube,fitted_patch
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES,smoothstep
HEIGHT=1.0
LILAC=(.83,.70,.85);FACE=(.88,.76,.90);EAR=(.20,.29,.39)
INK=(.045,.035,.075);VIOLET=(.27,.21,.45);IRIS=(.68,.71,.88);WHITE=(1.,.99,1.)
RUBY=(.85,.17,.30)
TAIL_POINTS=((0.,.365,-.275),(.055,.340,-.450),(.185,.440,-.570),(.255,.590,-.610),(.250,.695,-.610))
FORK_POINTS={-1:((.250,.665,-.610),(.220,.750,-.625),(.120,.835,-.645),(.045,.850,-.650)),1:((.250,.665,-.610),(.285,.755,-.625),(.360,.840,-.645),(.425,.865,-.650))}

def trunk():
 rows=curved_profiles(((-.32,.014,.023,.38),(-.25,.103,.119,.395),(-.11,.115,.126,.405),(.06,.100,.121,.425),(.17,.066,.095,.47),(.205,.014,.025,.49)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / slender lilac sun cat torso',LILAC,v,faces,True),40.)

def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / rounded lilac cat brow and tapered chin',FACE,curved_profiles(((.548,.029,.043,.258),(.575,.085,.088,.246),(.622,.135,.120,.221),(.685,.148,.125,.200),(.747,.125,.105,.182),(.788,.066,.062,.172),(.800,.010,.012,.169)),4),36)
 muzzle=ellipsoid('Muzzle / tiny lilac feline snout',FACE,(0.,.592,.322),(.058,.026,.031),24,14)
 p=[trunk(),head,muzzle,paddle('Neck / tall graceful lilac feline throat',LILAC,[(0.,.42,.11),(0.,.51,.16),(0.,.60,.20)],[.065,.066,.063],[.065,.068,.062],segments=20,steps=4)]
 p.append(ellipsoid('Tail hub / smooth lilac bifurcation junction',LILAC,(.250,.689,-.610),(.027,.034,.026),20,14))
 p.append(curved_tube('Tail / long slender lilac sun cat stem',LILAC,TAIL_POINTS,[.030,.029,.027,.023,.022],20))
 for side in (-1,1):
  p.append(curved_tube('Tail fork / independently curled lilac tip '+str(side),LILAC,FORK_POINTS[side],[.025,.023,.018,.006],18))
  ear=paddle('Ear / broad triangular lilac sun cat pinna '+str(side),LILAC,[(side*.106,.738,.174),(side*.207,.800,.145),(side*.319,.870,.108),(side*.389,.916,.084)],[.072,.085,.054,.0015],[.030,.025,.016,.0015],segments=20,steps=4);p.append(ear)
  xy=[(side*.137,.750),(side*.180,.823),(side*.362,.898),(side*.280,.810),(side*.200,.761)]
  p.append(paint('Ear inset / dark blue fitted feline pinna '+str(side),EAR,xy,ear,.003,3))
  xy=[(side*.041,.656),(side*.073,.691),(side*.128,.697),(side*.123,.661),(side*.100,.636),(side*.064,.638)]
  eye=paint('Eye / slanted violet sun cat gaze '+str(side),VIOLET,xy,head,.006,3);p.append(eye)
  iris=marking('Iris / pale silver violet feline eye '+str(side),IRIS,oval(side*.084,.663,.024,.022,20),eye,.0025);p.append(iris)
  p.append(marking('Glint / broad white feline eye spark '+str(side),WHITE,oval(side*.080,.671,.006,.008,16),iris,.0025))
  # A broad tapered sensory cheek fan on each side, rooted into the head.
  p.append(paddle('Cheek wisp / long swept lilac feline whisker fan '+str(side),FACE,[(side*.112,.622,.198),(side*.172,.582,.177),(side*.220,.525,.143)],[.047,.044,.0015],[.026,.021,.0015],segments=16,steps=4))
  p.append(ellipsoid('Shoulder / softly joined lilac feline foreleg '+str(side),LILAC,(side*.074,.421,.125),(.046,.064,.056),22,14))
  for row in (0,1):
   z=.137 if row==0 else -.220;x=side*(.080 if row==0 else .115)
   if row==1:p.append(ellipsoid('Haunch / rounded lilac feline hind thigh '+str(side),LILAC,(side*.095,.353,-.222),(.069,.101,.093),24,16))
   pts=[(side*(.070 if row==0 else .092),.432 if row==0 else .372,z-.009),(x,.202,z-(.040 if row else 0.)),(x,.054,z+.018)]
   p.append(curved_tube(f'Leg / slender lilac feline limb {side} {row}',LILAC,pts,[.040 if row==0 else .053,.027,.025],18))
   paw=ellipsoid(f'Paw / four planted lilac feline feet {side} {row}',LILAC,(x,.028,z+.037),(.039,.028,.049),22,12);p.append(paw)
   skin=surface(paw)
   for j,dx in enumerate((-.011,.011)):
    pts=[]
    for k in range(8):
     yy=.009+.018*k/7;xx=x+dx;pts.append((xx,yy,skin(yy,xx)+.0006))
    p.append(tube(f'Toe crease / fine feline paw divisions {side} {row} {j}',VIOLET,pts,[.00065]*8,6))
 gem=marking('Gem / fitted oval ruby forehead lens',RUBY,oval(0.,.731,.018,.023,24),head,.009);p.append(gem)
 p.append(marking('Gem glint / seated white ruby sparkle',WHITE,oval(-.005,.738,.004,.006,14),gem,.0025))
 p.append(ellipsoid('Nose / tiny dark feline button',INK,(0.,.606,.350),(.006,.004,.003),14,10))
 skin=surface(muzzle);smile=[]
 for k in range(21):
  x=-.031+.062*k/20;y=.581+.006*(abs(x)/.031)**2;smile.append((x,y,skin(y,x)+.0009))
 p.append(tube('Smile / fine fitted sun cat mouth',INK,smile,[.0009]*21,8))
 return {'name':'espeon','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from espeon_glb import export_espeon
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_espeon())
