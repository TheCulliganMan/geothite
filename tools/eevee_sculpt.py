#!/usr/bin/env python3
"""Original small evolution fox with a fluffy ruff, broad gaze and cream-tipped brush."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube,fitted_patch
from bovine_sculpt import curved_tube,oval,smooth
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from chikorita_sculpt import unit,sub,cross,dot,mul,add
from skin_glb import COORDINATES,smoothstep
HEIGHT=.80;CORE_DROP=.050
BROWN=(.69,.46,.27);FACE=(.74,.52,.31);CREAM=(.96,.88,.67)
EAR=(.29,.17,.085);INK=(.075,.043,.026);AMBER=(.46,.26,.085);WHITE=(1.,.98,.93)
TAIL_POINTS=((0.,.295,-.235),(.035,.390,-.340),(.075,.525,-.425),(.115,.655,-.475),(.150,.755,-.500))
TAIL_WIDTHS=(.034,.110,.145,.091,.002)
TAIL_DEPTHS=(.030,.085,.105,.068,.002)

def trunk():
 rows=curved_profiles(((-.29,.016,.025,.27),(-.23,.115,.125,.27),(-.08,.145,.145,.285),(.06,.135,.145,.30),(.14,.082,.102,.34),(.18,.018,.025,.36)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / compact horizontal brown evolution fox',BROWN,v,faces,True),40.)

def ruff():
 n=64;rows=curved_profiles(((.310,.060,.055,0.),(.350,.160,.125,0.),(.405,.190,.155,0.),(.463,.142,.105,0.),(.493,.073,.059,0.)),3);v=[];faces=[]
 for k,(y,rx,rz,_) in enumerate(rows):
  for j in range(n):
   a=j*math.tau/n;t=k/(len(rows)-1);fur=(.5+.5*math.cos(7*a+.25))**2;bulge=math.sin(math.pi*t);spread=1.+.08*fur*bulge
   yy=y-.045*bulge*max(0.,math.cos(a))-.025*fur*bulge*bulge
   v.append((rx*math.sin(a)*spread,yy,.092+rz*math.cos(a)*spread))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Ruff / continuous fluffy cream neck and chest collar',CREAM,v,faces,True),55.)

def tail():
 points,widths=smooth(TAIL_POINTS,TAIL_WIDTHS,6);_,depths=smooth(TAIL_POINTS,TAIL_DEPTHS,6);n=24;seam=15;rings=[];side=(1.,0.,0.)
 for k,p in enumerate(points):
  tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]));side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
  rings.append([add(p,add(mul(side,widths[k]*math.cos(j*math.tau/n)),mul(other,depths[k]*math.sin(j*math.tau/n)))) for j in range(n)])
 # Both material solids share this closed, gently serrated cut exactly.
 for j in range(n):
  wiggle=.007*math.cos(6*j*math.tau/n)
  rings[seam][j]=add(rings[seam][j],mul(unit(sub(points[seam+1],points[seam-1])),wiggle))
 result=[]
 for name,color,rr in [('Tail / continuous brown brush base',BROWN,rings[:seam+1]),('Tail tip / continuous cream brush end',CREAM,rings[seam:])]:
  v=[p for ring in rr for p in ring];faces=[tuple(reversed(range(n))),tuple((len(rr)-1)*n+j for j in range(n))]
  for k in range(len(rr)-1):
   for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
  result.append(crease_normals(part(name,color,v,faces,True),40.))
 return result

def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / broad rounded fox cheeks and tapered chin',FACE,curved_profiles(((.415,.045,.055,.235),(.448,.120,.110,.225),(.510,.173,.144,.204),(.590,.181,.145,.188),(.665,.147,.119,.173),(.706,.084,.074,.164),(.722,.012,.014,.160)),4),36)
 muzzle=ellipsoid('Muzzle / small soft brown fox snout',FACE,(0.,.470,.316),(.105,.042,.049),28,18)
 p=[trunk(),head,muzzle,ruff()]
 p.append(paddle('Ruff tuft / pointed cream chest lobe',CREAM,[(0.,.400,.235),(0.,.340,.265),(0.,.258,.232)],[.080,.078,.002],[.027,.028,.002],segments=20,steps=4))
 for side in (-1,1):
  p.append(paddle('Ruff tuft / swept cream cheek lobe '+str(side),CREAM,[(side*.121,.409,.145),(side*.158,.351,.166),(side*.151,.295,.135)],[.056,.057,.002],[.026,.025,.002],segments=18,steps=4))
 p.append(paddle('Neck / short rooted brown fox throat',BROWN,[(0.,.33,.10),(0.,.41,.15),(0.,.47,.20)],[.079,.078,.070],[.071,.073,.068],segments=20,steps=4))
 p+=tail()
 for side in (-1,1):
  ear=paddle('Ear / long pointed brown fox pinna '+str(side),BROWN,[(side*.104,.673,.157),(side*.186,.796,.138),(side*.287,.930,.115),(side*.331,.977,.104)],[.064,.075,.041,.0018],[.030,.026,.016,.0018],segments=20,steps=4);p.append(ear)
  xy=[(side*.122,.713),(side*.179,.855),(side*.306,.947),(side*.229,.787),(side*.170,.732)]
  p.append(paint('Ear inset / deep warm brown fitted pinna '+str(side),EAR,xy,ear,.003,2))
  x=side*.093;y=.563
  eye=marking('Eye / large dark oval fox gaze '+str(side),INK,oval(x,y,.040,.060,20),head,.006);p.append(eye)
  iris=marking('Iris / warm amber lower fox eye '+str(side),AMBER,oval(x,y-.018,.027,.033,18),eye,.002);p.append(iris)
  pupil=marking('Pupil / dark upright fox eye center '+str(side),INK,oval(x,y+.003,.021,.041,18),eye,.004);p.append(pupil)
  p.append(marking('Glint / broad white fox eye spark '+str(side),WHITE,oval(x-side*.008,y+.025,.007,.011,16),pupil,.0025))
  p.append(marking('Glint small / tiny lower fox eye spark '+str(side),WHITE,oval(x+side*.010,y-.031,.003,.004,10),iris,.003))
  for row in (0,1):
   z=.115 if row==0 else -.175;x=side*(.092 if row==0 else .122)
   if row==1:p.append(ellipsoid('Haunch / rounded brown fox hind thigh '+str(side),BROWN,(side*.106,.267,-.180),(.072,.092,.086),24,16))
   p.append(curved_tube(f'Leg / short sturdy fox limb {side} {row}',BROWN,[(side*(.075 if row==0 else .100),.330 if row==0 else .300,z-.012),(x,.163,z),(x,.057,z+.016)],[.055 if row==0 else .065,.040,.033],18))
   paw=ellipsoid(f'Paw / four planted rounded fox feet {side} {row}',BROWN,(x,.035,z+.036),(.048,.034,.060),24,14);p.append(paw)
   skin=surface(paw)
   for j,dx in enumerate((-.013,.013)):
    pts=[]
    for k in range(8):
     yy=.012+.023*k/7;xx=x+dx;pts.append((xx,yy,skin(yy,xx)+.0006))
    p.append(tube(f'Toe crease / subtle paired fox paw divisions {side} {row} {j}',EAR,pts,[.0007]*len(pts),6))
 nose=ellipsoid('Nose / tiny dark triangular fox button',INK,(0.,.499,.358),(.013,.008,.006),16,10);p.append(nose)
 smile=[];skin=surface(muzzle)
 for k in range(25):
  x=-.047+.094*k/24;y=.455+.010*(abs(x)/.047)**2;smile.append((x,y,skin(y,x)+.001))
 p.append(tube('Smile / fitted gentle curved fox mouth',INK,smile,[.0012]*len(smile),8))
 for j,(x,tip) in enumerate(((-.030,(-.049,.755,.153)),(.0,(.001,.769,.151)),(.029,(.043,.747,.153)))):
  p.append(paddle('Crown tuft / three short pointed fox wisps '+str(j),FACE,[(x,.694,.157),tip],[.021,.0015],[.014,.0015],segments=12,steps=3))
 for q in p:
  if q['part'].startswith(('Paw /','Toe crease /')):continue
  limb=q['part'].startswith('Leg /')
  for k in range(1,len(q['positions']),3):
   y=q['positions'][k];q['positions'][k]-=CORE_DROP*(smoothstep(.057,.290,y) if limb else 1.)
  if limb:q.update(crease_normals(q,35.))
 return {'name':'eevee','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from eevee_glb import export_eevee
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_eevee())
