#!/usr/bin/env python3
"""Original lightning fox: upright ears, angular gaze, white needle collar and bristled rump."""
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
HEIGHT=1.05;CORE_DROP=.025
YELLOW=(.99,.73,.095);FACE=(1.,.79,.16);WHITE=(.96,.97,.94)
INK=(.045,.025,.060);VIOLET=(.20,.075,.31);GLINT=(1.,1.,.98)

def trunk():
 rows=curved_profiles(((-.33,.014,.024,.40),(-.26,.115,.135,.415),(-.12,.137,.145,.43),(.06,.118,.135,.445),(.17,.077,.095,.475),(.205,.015,.025,.49)),4);n=28;v=[];faces=[]
 for z,rx,ry,y in rows:
  for j in range(n):a=j*math.tau/n;v.append((rx*math.sin(a),y+ry*math.cos(a),z))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Body / lean yellow lightning fox torso',YELLOW,v,faces,True),40.)

def ruff():
 # One closed collar. The intermediate ring forms twelve long occupied needles.
 n=72;v=[];faces=[]
 for row,(y,rx,rz,teeth) in enumerate(((.480,.069,.063,0.),(.505,.150,.130,.5),(.535,.190,.163,1.),(.595,.133,.107,.25),(.635,.070,.061,0.))):
  for j in range(n):
   a=j*math.tau/n;tooth=(.5+.5*math.cos(12*a))**3;extent=teeth*tooth
   v.append((math.sin(a)*(rx+.038*extent),y-.115*extent,.100+math.cos(a)*(rz+.028*extent)))
 faces=[tuple(reversed(range(n))),tuple((5-1)*n+j for j in range(n))]
 for k in range(4):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Ruff / continuous twelve-point white lightning collar',WHITE,v,faces,True),28.)

def marking(name,color,xy,parent,th=.004):return crease_normals(fitted_patch(name,color,xy,surface(parent),th),35.)

@lru_cache(maxsize=1)
def raw_sculpture():
 head=loft('Head / angular yellow fox cheeks and tapered chin',FACE,curved_profiles(((.535,.038,.052,.257),(.568,.097,.100,.240),(.624,.150,.126,.219),(.694,.157,.125,.198),(.755,.120,.100,.179),(.791,.062,.062,.167),(.803,.010,.012,.162)),4),32)
 muzzle=ellipsoid('Muzzle / short pointed yellow lightning snout',FACE,(0.,.590,.331),(.086,.035,.055),24,14)
 p=[trunk(),head,muzzle,ruff(),paddle('Neck / upright yellow lightning fox throat',YELLOW,[(0.,.43,.11),(0.,.53,.15),(0.,.61,.20)],[.075,.077,.067],[.070,.073,.065],segments=20,steps=4)]
 # Strong chest needles accompany the smaller needles on the continuous collar.
 for j,(x,y,z) in enumerate(((-.100,.335,.205),(0.,.285,.250),(.100,.335,.205))):
  p.append(paddle('Ruff tuft / long white chest needle '+str(j),WHITE,[(x*.6,.533,.184),(x*.9,.450,.224),(x,y,z)],[.060,.056,.0015],[.028,.024,.0015],segments=16,steps=4))
 for side in (-1,1):
  ear=paddle('Ear / long sharp yellow lightning pinna '+str(side),YELLOW,[(side*.090,.754,.160),(side*.153,.862,.137),(side*.235,1.008,.104),(side*.286,1.088,.088)],[.053,.061,.035,.0015],[.026,.023,.014,.0015],segments=18,steps=4);p.append(ear)
  xy=[(side*.109,.788),(side*.151,.921),(side*.265,1.054),(side*.207,.897),(side*.160,.805)]
  p.append(paint('Ear inset / deep violet fitted lightning pinna '+str(side),VIOLET,xy,ear,.003,2))
  # Narrower upper edge, rising outer corner: angular rather than Eevee's round gaze.
  xy=[(side*.046,.677),(side*.067,.706),(side*.127,.719),(side*.126,.680),(side*.106,.652),(side*.069,.653)]
  eye=paint('Eye / sharp dark almond lightning gaze '+str(side),INK,xy,head,.006,3);p.append(eye)
  iris=marking('Iris / lower violet lightning gaze '+str(side),VIOLET,oval(side*.090,.666,.022,.012,16),eye,.002);p.append(iris)
  p.append(marking('Glint / white lightning eye spark '+str(side),GLINT,oval(side*.073,.694,.007,.009,14),eye,.003))
  # Two short cheek bristles, seated within the face silhouette at their roots.
  for row in (0,1):
   yy=.661-row*.053
   p.append(paddle(f'Cheek quill / swept yellow facial bristle {side} {row}',FACE,[(side*.114,yy,.206),(side*.160,yy-.012,.178),(side*(.216-row*.012),yy-.043,.152)],[.044,.032,.0015],[.027,.022,.0015],segments=12,steps=3))
  for row in (0,1):
   z=.132 if row==0 else -.220;x=side*(.087 if row==0 else .128)
   if row==1:p.append(ellipsoid('Haunch / powerful yellow lightning hind thigh '+str(side),YELLOW,(side*.109,.363,-.225),(.076,.111,.098),24,16))
   points=[(side*(.075 if row==0 else .105),.449 if row==0 else .386,z-.008),(x,.212,z-(.050 if row else 0.)),(x,.056,z+.018)]
   p.append(curved_tube(f'Leg / long angular lightning fox limb {side} {row}',YELLOW,points,[.047 if row==0 else .061,.030,.028],18))
   paw=ellipsoid(f'Paw / four planted yellow lightning feet {side} {row}',YELLOW,(x,.030,z+.040),(.043,.030,.055),22,12);p.append(paw)
   skin=surface(paw)
   for j,dx in enumerate((-.012,.012)):
    pts=[]
    for k in range(8):
     yy=.010+.020*k/7;xx=x+dx;pts.append((xx,yy,skin(yy,xx)+.0006))
    p.append(tube(f'Toe crease / paired lightning paw divisions {side} {row} {j}',INK,pts,[.0007]*8,6))
  # Occupied sharp hindquarter fur rather than an Eevee brush tail.
  for row in (0,1):
   x=side*(.095+row*.022)
   p.append(paddle(f'Rump quill / swept lateral lightning bristle {side} {row}',YELLOW,[(x,.465-row*.040,-.202),(side*.153,.455-row*.055,-.280),(side*(.206-row*.010),.445-row*.082,-.405)],[.068,.052,.0015],[.047,.034,.0015],segments=14,steps=4))
 for j,(x,y,z) in enumerate(((-.055,.653,-.335),(0.,.699,-.245),(.055,.658,-.350),(0.,.534,-.471))):
  p.append(paddle('Back quill / raised yellow lightning spine '+str(j),YELLOW,[(x*.5,.490,-.205),(x*.9,.552,-.265),(x,y,z)],[.060,.052,.0015],[.048,.034,.0015],segments=14,steps=4))
 p.append(ellipsoid('Nose / tiny dark lightning button',INK,(0.,.610,.380),(.010,.006,.0045),16,10))
 skin=surface(muzzle);smile=[]
 for k in range(25):
  x=-.044+.088*k/24;y=.577+.009*(abs(x)/.044)**2;smile.append((x,y,skin(y,x)+.001))
 p.append(tube('Smile / fitted lightning fox mouth',INK,smile,[.0011]*25,8))
 for q in p:
  if q['part'].startswith(('Paw /','Toe crease /')):continue
  limb=q['part'].startswith('Leg /')
  for k in range(1,len(q['positions']),3):
   y=q['positions'][k];q['positions'][k]-=CORE_DROP*(smoothstep(.056,.350,y) if limb else 1.)
  if limb:q.update(crease_normals(q,35.))
 return {'name':'jolteon','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from jolteon_glb import export_jolteon
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_jolteon())
