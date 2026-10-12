#!/usr/bin/env python3
"""Original headless kicking fighter with continuous ribbed spring limbs."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,tube
from bovine_sculpt import oval,curved_tube,smooth
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from nidorino_sculpt import side_patch
from skin_glb import COORDINATES
HEIGHT=1.24
TAN=(.63,.52,.44);CREAM=(.88,.85,.67);WHITE=(.98,.98,.94)
INK=(.085,.085,.10);GOLD=(.94,.82,.34)

def spring(name,points,radii,ribs):
 pp,rr=smooth(points,radii,24);rr=[r+.012*math.sin(math.pi*ribs*k/(len(pp)-1))**2 for k,r in enumerate(rr)]
 return crease_normals(tube(name,CREAM,pp,rr,32),45)

def limbs(p):
 for s in (-1,1):
  hip=paddle('Hip / rooted tapered tan kicking thigh '+str(s),TAN,[(s*.085,.654,-.013),(s*.166,.567,.004),(s*.195,.512,.036)],[.081,.073,.056],[.085,.080,.062]);p.append(hip)
  p.append(spring('Leg spring / continuous cream nine-rib shin '+str(s),[(s*.189,.550,.027),(s*.213,.313,.077),(s*.218,.058,.150)],[.061,.059,.053],9))
  foot=ellipsoid('Foot / grounded broad tan three-toed pad '+str(s),TAN,(s*.218,.041,.170),(.107,.041,.143),36,24);p.append(foot)
  for j,dx in enumerate((-.063,0,.063)):
   p.append(curved_tube(f'Toe claw / three rooted ivory kicking points {s} {j}',WHITE,[(s*.218+dx*.71,.045,.264),(s*.218+dx,.031,.332),(s*.218+dx*.96,.020,.356)],[.027,.022,.003],24))
  p.append(side_patch('Foot mark / fitted round gold sole spot '+str(s),GOLD,oval(.173,.059,.039,.014,28),foot,s))
  arm=paddle('Arm / rooted bent tan upper forelimb '+str(s),TAN,[(s*.208,1.067,.006),(s*.350,.962,.035),(s*.414,.948,.094),(s*.432,.972,.143)],[.083,.065,.050,.043],[.086,.069,.052,.043]);p.append(arm)
  p.append(spring('Wrist spring / continuous cream five-rib forearm '+str(s),[(s*.411,.942,.094),(s*.448,.984,.171),(s*.470,1.021,.235)],[.052,.047,.041],5))
  palm=ellipsoid('Hand / tan three-finger martial palm '+str(s),TAN,(s*.477,1.040,.268),(.069,.057,.059),32,20);p.append(palm)
  for j,(dx,y,z) in enumerate(((-.042,1.062,.303),(0,1.044,.323),(.044,1.015,.298))):
   p.append(curved_tube(f'Finger / three rooted curled tan martial digits {s} {j}',TAN,[(s*(.477+dx*.5),y+.010,.271),(s*(.477+dx),y,z),(s*(.480+dx),y-.044,z+.036),(s*(.471+dx),y-.087,z+.028)],[.023,.020,.015,.009],24))

def sculpture():
 body=crease_normals(loft('Body / continuous headless tan kicking-fighter pear',TAN,curved_profiles(((.525,.062,.077,-.012),(.610,.128,.135,-.021),(.766,.191,.165,-.024),(1.000,.264,.191,-.014),(1.204,.244,.177,.002),(1.343,.146,.112,.022),(1.386,.033,.029,.027)),6),48),45)
 p=[body]
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.066,1.188),(.192,1.148),(.177,1.083),(.123,1.076),(.079,1.119))]
  eye=paint('Eye / fitted sloping black kicking-fighter lens '+str(s),INK,xy,body,.007,3);p.append(eye)
  pupil=paint('Pupil / fitted narrow ivory fighter stare '+str(s),WHITE,oval(s*.135,1.125,.012,.023,24),eye,.004,3);p.append(pupil)
  p.append(paint('Pupil core / tiny fitted dark fighter point '+str(s),INK,oval(s*.135,1.123,.0035,.010,20),pupil,.003,2))
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'hitmonlee','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'hitmonlee.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
