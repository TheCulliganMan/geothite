#!/usr/bin/env python3
"""Original stocky armored drill biped with plated belly and an open fanged jaw."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import curved_tube,oval,smooth
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from rhyhorn_sculpt import longitudinal
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=1.40
GRAY=(.59,.62,.68);ARMOR=(.65,.68,.73);SHADE=(.43,.47,.54)
CREAM=(.89,.83,.65);HORN=(.95,.89,.72);WHITE=(.98,.97,.91)
INK=(.07,.075,.10);RED=(.72,.22,.18);PINK=(.77,.43,.50)
TAIL_POINTS=[(0,.33,-.229),(.03,.235,-.527),(.13,.25,-.829),(.28,.423,-1.057),(.421,.574,-1.101)]
TAIL_RADII=[.175,.169,.128,.080,.003]
HORN_POINTS=[(0,1.208,.384),(0,1.323,.470),(0,1.465,.574),(0,1.517,.615)]
HORN_RADII=[.094,.071,.034,.002]

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
 return crease_normals(part('Tail armor / fitted segmented grey Rhydon collar '+str(j),SHADE,vertices,faces,True),35)

def anatomy():
 body=loft('Body / continuous heavy grey Rhydon torso',GRAY,curved_profiles(((.20,.115,.124,-.04),(.34,.292,.27,-.045),(.56,.365,.30,-.015),(.78,.345,.28,.01),(.97,.292,.235,.023),(1.10,.18,.16,.041),(1.19,.095,.09,.045)),5),44)
 head=longitudinal('Head / broad armored Rhydon skull',ARMOR,[(-.08,1.277,.08,.09),(.015,1.306,.19,.16),(.16,1.305,.25,.17),(.30,1.240,.25,.16),(.40,1.184,.19,.11)])
 upper=bill('Muzzle / broad rooted grey upper snout',GRAY,[(.19,.165,1.115,1.265),(.36,.22,1.052,1.225),(.50,.145,1.04,1.148),(.535,.055,1.063,1.104)])
 lower=bill('Jaw / open rooted grey lower muzzle',GRAY,[(.11,.128,.975,1.141),(.29,.18,.876,.960),(.44,.135,.894,.945),(.51,.045,.947,.972)])
 chamber=ellipsoid('Mouth back / recessed dark Rhydon oral chamber',INK,(0,1.015,.245),(.137,.112,.043),32,22)
 tongue=ellipsoid('Tongue / rooted pink Rhydon tongue',PINK,(0,.942,.394),(.105,.021,.063),32,20)
 p=[body,head,upper,lower,chamber,tongue]
 # Broad connected cream bands preserve the species' armored belly silhouette.
 for j,(lo,hi,w) in enumerate(((.285,.42,.174),(.431,.60,.255),(.612,.79,.272),(.802,.954,.24))):
  xy=[(-w*.83,lo),(-w,lo+.035),(-w,hi-.035),(-w*.82,hi),(w*.82,hi),(w,hi-.035),(w,lo+.035),(w*.83,lo)]
  p.append(paint('Belly plate / fitted cream Rhydon armor band '+str(j),CREAM,xy,body,.014,4))
 collar=[(-.23,.918),(-.265,.987),(-.18,1.076),(-.09,1.10),(0,1.08),(.09,1.10),(.18,1.076),(.265,.987),(.23,.918),(0,.997)]
 p.append(paint('Collar / fitted pointed grey Rhydon neck armor',ARMOR,collar,body,.021,4))
 for s in (-1,1):
  ear=paddle('Ear / rooted tall pointed grey Rhydon pinna '+str(s),GRAY,[(s*.157,1.327,.055),(s*.22,1.532,.020),(s*.217,1.752,.012)],[.077,.057,.002],[.053,.027,.0015]);p.append(ear)
  p.append(paint('Ear inset / fitted shaded Rhydon ear '+str(s),SHADE,[(s*x,y) for x,y in ((.165,1.403),(.184,1.535),(.212,1.663),(.244,1.535),(.231,1.404))],ear,.005,3))
  xy=[(s*x,y) for x,y in ((.10,1.273),(.208,1.32),(.233,1.26),(.191,1.222),(.129,1.221))]
  socket=paint('Socket / fitted angular dark Rhydon gaze '+str(s),INK,xy,head,.008,3);p.append(socket);cx=sum(x for x,y in xy)/5;cy=sum(y for x,y in xy)/5
  eye=paint('Eye / fitted narrow ivory Rhydon glare '+str(s),WHITE,[(cx+.85*(x-cx),cy+.82*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  iris=paint('Iris / fitted red Rhydon iris '+str(s),RED,oval(s*.177,1.267,.012,.021,24),eye,.003,2);p.append(iris)
  p.append(paint('Pupil / fitted dark Rhydon slit '+str(s),INK,oval(s*.177,1.267,.0045,.016,20),iris,.003,2))
  p.append(paint('Nostril / fitted dark Rhydon muzzle recess '+str(s),INK,oval(s*.082,1.121,.009,.005,20),upper,.003,2))
  p.append(curved_tube('Fang / rooted ivory Rhydon upper canine '+str(s),WHITE,[(s*.119,1.077,.377),(s*.123,1.031,.398),(s*.120,.993,.409)],[.026,.020,.002],24))
  # Jagged side armor roots into the actual skull and torso.
  for j,(x,y,z,tx,ty,tz,r) in enumerate(((.201,1.274,.122,.328,1.304,.147,.062),(.207,1.205,.18,.331,1.207,.221,.059),(.175,1.142,.201,.286,1.098,.276,.053))):
   p.append(curved_tube(f'Cheek armor / rooted grey Rhydon point {s} {j}',ARMOR,[(s*x,y,z),(s*(x+tx)/2,(y+ty)/2,(z+tz)/2),(s*tx,ty,tz)],[r,r*.65,.002],20))
  p.append(ellipsoid('Shoulder / rooted rounded grey Rhydon armor '+str(s),ARMOR,(s*.286,.942,.033),(.155,.181,.17),32,22))
  p.append(paddle('Arm / short powerful grey Rhydon forelimb '+str(s),GRAY,[(s*.30,.928,.048),(s*.444,.89,.134),(s*.55,.941,.208)],[.127,.111,.091],[.135,.119,.098]))
  hand=ellipsoid('Hand / broad three-clawed grey Rhydon hand '+str(s),GRAY,(s*.585,.983,.246),(.128,.133,.124),32,22);p.append(hand)
  for j,dx in enumerate((-.068,0,.068)):
   p.append(curved_tube(f'Hand claw / three rooted ivory Rhydon claws {s} {j}',WHITE,[(s*.585+dx*.8,1.041,.294),(s*.585+dx,1.125,.32),(s*.585+dx*.92,1.158,.348)],[.030,.022,.002],20))
  p.append(ellipsoid('Thigh / stocky grey Rhydon haunch '+str(s),GRAY,(s*.262,.348,-.009),(.20,.25,.225),36,24))
  p.append(paddle('Leg / short rooted grey Rhydon shin '+str(s),GRAY,[(s*.28,.329,.018),(s*.34,.17,.099),(s*.357,.093,.168)],[.123,.104,.098],[.137,.114,.077]))
  foot=ellipsoid('Foot / grounded broad grey Rhydon pad '+str(s),GRAY,(s*.355,.055,.207),(.149,.055,.17),36,24);p.append(foot)
  for j,dx in enumerate((-.068,.068)):
   p.append(paddle(f'Foot claw / two rooted ivory Rhydon toes {s} {j}',WHITE,[(s*.355+dx,.050,.318),(s*.355+dx*1.06,.031,.391),(s*.355+dx*1.07,.012,.421)],[.045,.037,.012],[.028,.019,.002]))
 horn=curved_tube('Horn / rooted cream Rhydon nasal drill',HORN,HORN_POINTS,HORN_RADII,32);p.append(horn)
 # A physical raised spiral wraps the tapering drill instead of a plain cone.
 points=[];radii=[];pp,rr=smooth(HORN_POINTS,HORN_RADII);ff=frames(pp)
 for j in range(85):
  t=j/84;a=math.tau*3*t+.6;u=.4+t*9.8;k=int(u);f=u-k;center=add(mul(pp[k],1-f),mul(pp[k+1],f));r=rr[k]*(1-f)+rr[k+1]*f+.0015;side=unit(add(mul(ff[k][0],1-f),mul(ff[k+1][0],f)));other=unit(add(mul(ff[k][1],1-f),mul(ff[k+1][1],f)))
  points.append(add(center,mul(add(mul(side,math.cos(a)),mul(other,math.sin(a))),r)));radii.append(.006*(1-.45*t))
 p.append(curved_tube('Horn spiral / rooted dark cream Rhydon drill ridge',(.72,.65,.49),points,radii,12))
 tail=curved_tube('Tail / thick rooted tapering grey Rhydon tail',GRAY,TAIL_POINTS,TAIL_RADII,32);p.append(tail)
 for j,(start,end) in enumerate(((2,4),(6,8),(10,12))):p.append(tail_collar(j,start,end))
 for j,(y,z,ty,tz,r) in enumerate(((1.045,-.092,1.218,-.236,.085),(.90,-.193,1.060,-.356,.087),(.714,-.267,.876,-.430,.084))):
  p.append(curved_tube('Back armor / rooted grey Rhydon dorsal point '+str(j),ARMOR,[(0,y,z),(0,(y+ty)/2,(z+tz)/2),(0,ty,tz)],[r,r*.72,.002],20))
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'rhydon','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'rhydon.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
