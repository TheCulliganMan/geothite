#!/usr/bin/env python3
"""Original spinning handstand fighter with blue spiked feet and a ball tail."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.25
TAN=(.66,.53,.40);CREASE=(.48,.37,.28);BLUE=(.31,.52,.74)
WHITE=(.96,.97,.89);INK=(.075,.07,.065)

def limbs(p):
 for s in (-1,1):
  p.append(paddle('Leg / rooted long curved tan handstand shin '+str(s),TAN,[(s*.127,.578,.018),(s*.259,.352,.099),(s*.323,.163,.163)],[.040,.035,.033],[.041,.036,.034]))
  p.append(curved_tube('Ankle wrap / rooted ivory handstand foot band '+str(s),WHITE,[(s*.309,.204,.151),(s*.326,.156,.169)],[.039,.044],28))
  foot=ellipsoid('Foot / broad blue three-spiked handstand ball '+str(s),BLUE,(s*.333,.127,.192),(.142,.128,.137),40,26);p.append(foot)
  paths=[([(s*.432,.135,.193),(s*.476,.144,.194),(s*.523,.153,.197)],[.041,.024,.002]),
         ([(s*.274,.041,.208),(s*.265,-.011,.223),(s*.253,-.064,.235)],[.040,.026,.002]),
         ([(s*.303,.143,.293),(s*.291,.153,.335),(s*.278,.169,.397)],[.038,.025,.002])]
  for j,(path,radii) in enumerate(paths):p.append(curved_tube(f'Foot spike / three rooted ivory handstand points {s} {j}',WHITE,path,radii,28))
  p.append(paint('Foot sole / fitted tan round handstand pad '+str(s),TAN,oval(s*.333,.128,.038,.055,28),foot,.005,1))
  p.append(paddle('Arm / rooted curved tan handstand forelimb '+str(s),TAN,[(s*.148,.759,.029),(s*.312,.807,.064),(s*.352,.932,.095)],[.035,.033,.029],[.036,.034,.030]))
  p.append(curved_tube('Wrist wrap / rooted ivory handstand cuff '+str(s),WHITE,[(s*.346,.908,.089),(s*.359,.967,.104)],[.036,.041],28))
  p.append(ellipsoid('Hand / round tan handstand fist '+str(s),TAN,(s*.364,1.000,.113),(.071,.073,.074),36,24))

def sculpture(normalize=True):
 body=ellipsoid('Body / compact blue handstand pear',BLUE,(0,.708,.011),(.194,.211,.174),44,28)
 neck=ellipsoid('Neck / short rooted tan handstand neck',TAN,(0,.882,.030),(.073,.096,.076),36,24)
 head=crease_normals(loft('Head / continuous broad tan spinning helmet',TAN,curved_profiles(((.901,.073,.084,.044),(.974,.226,.180,.028),(1.083,.286,.212,.004),(1.204,.407,.268,-.021),(1.302,.358,.239,-.037),(1.373,.130,.107,-.029),(1.400,.035,.028,-.027)),5),56),45)
 p=[body,neck,head]
 p.append(curved_tube('Spin point / rooted tan handstand balance cone',TAN,[(0,1.366,-.027),(0,1.439,-.024),(0,1.548,-.019)],[.069,.042,.002],32))
 p.append(paint('Belly / fitted pointed tan handstand shield',TAN,[(-.065,.529),(.065,.529),(.088,.625),(.046,.757),(0,.805),(-.046,.757),(-.088,.625)],body,.008,3))
 for s in (-1,1):
  eye=paint('Eye / fitted round black handstand gaze '+str(s),INK,oval(s*.117,1.070,.036,.046,32),head,.007,1);p.append(eye)
  p.append(paint('Pupil / fitted ivory round handstand stare '+str(s),WHITE,oval(s*.122,1.069,.013,.017,24),eye,.004,1))
  xy=[(s*x,y) for x,y in ((.251,1.213),(.299,1.226),(.338,1.243),(.353,1.251),(.349,1.244),(.335,1.237),(.296,1.218),(.252,1.207))]
  p.append(paint('Helmet crease / fitted tan side fold '+str(s),CREASE,xy,head,.003,2))
 p.append(paint('Mouth / fitted small handstand smile',CREASE,[(-.090,1.002),(-.036,.980),(0,.978),(.036,.980),(.090,1.002),(.080,.988),(.033,.972),(0,.970),(-.033,.972),(-.080,.988)],head,.004,2))
 limbs(p)
 tail=curved_tube('Tail / long curved tan third balance limb',TAN,[(0,.563,-.099),(.076,.409,-.312),(.107,.244,-.486),(.090,.190,-.553)],[.040,.036,.032,.028],28);p.append(tail)
 ball=ellipsoid('Tail ball / rooted small blue balance ball',BLUE,(.087,.171,-.579),(.077,.077,.080),36,24);p.append(ball)
 p.append(curved_tube('Tail spike / rooted ivory balance point',WHITE,[(.087,.176,-.630),(.087,.191,-.682),(.087,.211,-.732)],[.032,.020,.002],28))
 # Rotate the authored anatomy around Z: the face stays forward while the
 # pointed helmet becomes the support and the three blue balance ends rise.
 for q in p:
  for field in ('positions','normals'):q[field]=[(-v if i%3<2 else v) for i,v in enumerate(q[field])]
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 from sculpt_geometry import linear
 for q in p:
  n=q['part'];rgb=(0,0,0) if n.startswith(('Eye /','Helmet crease /','Mouth /')) else (255,255,255) if n.startswith(('Pupil /','Ankle wrap /','Foot spike /','Wrist wrap /','Tail spike /')) else (82,82,173) if n.startswith(('Body /','Foot /','Tail ball /')) else (198,156,33)
  q['base_color']=linear(tuple(v/255 for v in rgb))
 if normalize:
  for q in p:
   q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'hitmontop','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'hitmontop.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
