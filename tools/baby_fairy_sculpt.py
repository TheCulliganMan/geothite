#!/usr/bin/env python3
"""Original Cleffa and Igglybuff sculptures with fitted faces and distinct silhouettes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('cleffa','igglybuff');HEIGHTS={'cleffa':.60,'igglybuff':.58}
PEACH=(.98,.77,.75);PALE=(.98,.84,.85);BROWN=(.42,.34,.27);INK=(.13,.10,.12);WHITE=(.99,.99,.96);RUBY=(.81,.36,.44);BLUSH=(.95,.51,.49);TONGUE=(.89,.56,.64);CREASE=(.65,.42,.45)
def tilt(p,angle,center):
 c=math.cos(angle);s=math.sin(angle)
 for field in ('positions','normals'):
  a=[]
  for i in range(0,len(p[field]),3):
   x,y,z=p[field][i:i+3];cx,cy,cz=center if field=='positions' else (0,0,0);x-=cx;y-=cy;z-=cz;a.extend((c*x-s*y+cx,s*x+c*y+cy,z+cz))
  p[field]=a
 return p
def spiral(name,parent,cx,cy,radius,color):
 skin=surface(parent);pts=[]
 for k in range(43):
  t=k/42;a=.4*math.pi+math.tau*1.3*t;r=radius*(1-.88*t);x=cx+r*math.cos(a);y=cy+r*math.sin(a);pts.append((x,y,skin(y,x)+.003))
 return crease_normals(tube(name,color,pts,[.002]*len(pts),12),35)
def cleffa():
 body=loft('Body / continuous peach star-shaped fairy pear',PEACH,curved_profiles(((.035,.045,.050,-.025),(.11,.180,.150,-.02),(.22,.250,.200,-.015),(.38,.275,.210,-.01),(.52,.252,.196,0),(.65,.200,.170,.004),(.735,.130,.100,.006),(.765,.025,.028,.004)),4),40);skin=surface(body);p=[body]
 ear=contour((((.110,.735),(.247,.770),(.362,.704),(.447,.650)),((.447,.650),(.440,.607),(.343,.538),(.217,.505)),((.217,.505),(.170,.570),(.145,.680),(.110,.735))),10)
 for sign in (-1,1):
  q=fitted_contour('Ear / rounded broad brown star point '+str(sign),BROWN,[(sign*x,y) for x,y in ear],lambda y,x:.035+.035*math.exp(-((abs(x)-.29)/.12)**2-((y-.635)/.13)**2),.070,subdivisions=3);p.append(crease_normals(q,35))
  center=(sign*.186,.345,.144);p.append(tilt(ellipsoid('Arm / shy inward peach leaf hand '+str(sign),PEACH,center,(.065,.096,.055),32,22),sign*.60,center))
  center=(sign*.150,.069,.042);p.append(tilt(ellipsoid('Foot / grounded peach star toe '+str(sign),PEACH,center,(.088,.069,.096),32,22),sign*.30,center))
  x=sign*.088;eye=fitted_patch('Eye / small fitted dark Cleffa oval '+str(sign),INK,oval(x,.516,.013,.029,32),skin,.005);p.extend((eye,fitted_patch('Eye glint / fitted ivory Cleffa sparkle '+str(sign),WHITE,oval(x-.002,.528,.004,.007,24),surface(eye),.0015),fitted_patch('Cheek / fitted warm peach blush '+str(sign),BLUSH,oval(sign*.154,.440,.029,.010,32),skin,.002)))
 tuft=tilt(ellipsoid('Tuft / rooted rolled peach forehead pillow',PEACH,(.022,.754,.071),(.082,.133,.075),40,28),-.54,(.022,.754,.071));p.extend((tuft,spiral('Tuft crease / fitted rosy rolled curl',tuft,.018,.754,.045,CREASE),ellipsoid('Tail / short rooted peach rear puff',PEACH,(-.090,.225,-.246),(.090,.114,.100),32,22)))
 smile=contour((((-.052,.418),(-.022,.429),(-.018,.393),(.008,.404)),((.008,.404),(.024,.416),(.037,.419),(.052,.421)),((.052,.421),(.030,.413),(.021,.403),(.008,.400)),((.008,.400),(-.020,.389),(-.023,.423),(-.052,.414)),((-.052,.414),(-.052,.415),(-.052,.417),(-.052,.418))),6)
 p.append(fitted_contour('Mouth / fitted tiny wavy Cleffa smile',INK,smile,skin,.003));return p
def igglybuff():
 body=ellipsoid('Body / continuous pale pink balloon orb',PALE,(0,.405,0),(.293,.330,.235),40,28);skin=surface(body);p=[body]
 for j,(center,radii) in enumerate((((-.063,.741,-.020),(.068,.075,.058)),((.063,.741,-.020),(.068,.075,.058)),((0,.807,-.030),(.065,.074,.056)))):p.append(ellipsoid('Crown / rounded rooted balloon tuft lobe '+str(j),PALE,center,radii,28,20))
 for sign in (-1,1):
  center=(sign*.263,.365,.085);p.append(tilt(ellipsoid('Arm / tiny pale balloon leaf stub '+str(sign),PALE,center,(.062,.040,.042),28,18),sign*.35,center))
  center=(sign*.100,.070,.065);p.append(tilt(ellipsoid('Foot / grounded pale balloon petal '+str(sign),PALE,center,(.062,.080,.070),32,22),sign*.24,center))
  x=sign*.116;y=.457;border=fitted_contour('Eye border / fitted dark Igglybuff socket '+str(sign),INK,oval(x,y,.053,.068,32),skin,.006,subdivisions=3);iris=fitted_contour('Iris / fitted ruby balloon eye '+str(sign),RUBY,oval(x,y,.049,.064,32),surface(border),.003);pupil=fitted_patch('Pupil / fitted large dark balloon oval '+str(sign),INK,oval(x,y,.027,.046,36),surface(iris),.003);glint=fitted_patch('Eye glint / fitted ivory balloon reflection '+str(sign),WHITE,oval(x-.008,y+.020,.012,.014,28),surface(pupil),.002);p.extend((border,iris,pupil,glint))
 p.append(spiral('Forehead / fitted dark balloon spiral',body,0,.596,.052,INK))
 mouth=contour((((-.038,.374),(-.010,.369),(.012,.368),(.038,.375)),((.038,.375),(.025,.294),(-.024,.298),(-.038,.374))),12);q=fitted_contour('Mouth / fitted delighted balloon smile',INK,mouth,skin,.005,subdivisions=3);p.append(q);cx=sum(x for x,y in mouth)/len(mouth);cy=sum(y for x,y in mouth)/len(mouth);p.append(fitted_contour('Tongue / fitted rosy balloon smile interior',TONGUE,[(cx+(x-cx)*.74,cy+(y-cy)*.74) for x,y in mouth],surface(q),.003));return p
def sculpture(species):
 p={'cleffa':cleffa,'igglybuff':igglybuff}[species]()
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye /','Eye glint /','Eye border /','Iris /','Pupil /','Cheek /','Mouth /','Tongue /')):p[i]=crease_normals(q,35)
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for species in SPECIES:(args.output/(species+'.mesh.json')).write_text(json.dumps(sculpture(species),separators=(',',':')))
if __name__=='__main__':main()
