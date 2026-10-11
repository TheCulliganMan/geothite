#!/usr/bin/env python3
"""Original psychic coconut palm with folded crown and three distinct faces."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import tube,part,loft,curved_profiles,ellipsoid,fitted_contour,fitted_patch,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import contour
from bivalve_sculpt import ribbon_inside
from skin_glb import COORDINATES
HEIGHT=1.22
TAN=(.69,.62,.49);SOLE=(.86,.80,.54);GOLD=(.96,.90,.57);LEAF=(.40,.66,.35);VEIN=(.24,.44,.24);WHITE=(.99,.99,.94);INK=(.19,.16,.17);PINK=(.79,.51,.59)
ENDS=((- .73,1.43,.06),(-.64,1.70,-.06),(-.39,1.94,-.10),(-.09,2.04,-.12),(.28,2.04,-.10),(.54,1.85,-.03),(.73,1.58,.04),(.72,1.32,.12),(-.51,1.33,-.39),(0,1.58,-.58),(.54,1.42,-.42))
def frond(j,end):
 v=[];centers=[];n=4;count=25
 for k in range(count):
  t=k/(count-1);x=end[0]*(.70*t+.30*t*t);y=1.075+(end[1]-1.075)*t+.10*math.sin(math.pi*t);z=-.020+(end[2]+.020)*t+.04*math.sin(math.pi*t)
  dx=end[0]*(.70+.60*t);dy=end[1]-1.075+.10*math.pi*math.cos(math.pi*t);length=math.hypot(dx,dy);sx,sy=dy/length,-dx/length
  w=.004*(1-t)+.0015*t+(.060 if j in (3,4) else .075)*math.sin(math.pi*t)**.85
  v.extend(((x,y,z+.012),(x+sx*w,y+sy*w,z+.001),(x,y,z-.011),(x-sx*w,y-sy*w,z+.001)));centers.append((x,y,z+.012))
 f=[tuple(reversed(range(n))),tuple((count-1)*n+i for i in range(n))]
 for k in range(count-1):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part('Frond / continuous folded curved palm blade '+str(j),LEAF,v,f,True),40),centers

def sampled_tube(name,color,points,radii,segments):
 return crease_normals(tube(name,color,points,radii,segments),35)

def sculpture():
 trunk=loft('Trunk / continuous rounded segmented palm body',TAN,curved_profiles(((.10,.050,.045,0),(.17,.200,.170,0),(.35,.230,.200,0),(.60,.210,.180,-.015),(.81,.190,.160,-.025),(.97,.150,.130,-.020),(1.10,.040,.035,-.020),(1.12,.012,.012,-.020)),4),40);p=[trunk]
 for sign in (-1,1):
  p.append(curved_tube('Leg / stout rooted palm ankle '+str(sign),TAN,[(sign*.142,.235,.010),(sign*.197,.137,.090),(sign*.223,.083,.167)],[.073,.067,.068],28))
  foot=ellipsoid('Foot / broad rounded tan palm stump '+str(sign),TAN,(sign*.226,.085,.177),(.117,.085,.145),32,22);p.append(foot)
  sole=fitted_contour('Sole / fitted pale oval palm foot pad '+str(sign),SOLE,oval(sign*.226,.075,.075,.057,32),surface(foot),.006,subdivisions=3);p.append(sole)
  for j,off in enumerate((-.061,0,.061)):
   x=sign*.226+off;y=.128;z=surface(foot)(y,x)-.010
   p.append(curved_tube('Claw / rooted short ivory foot spike '+str(sign)+' '+str(j),WHITE,[(x,y,z),(x,.191,z+.014),(x,.218,z+.025)],[.021,.015,.0025],20))
 for rear in (False,True):
  parent=reverse_front(trunk) if rear else trunk;ss=surface(parent)
  for j,y in enumerate((.320,.530,.735)):
   pts=[]
   for k in range(31):
    x=-.170+.340*k/30;yy=y+.024*(x/.170)**2;pts.append((x,yy,ss(yy,x)+.0026))
   band=sampled_tube('Trunk crease / supported curved segment band '+('rear ' if rear else 'front ')+str(j),(.46,.41,.33),pts,[.0018]*len(pts),10);p.append(reverse_front(band) if rear else band)
 for j,(cx,cy,cz) in enumerate(((-.245,.980,-.055),(-.025,.945,.167),(.205,1.012,.040))):
  head=ellipsoid('Head / individual rounded golden coconut '+str(j),GOLD,(cx,cy,cz),(.153,.166,.145),36,24);p.append(head);ss=surface(head)
  for sign in (-1,1):
   xy=[(.026,.057),(.104,.068),(.093,.025),(.035,.021)] if j!=1 else [(.024,.035),(.104,.070),(.098,.005),(.040,.013)]
   xy=[(cx+sign*x,cy+y) for x,y in xy];eye=fitted_contour('Eye white / fitted distinct coconut eye '+str(j)+' '+str(sign),WHITE,xy,ss,.006,subdivisions=3);p.append(eye)
   p.append(fitted_patch('Pupil / fitted fine coconut pupil '+str(j)+' '+str(sign),INK,oval(cx+sign*.065,cy+.040,.0028,.009,20),surface(eye),.002))
   a,b=xy[:2];pts=[];es=surface(eye)
   for k in range(17):
    t=.04+.92*k/16;x=a[0]+(b[0]-a[0])*t;y=a[1]+(b[1]-a[1])*t-.002;pts.append((x,y,es(y,x)+.002))
   p.append(sampled_tube('Lid / supported sleepy coconut upper lid '+str(j)+' '+str(sign),INK,pts,[.0014]*len(pts),10))
  if j==1:
   xy=contour((((-.092,-.015),(-.025,-.034),(.031,-.034),(.093,-.015)),((.093,-.015),(.066,-.135),(-.061,-.139),(-.092,-.015))),16)
  elif j==0:xy=oval(0,-.053,.026,.034,32)
  else:
   xy=contour((((-.066,-.047),(-.015,-.063),(.032,-.061),(.071,-.032)),((.071,-.032),(.046,-.090),(-.015,-.085),(-.066,-.047))),12)
  mouth=fitted_contour('Mouth / fitted distinct coconut expression '+str(j),INK,[(cx+x,cy+y) for x,y in xy],ss,.007,subdivisions=3);p.append(mouth)
  ms=surface(mouth)
  if j==1:
   p.append(fitted_patch('Tongue / fitted rounded pink coconut mouth interior',PINK,oval(cx,cy-.067,.034,.015,32),ms,.003))
   for sign in (-1,1):p.append(fitted_contour('Fang / fitted ivory open coconut mouth canine '+str(sign),WHITE,[(cx+sign*.069,cy-.030),(cx+sign*.039,cy-.035),(cx+sign*.055,cy-.067)],ms,.003,subdivisions=2))
  elif j==0:p.append(fitted_patch('Slack tongue / fitted small pink tired coconut tongue',PINK,oval(cx,cy-.061,.012,.017,24),ms,.0025))
  elif j==2:p.append(fitted_contour('Smile lining / fitted muted pink coconut smirk',PINK,[(cx+x,cy+y) for x,y in ribbon_inside(xy,.052,amount=.32)],ms,.003,subdivisions=3))
 for j,end in enumerate(ENDS):
  leaf,centers=frond(j,end);p.append(leaf);ss=surface(leaf);pts=[(x,y,ss(y,x)+.0018) for x,y,z in centers[1:-2]]
  p.append(sampled_tube('Midrib / supported narrow palm blade ridge '+str(j),VEIN,pts,[.0012]*len(pts),10))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye white /','Pupil /','Mouth /','Tongue /','Fang /','Smile lining /','Sole /')):p[i]=crease_normals(q,35)
 # Compact the woody body; preserve coconut and blade shapes by translation.
 for q in p:
  if q['part'].startswith(('Trunk /','Trunk crease /')):
   q['positions']=[v*.65 if i%3==1 else v for i,v in enumerate(q['positions'])]
   normals=[]
   for i in range(0,len(q['normals']),3):
    x,y,z=q['normals'][i:i+3];y/=.65;length=math.sqrt(x*x+y*y+z*z);normals.extend((x/length,y/length,z/length))
   q['normals']=normals
  elif not q['part'].startswith(('Leg /','Foot /','Sole /','Claw /')):
   q['positions']=[v-.36 if i%3==1 else v for i,v in enumerate(q['positions'])]
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'exeggutor','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);m=sculpture();(args.output/'exeggutor.mesh.json').write_text(json.dumps(m,separators=(',',':')));print(len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']),flush=True)
if __name__=='__main__':main()
