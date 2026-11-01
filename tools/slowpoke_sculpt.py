#!/usr/bin/env python3
"""Original low pink Slowpoke with a sleepy open muzzle and raised pale-tipped tail."""
import argparse,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,translate,tube
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.0
PINK=(1.,82/255,140/255);SHADOW=(173/255,41/255,66/255)
CREAM=(1.,1.,1.);INK=(0.,0.,0.)
TAIL_POINTS=((0.,.22,-.36),(.055,.255,-.55),(.16,.38,-.72),(.23,.54,-.72),(.23,.65,-.63))

def sculpture(*,normalize=True):
 body=ellipsoid('Body / long low continuous pink Slowpoke torso',PINK,(0.,.255,-.095),(.275,.225,.405),32,18)
 head=ellipsoid('Head / broad round sleepy pink Slowpoke head',PINK,(0.,.445,.29),(.270,.225,.235),32,20)
 lip=ellipsoid('Muzzle / broad cream upper Slowpoke lip',CREAM,(0.,.369,.497),(.246,.077,.132),28,16)
 jaw=ellipsoid('Jaw / low cream open Slowpoke lower lip',CREAM,(0.,.265,.489),(.215,.043,.116),28,14)
 back=ellipsoid('Mouth back / recessed dark sleepy oral chamber',INK,(0.,.317,.488),(.179,.060,.032),24,14)
 tongue=ellipsoid('Tongue / small rooted pink sleepy tongue',SHADOW,(0.,.293,.555),(.132,.012,.052),24,10)
 p=[body,head,lip,jaw,back,tongue]
 for side in (-1,1):
  eye=paint('Eye / fitted white empty Slowpoke stare '+str(side),CREAM,oval(side*.143,.488,.047,.065,32),head,.008,3);p.append(eye)
  p.append(paint('Pupil / fitted tiny black Slowpoke dot '+str(side),INK,oval(side*.150,.501,.0055,.008,20),eye,.003,2))
  ear=ellipsoid('Ear / rounded pink Slowpoke pinna '+str(side),PINK,(side*.216,.626,.221),(.085,.090,.055),24,16);p.append(ear)
  p.append(paint('Ear inset / fitted shadow-pink inner pinna '+str(side),SHADOW,oval(side*.216,.633,.040,.047,24),ear,.004,3))
  p.append(paint('Nostril / fitted small muzzle dot '+str(side),INK,oval(side*.058,.391,.004,.007,16),lip,.003,2))
  p.append(curved_tube('Fang / small rooted cream Slowpoke canine '+str(side),CREAM,[(side*.140,.343,.572),(side*.141,.307,.582),(side*.138,.292,.585)],[.013,.010,.0015],14))
  for rear,z in ((False,.225),(True,-.29)):
   label='rear' if rear else 'fore'
   p.append(ellipsoid('Leg / short pink '+label+' Slowpoke leg '+str(side),PINK,(side*.219,.158,z),(.099,.119,.113),24,14))
   p.append(translate(loft('Foot / planted broad '+label+' Slowpoke pad '+str(side),PINK,[(0.,.082,.090,z+.016),(.028,.105,.110,z+.020),(.070,.103,.103,z+.005),(.105,.075,.072,z)],20),side*.219,0.,0.))
   for j,dx in enumerate((-.052,0.,.052)):
    p.append(curved_tube('Toe / small rooted cream '+label+' Slowpoke claw '+str(side)+' '+str(j),CREAM,[(side*.219+dx,.052,z+.105),(side*.219+dx,.028,z+.141),(side*.219+dx,.010,z+.161)],[.018,.014,.002],12))
 p.append(curved_tube('Tail / thick continuous raised pink Slowpoke tail',PINK,TAIL_POINTS,[.094,.090,.081,.073,.066],24))
 p.append(ellipsoid('Tail tip / rounded pale Slowpoke tail end',CREAM,(.23,.653,-.625),(.067,.091,.068),24,14))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if j%3==1 else 0.))*scale,7) for j,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'slowpoke','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from slowpoke_glb import export_slowpoke
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_slowpoke())
if __name__=='__main__':main()
