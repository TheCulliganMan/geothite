#!/usr/bin/env python3
"""Original shared Wigglytuff: smooth balloon, pointed ears and fitted blue eyes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval
from jigglypuff_sculpt import pillow_ear
from chikorita_sculpt import linear
from baby_fairy_sculpt import tilt,spiral
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.0
PINK=(.98,.80,.85);INK=(.13,.15,.20);WHITE=(.98,.99,.97);BLUE=(.13,.63,.73);LOWER=(.41,.79,.84)
def sculpture():
 body=loft('Body / continuous tall pink balloon pear',PINK,curved_profiles(((.025,.040,.050,0),(.09,.180,.130,0),(.19,.270,.180,0),(.36,.300,.225,0),(.53,.275,.215,0),(.68,.225,.170,0),(.79,.150,.090,0),(.82,.022,.025,0)),4),40);skin=surface(body);p=[body]
 bellyxy=((0,.595),(.095,.569),(.174,.489),(.223,.352),(.245,.215),(.184,.105),(.103,.063),(0,.051),(-.103,.063),(-.184,.105),(-.245,.215),(-.223,.352),(-.174,.489),(-.095,.569))
 belly=fitted_contour('Belly / fitted broad ivory balloon pear',WHITE,bellyxy,skin,.008,subdivisions=4);p.append(belly)

 for sign in (-1,1):
  xy=contour((((.115,.721),(.105,.884),(.268,1.100),(.445,1.174)),((.445,1.174),(.459,1.023),(.335,.773),(.218,.711)),((.218,.711),(.192,.694),(.153,.702),(.115,.721))),12)
  ear=pillow_ear('Ear / long swept pink balloon rabbit leaf '+str(sign),[(sign*x,y) for x,y in xy]);ear['base_color']=linear(PINK);p.append(ear)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  inset=[(sign*(cx+(x-cx)*.74),cy+(y-cy)*.74) for x,y in xy[::3]];p.append(fitted_contour('Ear lining / fitted charcoal balloon rabbit leaf '+str(sign),INK,inset,surface(ear),.006,subdivisions=3))

  center=(sign*.274,.415,.071);p.append(tilt(ellipsoid('Arm / short rounded pink balloon leaf '+str(sign),PINK,center,(.068,.106,.060),32,22),-sign*.65,center))
  center=(sign*.123,.059,.070);p.append(tilt(ellipsoid('Foot / grounded splayed pink balloon petal '+str(sign),PINK,center,(.085,.062,.105),32,22),sign*.22,center))
  x=sign*.112;y=.655;border=fitted_contour('Eye border / fitted dark balloon socket '+str(sign),INK,oval(x,y,.072,.093,36),skin,.006,subdivisions=3);white=fitted_contour('Eye white / fitted ivory balloon oval '+str(sign),WHITE,oval(x,y,.067,.088,36),surface(border),.003,subdivisions=3);iris=fitted_patch('Iris / fitted large blue balloon eye '+str(sign),BLUE,oval(x,y+.002,.052,.071,40),surface(white),.004);p.extend((border,white,iris))
  p.append(fitted_patch('Iris sheen / fitted soft turquoise lower reflection '+str(sign),LOWER,oval(x,y-.038,.027,.016,28),surface(iris),.0015))
  p.append(fitted_patch('Eye glint / fitted bright ivory balloon sparkle '+str(sign),WHITE,oval(x+.022,y+.037,.014,.016,28),surface(iris),.002))
 tuft=tilt(ellipsoid('Tuft / large angled rolled pink balloon forelock',PINK,(.024,.827,.105),(.107,.117,.078),40,28),-.4,(.024,.827,.105));p.extend((tuft,spiral('Tuft crease / fitted rose balloon curl',tuft,.018,.825,.065,(.59,.39,.48))))
 mouthxy=contour((((-.041,.574),(-.018,.569),(.018,.569),(.041,.574)),((.041,.574),(.027,.520),(-.027,.520),(-.041,.574))),12)
 mouth=fitted_contour('Mouth / fitted friendly open balloon smile',INK,mouthxy,surface(belly),.005,subdivisions=3);p.append(mouth);p.append(fitted_patch('Tongue / fitted rosy balloon smile interior',(.76,.51,.64),oval(0,.541,.017,.006,28),surface(mouth),.003))

 for i,q in enumerate(p):
  if q['part'].startswith(('Eye border /','Eye white /','Iris /','Iris sheen /','Eye glint /','Ear lining /','Mouth /','Tongue /','Belly /')):p[i]=crease_normals(q,35)
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'wigglytuff','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'wigglytuff.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
