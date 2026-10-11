#!/usr/bin/env python3
"""Original six psychic eggs, distinct expressions and a truly broken shell."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,fitted_contour,fitted_patch,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.22
PINK=(.92,.70,.74);WHITE=(.99,.99,.96);INK=(.21,.17,.20);YOLK=(.99,.83,.25)
# Front angry eggs, a smirking middle egg, broken left rear, tired right rear.
EGGS=((- .290,.227,.145,.158,.227,.143),(0,.241,.176,.168,.241,.147),(.290,.219,.115,.153,.219,.142),(-.360,.394,-.055,.154,.224,.142),(0,.404,-.060,.161,.259,.148),(.310,.415,-.065,.156,.270,.142))
def broken_egg(c):
 cx,cy,cz,rx,ry,rz=c;n=64;rings=24;v=[]
 for inside in (False,True):
  for k in range(rings+1):
   for j in range(n):
    a=j*math.tau/n;rim=.92+.12*math.cos(7*a)+.055*math.sin(13*a)+.23*max(0,math.cos(a));t=math.pi-.0008-(math.pi-.0008-rim)*k/rings
    w=rx-(.014 if inside else 0);h=ry-(.016 if inside else 0);d=rz-(.013 if inside else 0)
    v.append((cx+w*math.sin(t)*math.sin(a),cy+h*math.cos(t),cz+d*math.sin(t)*math.cos(a)))
 count=(rings+1)*n;f=[tuple(reversed(range(n))),tuple(count+j for j in range(n))]
 for k in range(rings):
  for j in range(n):
   a=k*n+j;b=k*n+(j+1)%n;f.extend(((a,b,b+n,a+n),(count+a,count+a+n,count+b+n,count+b)))
 for j in range(n):a=rings*n+j;b=rings*n+(j+1)%n;f.append((a,b,count+b,count+a))
 return crease_normals(part('Egg / hollow jagged broken rear shell 3',PINK,v,f,True),45)
def sculpture():
 p=[]
 for j,c in enumerate(EGGS):
  cx,cy,cz,rx,ry,rz=c
  body=broken_egg(c) if j==3 else ellipsoid('Egg / individual rounded psychic egg '+str(j),PINK,(cx,cy,cz),(rx,ry,rz),40,28);p.append(body);ss=surface(body)
  if j==3:
   p.append(ellipsoid('Yolk / recessed golden broken egg interior',YOLK,(cx,cy-.047,cz-.010),(.126,.164,.118),32,22));continue
  face_y=cy+(.140 if j==4 else .110 if j==5 else 0)
  for sign in (-1,1):
   if j==5:xy=[(.025,.037),(.115,.027),(.098,-.009),(.034,.003)]
   elif j==4:xy=[(.027,.021),(.096,.024),(.087,-.007),(.035,-.009)]
   elif j==2:xy=[(.025,.044),(.108,.072),(.103,.017),(.034,.011)]
   else:xy=[(.022,.036),(.112,.079),(.104,.020),(.031,.013)]
   xy=[(cx+sign*x,face_y+y) for x,y in xy];eye=fitted_contour('Eye white / fitted distinct psychic egg eye '+str(j)+' '+str(sign),WHITE,xy,ss,.0055,subdivisions=3);p.append(eye)
   x=cx+sign*.072;y=face_y+(.002 if j==4 else .012 if j==5 else .039)
   p.append(fitted_patch('Pupil / fitted fine psychic egg pupil '+str(j)+' '+str(sign),INK,oval(x,y,.0025,.008,20),surface(eye),.002))
   # Upper eyelid defines the expression without detached eyebrows.
   a,b=xy[:2];pts=[]
   for k in range(17):
    t=.04+.92*k/16;x=a[0]+(b[0]-a[0])*t;y=a[1]+(b[1]-a[1])*t-.002;pts.append((x,y,surface(eye)(y,x)+.0019))
   p.append(curved_tube('Lid / supported distinct psychic egg eye line '+str(j)+' '+str(sign),INK,pts,[.0013]*len(pts),10))
  if j==4:local=contour((((-.073,-.021),(-.024,-.030),(.043,-.027),(.078,.005)),((.078,.005),(.044,-.035),(-.025,-.037),(-.073,-.026))),12)
  elif j==5:local=contour((((-.049,-.072),(-.008,-.040),(.021,-.042),(.060,-.075)),((.060,-.075),(.019,-.059),(-.009,-.058),(-.049,-.080))),12)
  else:local=contour((((-.056,-.072),(-.014,-.045),(.017,-.045),(.059,-.069)),((.059,-.069),(.017,-.055),(-.015,-.055),(-.056,-.078))),12)
  p.append(fitted_contour('Mouth / fitted distinct psychic egg expression '+str(j),INK,[(cx+x,face_y+y) for x,y in local],ss,.0045,subdivisions=3))
  if j in (0,1,2,5):
   sign=-1 if j in (0,5) else 1;xy=[(cx+sign*x,cy+y) for x,y in ((.039,.194),(.025,.153),(.049,.126),(.026,.096),(.028,.061))];pts=[(x,y,ss(y,x)+.0020) for x,y in xy]
   p.append(curved_tube('Crack / supported angular psychic egg shell seam '+str(j),INK,pts,[.0014]*len(pts),10))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye white /','Pupil /','Mouth /')):p[i]=crease_normals(q,35)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'exeggcute','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);m=sculpture();(args.output/'exeggcute.mesh.json').write_text(json.dumps(m,separators=(',',':')));print(len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']),flush=True)
if __name__=='__main__':main()
