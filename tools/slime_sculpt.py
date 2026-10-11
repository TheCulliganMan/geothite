#!/usr/bin/env python3
"""Original distinct Grimer, Muk and Ditto with continuous flowing slime contours."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('grimer','muk','ditto');HEIGHT={s:.96 for s in SPECIES}
COLORS={'grimer':(.63,.52,.70),'muk':(.55,.46,.60),'ditto':(.75,.65,.83)}
INK=(.12,.14,.17);WHITE=(.98,.97,.96);TONGUE=(.59,.67,.75);FOLD=(.38,.31,.44)

def body(species):
 if species=='ditto':rows=((.02,.300,.215,0),(.075,.333,.225,0),(.14,.320,.225,0),(.23,.272,.205,0),(.32,.265,.187,0),(.395,.275,.158,0),(.455,.230,.138,0),(.50,.170,.114,0),(.53,.070,.075,0),(.55,.015,.020,0))
 elif species=='grimer':rows=((.02,.330,.240,-.03),(.06,.370,.270,-.03),(.12,.380,.275,-.03),(.19,.290,.220,-.02),(.25,.330,.250,-.02),(.33,.250,.210,-.015),(.42,.255,.190,0),(.51,.210,.170,.02),(.60,.205,.160,.04),(.69,.205,.150,.06),(.78,.170,.140,.07),(.83,.090,.080,.07),(.85,.016,.018,.07))
 else:rows=((.02,.400,.290,-.04),(.06,.450,.330,-.04),(.13,.430,.330,-.04),(.21,.380,.290,-.03),(.27,.370,.270,-.02),(.37,.300,.240,0),(.46,.290,.230,.025),(.56,.235,.185,.04),(.67,.230,.180,.055),(.74,.190,.150,.065),(.80,.110,.095,.065),(.82,.020,.020,.065))
 profiles=curved_profiles(rows,4);n=48;v=[]
 for row,(y,rx,rz,cz) in enumerate(profiles):
  for j in range(n):
   a=j*math.tau/n;wave=1 if row in (0,len(profiles)-1) else 1+(.030*math.sin(3*a+.7)+.022*math.sin(5*a+1.8)+.010*math.sin(7*a-.4) if species=='ditto' else .045*math.sin(3*a+.7)+.035*math.sin(5*a+1.8)+.018*math.sin(7*a-.4))*min(1.,max(0.,(y-rows[0][0])/.055))
   x=rx*math.sin(a)*wave;z=cz+rz*math.cos(a)*wave;yy=y
   if species=='ditto':
    yy+=(.095*math.sin(a)**8*(rx/.3)**2*math.exp(-((y-.45)/.13)**2) if row else 0)
    if x>0:x*=1+.16*math.exp(-((y-.35)/.09)**2)
   else:x+=(-.052 if species=='muk' else .006)*math.sin(y/rows[-1][0]*math.pi/2)
   v.append((x,yy,z))
 f=[tuple(reversed(range(n))),tuple((len(profiles)-1)*n+j for j in range(n))]
 for k in range(len(profiles)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return part('Body / continuous '+('soft asymmetric three-lobed transform blob' if species=='ditto' else 'flowing sludge head and rippled puddle skirt'),COLORS[species],v,f,True)

def lip(color,cx,cy,rx,ry,skin):
 n=64;v=[]
 for r,offset in ((1.12,.005),(1.035,.024),(.985,.027),(1.12,-.005),(.985,-.005)):
  v.extend((cx+rx*r*math.cos(j*math.tau/n),cy+ry*r*math.sin(j*math.tau/n),skin(cy+ry*r*math.sin(j*math.tau/n),cx+rx*r*math.cos(j*math.tau/n))+offset) for j in range(n))
 f=[]
 for aa,bb in ((0,1),(1,2),(2,4),(4,3),(3,0)):
  for j in range(n):a=aa*n+j;b=aa*n+(j+1)%n;c=bb*n+j;d=bb*n+(j+1)%n;f.append((a,c,d,b))
 return crease_normals(part('Lip / true annular raised flowing mouth rim',color,v,f,True),45)

def arms(species,color):
 p=[]
 for sign in (-1,1):
  if species=='grimer':points=[(sign*.180,.370,.030),(sign*.290,.390,.100),(sign*.370,.370,.220),(sign*.405,.350,.305)];r=[.077,.080,.072,.058];center=(sign*.410,.350,.305);radii=(.077,.063,.055)
  elif sign==1:points=[(.190,.400,.025),(.340,.425,.100),(.510,.405,.115),(.670,.355,.150)];r=[.098,.112,.105,.079];center=(.690,.345,.165);radii=(.105,.084,.075)
  else:points=[(-.230,.290,.050),(-.350,.260,.120),(-.450,.205,.170)];r=[.088,.095,.063];center=(-.465,.204,.185);radii=(.077,.062,.057)
  arm=curved_tube('Arm / '+('long sagging adult sludge stream ' if species=='muk' and sign==1 else 'short forward flowing sludge stream ')+str(sign),color,points,r,24);p.append(arm)
  hand=ellipsoid('Hand / rounded flowing three-finger palm '+str(sign),color,center,radii,32,22);p.append(hand)
  for j in range(3):
   xx=center[0]+sign*(j-1)*radii[0]*.60;yy=center[1]+(.020 if j==0 else -.004 if j==1 else -.025);zz=center[2]+.030
   p.append(curved_tube('Finger / rounded drooping sludge digit '+str(sign)+' '+str(j),color,[(xx,yy,zz),(xx+sign*.009,yy-.027,zz+.050),(xx+sign*.004,yy-.051,zz+.067)],[.027 if species=='grimer' else .030,.025,.011],20))
 return p

def sculpture(species):
 if species not in SPECIES:raise ValueError(species)
 color=COLORS[species];b=body(species);skin=surface(b);p=[b]
 if species=='ditto':
  for sign in (-1,1):
   x=sign*.067;y=.467+(-sign*.006);p.append(fitted_patch('Eye / tiny fitted transform dot '+str(sign),INK,oval(x,y,.0045,.006,24),skin,.0025))
  xy=contour((((-.085,.442),(-.040,.414),(.050,.419),(.082,.439)),((.082,.439),(.040,.402),(-.055,.403),(-.085,.442))),16)
  mouth=fitted_contour('Mouth / fitted gentle asymmetric transform smile',INK,xy,skin,.004,subdivisions=3);p.append(mouth)
  # Follow the actual concave smile outline instead of assuming an ellipse
  # fits inside the narrow curved ribbon.
  upper=[];lower=[]
  for k in range(17):
   x=-.069+.138*k/16;hits=[]
   for a,b in zip(xy,xy[1:]+xy[:1]):
    if min(a[0],b[0])<x<max(a[0],b[0]):hits.append(a[1]+(b[1]-a[1])*(x-a[0])/(b[0]-a[0]))
   lo=min(hits);hi=max(hits);mid=(lo+hi)/2;upper.append((x,mid+(hi-lo)*.24));lower.append((x,mid-(hi-lo)*.24))
  p.append(fitted_contour('Smile interior / soft pink transform grin',(.90,.73,.80),upper+list(reversed(lower)),surface(mouth),.002,subdivisions=3))
  for j,(x,y,rx,ry) in enumerate(((0,.525,.018,.009),(-.180,.468,.017,.011),(.160,.478,.014,.008))):p.append(fitted_patch('Body sheen / soft wet lavender highlight '+str(j),(.87,.81,.93),oval(x,y,rx,ry,24),skin,.002))
 else:
  p+=arms(species,color);adult=species=='muk';cx=-.025 if adult else 0;cy=.580 if adult else .586;rx=.130 if adult else .095;ry=.104 if adult else .133
  mouth=fitted_contour('Mouth / fitted deep open sludge cavity',INK,oval(cx,cy,rx,ry,48),skin,.012,subdivisions=3);p.append(mouth);p.append(lip(color,cx,cy,rx,ry,skin))
  p.append(fitted_patch('Tongue / fitted blue-gray lower sludge tongue',TONGUE,oval(cx,cy-ry*.56,rx*.75,ry*.26,40),surface(mouth),.005))
  for sign in (-1,1):
   x=sign*(.098 if adult else .094)+(-.034 if adult else 0);y=.726 if adult else .765
   if adult:xy=[(x-.038,y+.008),(x+.034,y+.030),(x+.029,y-.013),(x-.031,y-.011)]
   else:xy=oval(x,y,.038,.044,28)
   eye=fitted_contour('Eye border / fitted dark sludge socket '+str(sign),INK,xy,skin,.006,subdivisions=3);p.append(eye);ccx=sum(x for x,y in xy)/len(xy);ccy=sum(y for x,y in xy)/len(xy)
   white=fitted_contour('Eye white / fitted pale sludge stare '+str(sign),WHITE,[(ccx+(x-ccx)*.91,ccy+(y-ccy)*.91) for x,y in xy],surface(eye),.003,subdivisions=3);p.append(white)
   p.append(fitted_patch('Pupil / tiny fitted sludge point '+str(sign),INK,oval(x+.006,y+.008 if adult else y+.011,.0035,.0055,24),surface(white),.0025))
  for rear in (False,True):
   parent=reverse_front(b) if rear else b;ss=surface(parent)
   for j,(y,span) in enumerate(((.150,.29 if adult else .25),(.270,.24 if adult else .21),(.395,.17))):
    points=[]
    for k in range(17):
     x=-span+2*span*k/16;yy=y+.014*math.sin(x/span*math.pi+1.1);points.append((x,yy,ss(yy,x)+.0025))
    q=curved_tube('Body fold / supported soft sludge crease '+str(int(rear))+' '+str(j),FOLD if adult else (.46,.37,.53),points,[.0025]*len(points),10);p.append(reverse_front(q) if rear else q)
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye /','Eye border /','Eye white /','Pupil /','Mouth /','Tongue /','Smile interior /','Body sheen /')):p[i]=crease_normals(q,35)
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[species]/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--only',choices=SPECIES);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in ([args.only] if args.only else SPECIES):
  m=sculpture(s);(args.output/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(s,len(m['primitives']),'parts',sum(len(p['indices'])//3 for p in m['primitives']),'triangles',flush=True)
if __name__=='__main__':main()
