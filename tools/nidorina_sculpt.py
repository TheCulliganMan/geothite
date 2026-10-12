#!/usr/bin/env python3
"""Original hornless poison-pin creature with notched ears and fitted markings."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from goldfish_sculpt import zy
from skin_glb import COORDINATES
from wing_insect_sculpt import contour
HEIGHT=1.12
BLUE=(.53,.70,.77);PALE=(.69,.81,.85);NAVY=(.23,.32,.53)
WHITE=(.98,.98,.94);INK=(.065,.08,.12);ORANGE=(.76,.30,.18)

def ears(p):
 for s in (-1,1):
  outline=[(.115,.962),(.144,1.092),(.189,1.287),(.206,1.201),(.385,1.357),(.379,1.287),(.437,1.205),(.410,1.178),(.426,1.126),(.361,1.068),(.259,1.010),(.172,.976)]
  ear=sheet('Ear / thick notched blue pinna '+str(s),BLUE,[(s*x,y) for x,y in outline],lambda y,x:.033-.18*(y-.96),.036,3);p.append(ear)
  inset=[(.162,1.040),(.194,1.147),(.339,1.263),(.379,1.224),(.381,1.171),(.343,1.110),(.268,1.060),(.208,1.034)]
  inner=paint('Ear inset / fitted navy inner field '+str(s),NAVY,[(s*x,y) for x,y in inset],ear,.006,3);p.append(inner)
  fold=[(.184,1.040),(.224,1.128),(.281,1.177),(.311,1.152),(.328,1.105),(.275,1.067),(.222,1.045)]
  p.append(paint('Ear fold / fitted blue lower pinna '+str(s),BLUE,[(s*x,y) for x,y in fold],inner,.006,3))

def side_mark(p,parent,s,outline,name):
 # Rotate the actual parent into a +Z-facing chart, then rotate both back.
 chart=zy(parent)
 if s==1:chart=reverse_front(chart)
 xy=[((z if s==-1 else -z),y) for z,y in outline]
 patch=paint(name,NAVY,xy,chart,.004,3)
 if s==1:patch=reverse_front(patch)
 p.append(zy(zy(zy(patch))))

def limbs(p,body):
 for s in (-1,1):
  arm=paddle('Arm / rooted bent blue forelimb '+str(s),BLUE,[(s*.203,.585,.075),(s*.275,.456,.151),(s*.277,.323,.237)],[.085,.073,.077],[.085,.074,.068]);p.append(arm)
  hand=ellipsoid('Hand / rounded downward blue paw '+str(s),BLUE,(s*.274,.305,.239),(.081,.105,.087),32,24);p.append(hand)
  for j,dx in enumerate((-.042,0,.042)):
   p.append(curved_tube(f'Hand claw / three rooted ivory points {s} {j}',WHITE,[(s*.274+dx,.259,.265),(s*.274+dx*.9,.226,.285),(s*.274+dx*.8,.215,.292)],[.020,.014,.002],20))
  thigh=ellipsoid('Haunch / substantial spotted blue hindquarter '+str(s),BLUE,(s*.212,.238,-.098),(.136,.207,.174),40,26);p.append(thigh)
  p.append(paddle('Leg / short rooted blue hind ankle '+str(s),BLUE,[(s*.232,.160,-.038),(s*.246,.088,.018),(s*.251,.044,.087)],[.074,.072,.073],[.079,.071,.042]))
  foot=ellipsoid('Foot / grounded blue hind paw '+str(s),BLUE,(s*.251,.033,.091),(.089,.033,.119),32,20);p.append(foot)
  for j,dx in enumerate((-.052,0,.052)):
   p.append(curved_tube(f'Foot claw / three rooted ivory toes {s} {j}',WHITE,[(s*.251+dx*.8,.030,.169),(s*.251+dx,.018,.218),(s*.251+dx,.011,.240)],[.024,.020,.003],20))
  for j,(z,y,rz,ry) in enumerate(((-.096,.315,.049,.066),(-.090,.195,.028,.038),(-.140,.404,.020,.029))):
   side_mark(p,thigh,s,oval(z,y,rz,ry,24),f'Haunch spot / fitted navy poison marking {s} {j}')
  side_mark(p,body,s,oval(-.125,.554,.023,.032,24),'Flank spot / fitted navy poison marking '+str(s))

def sculpture():
 body=crease_normals(loft('Body / continuous squat blue poison-pin pear',BLUE,curved_profiles(((.119,.115,.135,-.077),(.235,.238,.216,-.083),(.414,.284,.230,-.053),(.568,.256,.202,-.029),(.678,.184,.144,.011),(.745,.075,.073,.042)),4),44),45)
 head=crease_normals(loft('Head / broad hornless blue angular cranium',BLUE,curved_profiles(((.637,.075,.079,.121),(.726,.186,.142,.157),(.843,.235,.180,.100),(.964,.197,.152,.061),(1.029,.103,.083,.049),(1.041,.025,.025,.049)),4),48),45)
 muzzle=ellipsoid('Muzzle / low broad hornless blue snout',BLUE,(0,.746,.256),(.176,.075,.123),44,26)
 p=[body,head,muzzle]
 p.append(paint('Breast / fitted pale blue belly',PALE,contour([((-.115,.572),(-.185,.530),(-.187,.345),(-.064,.224)),((-.064,.224),(-.035,.203),(.035,.203),(.064,.224)),((.064,.224),(.187,.345),(.185,.530),(.115,.572)),((.115,.572),(.064,.596),(-.064,.596),(-.115,.572))],8),body,.008,4))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.051,.832),(.171,.911),(.198,.850),(.171,.809),(.095,.802))]
  socket=paint('Eye socket / fitted angular dark recess '+str(s),INK,xy,head,.008,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow ivory lens '+str(s),WHITE,[(cx+.91*(x-cx),cy+.88*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted orange poison-pin gaze '+str(s),ORANGE,oval(s*.143,.846,.019,.024,28),eye,.004,3);p.append(iris)
  pupil=paint('Pupil / fitted dark poison-pin slit '+str(s),INK,oval(s*.143,.846,.008,.020,24),iris,.003,3);p.append(pupil)
  p.append(paint('Glint / tiny ivory eye sparkle '+str(s),WHITE,oval(s*.143-.003,.855,.0025,.0045,16),pupil,.002,2))
  p.append(paint('Nostril / small fitted blue snout recess '+str(s),INK,oval(s*.044,.758,.005,.003,16),muzzle,.003,2))
  p.append(curved_tube('Fang / small rooted ivory canine '+str(s),WHITE,[(s*.134,.715,.322),(s*.133,.693,.339),(s*.127,.669,.343)],[.017,.013,.002],24))
 p.append(paint('Mouth / fitted shallow poison-pin smile',INK,[(-.15,.716),(-.090,.700),(0,.687),(.09,.70),(.15,.716),(.125,.699),(.074,.684),(0,.678),(-.074,.684),(-.125,.699)],muzzle,.003,3))
 ears(p);limbs(p,body)
 for j,(y,z,tipy,tipz,r) in enumerate(((.629,-.135,.835,-.259,.091),(.555,-.250,.740,-.376,.079),(.443,-.283,.590,-.432,.066))):
  p.append(curved_tube('Back spine / rooted swept blue poison barb '+str(j),BLUE,[(0,y,z),(0,y+.067,z-.049),(0,tipy,tipz)],[r,r*.65,.002],28))
 p.append(curved_tube('Tail / short rooted blue rear point',BLUE,[(0,.291,-.205),(0,.311,-.390),(.02,.421,-.531)],[.100,.081,.002],28))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'nidorina','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'nidorina.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
