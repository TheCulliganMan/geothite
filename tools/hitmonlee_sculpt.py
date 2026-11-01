#!/usr/bin/env python3
"""Original headless kicker with a broad shoulder mass and upright spring supports."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,tube,translate,fitted_triangle_contour
from bovine_sculpt import oval,curved_tube,smooth
from lugia_sculpt import paddle
from bulldog_sculpt import rounded_loft
from cubone_sculpt import skin_shading
from nidorino_sculpt import rig_side_patch as side_patch
from skin_glb import COORDINATES
HEIGHT=1.24
TAN=(.63,.52,.44);CREAM=(.88,.85,.67);WHITE=(.98,.98,.94)
INK=(.085,.085,.10);GOLD=(.94,.82,.34)

def paint(name,color,xy,parent,th=.006,sub=2):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th),45),parent)

def spring(name,points,radii,ribs):
 pp,rr=smooth(points,radii,24);rr=[r+.014*math.sin(math.pi*ribs*k/(len(pp)-1))**2 for k,r in enumerate(rr)]
 return crease_normals(tube(name,CREAM,pp,rr,32),45)

def arm_path(s):
 if s==-1:
  return [(s*.240,1.080,.004),(s*.365,.995,.025),(s*.436,.987,.115)],[(s*.422,.979,.113),(s*.430,1.085,.250),(s*.408,1.143,.306)],(s*.398,1.178,.344)
 return [(s*.240,1.072,.004),(s*.380,.968,.001),(s*.454,.908,.090)],[(s*.449,.914,.070),(s*.435,.969,.229),(s*.400,1.023,.302)],(s*.382,1.052,.347)

def limbs(p):
 for s in (-1,1):
  p.append(paddle('Hip / rooted tapered tan kicking thigh '+str(s),TAN,[(s*.139,.671,.014),(s*.195,.581,.019),(s*.220,.520,.020)],[.093,.087,.075],[.096,.089,.079],segments=24,steps=5))
  p.append(spring('Leg spring / continuous cream nine-rib shin '+str(s),[(s*.214,.554,.020),(s*.237,.319,.020),(s*.242,.088,.020)],[.080,.078,.068],9))
  foot=translate(rounded_loft('Foot / grounded broad tan three-toed pad '+str(s),TAN,[(0.,.126,.163,.078),(.041,.132,.170,.078),(.101,.119,.145,.070),(.145,.075,.083,.023)],28,.75),s*.242,0,0)
  # The heel rolls off its sole as the kick begins; the mid/forefoot stays flat.
  for i in range(0,len(foot['positions']),3):
   x,y,z=foot['positions'][i:i+3];roll=.025*max(0.,min(1.,(.050-z)/.135))**2*(1.-max(0.,min(1.,(y-.040)/.080)));foot['positions'][i+1]=y+roll
  p.append(crease_normals(foot,45))
  for j,dx in enumerate((-.075,0.,.075)):
   p.append(curved_tube(f'Toe claw / three rooted ivory kicking points {s} {j}',WHITE,[(s*.242+dx*.80,.065,.203),(s*.242+dx,.043,.285),(s*.242+dx*.98,.025,.326)],[.030,.022,.003],20))
  p.append(side_patch('Foot mark / fitted round gold sole spot '+str(s),GOLD,oval(.095,.078,.042,.035,20),foot,s))
  arm,wrist,center=arm_path(s)
  p.append(paddle('Arm / rooted bent tan upper forelimb '+str(s),TAN,arm,[.087,.065,.052],[.090,.068,.055],segments=24,steps=5))
  p.append(spring('Wrist spring / continuous cream five-rib forearm '+str(s),wrist,[.063,.057,.048],5))
  cx,cy,cz=center;p.append(ellipsoid('Hand / tan three-finger martial palm '+str(s),TAN,center,(.078,.069,.071),28,18))
  for j,dx in enumerate((-.048,0.,.048)):
   path=[(cx+s*dx*.55,cy+.027,cz+.018),(cx+s*dx,cy+.045,cz+.064),(cx+s*dx,cy-.010,cz+.108),(cx+s*dx*.96,cy-.072,cz+.072)]
   p.append(curved_tube(f'Finger / three rooted curled tan martial digits {s} {j}',TAN,path,[.025,.023,.017,.009],18))

def sculpture(normalize=True):
 body=crease_normals(loft('Body / continuous headless tan kicking-fighter pear',TAN,curved_profiles(((.532,.150,.110,-.012),(.627,.192,.166,-.018),(.817,.244,.218,-.018),(1.056,.301,.239,-.010),(1.234,.274,.212,.005),(1.365,.166,.130,.015),(1.406,.070,.068,.016)),5),40),45)
 p=[body]
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.045,1.174),(.220,1.261),(.205,1.162),(.138,1.113),(.064,1.138))]
  eye=paint('Eye / fitted sloping black kicking-fighter lens '+str(s),INK,xy,body,.006);p.append(eye)
  pupil=paint('Pupil / fitted ivory fighter stare '+str(s),WHITE,oval(s*.138,1.180,.021,.029,24),eye,.0035);p.append(pupil)
  p.append(paint('Pupil core / fitted dark fighter point '+str(s),INK,oval(s*.139,1.180,.006,.016,20),pupil,.002))
 limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Eye /','Pupil core /')) else (255,255,255) if n.startswith(('Pupil /','Toe claw /')) else (123,132,123) if n.startswith(('Leg spring /','Wrist spring /','Foot mark /')) else (181,115,41)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:
   q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
 return {'name':'hitmonlee','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'hitmonlee.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
