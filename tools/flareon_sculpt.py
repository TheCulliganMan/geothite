#!/usr/bin/env python3
"""Original flame fox with voluminous cream collar, swept forehead tuft and broad curled plume."""
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
HEIGHT=.97;CORE_DROP=.045;HEAD_LIFT=.040
ORANGE=(.98,.48,.17);FACE=(1.,.54,.23);CREAM=(.99,.91,.62)
EAR=(.055,.055,.070);INK=(.018,.025,.030);BLUE=(.015,.22,.32);WHITE=(1.,.98,.93)
TAIL_POINTS=((0.,.295,-.235),(.050,.400,-.400),(.170,.560,-.490),(.250,.730,-.540),(.230,.920,-.480))
TAIL_WIDTHS=(.045,.170,.230,.160,.002)
TAIL_DEPTHS=(.040,.130,.180,.130,.002)

def trunk():
 rows=curved_profiles(((-.29,.016,.025,.27),(-.23,.115,.125,.27),(-.08,.145,.145,.285),(.06,.135,.145,.30),(.14,.082,.102,.34),(.18,.018,.025,.36)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / compact horizontal orange flame fox',ORANGE,v,faces,True),40.)

def ruff():
 n=64;rows=curved_profiles(((.290,.070,.062,0.),(.340,.205,.165,0.),(.420,.240,.190,0.),(.495,.200,.148,0.),(.545,.077,.065,0.)),3);v=[];faces=[]
 for k,(y,rx,rz,_) in enumerate(rows):
  for j in range(n):
   a=j*math.tau/n;t=k/(len(rows)-1);fur=(.5+.5*math.cos(7*a+.25))**2;bulge=math.sin(math.pi*t);spread=1.+.08*fur*bulge
   yy=y-.045*bulge*max(0.,math.cos(a))-.025*fur*bulge*bulge
   v.append((rx*math.sin(a)*spread,yy,.092+rz*math.cos(a)*spread))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Ruff / voluminous continuous cream flame collar',CREAM,v,faces,True),55.)

def tail():
 points,widths=smooth(TAIL_POINTS,TAIL_WIDTHS,7);_,depths=smooth(TAIL_POINTS,TAIL_DEPTHS,7);n=32;rings=[];side=(1.,0.,0.)
 for k,p in enumerate(points):
  tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]));side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
  ring=[]
  for j in range(n):
   a=j*math.tau/n;t=k/(len(points)-1);fur=1.+.065*math.sin(math.pi*t)*math.cos(7*a+.8*t)
   ring.append(add(p,add(mul(side,widths[k]*math.cos(a)*fur),mul(other,depths[k]*math.sin(a)*fur))))
  rings.append(ring)
 v=[p for ring in rings for p in ring];faces=[tuple(reversed(range(n))),tuple((len(rings)-1)*n+j for j in range(n))]
 for k in range(len(rings)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return [crease_normals(part('Tail / broad continuous cream curled flame plume',CREAM,v,faces,True),45.)]

def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / broad rounded fox cheeks and tapered chin',FACE,curved_profiles(((.415,.045,.055,.235),(.448,.120,.110,.225),(.510,.173,.144,.204),(.590,.181,.145,.188),(.665,.147,.119,.173),(.706,.084,.074,.164),(.722,.012,.014,.160)),4),36)
 muzzle=ellipsoid('Muzzle / small soft orange fox snout',FACE,(0.,.470,.316),(.105,.042,.049),28,18)
 p=[trunk(),head,muzzle,ruff()]
 p.append(paddle('Ruff tuft / pointed cream chest lobe',CREAM,[(0.,.420,.245),(0.,.335,.295),(0.,.240,.243)],[.110,.098,.002],[.045,.040,.002],segments=20,steps=4))
 for side in (-1,1):
  p.append(paddle('Ruff tuft / swept cream cheek lobe '+str(side),CREAM,[(side*.153,.425,.145),(side*.207,.351,.176),(side*.194,.280,.145)],[.079,.072,.002],[.037,.034,.002],segments=18,steps=4))
 p.append(paddle('Neck / short rooted orange fox throat',ORANGE,[(0.,.33,.10),(0.,.41,.15),(0.,.51,.20)],[.079,.078,.070],[.071,.073,.068],segments=20,steps=4))
 p+=tail()
 for side in (-1,1):
  ear=paddle('Ear / long pointed orange fox pinna '+str(side),ORANGE,[(side*.104,.673,.157),(side*.186,.796,.138),(side*.287,.930,.115),(side*.331,.977,.104)],[.064,.075,.041,.0018],[.030,.026,.016,.0018],segments=20,steps=4);p.append(ear)
  xy=[(side*.122,.713),(side*.179,.855),(side*.306,.947),(side*.229,.787),(side*.170,.732)]
  p.append(paint('Ear inset / deep charcoal fitted flame pinna '+str(side),EAR,xy,ear,.003,2))
  x=side*.093;y=.563
  eye=marking('Eye / large dark oval fox gaze '+str(side),INK,oval(x,y,.040,.060,20),head,.006);p.append(eye)
  iris=marking('Iris / deep blue lower fox eye '+str(side),BLUE,oval(x,y-.018,.027,.033,18),eye,.002);p.append(iris)
  pupil=marking('Pupil / dark upright fox eye center '+str(side),INK,oval(x,y+.003,.021,.041,18),eye,.004);p.append(pupil)
  p.append(marking('Glint / broad white fox eye spark '+str(side),WHITE,oval(x-side*.008,y+.025,.007,.011,16),pupil,.0025))
  p.append(marking('Glint small / tiny lower fox eye spark '+str(side),WHITE,oval(x+side*.010,y-.031,.003,.004,10),iris,.003))
  for row in (0,1):
   z=.115 if row==0 else -.175;x=side*(.092 if row==0 else .122)
   if row==1:p.append(ellipsoid('Haunch / rounded orange fox hind thigh '+str(side),ORANGE,(side*.106,.267,-.180),(.072,.092,.086),24,16))
   p.append(curved_tube(f'Leg / short sturdy fox limb {side} {row}',ORANGE,[(side*(.075 if row==0 else .100),.330 if row==0 else .300,z-.012),(x,.163,z),(x,.057,z+.016)],[.055 if row==0 else .065,.040,.033],18))
   paw=ellipsoid(f'Paw / four planted rounded fox feet {side} {row}',ORANGE,(x,.035,z+.036),(.048,.034,.060),24,14);p.append(paw)
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
 p.append(paddle('Crown tuft / broad swept cream forehead flame',CREAM,[(0.,.666,.230),(-.018,.745,.242),(.012,.820,.187),(.066,.894,.123)],[.069,.075,.047,.0015],[.034,.042,.028,.0015],segments=20,steps=4))
 for side in (-1,1):
  p.append(paddle('Crown tuft / seated cream flame wisp '+str(side),CREAM,[(side*.033,.698,.194),(side*.062,.761,.180),(side*.082,.822,.132)],[.031,.027,.0015],[.021,.018,.0015],segments=14,steps=4))
 for q in p:
  if q['part'].startswith(('Paw /','Toe crease /')):continue
  limb=q['part'].startswith('Leg /')
  if q['part'].startswith(('Head /','Muzzle /','Eye /','Iris /','Pupil /','Glint /','Glint small /','Nose /','Smile /','Ear /','Ear inset /','Crown tuft /')):
   for k in range(1,len(q['positions']),3):q['positions'][k]+=HEAD_LIFT
  for k in range(1,len(q['positions']),3):
   y=q['positions'][k];q['positions'][k]-=CORE_DROP*(smoothstep(.057,.290,y) if limb else 1.)
  if limb:q.update(crease_normals(q,35.))
 return {'name':'flareon','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from flareon_glb import export_flareon
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_flareon())
