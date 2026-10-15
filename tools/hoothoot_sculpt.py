#!/usr/bin/env python3
"""Original Hoothoot owl: clock brows, fitted round eyes and a tucked second foot."""
import argparse,math
from pathlib import Path
from sculpt_geometry import ellipsoid,tube
from raptor_bird_sculpt import paint
from bovine_sculpt import oval
from lugia_sculpt import paddle
from skin_glb import COORDINATES
HEIGHT=1.0
# Verified normal pack RGB555 (23,13,6) and (4,6,16), bit-replicated to RGB8.
OCHRE=(189/255,107/255,49/255);NAVY=(33/255,49/255,132/255)
WHITE=(1.,1.,1.);INK=(0.,0.,0.)

def sculpture(*,normalize=True):
 body=ellipsoid('Body / continuous round ochre owl',OCHRE,(0.,.40,0.),(.295,.31,.245),32,22);p=[body]
 for s in (-1,1):
  mask=paint('Eye mask / fitted navy clock ring '+str(s),NAVY,oval(s*.115,.535,.105,.110,28),body,.005,3);p.append(mask)
  eye=paint('Eye / fitted round white owl lens '+str(s),WHITE,oval(s*.115,.535,.079,.084,28),mask,.003,3);p.append(eye)
  pupil=paint('Pupil / small focused black owl gaze '+str(s),INK,oval(s*.105,.536,.023,.030,24),eye,.003,2);p.append(pupil)
  p.append(paint('Glint / restrained white owl eye '+str(s),WHITE,oval(s*.105-.007,.548,.007,.009,16),pupil,.002,2))
  p.append(paddle('Clock brow / upright hand '+str(s),NAVY,[(s*.034,.629,.182),(s*.106,.713,.127),(s*.18,.803,.054)],[.024,.031,.002],[.021,.018,.002],segments=16,steps=4))
  p.append(paddle('Clock brow / hooked outer hand '+str(s),NAVY,[(s*.034,.629,.182),(s*.191,.752,.055),(s*.345,.744,-.005)],[.024,.033,.003],[.021,.019,.003],segments=16,steps=4))
  p.append(paddle('Wing / folded ochre owl shoulder '+str(s),OCHRE,[(s*.246,.47,-.010),(s*.308,.30,-.04),(s*.242,.155,-.115)],[.050,.071,.015],[.047,.035,.011],segments=18,steps=4))
  for j in range(2):
   p.append(paddle('Flight feather / seated owl wing tip '+str(s)+' '+str(j),OCHRE,[(s*(.281+.017*j),.303,-.063),(s*(.278+.009*j),.211,-.103),(s*(.230+.018*j),.136,-.152)],[.030,.030,.002],[.026,.019,.002],segments=14,steps=4))
 for j,(y,w) in enumerate(((.355,.12),(.282,.095),(.215,.065))):
  # Concave fitted chevrons, not raised spheres on the belly.
  xy=[(-w,y+.024),(0.,y-.020),(w,y+.024),(w*.76,y+.034),(0.,y+.003),(-w*.76,y+.034)]
  p.append(paint('Breast mark / fitted pale feather chevron '+str(j),WHITE,xy,body,.004,2))
 p.append(paddle('Upper beak / short ochre owl hook',OCHRE,[(0.,.468,.217),(0.,.449,.279),(0.,.417,.325)],[.034,.029,.002],[.025,.021,.002],segments=18,steps=4))
 p.append(paddle('Lower beak / short fitted owl jaw',OCHRE,[(0.,.435,.224),(0.,.420,.274),(0.,.419,.303)],[.028,.023,.002],[.015,.012,.002],segments=16,steps=4))
 # Hoothoot displays one foot; the other is explicitly tucked behind the belly.
 p.append(tube('Leg / central weight-bearing owl shank',OCHRE,[(.027,.157,.01),(.027,.087,.037),(.027,.032,.064)],[.028,.022,.018],14))
 p.append(ellipsoid('Foot / single planted owl pad',OCHRE,(.027,.026,.075),(.067,.026,.064),18,12))
 for j,dx in enumerate((-.041,0.,.041)):
  p.append(tube('Toe / three forward owl digits '+str(j),OCHRE,[(.027+dx*.2,.033,.122),(.027+dx,.022,.178),(.027+dx*1.1,.019,.220)],[.014,.012,.006],12))
  p.append(paddle('Claw / small navy owl talon '+str(j),NAVY,[(.027+dx*1.1,.020,.212),(.027+dx*1.1,.014,.235),(.027+dx*1.1,.007,.254)],[.007,.006,.001],[.008,.005,.001],segments=12,steps=3))
 p.append(tube('Rear toe / opposing owl hallux',OCHRE,[(.027,.028,.061),(.039,.016,.011),(.049,.012,-.025)],[.012,.010,.006],12))
 p.append(tube('Leg / tucked second owl shank',OCHRE,[(-.095,.226,-.075),(-.087,.154,-.054),(-.070,.120,-.025)],[.022,.018,.014],12))
 p.append(ellipsoid('Foot / tucked second owl pad',OCHRE,(-.070,.119,-.030),(.037,.017,.045),16,10))
 for j,x in enumerate((-.038,0.,.038)):
  p.append(paddle('Tail / compact ochre owl fan '+str(j),OCHRE,[(x*.5,.185,-.196),(x,.155,-.325),(x*1.4,.211,-.409)],[.025,.035,.003],[.022,.015,.002],segments=14,steps=4))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'hoothoot','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from hoothoot_glb import export_hoothoot
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_hoothoot())
if __name__=='__main__':main()
