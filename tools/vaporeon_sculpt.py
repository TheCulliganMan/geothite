#!/usr/bin/env python3
"""Original aquatic evolution cat, pleated frill, fan fins and long mermaid tail."""
import argparse,math
from pathlib import Path
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube,fitted_patch
from bovine_sculpt import curved_tube,oval,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from goldfish_sculpt import surface
from spider_sculpt import upper_skin
from skin_glb import COORDINATES
HEIGHT=1.00
CYAN=(.40,.73,.82);NAVY=(.10,.38,.52);CREAM=(.96,.94,.76);WHITE=(.94,.96,.98)
INK=(.045,.065,.11);VIOLET=(.28,.22,.43);GLINT=(1.,.99,.96)
TAIL_POINTS=((0.,.315,-.300),(0.,.255,-.460),(.15,.190,-.610),(.36,.220,-.650),(.50,.330,-.600),(.54,.450,-.540))
TAIL_RADII=(.088,.080,.070,.055,.038,.020)

def trunk():
 rows=curved_profiles(((-.355,.018,.025,.305),(-.29,.115,.126,.305),(-.12,.145,.143,.320),(.045,.132,.145,.350),(.14,.091,.123,.395),(.19,.020,.028,.425)),4);n=32;v=[];f=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part('Body / long rounded cyan aquatic cat',CYAN,v,f,True),40.)

def frill():
 n=80;v=[];f=[];layers=5
 for face in (1,-1):
  for k in range(layers):
   t=k/(layers-1)
   for j in range(n):
    a=j*math.tau/n;wave=.92+.08*math.cos(10*a);r=(1.-t)+t*wave
    x=(.061+.219*t)*math.sin(a)*r;z=.140+(.055+.155*t)*math.cos(a)*r
    y=.505*(1.-t)+t*(.360+.370*(1.-math.cos(a))*.5)+.012*math.sin(math.pi*t)+face*.004
    v.append((x,y,z))
 block=layers*n
 for face in range(2):
  off=face*block
  for k in range(layers-1):
   for j in range(n):
    a=off+k*n+j;b=off+k*n+(j+1)%n;f.append((a,b,b+n,a+n) if face==0 else (a,a+n,b+n,b))
 for j in range(n):
  a=j;b=(j+1)%n;f.append((a,a+block,b+block,b))
  a=(layers-1)*n+j;b=(layers-1)*n+(j+1)%n;f.append((a,b,b+block,a+block))
 return crease_normals(part('Frill / continuous scalloped white neck fan',WHITE,v,f,True),35.)

def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

def head_fin(p,side):
 xy=[(side*.120,.668),(side*.480,.835),(side*.405,.705),(side*.430,.580),(side*.130,.565)]
 fin=sheet('Head fin / blue-rimmed aquatic fan '+str(side),NAVY,xy,lambda y,x:.225-.13*(abs(x)-.13)+.03*(y-.65),.012,2);p.append(fin)
 inner=[(side*.158,.658),(side*.442,.799),(side*.378,.702),(side*.396,.611),(side*.158,.591)]
 for face in (1,-1):
  parent=fin if face==1 else reverse_front(fin);outline=inner if face==1 else [(-x,y) for x,y in inner]
  membrane=paint(f'Head fin membrane / fitted cream fan {side} {face}',CREAM,outline,parent,.0035,2)
  p.append(membrane if face==1 else reverse_front(membrane));skin=surface(membrane)
  for j,(endx,endy) in enumerate(((side*.390,.743),(side*.388,.649))):
   pts=[]
   for k in range(11):
    t=.08+.84*k/10;x=side*.168+(endx-side*.168)*t;y=.630+(endy-.630)*t
    if face<0:x=-x
    pts.append((x,y,skin(y,x)+.0007))
   rib=tube(f'Head fin ray / two seated blue fan ribs {side} {face} {j}',NAVY,pts,[.0014]*len(pts),6)
   p.append(rib if face==1 else reverse_front(rib))

def ridge(name,points):
 # Each navy pennant is a single closed serrated sheet seated in actual skin.
 v=[];f=[]
 for i,(x,y,z) in enumerate(points):
  height=.014 if i in (0,len(points)-1) else .038 if i%2 else .017
  for xx,yy in ((x-.007,y-.009),(x+.007,y-.009),(x+.006,y+height),(x-.006,y+height)):v.append((xx,yy,z))
 f=[(3,2,1,0),tuple((len(points)-1)*4+j for j in range(4))]
 for k in range(len(points)-1):
  for j in range(4):a=k*4+j;b=k*4+(j+1)%4;f.append((a,b,b+4,a+4))
 return crease_normals(part(name,NAVY,v,f),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=trunk();head=ellipsoid('Head / broad smooth cyan aquatic-cat face',CYAN,(0.,.660,.240),(.180,.165,.150),40,26)
 muzzle=ellipsoid('Muzzle / small soft aquatic-cat snout',CYAN,(0.,.554,.347),(.094,.042,.044),28,16)
 neck=paddle('Neck / long upright cyan aquatic throat',CYAN,[(0.,.38,.10),(0.,.47,.16),(0.,.57,.21)],[.085,.080,.073],[.079,.075,.070],segments=24,steps=4)
 p=[body,head,muzzle,neck,frill()]
 cap=[(-.140,.730),(-.085,.795),(0.,.817),(.085,.795),(.140,.730),(.070,.760),(0.,.746),(-.070,.760)]
 for face in (1,-1):
  parent=head if face==1 else reverse_front(head);outline=cap if face==1 else [(-x,y) for x,y in cap]
  mark=paint('Head cap / fitted navy aquatic crown '+str(face),NAVY,outline,parent,.008,3);p.append(mark if face==1 else reverse_front(mark))
 # The central blue fin is a narrow closed blade in the YZ plane.
 yz=[(.758,.185),(1.025,.130),(.792,.300)];v=[(x,y,z) for x in (-.008,.008) for y,z in yz]
 p.append(part('Crest / tall pointed navy aquatic head fin',NAVY,v,[(2,1,0),(3,4,5),(0,1,4,3),(1,2,5,4),(2,0,3,5)]))
 for side in (-1,1):
  head_fin(p,side);x=side*.095;y=.660
  eye=marking('Eye / large dark aquatic-cat gaze '+str(side),INK,oval(x,y,.039,.045,20),head,.006);p.append(eye)
  iris=marking('Iris / deep violet aquatic-cat eye '+str(side),VIOLET,oval(x,y-.014,.027,.025,18),eye,.002);p.append(iris)
  pupil=marking('Pupil / dark aquatic-cat eye center '+str(side),INK,oval(x,y+.005,.022,.030,18),eye,.004);p.append(pupil)
  p.append(marking('Glint / bright aquatic-cat eye spark '+str(side),GLINT,oval(x-side*.008,y+.020,.008,.010,16),pupil,.0025))
  for row in (0,1):
   x=side*(.103 if row==0 else .125);z=.145 if row==0 else -.230
   if row==0:p.append(ellipsoid('Shoulder / rounded cyan aquatic-cat forelimb root '+str(side),CYAN,(side*.090,.355,z),(.063,.090,.080),24,16))
   if row==1:p.append(ellipsoid('Haunch / rounded cyan aquatic-cat hind thigh '+str(side),CYAN,(side*.108,.293,z),(.077,.100,.095),24,16))
   p.append(curved_tube(f'Leg / tapered aquatic-cat limb {side} {row}',CYAN,[(side*(.080 if row==0 else .100),.394 if row==0 else .330,z-.005),(x,.181,z),(x,.063,z+.021)],[.057 if row==0 else .066,.039,.034],18))
   paw=ellipsoid(f'Paw / four planted aquatic-cat feet {side} {row}',CYAN,(x,.036,z+.043),(.049,.035,.063),24,14);p.append(paw);skin=surface(paw)
   for j,dx in enumerate((-.014,.014)):
    pts=[]
    for k in range(8):
     yy=.013+.025*k/7;xx=x+dx;pts.append((xx,yy,skin(yy,xx)+.0006))
    p.append(tube(f'Toe crease / paired aquatic-cat paw divisions {side} {row} {j}',NAVY,pts,[.0007]*len(pts),6))
 p.append(ellipsoid('Nose / small dark aquatic-cat button',INK,(0.,.583,.377),(.010,.006,.006),16,10))
 smile=[];skin=surface(muzzle)
 for k in range(25):
  x=-.043+.086*k/24;y=.541+.009*(abs(x)/.043)**2;smile.append((x,y,skin(y,x)+.001))
 p.append(tube('Smile / gentle fitted aquatic-cat mouth',INK,smile,[.0012]*len(smile),8))
 tail=curved_tube('Tail / long continuous curling aquatic tail',CYAN,TAIL_POINTS,TAIL_RADII,24);p.append(tail)
 for side in (-1,1):
  p.append(paddle('Fluke / two broad mermaid tail leaves '+str(side),CYAN,[(.535,.445,-.544),(.613,.450+side*.061,-.538),(.714,.450+side*.145,-.532),(.790,.450+side*.191,-.530)],[.022,.074,.071,.002],[.014,.020,.016,.002],segments=24,steps=4))
 back=upper_skin(body);points=[]
 for i in range(21):
  z=.12-.445*i/20;points.append((0.,back(z,0.),z))
 p.append(ridge('Back ridge / continuous scalloped navy dorsal fin',points))
 # Tail ridge follows the actual upper silhouette, not a floating second tail.
 top=upper_skin(tail);points=[]
 for i in range(6):
  x,y,z=TAIL_POINTS[i]
  points.append((x,top(z,x),z))
 p.append(ridge('Tail ridge / scalloped navy aquatic tail fin',points))
 return {'name':'vaporeon','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from vaporeon_glb import export_vaporeon
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_vaporeon())
