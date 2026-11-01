#!/usr/bin/env python3
"""Original shared Jigglypuff: smooth balloon, pointed ears and fitted blue eyes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval
from baby_fairy_sculpt import tilt,spiral
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=.82
PINK=(.98,.77,.83);INK=(.13,.15,.20);WHITE=(.98,.99,.97);BLUE=(.13,.63,.73);LOWER=(.41,.79,.84)
def pillow_ear(name,outline,*,center=None):
 # Closed concentric front/back surfaces round the rim as well as the face.
 n=len(outline);cx,cy=center if center is not None else (sum(x for x,y in outline)/n,sum(y for x,y in outline)/n)
 xy=[(cx,cy,0.)]
 for r in (.23,.47,.70,.88,1.):xy.extend((cx+(x-cx)*r,cy+(y-cy)*r,r) for x,y in outline)
 count=len(xy);v=[(x,y,.01+sign*.060*math.sqrt(max(.0005,1-r*r))) for sign in (1,-1) for x,y,r in xy]
 f=[(0,1+i,1+(i+1)%n) for i in range(n)]
 for ring in range(4):
  for j in range(n):
   a=1+ring*n+j;b=1+ring*n+(j+1)%n;f.append((a,a+n,b+n,b))
 f += [tuple(i+count for i in reversed(q)) for q in list(f)]
 last=1+4*n
 f += [(last+(j+1)%n,last+j,last+j+count,last+(j+1)%n+count) for j in range(n)]
 return part(name,PINK,v,f,True)
def sculpture():
 body=ellipsoid('Body / continuous round pink balloon',PINK,(0,.405,0),(.300,.330,.238),40,28);skin=surface(body);p=[body]
 for sign in (-1,1):
  xy=contour((((.139,.617),(.164,.721),(.203,.810),(.235,.823)),((.235,.823),(.278,.811),(.344,.707),(.360,.646)),((.360,.646),(.320,.597),(.250,.546),(.205,.545)),((.205,.545),(.175,.562),(.149,.595),(.139,.617))),10)
  ear=pillow_ear('Ear / rounded pointed pink balloon ear '+str(sign),[(sign*x,y) for x,y in xy]);p.append(ear)
  inset=[(sign*x,y) for x,y in ((.188,.648),(.235,.775),(.313,.650),(.230,.584))];p.append(fitted_contour('Ear lining / fitted charcoal triangular inset '+str(sign),INK,inset,surface(ear),.004,subdivisions=3))
  center=(sign*.285,.359,.027);p.append(tilt(ellipsoid('Arm / short rounded pink balloon leaf '+str(sign),PINK,center,(.075,.061,.060),32,22),-sign*.4,center))
  center=(sign*.123,.063,.070);p.append(tilt(ellipsoid('Foot / grounded splayed pink balloon petal '+str(sign),PINK,center,(.084,.065,.119),32,22),sign*.22,center))
  x=sign*.117;y=.473;border=fitted_contour('Eye border / fitted dark balloon socket '+str(sign),INK,oval(x,y,.074,.090,36),skin,.006,subdivisions=2);white=fitted_contour('Eye white / fitted ivory balloon oval '+str(sign),WHITE,oval(x,y,.069,.085,36),surface(border),.003,subdivisions=2);iris=fitted_patch('Iris / fitted large blue balloon eye '+str(sign),BLUE,oval(x,y+.002,.054,.068,40),surface(white),.004);p.extend((border,white,iris))
  p.append(fitted_patch('Iris sheen / fitted soft turquoise lower reflection '+str(sign),LOWER,oval(x,y-.038,.027,.016,28),surface(iris),.0015))
  p.append(fitted_patch('Eye glint / fitted bright ivory balloon sparkle '+str(sign),WHITE,oval(x+.022,y+.037,.014,.016,28),surface(iris),.002))
 tuft=tilt(ellipsoid('Tuft / large angled rolled pink balloon forelock',PINK,(.024,.697,.130),(.116,.135,.082),40,28),-.4,(.024,.697,.130));p.extend((tuft,spiral('Tuft crease / fitted rose balloon curl',tuft,.018,.690,.073,(.59,.39,.48))))
 smile=contour((((-.047,.370),(-.028,.330),(.022,.326),(.050,.363)),((.050,.363),(.020,.320),(-.029,.324),(-.050,.367)),((-.050,.367),(-.049,.368),(-.048,.369),(-.047,.370))),12);p.append(fitted_contour('Mouth / fitted gentle curved balloon smile',INK,smile,skin,.003,subdivisions=2))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye border /','Eye white /','Iris /','Iris sheen /','Eye glint /','Ear lining /','Mouth /')):p[i]=crease_normals(q,35)
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[0.0 if q['part'].startswith('Ear /') and i%3==1 and round(v,7)==0. else round(v,7) for i,v in enumerate(q['normals'])]
 return {'name':'icon_jigglypuff','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 from jigglypuff_glb import main_export
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--output',type=Path,required=True)
 main_export(parser.parse_args().output)
if __name__=='__main__':main()
