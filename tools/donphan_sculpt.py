#!/usr/bin/env python3
"""Original armored elephant with a continuous rounded tire and transverse tread."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,smooth
from skarmory_sculpt import blade
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.02
BODY=(.75,.81,.82);ARMOR=(.22,.25,.29);TREAD=(.34,.39,.42);RED=(.52,.22,.29);IVORY=(.96,.96,.88);INK=(.085,.10,.12)
def tire_rows():
 path=[(0,.580,-.370),(0,.676,-.230),(0,.716,-.025),(0,.784,.195),(0,.757,.337),(0,.650,.460),(0,.445,.527),(0,.240,.554),(0,.138,.595),(0,.157,.677)]
 widths=[.100,.151,.172,.170,.155,.139,.122,.104,.083,.025];depths=[.027,.040,.047,.055,.056,.053,.046,.040,.030,.016]
 pp,ww=smooth(path,widths);_,dd=smooth(path,depths)
 rows=[]
 for i,(p,w,d) in enumerate(zip(pp,ww,dd)):
  a=pp[max(0,i-1)];b=pp[min(len(pp)-1,i+1)];dy=b[1]-a[1];dz=b[2]-a[2];length=math.hypot(dy,dz);rows.append((p,w,d,(dz/length,-dy/length)))
 return rows

def ribbon(name,color,rows,offset=0.,width_scale=1.,depth_scale=1.):
 n=32;v=[]
 for (x,y,z),w,d,(ny,nz) in rows:
  for j in range(n):
   a=math.tau*j/n;q=d*depth_scale*math.sin(a)+offset;v.append((x+w*width_scale*math.cos(a),y+ny*q,z+nz*q))
 f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for k in range(len(rows)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part(name,color,v,f,True),38.)

def sculpture():
 body=ellipsoid('Body / broad pale armored elephant barrel',BODY,(0,.400,-.057),(.330,.281,.387),40,28)
 head=ellipsoid('Head / broad pale lowered elephant cranium',BODY,(0,.554,.249),(.284,.238,.239),40,28);p=[body,head];hs=surface(head)
 rows=tire_rows();p.append(ribbon('Tire / continuous charcoal back and hanging trunk',ARMOR,rows))
 for j in range(9):
  begin=j*4;piece=rows[begin:begin+4]
  if len(piece)>1:p.append(ribbon('Tread / rounded raised transverse armor plate '+str(j),TREAD,piece,.023,.96,.64))
 ear_outline=contour([((.145,.700),(.269,.761),(.486,.681),(.654,.609)),((.654,.609),(.598,.635),(.557,.601),(.515,.566)),((.515,.566),(.378,.609),(.190,.630),(.145,.700))],12)
 for sign in (-1,1):
  xy=[(sign*x,y) for x,y in ear_outline];ear=blade('Ear / broad swept charcoal shield '+str(sign),ARMOR,xy,lambda x,y:.148-.13*(abs(x)-.20),.029);p.append(ear)
  lining=contour([((.249,.700),(.342,.705),(.476,.649),(.570,.612)),((.570,.612),(.483,.597),(.354,.634),(.249,.700))],12)
  p.append(fitted_contour('Ear lining / fitted burgundy shield underside '+str(sign),RED,[(sign*x,y) for x,y in lining],surface(ear),.004))
  xy=[(sign*x,y) for x,y in ((.164,.655),(.231,.680),(.231,.650),(.176,.636))]
  border=fitted_contour('Eye / fitted dark watchful socket '+str(sign),INK,xy,hs,.005,subdivisions=3);p.append(border)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy);white=fitted_contour('Eye white / narrow ivory watchful wedge '+str(sign),IVORY,[(cx+(x-cx)*.80,cy+(y-cy)*.73) for x,y in xy],surface(border),.003,subdivisions=3);p.append(white)
  pupil=fitted_patch('Pupil / fitted small dark watchful slit '+str(sign),INK,oval(sign*.201,.657,.004,.007,20),surface(white),.002);p.append(pupil)
  p.append(curved_tube('Tusk / long outward sweeping ivory lance '+str(sign),IVORY,[(sign*.165,.478,.408),(sign*.253,.429,.504),(sign*.344,.449,.590),(sign*.418,.526,.664)],[.057,.062,.040,.001],32))
 for sign in (-1,1):
  for front in (False,True):
   z=.183 if front else -.292;name=str(sign)+' '+('front' if front else 'rear')
   profiles=curved_profiles(((.028,.042,.047,z+.024),(.100,.087,.088,z+.015),(.211,.095,.097,z),(.315,.084,.088,z-.015),(.400,.025,.030,z-.020)),4)
   leg=loft('Leg / rooted stocky pale pillar '+name,BODY,profiles,32);leg['positions']=[v+sign*.218 if i%3==0 else v for i,v in enumerate(leg['positions'])];p.append(leg)
   bandrows=[]
   for y,rx,rz,cz in profiles:
    if .100<=y<=.211:bandrows.append((y,rx+.003,rz+.003,cz))
   cuff=loft('Ankle armor / fitted charcoal leg cuff '+name,ARMOR,bandrows,32);cuff['positions']=[v+sign*.218 if i%3==0 else v for i,v in enumerate(cuff['positions'])];p.append(cuff)
   foot=ellipsoid('Foot / grounded rounded pale elephant pad '+name,BODY,(sign*.218,.046,z+.037),(.096,.046,.112),32,20);p.append(foot)
   fs=surface(foot)
   for j,dx in enumerate((-.047,0,.047)):
    p.append(fitted_patch('Toenail / fitted ivory elephant toe '+name+' '+str(j),IVORY,oval(sign*.218+dx,.033,.020,.013,24),fs,.004))
 p.append(curved_tube('Tail / short armored rear flick',ARMOR,[(0,.425,-.405),(.045,.451,-.465),(.096,.461,-.493)],[.033,.026,.001],24))
 p=[crease_normals(q,35.) if q['part'].startswith(('Ear lining /','Eye /','Eye white /','Pupil /','Toenail /')) else q for q in p]
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'donphan','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'donphan.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
