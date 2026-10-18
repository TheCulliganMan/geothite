#!/usr/bin/env python3
"""Original six-tailed fox with orange curl fan, curled forelock and dark planted paws."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube,fitted_patch,fitted_adaptive_contour
from bovine_sculpt import curved_tube,oval,smooth
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from raptor_bird_sculpt import paint
from chikorita_sculpt import unit,sub,cross,dot,mul,add
from skin_glb import COORDINATES,smoothstep
HEIGHT=.85;CORE_DROP=.050
BROWN=(.70,.44,.30);FACE=(.76,.50,.35);CREAM=(.98,.91,.72)
ORANGE=(.98,.55,.25);PAW=(.52,.29,.20)
EAR=(.39,.20,.12);INK=(.12,.09,.08);AMBER=(.37,.25,.22);WHITE=(1.,.98,.93)
def tail_paths():
 paths=[]
 for layer,(cx,cy,z,r) in enumerate(((.220,.500,-.710,.140),(.170,.620,-.570,.145),(.115,.730,-.440,.140))):
  for side in (-1,1):
   points=[(side*.025,.270,-.240),(side*cx*.65,.350+layer*.055,-.400)]
   for k in range(7):
    t=k/6;a=-math.pi/2+t*math.pi*1.80;radius=r*(1.-.80*t)
    points.append((side*(cx+radius*math.cos(a)),cy+radius*math.sin(a),z-.018*t))
   paths.append(tuple(points))
 return tuple(paths)
TAIL_PATHS=tail_paths()

def trunk():
 rows=curved_profiles(((-.29,.016,.025,.27),(-.23,.115,.125,.27),(-.08,.145,.145,.285),(.06,.135,.145,.30),(.14,.082,.102,.34),(.18,.018,.025,.36)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / compact horizontal brown evolution fox',BROWN,v,faces,True),40.)

def tails():
 result=[]
 for j,points in enumerate(TAIL_PATHS):
  radii=(.040,.072,.090,.084,.073,.060,.045,.030,.023)
  pp,rr=smooth(points,radii,3)
  result.append(crease_normals(tube('Tail / curled orange fox plume '+str(j),ORANGE,pp,rr,16),40.))
 return result


def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / broad rounded fox cheeks and tapered chin',FACE,curved_profiles(((.415,.045,.055,.235),(.448,.120,.110,.225),(.510,.173,.144,.204),(.590,.181,.145,.188),(.665,.147,.119,.173),(.706,.084,.074,.164),(.722,.012,.014,.160)),4),36)
 muzzle=ellipsoid('Muzzle / small soft brown fox snout',FACE,(0.,.470,.316),(.105,.042,.049),28,18)
 p=[trunk(),head,muzzle]
 neck=paddle('Neck / short rooted brown fox throat',BROWN,[(0.,.33,.10),(0.,.41,.15),(0.,.47,.20)],[.079,.078,.070],[.071,.073,.068],segments=20,steps=4);p.append(neck)
 p.append(fitted_adaptive_contour('Bib / fitted small cream fox chest',CREAM,[(-.030,.340),(-.042,.380),(-.030,.420),(0.,.435),(.030,.420),(.042,.380),(.030,.340),(0.,.321)],neck,.004,surface_at=surface(neck)))
 p+=tails()
 for side in (-1,1):
  ear=paddle('Ear / long pointed brown fox pinna '+str(side),BROWN,[(side*.104,.653,.157),(side*.164,.725,.138),(side*.234,.791,.115),(side*.284,.830,.104)],[.056,.065,.037,.0018],[.030,.026,.016,.0018],segments=20,steps=4);p.append(ear)
  xy=[(side*.121,.682),(side*.153,.754),(side*.262,.809),(side*.209,.724),(side*.161,.689)]
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
   paw=ellipsoid(f'Paw / four planted dark fox feet {side} {row}',PAW,(x,.035,z+.036),(.048,.034,.060),24,14);p.append(paw)
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
 for j,x in enumerate((-.075,0.,.075)):
  points=[(x,.669,.182),(x-.018,.733,.172)]
  for k in range(7):
   t=k/6;a=math.pi+t*math.pi*1.45;r=.047*(1.-.68*t)
   points.append((x+.025+r*math.cos(a),.754+r*math.sin(a),.173+.009*t))
  pp,rr=smooth(points,(.026,.035,.037,.034,.029,.024,.020,.015,.009),3)
  p.append(crease_normals(tube('Forelock / curled orange fox wave '+str(j),ORANGE,pp,rr,14),40.))
 for q in p:
  if q['part'].startswith(('Paw /','Toe crease /')):continue
  limb=q['part'].startswith('Leg /')
  for k in range(1,len(q['positions']),3):
   y=q['positions'][k];q['positions'][k]-=CORE_DROP*(smoothstep(.057,.290,y) if limb else 1.)
  if limb:q.update(crease_normals(q,35.))
 return {'name':'vulpix','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from vulpix_glb import export_vulpix
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_vulpix())
