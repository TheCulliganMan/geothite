#!/usr/bin/env python3
"""Original stocky tongue bearer with a recessed mouth and broad pink tongue."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,linear
from bovine_sculpt import oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from cubone_sculpt import recessed,skin_shading
from moltres_sculpt import flowing
from skin_glb import COORDINATES
HEIGHT=1.18
PINK=(.94,.64,.70);TONGUE=(.95,.69,.75);CREAM=(.96,.91,.69)
WHITE=(.95,.96,.95);INK=(.17,.13,.19);GROOVE=(.89,.60,.67)

def anatomy():
 body=loft('Body / stocky rounded pink Lickitung torso',PINK,curved_profiles(((.13,.07,.09,-.04),(.25,.24,.23,-.03),(.43,.33,.28,-.012),(.64,.32,.255,.015),(.83,.26,.205,.05),(.98,.13,.12,.08),(1.025,.035,.04,.08)),5),48)
 neck=ellipsoid('Neck / rooted short pink Lickitung throat',PINK,(0,1.007,.08),(.15,.17,.14),36,24)
 head=recessed(ellipsoid('Head / rounded pink Lickitung head with physical oral recess',PINK,(0,1.20,.09),(.304,.28,.263),80,56),[(0,1.05,.215,.125,.177)])
 # The recessed helper rebuilds a part; retain this species' original palette.
 head['base_color']=linear(PINK)
 p=[body,neck,head]
 mouth=skin_shading(paint('Mouth lining / fitted broad dark Lickitung oral bowl',INK,oval(0,1.050,.202,.111,48),head,.004,4),head);p.append(mouth)
 tongue=paddle('Tongue / rooted long broad flattened pink Lickitung tongue',TONGUE,[(0,1.025,.025),(0,.945,.425),(-.012,.713,.704),(-.022,.466,.902),(-.022,.392,.932)],[.140,.152,.151,.115,.004],[.048,.050,.048,.049,.003]);p.append(tongue)
 # A shallow fitted marking follows the actual stored tongue, not a floating rod.
 p.append(skin_shading(paint('Tongue groove / fitted long subtle Lickitung center line',GROOVE,[(-.008,.455),(-.007,.84),(.004,.84),(.003,.455)],tongue,.004,4),tongue))
 for j,(low,high,w) in enumerate(((.26,.365,.18),(.405,.555,.235),(.603,.760,.23),(.805,.928,.16))):
  xy=[(-w*.66,low),(-w,low+.045),(-w,high-.04),(-w*.78,high),(w*.78,high),(w,high-.04),(w,low+.045),(w*.66,low)]
  p.append(skin_shading(paint('Belly band / fitted broad cream Lickitung abdomen '+str(j),CREAM,flowing(xy,3),body,.007,3),body))
 for s in (-1,1):
  eye=paint('Eye / fitted small dark Lickitung eye '+str(s),INK,oval(s*.157,1.277,.019,.028,28),head,.004,3);p.append(eye)
  p.append(paint('Eye glint / fitted white Lickitung gleam '+str(s),WHITE,oval(s*.153,1.287,.006,.007,20),eye,.002,2))
  p.append(paint('Nostril / fitted subtle Lickitung nostril '+str(s),GROOVE,oval(s*.038,1.17,.003,.008,20),head,.003,2))
  shoulder=ellipsoid('Shoulder / rooted round pink Lickitung shoulder '+str(s),PINK,(s*.269,.839,.021),(.125,.136,.133),36,24);p.append(shoulder)
  arm=paddle('Arm / rooted broad blunt pink Lickitung forelimb '+str(s),PINK,[(s*.268,.861,.028),(s*.421,.86,.120),(s*.503,.802,.217)],[.109,.126,.080],[.11,.104,.073]);p.append(arm)
  hand=ellipsoid('Hand / rooted blunt pink Lickitung palm '+str(s),PINK,(s*.494,.806,.223),(.081,.082,.08),36,24);p.append(hand)
  p.append(ellipsoid('Hand tip / rooted small white Lickitung nail '+str(s),WHITE,(s*.501,.76,.279),(.024,.025,.036),28,20))
  thigh=ellipsoid('Thigh / rooted powerful pink Lickitung haunch '+str(s),PINK,(s*.256,.280,.020),(.173,.213,.225),40,28);p.append(thigh)
  ring=skin_shading(paint('Thigh ring / fitted cream Lickitung leg ring '+str(s),CREAM,oval(s*.272,.306,.076,.093,40),thigh,.005,3),thigh);p.append(ring)
  p.append(skin_shading(paint('Thigh center / fitted pink Lickitung ring center '+str(s),PINK,oval(s*.272,.306,.043,.059,32),ring,.003,3),ring))
  foot=ellipsoid('Foot / grounded pink Lickitung foot '+str(s),PINK,(s*.269,.055,.156),(.107,.055,.162),36,24);p.append(foot)
  p.append(ellipsoid('Toe / rooted broad white Lickitung toe tip '+str(s),WHITE,(s*.269,.048,.282),(.075,.039,.075),36,24))
 p.append(paddle('Tail / rooted heavy upward curling pink Lickitung tail',PINK,[(0,.327,-.206),(.028,.502,-.480),(.025,.80,-.615),(.025,1.00,-.489),(.025,1.08,-.38)],[.190,.244,.223,.14,.003],[.139,.174,.148,.10,.002]))
 return p

def sculpture():
 p=anatomy();low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'lickitung','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'lickitung.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
