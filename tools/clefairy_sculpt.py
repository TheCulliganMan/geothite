#!/usr/bin/env python3
"""Original shared Clefairy sculpture, grounded and front +Z."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,fitted_patch,fitted_contour,tube,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,reverse_front
from wing_insect_sculpt import contour
from baby_fairy_sculpt import tilt,spiral
from skin_glb import COORDINATES
HEIGHT=.82
PINK=(.97,.73,.77);BROWN=(.43,.36,.29);INK=(.13,.12,.16);WHITE=(.98,.98,.94);BLUSH=(.94,.55,.65);TONGUE=(.77,.53,.66);WING=(.95,.88,.81)
def sculpture():
 body=loft('Body / continuous round pink fairy pear',PINK,curved_profiles(((.05,.05,.07,0),(.11,.16,.14,0),(.22,.26,.21,0),(.38,.30,.24,0),(.53,.27,.22,0),(.65,.22,.18,0),(.72,.15,.12,0),(.76,.04,.03,0)),4),40);skin=surface(body);p=[body]
 for sign in (-1,1):
  xy=contour((((.125,.650),(.159,.698),(.269,.757),(.350,.790)),((.350,.790),(.352,.707),(.334,.573),(.235,.530)),((.235,.530),(.197,.556),(.154,.609),(.125,.650))),10)
  ear=crease_normals(fitted_contour('Ear / swept triangular pink fairy point '+str(sign),PINK,[(sign*x,y) for x,y in xy],lambda y,x:.01+.064*math.exp(-((abs(x)-.25)/.13)**2-((y-.65)/.14)**2),.060,subdivisions=2),35);p.append(ear)
  tip=[(sign*x,y) for x,y in ((.281,.738),(.347,.779),(.339,.710))];p.append(fitted_contour('Ear tip / fitted muted brown fairy triangle '+str(sign),BROWN,tip,surface(ear),.004,subdivisions=3))
  center=(sign*.279,.386,.035);arm=tilt(ellipsoid('Arm / raised rounded fairy hand '+str(sign),PINK,center,(.091,.140,.075),24,16),-sign*.55,center);p.append(arm)
  for j in (-1,0,1):p.append(ellipsoid('Hand claw / rooted ivory fairy fingertip '+str(sign)+' '+str(j),WHITE,(sign*(.346+j*.023),.496-abs(j)*.013,.062),(.014,.023,.024),20,14))
  center=(sign*.124,.077,.066);foot=tilt(ellipsoid('Foot / grounded pointed pink fairy petal '+str(sign),PINK,center,(.078,.080,.104),24,16),sign*.22,center);p.append(foot)
  p.append(fitted_patch('Toe / fitted ivory fairy toenail '+str(sign),WHITE,oval(sign*.129,.041,.031,.012,28),surface(foot),.003))
  x=sign*.084;eye=fitted_patch('Eye / fitted dark narrow fairy oval '+str(sign),INK,oval(x,.503,.014,.033,32),skin,.005);p.extend((eye,fitted_patch('Eye glint / fitted ivory fairy reflection '+str(sign),WHITE,oval(x-.002,.516,.0045,.008,24),surface(eye),.0015),fitted_patch('Cheek / fitted pink fairy blush '+str(sign),BLUSH,oval(sign*.147,.446,.030,.009,32),skin,.002)))
  # Three overlapping rounded wing lobes form a small scalloped rear wing.
  for j in range(3):
   center=(sign*(.225+j*.038),.370+j*.035,-.162-j*.010);p.append(tilt(ellipsoid('Wing / rooted soft scalloped fairy feather '+str(sign)+' '+str(j),WING,center,(.065,.097-j*.008,.033),20,14),-sign*.7,center))
 tuft=ellipsoid('Tuft / large rooted round rolled fairy forelock',PINK,(0,.699,.120),(.113,.115,.070),32,20);p.extend((tuft,spiral('Tuft crease / fitted dark rose fairy curl',tuft,0,.699,.074,(.56,.37,.44))))
 mouth=contour((((-.047,.416),(-.015,.411),(.015,.412),(.047,.418)),((.047,.418),(.039,.313),(-.035,.317),(-.047,.416))),12);q=fitted_contour('Mouth / fitted delighted open fairy smile',INK,mouth,skin,.005,subdivisions=2);p.append(q);cx=sum(x for x,y in mouth)/len(mouth);cy=sum(y for x,y in mouth)/len(mouth);p.append(fitted_contour('Tongue / fitted rose fairy smile interior',TONGUE,[(cx+(x-cx)*.75,cy+(y-cy)*.75) for x,y in mouth],surface(q),.003))
 tooth=fitted_contour('Tooth / fitted small ivory fairy fang',WHITE,[(.014,.410),(.027,.411),(.023,.392)],surface(q),.004,subdivisions=2);p.append(tooth)
 pts=[(0,.25,-.16),(.08,.22,-.25),(.15,.17,-.32)];radii=[.073,.070,.067]
 for j in range(1,45):
  t=j/44;a=-math.pi/2+math.tau*1.12*t;r=.13*(1-.75*t);pts.append((.15+r*math.cos(a),.30+r*math.sin(a),-.34));radii.append(.065*(1-.70*t))
 p.append(crease_normals(tube('Tail / rooted thick curled pink fairy spiral',PINK,pts,radii,20),35))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye /','Eye glint /','Cheek /','Mouth /','Tongue /','Tooth /','Ear tip /','Toe /')):p[i]=crease_normals(q,35)
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'icon_clefairy','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'icon_clefairy.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
