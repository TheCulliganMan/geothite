#!/usr/bin/env python3
"""Original powerful Raikou with fitted tiger stripes, saber fangs and a storm mantle."""
import argparse,math
from copy import deepcopy
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,translate,tube,fitted_contour,crease_normals
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.30
YELLOW=(1.,1.,0.);PURPLE=(156/255,49/255,148/255)
WHITE=(1.,1.,1.);INK=(0.,0.,0.);HELMET=(.22,.24,.25);CYAN=(.64,.88,.91)
TAIL_POINTS=((0.,.65,-.65),(.16,.54,-.80),(.29,.49,-.90),(.29,.98,-1.01),(.40,1.24,-1.12),(.48,.82,-1.25))

def plate(name,color,xy,z,th=.025):
 skin=z if callable(z) else lambda y,x:z
 return crease_normals(fitted_contour(name,color,xy,skin,th,2),35.)

def side_frame(p,side,inverse=False):
 q=deepcopy(p)
 for key in ('positions','normals'):
  q[key]=[v for i in range(0,len(q[key]),3) for v in ((side*q[key][i+2],q[key][i+1],-side*q[key][i]) if inverse else (-side*q[key][i+2],q[key][i+1],side*q[key][i]))]
 return q

def side_patch(name,zy,parent,side):
 patch=paint(name,INK,[(-side*z,y) for z,y in zy],side_frame(parent,side),.004,3)
 return side_frame(patch,side,True)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Body / powerful yellow Raikou torso',YELLOW,(0.,.59,-.17),(.305,.285,.535),32,18)
 neck=ellipsoid('Neck / raised Raikou shoulder and neck',YELLOW,(0.,.80,.15),(.26,.235,.25),28,16)
 head=ellipsoid('Head / broad white Raikou skull',WHITE,(0.,1.15,.34),(.275,.205,.23),32,18)
 p=[body,neck,head]
 p.append(ellipsoid('Bib / broad white Raikou throat',WHITE,(0.,.75,.377),(.16,.21,.085),24,16))
 for side in (-1,1):
  # Patches follow actual stored side triangles instead of standing on flat cards.
  for j,c in enumerate((.14,-.12,-.37)):
   xy=[(c-.045,.77),(c+.023,.71),(c+.008,.61),(c+.085,.59),(c+.025,.48),(c-.034,.53),(c-.055,.64)]
   p.append(side_patch('Body stripe / fitted jagged Raikou flank '+str(side)+' '+str(j),xy,body,side))
  for rear,z in ((False,.28),(True,-.50)):
   label='rear' if rear else 'fore';x=side*.235
   leg=translate(loft('Leg / rooted '+label+' Raikou leg '+str(side),YELLOW,
    [(.07,.067,.070,z+.045),(.17,.086,.090,z+.040),(.27,.085,.100,z-.015),
     (.42,.110,.100,z),(.55,.110,.120,z-.020),(.62,.045,.060,z-.030)],24),x,0.,0.);p.append(leg)
   xy=[(z-.035,.49),(z+.035,.45),(z+.012,.35),(z+.065,.33),(z+.025,.24),(z-.050,.34)]
   p.append(side_patch('Leg stripe / fitted '+label+' Raikou lightning mark '+str(side),xy,leg,side))
   p.append(translate(loft('Foot / broad planted '+label+' Raikou paw '+str(side),WHITE,[(0.,.105,.115,z+.035),(.026,.129,.151,z+.047),(.078,.126,.145,z+.028),(.112,.084,.101,z)],24),x,0.,0.))
   for j,dx in enumerate((-.072,0.,.072)):
    p.append(ellipsoid('Toe / round white '+label+' Raikou toe '+str(side)+' '+str(j),WHITE,(x+dx,.047,z+.151),(.034,.046,.059),16,10))
  cheek=ellipsoid('Cheek / full white Raikou muzzle ruff '+str(side),WHITE,(side*.155,1.062,.42),(.19,.158,.175),28,16);p.append(cheek)
  ear=ellipsoid('Ear / pointed white Raikou ear '+str(side),WHITE,(side*.19,1.337,.21),(.075,.105,.064),20,14);p.append(ear)
  p.append(paint('Ear inset / dark fitted Raikou ear '+str(side),INK,oval(side*.19,1.348,.036,.050,16),ear,.005,1))
  p.append(curved_tube('Fang / long tapered Raikou saber '+str(side),WHITE,[(side*.105,1.065,.584),(side*.105,.92,.613),(side*.09,.785,.615)],[.033,.021,.0018],16))
 helmet=plate('Helmet / dark angular Raikou forehead armor',HELMET,[(-.26,1.305),(-.15,1.365),(0.,1.365),(.15,1.365),(.26,1.305),(.20,1.22),(.075,1.25),(0.,1.10),(-.075,1.25),(-.20,1.22)],lambda y,x:.56-2.8*x*x-.20*max(0.,y-1.27),.035);p.append(helmet)
 # Raised dark ovals sit on the forehead armor, not a separate hovering hat.
 for j,(x,y) in enumerate(((-.095,1.32),(0.,1.332),(.095,1.32))):
  p.append(paint('Armor ridge / fitted dark Raikou forehead stone '+str(j),HELMET,oval(x,y,.022,.027,12),helmet,.007,1))
 whiskers=[(-.075,1.10),(-.32,1.20),(-.20,1.08),(-.30,.97),(-.07,1.035),(0.,1.01),(.07,1.035),(.30,.97),(.20,1.08),(.32,1.20),(.075,1.10),(0.,1.15)]
 p.append(plate('Whisker mask / pale cyan Raikou muzzle blades',CYAN,whiskers,lambda y,x:.615-1.6*x*x,.024))
 p.append(ellipsoid('Jaw / rounded white Raikou lower muzzle',WHITE,(0.,.993,.525),(.145,.083,.110),24,14))
 # Fitted eye layers remain above the white cheeks and below the armor rim.
 for side in (-1,1):
  x=side*.143;xy=[(x-side*.027,1.286),(x+side*.031,1.272),(x+side*.024,1.247),(x-side*.018,1.252)]
  socket=paint('Eye socket / fitted black Raikou stare '+str(side),INK,xy,helmet,.010,3);p.append(socket)
  white=paint('Eye / fitted white Raikou sclera '+str(side),WHITE,[(x+(a-x)*.84,1.266+(b-1.266)*.79) for a,b in xy],socket,.003,3);p.append(white)
  iris=paint('Iris / fitted amber Raikou iris '+str(side),YELLOW,oval(x,1.267,.008,.008,12),white,.003,1);p.append(iris)
  p.append(paint('Pupil / narrow black Raikou pupil '+str(side),INK,oval(x,1.267,.003,.005,12),iris,.002,1))
 p.append(ellipsoid('Storm mane / continuous purple Raikou mantle',PURPLE,(0.,1.04,-.32),(.32,.145,.48),32,16))
 for j,(x,y,z,rx,ry,rz) in enumerate(((.02,1.23,-.08,.19,.16,.19),(-.04,1.21,-.32,.19,.145,.20),(.05,1.15,-.58,.17,.14,.19))):
  p.append(ellipsoid('Storm curl / raised purple Raikou billow '+str(j),PURPLE,(x,y,z),(rx,ry,rz),24,14))
 p.append(tube('Tail / angular cyan Raikou lightning cable',CYAN,TAIL_POINTS,[.027,.025,.025,.023,.020,.018],12))
 x,y,z=TAIL_POINTS[-1];star=[]
 for i in range(12):
  a=math.pi/2+i*math.pi/6;r=.115 if i%2==0 else .048;star.append((x+r*math.cos(a),y+r*math.sin(a)))
 p.append(plate('Tail star / pointed cyan Raikou terminal bolt',CYAN,star,z,.024))
 # The tiger carries its head in front of the shoulders, with a low flowing cape.
 head_parts=('Head /','Cheek /','Ear /','Ear inset /','Fang /','Helmet /','Armor ridge /','Whisker mask /','Jaw /','Eye socket /','Eye /','Iris /','Pupil /')
 p=[translate(q,0.,-.12,0.) if q['part'].startswith(head_parts) else translate(q,0.,-.08,0.) if q['part'].startswith(('Storm mane /','Storm curl /')) else q for q in p]
 return {'name':'raikou','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(*,normalize=True):
 model=deepcopy(raw_sculpture())
 if normalize:
  parts=model['primitives'];low=min(v for p in parts for v in p['positions'][1::3]);high=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHT/(high-low)
  for p in parts:p['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return model

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from raikou_glb import export_raikou
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_raikou())
