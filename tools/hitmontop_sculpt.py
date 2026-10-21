#!/usr/bin/env python3
"""Original Crystal spinning fighter with a deep helmet, flanges and compact raised limbs."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,fitted_contour,fitted_triangle_contour,translate
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from bulldog_sculpt import rounded_loft
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES
HEIGHT=1.25
TAN=(.66,.53,.40);CREASE=(.48,.37,.28);BLUE=(.31,.52,.74)
WHITE=(.96,.97,.89);INK=(.075,.07,.065)

def paint(name,color,xy,parent,th=.006,sub=2):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th),45),parent)

def arm_path(s):return [(s*.180,.803,.025),(s*.338,.853,.060),(s*.432,.884,.139)]

def helmet_flange(s):
 xy=[(s*x,y) for x,y in ((.160,1.255),(.280,1.250),(.480,1.055),(.455,.990),(.350,1.020),(.200,1.080))]
 return crease_normals(fitted_contour('Helmet flange / thick swept tan headstand brim '+str(s),TAN,xy,lambda y,x:.050-.20*(abs(x)-.16),.110,2),55)

def limbs(p):
 for s in (-1,1):
  p.append(paddle('Leg / rooted short bent tan handstand shin '+str(s),TAN,[(s*.150,.592,.015),(s*.280,.542,.045),(s*.354,.462,.094)],[.064,.058,.052],[.064,.059,.054],segments=24,steps=5))
  p.append(curved_tube('Ankle wrap / rooted ivory handstand foot band '+str(s),WHITE,[(s*.318,.502,.073),(s*.370,.436,.125)],[.064,.067],24))
  foot=translate(rounded_loft('Foot / broad blue three-spiked handstand boot '+str(s),BLUE,[(.295,.085,.085,.145),(.336,.132,.130,.145),(.425,.140,.135,.145),(.503,.089,.092,.133)],28,.70),s*.397,0,0);p.append(foot)
  paths=[([(s*.492,.285,.143),(s*.548,.286,.144),(s*.604,.288,.146)],[.040,.025,.002]),([(s*.382,.205,.153),(s*.367,.149,.158),(s*.352,.095,.164)],[.040,.025,.002]),([(s*.397,.285,.243),(s*.397,.288,.300),(s*.397,.293,.360)],[.039,.025,.002])]
  for j,(path,radii) in enumerate(paths):p.append(curved_tube(f'Foot spike / three rooted ivory handstand points {s} {j}',WHITE,[(x,y+.110,z) for x,y,z in path],radii,24))
  p.append(paint('Foot sole / fitted tan round handstand pad '+str(s),TAN,oval(s*.397,.397,.047,.057,28),foot,.005))
  p.append(paddle('Arm / rooted curved tan handstand forelimb '+str(s),TAN,arm_path(s),[.054,.050,.044],[.055,.051,.046],segments=24,steps=5))
  p.append(curved_tube('Wrist wrap / rooted ivory handstand cuff '+str(s),WHITE,[(s*.409,.873,.118),(s*.462,.908,.168)],[.052,.055],24))
  p.append(ellipsoid('Hand / round tan handstand fist '+str(s),TAN,(s*.490,.925,.188),(.083,.083,.084),28,18))

def sculpture(normalize=True):
 body=ellipsoid('Body / compact blue handstand pear',BLUE,(0,.755,.007),(.230,.225,.200),32,22)
 neck=ellipsoid('Neck / short rooted tan handstand neck',TAN,(0,.946,.017),(.085,.099,.085),28,18)
 head=crease_normals(loft('Head / continuous deep tan spinning helmet',TAN,curved_profiles(((.945,.098,.092,.023),(1.015,.207,.177,.015),(1.142,.251,.210,.004),(1.245,.242,.197,-.008),(1.328,.173,.146,-.018),(1.383,.068,.066,-.021),(1.402,.026,.025,-.021)),4),40),45)
 p=[body,neck,head]
 p.append(curved_tube('Spin point / rooted tan handstand balance cone',TAN,[(0,1.370,-.021),(0,1.452,-.019),(0,1.575,-.015)],[.069,.043,.002],28))
 p.append(paint('Belly / fitted pointed tan handstand shield',TAN,[(-.078,.559),(.078,.559),(.102,.671),(.058,.820),(0,.862),(-.058,.820),(-.102,.671)],body,.007))
 for s in (-1,1):
  flange=helmet_flange(s);p.append(flange)
  eye=paint('Eye / fitted round black handstand gaze '+str(s),INK,oval(s*.110,1.116,.030,.039,28),head,.005);p.append(eye)
  p.append(paint('Pupil / fitted ivory round handstand stare '+str(s),WHITE,oval(s*.113,1.114,.011,.015,20),eye,.003))
  for j in range(3):
   y=1.218-.039*j;xy=[(s*.222,y),(s*.354,y-.065),(s*.353,y-.071),(s*.221,y-.006)]
   p.append(paint(f'Helmet crease / fitted three headstand brim folds {s} {j}',CREASE,xy,flange,.003))
 p.append(paint('Mouth / fitted small handstand smile',CREASE,[(-.083,1.037),(-.032,1.020),(0,1.018),(.032,1.020),(.083,1.037),(.076,1.028),(.030,1.012),(0,1.010),(-.030,1.012),(-.076,1.028)],head,.003))
 limbs(p)
 p.append(curved_tube('Tail / long curved tan third balance limb',TAN,[(0,.601,-.129),(.060,.490,-.314),(.137,.400,-.481),(.159,.360,-.551)],[.047,.043,.038,.032],24))
 p.append(ellipsoid('Tail ball / rooted small blue balance ball',BLUE,(.165,.340,-.588),(.084,.084,.087),28,18))
 p.append(curved_tube('Tail spike / rooted ivory balance point',WHITE,[(.165,.343,-.645),(.165,.356,-.700),(.165,.376,-.754)],[.034,.021,.002],24))
 # Crystal and BW use the inverted spin idle. Keep its coherent authoring
 # frame, then rotate the full original cage around Z with the face forward.
 for q in p:
  for field in ('positions','normals'):q[field]=[(-v if i%3<2 else v) for i,v in enumerate(q[field])]
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Eye /','Mouth /')) else (255,255,255) if n.startswith(('Pupil /','Ankle wrap /','Foot spike /','Wrist wrap /','Tail spike /')) else (82,82,173) if n.startswith(('Body /','Foot /','Tail ball /','Helmet crease /')) else (198,156,33)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:
   q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'hitmontop','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'hitmontop.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
