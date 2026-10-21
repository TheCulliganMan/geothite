#!/usr/bin/env python3
"""Original stocky armored drill biped with plated belly and an open fanged jaw."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,tube,fitted_triangle_contour,translate
from bovine_sculpt import curved_tube,oval,smooth
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from lugia_sculpt import paddle
from cubone_sculpt import skin_shading
from bulldog_sculpt import rounded_loft
from rhyhorn_sculpt import longitudinal
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=1.40
GRAY=(.59,.62,.68);ARMOR=(.65,.68,.73);SHADE=(.43,.47,.54)
CREAM=(.89,.83,.65);HORN=(.95,.89,.72);WHITE=(.98,.97,.91)
INK=(.07,.075,.10);RED=(.72,.22,.18);PINK=(.77,.43,.50)
TAIL_POINTS=[(0,.31,-.229),(.02,.240,-.540),(.05,.290,-.850),(.10,.470,-1.015),(.175,.690,-.995)]
TAIL_RADII=[.200,.180,.140,.090,.003]
HORN_POINTS=[(0,1.212,.431),(0,1.260,.553),(0,1.323,.691),(0,1.362,.775)]
HORN_RADII=[.096,.073,.036,.002]

def paint(name,color,xy,parent,th=.004,sub=1):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),45),parent)

def arm_path(s):
 return [(-.300,.921,.040),(-.440,.795,.140),(-.480,.795,.300)] if s==-1 else [(.300,.921,.040),(.460,.760,.100),(.490,.660,.250)]

def palm_center(s):return (-.490,.780,.371) if s==-1 else (.490,.633,.315)

def frames(points):
 result=[];side=None
 for k,p in enumerate(points):
  tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
  if side is None:side=cross(tangent,(0,1,0) if abs(tangent[1])<.9 else (1,0,0))
  side=unit(sub(side,mul(tangent,dot(side,tangent))));result.append((side,cross(tangent,side)))
 return result

def tail_collar(j,start,end):
 pp,rr=smooth(TAIL_POINTS,TAIL_RADII);ff=frames(pp);n=32;count=end-start+1;vertices=[]
 for offset in (.008,-.012):
  for k in range(start,end+1):
   side,other=ff[k];vertices.extend(add(pp[k],mul(add(mul(side,math.cos(i*math.tau/n)),mul(other,math.sin(i*math.tau/n))),rr[k]+offset)) for i in range(n))
 faces=[];inner=count*n
 for k in range(count-1):
  for i in range(n):
   a=k*n+i;b=k*n+(i+1)%n;faces.append((a,b,b+n,a+n));faces.append((a+inner+n,b+inner+n,b+inner,a+inner))
 for i in range(n):
  a=i;b=(i+1)%n;faces.append((b,a,a+inner,b+inner));a=(count-1)*n+i;b=(count-1)*n+(i+1)%n;faces.append((a,b,b+inner,a+inner))
 return crease_normals(part('Tail armor / fitted segmented grey Rhydon collar '+str(j),GRAY,vertices,faces,True),35)

def anatomy():
 body=loft('Body / continuous heavy grey Rhydon torso',GRAY,curved_profiles(((.17,.123,.140,-.045),(.31,.320,.285,-.040),(.53,.393,.325,-.017),(.75,.373,.300,.008),(.95,.303,.249,.024),(1.09,.190,.170,.034),(1.19,.104,.098,.037)),4),36)
 head=longitudinal('Head / broad armored Rhydon skull',ARMOR,[(-.100,1.286,.102,.114),(.015,1.311,.235,.171),(.170,1.307,.298,.174),(.335,1.239,.282,.154),(.444,1.185,.193,.107)],segments=32)
 upper=bill('Muzzle / broad rooted grey upper snout',GRAY,[(.170,.188,1.115,1.310),(.350,.245,1.055,1.254),(.535,.161,1.042,1.172),(.596,.045,1.070,1.118)])
 lower=bill('Jaw / open rooted grey lower muzzle',GRAY,[(.11,.140,1.025,1.160),(.29,.195,.960,1.005),(.44,.148,.970,1.012),(.56,.052,1.010,1.031)])
 chamber=ellipsoid('Mouth back / recessed dark Rhydon oral chamber',INK,(0,1.068,.245),(.137,.100,.043),24,16)
 tongue=ellipsoid('Tongue / rooted pink Rhydon tongue',PINK,(0,1.010,.410),(.105,.017,.063),24,14)
 p=[body,head,upper,lower,chamber,tongue]
 # Broad connected cream bands preserve the species' armored belly silhouette.
 for j,(lo,hi,w) in enumerate(((.230,.350,.187),(.361,.490,.276),(.501,.640,.294),(.651,.790,.270))):
  xy=[(-w*.83,lo),(-w,lo+.035),(-w,hi-.035),(-w*.82,hi),(w*.82,hi),(w,hi-.035),(w,lo+.035),(w*.83,lo)]
  p.append(paint('Belly plate / fitted cream Rhydon armor band '+str(j),CREAM,xy,body,.014,2))
 collar=[(-.23,.950),(-.271,1.008),(-.185,1.100),(-.09,1.129),(0,1.104),(.09,1.129),(.185,1.100),(.271,1.008),(.23,.950),(0,1.023)]
 collar=[(x*.88,y) for x,y in collar]
 p.append(paint('Collar / fitted pointed grey Rhydon neck armor',ARMOR,collar,body,.014,2))
 for s in (-1,1):
  ear=paddle('Ear / rooted tall pointed grey Rhydon pinna '+str(s),GRAY,[(s*.186,1.355,.042),(s*.254,1.495,.007),(s*.280,1.611,-.013)],[.081,.056,.006],[.058,.035,.004],segments=20,steps=4);p.append(ear)
  p.append(paint('Ear inset / fitted shaded Rhydon ear '+str(s),SHADE,[(s*x,y) for x,y in ((.199,1.416),(.232,1.497),(.276,1.569),(.285,1.492),(.252,1.413))],ear,.004,2))
  xy=[(s*x,y) for x,y in ((.121,1.278),(.248,1.327),(.265,1.274),(.219,1.234),(.149,1.232))]
  socket=paint('Socket / fitted angular dark Rhydon gaze '+str(s),INK,xy,head,.008,2);p.append(socket);cx=sum(x for x,y in xy)/5;cy=sum(y for x,y in xy)/5
  eye=paint('Eye / fitted narrow ivory Rhydon glare '+str(s),WHITE,[(cx+.85*(x-cx),cy+.82*(y-cy)) for x,y in xy],socket,.005,2);p.append(eye)
  iris=paint('Iris / fitted red Rhydon iris '+str(s),RED,oval(s*.207,1.277,.013,.023,20),eye,.003,1);p.append(iris)
  p.append(paint('Pupil / fitted dark Rhydon slit '+str(s),INK,oval(s*.207,1.277,.0045,.017,16),iris,.002,1))
  p.append(paint('Nostril / fitted dark Rhydon muzzle recess '+str(s),INK,oval(s*.082,1.121,.009,.005,20),upper,.003,2))
  p.append(curved_tube('Fang / rooted ivory Rhydon upper canine '+str(s),WHITE,[(s*.119,1.077,.377),(s*.123,1.050,.398),(s*.120,1.024,.409)],[.026,.020,.002],24))
  # Jagged side armor roots into the actual skull and torso.
  for j,(x,y,z,tx,ty,tz,r) in enumerate(((.252,1.355,.080,.389,1.404,.146,.064),(.260,1.263,.160,.385,1.274,.224,.061),(.223,1.167,.211,.350,1.101,.304,.058))):
   p.append(paddle(f'Cheek armor / rooted grey Rhydon point {s} {j}',ARMOR,[(s*x,y,z),(s*(x+tx)/2,(y+ty)/2,(z+tz)/2),(s*tx,ty,tz)],[r,r*.65,.002],[r*.75,r*.51,.002],segments=16,steps=4))
  p.append(ellipsoid('Shoulder / rooted rounded grey Rhydon armor '+str(s),ARMOR,(s*.300,.921,.040),(.155,.169,.165),24,16))
  p.append(paddle('Arm / short powerful grey Rhydon forelimb '+str(s),GRAY,arm_path(s),[.116,.102,.084],[.123,.110,.087],segments=20,steps=4))
  px,py,pz=palm_center(s);hand=ellipsoid('Hand / broad three-clawed grey Rhydon hand '+str(s),GRAY,(px,py,pz),(.117,.097,.108),24,18);p.append(hand)
  for j,dx in enumerate((-.068,0,.068)):
   p.append(curved_tube(f'Hand claw / three rooted ivory Rhydon claws {s} {j}',WHITE,[(px+dx*.8,py-.016,pz+.068),(px+dx,py-.040,pz+.135),(px+dx*.92,py-.072,pz+.164)],[.030,.022,.002],16))
  p.append(ellipsoid('Thigh / stocky grey Rhydon haunch '+str(s),GRAY,(s*.277,.331,-.032),(.218,.247,.234),28,18))
  p.append(paddle('Leg / short rooted grey Rhydon shin '+str(s),GRAY,[(s*.280,.321,-.030),(s*.343,.170,-.027),(s*.361,.085,-.031)],[.130,.110,.096],[.136,.112,.084],segments=24,steps=4))
  foot=translate(rounded_loft('Foot / grounded broad grey Rhydon pad '+str(s),GRAY,[(0.,.134,.155,.056),(.030,.159,.184,.065),(.080,.149,.170,.059),(.109,.097,.106,-.018)],28,.65),s*.359,0,0);p.append(foot)
  for j,dx in enumerate((-.068,.068)):
   p.append(paddle(f'Foot claw / two rooted ivory Rhydon toes {s} {j}',WHITE,[(s*.359+dx,.052,.195),(s*.359+dx*1.06,.032,.286),(s*.359+dx*1.07,.014,.319)],[.046,.037,.012],[.028,.019,.002],segments=20,steps=4))
 horn=curved_tube('Horn / rooted cream Rhydon nasal drill',HORN,HORN_POINTS,HORN_RADII,32);p.append(horn)
 # A physical raised spiral wraps the tapering drill instead of a plain cone.
 points=[];radii=[];pp,rr=smooth(HORN_POINTS,HORN_RADII);ff=frames(pp)
 for j in range(85):
  t=j/84;a=math.tau*3*t+.6;u=.4+t*9.8;k=int(u);f=u-k;center=add(mul(pp[k],1-f),mul(pp[k+1],f));r=rr[k]*(1-f)+rr[k+1]*f+.0015;side=unit(add(mul(ff[k][0],1-f),mul(ff[k+1][0],f)));other=unit(add(mul(ff[k][1],1-f),mul(ff[k+1][1],f)))
  points.append(add(center,mul(add(mul(side,math.cos(a)),mul(other,math.sin(a))),r)));radii.append(.006*(1-.45*t))
 p.append(crease_normals(tube('Horn spiral / rooted dark cream Rhydon drill ridge',(.72,.65,.49),points,radii,8),35))
 tail=curved_tube('Tail / thick rooted tapering grey Rhydon tail',GRAY,TAIL_POINTS,TAIL_RADII,32);p.append(tail)
 for j,(start,end) in enumerate(((2,4),(6,8),(10,12))):p.append(tail_collar(j,start,end))
 for j,(y,z,ty,tz,r) in enumerate(((1.045,-.092,1.218,-.236,.085),(.90,-.193,1.060,-.356,.087),(.714,-.267,.876,-.430,.084))):
  p.append(paddle('Back armor / rooted grey Rhydon dorsal point '+str(j),ARMOR,[(0,y,z),(0,(y+ty)/2,(z+tz)/2),(0,ty,tz)],[r*.52,r*.36,.002],[r,r*.72,.003],segments=12,steps=4))
 return p

@lru_cache(maxsize=1)
def raw_sculpture():return {'name':'rhydon','version':1,'coordinate_system':COORDINATES,'primitives':anatomy()}
def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());p=m['primitives']
 if not normalize:return m
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
 return m
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from rhino_glb import export_rhydon
 blob=export_rhydon();(args.output/'rhydon.glb').write_bytes(blob);print(len(blob),'bytes')
if __name__=='__main__':main()
